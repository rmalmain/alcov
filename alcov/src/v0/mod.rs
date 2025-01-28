use lzma_rs::lzma2_decompress;
use std::ffi::{CStr, CString};
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::io;

pub mod error;
pub use error::Error;

pub mod block;
pub use block::{AlcovBlock, AlcovBlockMetadata};

pub mod edge;
pub use edge::{AlcovEdges, AlcovDstBlockEdgeMetadata, AlcovBlockEdges, AlcovBlockEdgesMetadata, AlcovDstBlockEdge};

pub mod header;
pub use header::{AlcovHeaderMetadata, AlcovHeader, AlcovFlags};

pub mod module;
pub use module::{AlcovModule, AlcovSegment};


mod bindings {
    #![expect(non_camel_case_types)]
    #![expect(unsafe_op_in_unsafe_fn)]
    #![expect(dead_code)]
    #![expect(non_upper_case_globals)]

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub type ED = byteorder::LE;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alcov {
    pub hdr: AlcovHeader,
    pub modules: Vec<AlcovModule>,
    pub blocks: Vec<AlcovBlock>,
    pub edges: Option<AlcovEdges>,
}

fn write_path(cursor: &mut Cursor<&mut Vec<u8>>, path: &Path) -> Result<i64, Error> {
    let offset = cursor.position();
    let path_bytes = path.as_os_str().to_str().ok_or(Error::PathEncodingError)?;

    if !path_bytes.is_ascii() {
        return Err(Error::PathEncodingError);
    }

    // we can unwrap since it's ascii
    let path_cstr = CString::from_str(path_bytes).unwrap();

    cursor.write_all(path_cstr.as_bytes_with_nul())?;

    Ok(i64::try_from(offset)?)
}

// read size bytes from reader and put the result in a new vector.
#[expect(clippy::uninit_vec)]
fn read_alloc<R>(reader: &mut R, size: usize) -> Result<Vec<u8>, io::Error>
where
    R: Read,
{
    let mut buf: Vec<u8> = Vec::with_capacity(size);
    unsafe {
        buf.set_len(size);
    }

    reader.read_exact(&mut buf)?;

    Ok(buf)
}

impl Alcov {
    pub fn new(
        hdr: AlcovHeader,
        modules: Vec<AlcovModule>,
        blocks: Vec<AlcovBlock>,
        edges: Option<AlcovEdges>,
    ) -> Self {
        Self {
            hdr,
            modules,
            blocks,
            edges,
        }
    }

    pub fn get_flags(&self) -> AlcovFlags {
        let mut flags = AlcovFlags::empty();

        if self.hdr.input_path.is_some() {
            flags |= AlcovFlags::InputPath;
        }

        if self.hdr.compress {
            flags |= AlcovFlags::Compress;
        }

        if self.edges.is_some() {
            flags |= AlcovFlags::Edges;
        }

        flags
    }

    pub fn write<W>(&self, writer: &mut W) -> Result<(), Error>
    where
        W: Write,
    {
        let flags = self.get_flags();

        let modules_start: u64 = size_of::<bindings::alcov_hdr>() as u64;

        let mut post_hdr_buf: Vec<u8> = Vec::new();
        let mut post_hdr_cursor = Cursor::new(&mut post_hdr_buf);

        let mut paths_buf: Vec<u8> = Vec::new();
        let mut paths_cursor = Cursor::new(&mut paths_buf);

        if let Some(input_path) = &self.hdr.input_path {
            assert!(flags.intersects(AlcovFlags::InputPath));
            write_path(&mut paths_cursor, input_path.as_path())?;
        }

        for module in &self.modules {
            let offset: i64 = if let Some(path) = &module.path {
                write_path(&mut paths_cursor, path.as_path())?
            } else {
                -1
            };

            module.write(&mut post_hdr_cursor, offset)?;
        }

        let paths_start = modules_start + post_hdr_cursor.position();

        post_hdr_cursor.write_all(&paths_buf)?;

        let blocks_start = modules_start + post_hdr_cursor.position();

        let edges_start: u64 = if let Some(edge) = &self.edges {
            // edges flag should be set. otherwise, the alcov format is malformed.
            assert!(flags.intersects(AlcovFlags::Edges));

            let mut edges_buf: Vec<u8> = Vec::new();
            let mut edges_cursor = Cursor::new(&mut edges_buf);

            let mut blocks_buf: Vec<u8> = Vec::new();
            let mut blocks_cursor = Cursor::new(&mut blocks_buf);

            for (i, block) in self.blocks.iter().enumerate() {
                let offset = edges_cursor.position();

                let block_edges = &edge.adj_list[i];
                let block_edges_md = AlcovBlockEdgesMetadata {
                    out_edges_offset: offset,
                };

                block.write(&mut blocks_cursor, Some((block_edges, &block_edges_md)))?;
                block_edges.write(&mut edges_cursor)?;
            }

            if flags.intersects(AlcovFlags::Compress) {
                let mut blocks_compressed_buf: Vec<u8> = Vec::new();
                blocks_cursor.set_position(0);
                lzma_rs::lzma2_compress(&mut blocks_cursor, &mut blocks_compressed_buf)?;
                post_hdr_cursor.write_all(&blocks_compressed_buf)?;
            } else {
                post_hdr_cursor.write_all(&blocks_buf)?;
            }

            let edge_offset = modules_start + post_hdr_cursor.position();

            let mut edges_compressed_buf: Vec<u8> = Vec::new();
            if flags.intersects(AlcovFlags::Compress) {
                edges_cursor.set_position(0);
                lzma_rs::lzma2_compress(&mut edges_cursor, &mut edges_compressed_buf)?;
                post_hdr_cursor.write_all(&edges_compressed_buf)?;
            } else {
                post_hdr_cursor.write_all(&edges_buf)?;
            }

            post_hdr_cursor.write_all(&edges_buf)?;

            edge_offset
        } else {
            let mut blocks_buf: Vec<u8> = Vec::new();
            let mut blocks_cursor = Cursor::new(&mut blocks_buf);
            for block in &self.blocks {
                block.write(&mut blocks_cursor, None)?;
            }

            if flags.intersects(AlcovFlags::Compress) {
                let mut blocks_compressed_buf: Vec<u8> = Vec::new();
                blocks_cursor.set_position(0);
                lzma_rs::lzma2_compress(&mut blocks_cursor, &mut blocks_compressed_buf)?;
                post_hdr_cursor.write_all(&blocks_compressed_buf)?;
            } else {
                post_hdr_cursor.write_all(&blocks_buf)?;
            }

            0
        };

        let nb_edges: u64 = if let Some(edges) = &self.edges {
            edges.nb_edges()
        } else {
            0
        };

        let hdr_md = AlcovHeaderMetadata {
            version_major: self.hdr.version_major,
            version_minor: self.hdr.version_minor,
            modules_start,
            paths_start,
            blocks_start,
            edges_start,
            nb_modules: self.modules.len() as u32,
            nb_blocks: self.blocks.len() as u32,
            nb_edges,
            flags,
        };

        hdr_md.write(writer)?;
        writer.write_all(&post_hdr_buf)?;

        Ok(())
    }

    pub fn read<R>(reader: &mut R) -> Result<Self, Error>
    where
        R: Read,
    {
        let hdr_md = AlcovHeaderMetadata::read(reader)?;

        let modules_len = (hdr_md.paths_start - hdr_md.modules_start) as usize;
        let modules_buf = read_alloc(reader, modules_len)?;
        let mut modules_rdr = Cursor::new(modules_buf);

        let paths_len = (hdr_md.blocks_start - hdr_md.paths_start) as usize;
        let paths_buf = read_alloc(reader, paths_len)?;

        let input_path: Option<PathBuf> = if hdr_md.flags.intersects(AlcovFlags::InputPath) {
            let path_cstr = CStr::from_bytes_until_nul(&paths_buf)?;
            let path_str = path_cstr.to_str().unwrap();
            Some(PathBuf::from(path_str))
        } else {
            None
        };

        let hdr = AlcovHeader {
            input_path,
            version_major: hdr_md.version_major,
            version_minor: hdr_md.version_minor,
            compress: hdr_md.flags.intersects(AlcovFlags::Compress),
        };

        let raw_blocks_buf = if hdr_md.flags.intersects(AlcovFlags::Edges) {
            let blocks_len = (hdr_md.edges_start - hdr_md.blocks_start) as usize;
            read_alloc(reader, blocks_len)?
        } else {
            let mut blocks_buf = Vec::new();
            reader.read_to_end(&mut blocks_buf)?;
            blocks_buf
        };

        let blocks_buf = if hdr_md.flags.intersects(AlcovFlags::Compress) {
            let mut blocks_compressed_buf_cursor = Cursor::new(&raw_blocks_buf);

            let mut blocks_buf = Vec::new();
            lzma2_decompress(&mut blocks_compressed_buf_cursor, &mut blocks_buf)?;
            blocks_buf
        } else {
            raw_blocks_buf
        };
        let mut blocks_rdr = Cursor::new(blocks_buf);

        let mut modules: Vec<AlcovModule> = Vec::new();
        for _ in 0..hdr_md.nb_modules {
            modules.push(AlcovModule::read(&mut modules_rdr, &paths_buf)?);
        }

        let mut blocks: Vec<AlcovBlock> = Vec::new();
        if hdr_md.flags.intersects(AlcovFlags::Edges) {
            let edges_buf: Vec<u8> = if hdr_md.flags.intersects(AlcovFlags::Compress) {
                let mut edges_compressed_buf = Vec::new();
                reader.read_to_end(&mut edges_compressed_buf)?;
                let mut edges_compressed_buf_cursor = Cursor::new(edges_compressed_buf);

                let mut edges_buf = Vec::new();
                lzma2_decompress(&mut edges_compressed_buf_cursor, &mut edges_buf)?;
                edges_buf
            } else {
                let mut edges_buf = Vec::new();
                reader.read_to_end(&mut edges_buf)?;
                edges_buf
            };

            let mut edges = AlcovEdges::default();
            for _ in 0..hdr_md.nb_blocks {
                let (block, edge_info) = AlcovBlock::read(&mut blocks_rdr)?;
                let out_edges = AlcovBlockEdges::read(&edges_buf, &edge_info)?;

                blocks.push(block);
                edges.adj_list.push(out_edges);
            }

            Ok(Self {
                hdr,
                modules,
                blocks,
                edges: Some(edges),
            })
        } else {
            for _ in 0..hdr_md.nb_blocks {
                let (block, _) = AlcovBlock::read(&mut blocks_rdr)?;
                blocks.push(block);
            }

            Ok(Self {
                hdr,
                modules,
                blocks,
                edges: None,
            })
        }
    }

    pub fn should_compress(&self) -> bool {
        self.hdr.compress
    }

    pub fn has_edges(&self) -> bool {
        self.edges.is_some()
    }

    pub fn has_input(&self) -> bool {
        self.hdr.input_path.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_simple() {
        let hdr = AlcovHeader::new(Some("abcd"), true);

        let modules = vec![
            AlcovModule::new(0, Some(PathBuf::from("/home/abc")), vec![
                AlcovSegment::new(0..0x1000),
                AlcovSegment::new(0x2000..0x3000),
                AlcovSegment::new(0xaaaaaaaaa..0xbbbbbbbbbbbbbb),
            ])
            .unwrap(),
            AlcovModule::new(0x12345, None, vec![
                AlcovSegment::new(0..0x1000),
                AlcovSegment::new(0xaaaaaaaaa..0xbbbbbbbbbbbbbb),
            ])
            .unwrap(),
        ];

        let blocks = vec![
            AlcovBlock::new(0, 0, 500, 32, 12),
            AlcovBlock::new(0, 0, 560, 16, 3),
            AlcovBlock::new(0, 0, 620, 47, 1),
        ];

        let mut edges = AlcovEdges::new();
        edges.add(0, 1);
        edges.add(0, 1);
        edges.add(1, 2);
        edges.add(2, 0);

        let alcov = Alcov::new(hdr, modules, blocks, Some(edges));

        let mut out_buf: Vec<u8> = Vec::new();
        {
            let mut out_cursor = Cursor::new(&mut out_buf);
            alcov.write(&mut out_cursor).unwrap();
        }

        let mut tmpfile = NamedTempFile::new().unwrap();
        println!("path: {}", tmpfile.path().display());
        tmpfile.write_all(&out_buf).unwrap();
        tmpfile.keep().unwrap();

        let mut out_cursor = Cursor::new(&mut out_buf);
        let new_alcov = Alcov::read(&mut out_cursor).unwrap();

        if alcov != new_alcov {
            println!("alcov: {:#?}", alcov);
            println!("new alcov: {:#?}", new_alcov);
            panic!("alcov serialization is incorrect.");
        }
    }
}
