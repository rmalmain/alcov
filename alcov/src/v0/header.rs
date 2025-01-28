use std::{fmt, iter};
use std::fmt::{Display, Formatter};
use std::io::{Read, Write};
use std::path::PathBuf;
use bitflags::bitflags;
use byteorder::ReadBytesExt;
use crate::v0::{bindings, Error, ED};

bitflags! {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct AlcovFlags: u16 {
        const Edges = bindings::ALCOV_FLAG_EDGES as u16;
        const Compress = bindings::ALCOV_FLAG_COMPRESS as u16;
        const InputPath = bindings::ALCOV_FLAG_INPUT_PATH as u16;
    }
}

/// helper for parsing
#[derive(Debug, Clone)]
pub struct AlcovHeaderMetadata {
    pub version_major: u64,
    pub version_minor: u64,
    pub nb_modules: u32,
    pub nb_blocks: u32,
    pub nb_edges: u64,
    pub modules_start: u64,
    pub paths_start: u64,
    pub blocks_start: u64,
    pub edges_start: u64,
    pub flags: AlcovFlags,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlcovHeader {
    pub version_major: u64,
    pub version_minor: u64,
    pub compress: bool,
    pub input_path: Option<PathBuf>,
}

impl Display for AlcovFlags {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut sep = iter::once("");
        for (flag, _) in self.iter_names() {
            write!(f, "{}{}", sep.next().unwrap_or(", "), flag)?;
        }

        Ok(())
    }
}

impl AlcovHeaderMetadata {
    pub fn write<W>(&self, writer: &mut W) -> Result<(), Error>
    where
        W: Write,
    {
        writer.write_all(&bindings::ALCOV_MAGIC.to_le_bytes())?;
        writer.write_all(&self.version_major.to_le_bytes())?;
        writer.write_all(&self.version_minor.to_le_bytes())?;
        writer.write_all(&self.nb_modules.to_le_bytes())?;
        writer.write_all(&self.nb_blocks.to_le_bytes())?;
        writer.write_all(&self.nb_edges.to_le_bytes())?;
        writer.write_all(&self.modules_start.to_le_bytes())?;
        writer.write_all(&self.paths_start.to_le_bytes())?;
        writer.write_all(&self.blocks_start.to_le_bytes())?;
        writer.write_all(&self.edges_start.to_le_bytes())?;
        writer.write_all(&self.flags.bits().to_le_bytes())?;

        Ok(())
    }

    pub fn read<R>(reader: &mut R) -> Result<Self, Error>
    where
        R: Read,
    {
        let magic = reader.read_u64::<ED>()?;

        if !magic == bindings::ALCOV_MAGIC {
            return Err(Error::WrongMagic);
        }

        let version_major = reader.read_u64::<ED>()?;
        let version_minor = reader.read_u64::<ED>()?;
        let nb_modules = reader.read_u32::<ED>()?;
        let nb_blocks = reader.read_u32::<ED>()?;
        let nb_edges = reader.read_u64::<ED>()?;
        let modules_start = reader.read_u64::<ED>()?;
        let paths_start = reader.read_u64::<ED>()?;
        let blocks_start = reader.read_u64::<ED>()?;
        let edges_start = reader.read_u64::<ED>()?;

        let flags_int: u16 = reader.read_u16::<ED>()?;
        let flags = AlcovFlags::from_bits(flags_int).ok_or(Error::WrongFlags(flags_int))?;

        Ok(AlcovHeaderMetadata {
            version_major,
            version_minor,
            nb_modules,
            nb_blocks,
            nb_edges,
            modules_start,
            paths_start,
            blocks_start,
            edges_start,
            flags,
        })
    }
}

impl AlcovHeader {
    pub fn new<P>(input_path: Option<P>, compress: bool) -> Self
    where
        P: Into<PathBuf>,
    {
        Self {
            version_major: bindings::ALCOV_VERSION_MAJOR,
            version_minor: bindings::ALCOV_VERSION_MINOR,
            compress,
            input_path: input_path.map(Into::into),
        }
    }
}

