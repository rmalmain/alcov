use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::io::{Cursor, Read, Write};
use byteorder::{ReadBytesExt, WriteBytesExt};
use crate::v0::{AlcovBlockMetadata, Error, ED};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AlcovDstBlockEdge {
    pub dst_block_id: u64,
}

impl From<u64> for AlcovDstBlockEdge {
    fn from(value: u64) -> Self {
        Self {
            dst_block_id: value,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlcovDstBlockEdgeMetadata {
    pub nb_taken: u64,
}

pub struct AlcovBlockEdgesMetadata {
    /// offset in the edge chunk.
    pub out_edges_offset: u64,
}

#[derive(Debug, Clone, Default)]
pub struct AlcovBlockEdges {
    /// out edges metadata
    pub dst_modules: HashMap<AlcovDstBlockEdge, AlcovDstBlockEdgeMetadata>,
}

impl PartialEq for AlcovBlockEdges {
    fn eq(&self, other: &Self) -> bool {
        for (k, v) in &self.dst_modules {
            if let Some(other_dst) = other.dst_modules.get(k) {
                if other_dst != v {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
}

impl Eq for AlcovBlockEdges {}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AlcovEdges {
    pub adj_list: Vec<AlcovBlockEdges>,
}

impl AlcovEdges {
    pub fn new() -> Self {
        Self {
            adj_list: Vec::new(),
        }
    }

    /// Adds an edge to the collection.
    ///
    /// If the edge is already present, it will only increment to taken counter for this
    /// particular edge.
    ///
    /// Warning: no check is performed on src_block or dst_block. It can lead to important
    /// memory increases.
    pub fn add(&mut self, src_block: u64, dst_block: u64) {
        let src_idx = usize::try_from(src_block).unwrap();

        if src_idx >= self.adj_list.len() {
            for _ in self.adj_list.len()..=src_idx {
                self.adj_list.push(AlcovBlockEdges::default());
            }
        }

        match self.adj_list[src_idx].dst_modules.entry(dst_block.into()) {
            Entry::Occupied(mut occ_entry) => {
                let md = occ_entry.get_mut();
                md.nb_taken += 1;
            }
            Entry::Vacant(vac_entry) => {
                vac_entry.insert(AlcovDstBlockEdgeMetadata { nb_taken: 1 });
            }
        }
    }

    /// total number of edges
    pub fn nb_edges(&self) -> u64 {
        self.adj_list
            .iter()
            .map(|block_edges| block_edges.dst_modules.len() as u64)
            .sum()
    }
}

impl AlcovDstBlockEdge {
    pub fn write<W>(&self, writer: &mut W) -> Result<(), Error>
    where
        W: Write,
    {
        Ok(writer.write_u64::<ED>(self.dst_block_id)?)
    }

    pub fn read<R>(reader: &mut R) -> Result<Self, Error>
    where
        R: Read,
    {
        let dst_block_id = reader.read_u64::<ED>()?;

        Ok(Self { dst_block_id })
    }
}

impl AlcovDstBlockEdgeMetadata {
    pub fn write<W>(&self, writer: &mut W) -> Result<(), Error>
    where
        W: Write,
    {
        Ok(writer.write_u64::<ED>(self.nb_taken)?)
    }

    pub fn read<R>(reader: &mut R) -> Result<Self, Error>
    where
        R: Read,
    {
        let nb_taken = reader.read_u64::<ED>()?;

        Ok(Self { nb_taken })
    }
}

impl AlcovBlockEdges {
    pub fn write<W>(&self, writer: &mut W) -> Result<(), Error>
    where
        W: Write,
    {
        for (dst_edge, dst_edge_metadata) in &self.dst_modules {
            dst_edge.write(writer)?;
            dst_edge_metadata.write(writer)?;
        }

        Ok(())
    }

    pub fn read(edges_buf: &[u8], edge_chunk_info: &AlcovBlockMetadata) -> Result<Self, Error> {
        let mut buf_cursor = Cursor::new(edges_buf);
        buf_cursor.set_position(edge_chunk_info.out_edges_offset);

        let mut dst_modules: HashMap<AlcovDstBlockEdge, AlcovDstBlockEdgeMetadata> =
            HashMap::default();
        for _ in 0..edge_chunk_info.nb_out_edges {
            let dst_edge = AlcovDstBlockEdge::read(&mut buf_cursor)?;
            let dst_edge_md = AlcovDstBlockEdgeMetadata::read(&mut buf_cursor)?;

            dst_modules.insert(dst_edge, dst_edge_md);
        }

        Ok(Self { dst_modules })
    }
}

