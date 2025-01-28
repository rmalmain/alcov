use std::ffi::CStr;
use std::io::{Read, Write};
use std::ops::Range;
use std::path::PathBuf;
use byteorder::{ReadBytesExt, WriteBytesExt};
use crate::v0::{Error, ED};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlcovSegment {
    pub module_range: Range<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlcovModule {
    pub base_address: u64,
    pub path: Option<PathBuf>,
    pub segments: Vec<AlcovSegment>,
}

impl AlcovSegment {
    pub fn new(module_range: Range<u64>) -> Self {
        Self { module_range }
    }

    pub fn write<W>(&self, writer: &mut W) -> Result<(), Error>
    where
        W: Write,
    {
        writer.write_u64::<ED>(self.module_range.start)?;
        writer.write_u64::<ED>(self.module_range.clone().count() as u64)?;

        Ok(())
    }

    pub fn read<R>(reader: &mut R) -> Result<Self, Error>
    where
        R: Read,
    {
        let module_offset = reader.read_u64::<ED>()?;
        let size = reader.read_u64::<ED>()?;

        Ok(Self {
            module_range: module_offset..(module_offset + size),
        })
    }
}

impl AlcovModule {
    pub fn new(
        base_address: u64,
        path: Option<PathBuf>,
        segments: Vec<AlcovSegment>,
    ) -> Result<Self, Error> {
        if segments.is_empty() {
            return Err(Error::EmptyModule);
        }

        Ok(Self {
            base_address,
            path,
            segments,
        })
    }

    pub fn write<W>(&self, writer: &mut W, path_offset: i64) -> Result<(), Error>
    where
        W: Write,
    {
        writer.write_u64::<ED>(self.base_address)?;
        writer.write_i64::<ED>(path_offset)?;
        writer.write_u8(u8::try_from(self.segments.len())?)?;

        for segment in &self.segments {
            segment.write(writer)?;
        }

        Ok(())
    }

    pub fn read<R>(reader: &mut R, path_chunk: &[u8]) -> Result<Self, Error>
    where
        R: Read,
    {
        let base_address = reader.read_u64::<ED>()?;
        let path_offset = reader.read_i64::<ED>()?;
        let nb_segments = reader.read_u8()?;

        if nb_segments == 0 {
            return Err(Error::EmptyModule);
        }

        let mut segments: Vec<AlcovSegment> = Vec::new();
        for _ in 0..nb_segments {
            segments.push(AlcovSegment::read(reader)?);
        }

        let path = if path_offset >= 0 {
            let path_cstr =
                CStr::from_bytes_until_nul(&path_chunk[(path_offset as usize)..path_chunk.len()])?;
            let path_str = path_cstr.to_str().unwrap();
            Some(PathBuf::from(path_str))
        } else {
            None
        };

        Ok(Self {
            base_address,
            path,
            segments,
        })
    }
}

