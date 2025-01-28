use std::io::{Read, Write};
use byteorder::{ReadBytesExt, WriteBytesExt};
use crate::v0::{AlcovBlockEdges, AlcovBlockEdgesMetadata, Error, ED};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlcovBlock {
    pub module_id: u8,
    pub segment_id: u8,
    pub segment_offset: u32,
    pub size: u16,
    pub nb_taken: u64,
}

#[derive(Debug, Clone)]
pub struct AlcovBlockMetadata {
    pub nb_out_edges: u64,
    pub out_edges_offset: u64,
}

impl AlcovBlock {
    pub fn new(
        module_id: u8,
        segment_id: u8,
        segment_offset: u32,
        size: u16,
        nb_taken: u64,
    ) -> Self {
        Self {
            module_id,
            segment_id,
            segment_offset,
            size,
            nb_taken,
        }
    }

    pub fn write<W>(
        &self,
        writer: &mut W,
        out_edges: Option<(&AlcovBlockEdges, &AlcovBlockEdgesMetadata)>,
    ) -> Result<(), Error>
    where
        W: Write,
    {
        writer.write_u32::<ED>(self.segment_offset)?;
        writer.write_u16::<ED>(self.size)?;
        writer.write_u8(self.module_id)?;
        writer.write_u8(self.segment_id)?;
        if let Some((out_edges, out_edges_md)) = out_edges {
            writer.write_u64::<ED>(out_edges.dst_modules.len() as u64)?;
            writer.write_u64::<ED>(out_edges_md.out_edges_offset)?;
        } else {
            writer.write_u64::<ED>(0)?;
            writer.write_u64::<ED>(0)?;
        }
        writer.write_u64::<ED>(self.nb_taken)?;

        Ok(())
    }

    pub fn read<R>(reader: &mut R) -> Result<(Self, AlcovBlockMetadata), Error>
    where
        R: Read,
    {
        let segment_offset = reader.read_u32::<ED>()?;
        let size = reader.read_u16::<ED>()?;
        let module_id = reader.read_u8()?;
        let segment_id = reader.read_u8()?;
        let nb_out_edges = reader.read_u64::<ED>()?;
        let out_edges_offset = reader.read_u64::<ED>()?;
        let nb_taken = reader.read_u64::<ED>()?;

        Ok((
            Self {
                segment_offset,
                size,
                module_id,
                segment_id,
                nb_taken,
            },
            AlcovBlockMetadata {
                nb_out_edges,
                out_edges_offset,
            },
        ))
    }
}

