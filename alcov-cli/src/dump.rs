use clap::Args;
use std::io;
use std::io::Write;

#[cfg(feature = "v0")]
use alcov::v0::{Alcov, Error};
use clap_stdin::FileOrStdin;

/// Read an alcov file
#[derive(Clone, Debug, Args)]
pub struct Dump {
    /// Show the metadata of the file
    #[arg(short, long)]
    pub metadata: bool,
    /// Show blocks
    #[arg(short, long)]
    pub blocks: bool,
    /// Show edges
    #[arg(short, long)]
    pub edges: bool,
    /// Input, or empty to get from STDIN.
    #[arg(default_value = "-")]
    input: FileOrStdin,
}

impl Dump {
    pub fn run(self) -> Result<(), Error> {
        let mut stdout = io::stdout();

        let mut input_rdr = self.input.into_reader().unwrap();
        let alcov = Alcov::read(&mut input_rdr)?;

        if self.metadata {
            Self::write_md(&mut stdout, &alcov)?;
        }

        Ok(())
    }

    pub fn write_md<W>(writer: &mut W, alcov: &Alcov) -> Result<(), Error>
    where
        W: Write,
    {
        writeln!(
            writer,
            "alcov file v{}.{}",
            alcov.hdr.version_major, alcov.hdr.version_minor
        )?;
        writeln!(writer)?;

        let flags = alcov.get_flags();
        writeln!(writer, "Flags: {:}", flags)?;
        
        if let Some(input_path) = &alcov.hdr.input_path {
            writeln!(writer, "Input path: {}", input_path.display())?;
        }

        writeln!(writer, "# {} Blocks", alcov.blocks.len())?;

        if let Some(edges) = &alcov.edges {
            writeln!(writer, "# {} Edges", edges.nb_edges())?;
        }

        writeln!(writer)?;

        writeln!(writer, "# {} Modules", alcov.modules.len())?;
        for module in &alcov.modules {
            writeln!(writer, "\tBase address: {}", module.base_address)?;
            if let Some(path) = &module.path {
                writeln!(writer, "\tPath: {}", path.display())?;
            } else {
                writeln!(writer, "\t<no path>")?;
            }

            writeln!(writer, "\t# {} Segments", module.segments.len())?;
            for segment in &module.segments {
                writeln!(
                    writer,
                    "\t\t Range {:#x} -> {:#x} from module base.",
                    segment.module_range.start, segment.module_range.end
                )?;
            }
            writeln!(writer)?;
        }

        Ok(())
    }
}
