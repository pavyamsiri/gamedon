use clap::Parser as ClapParser;
use color_eyre::Report;
use gamedon_file::read_binary_file;
use gamedon_opcode::disassemble;
use std::path::PathBuf;

/// The command line arguments.
#[derive(ClapParser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The path to the ROM.
    path: PathBuf,
}

#[expect(
    clippy::print_stdout,
    reason = "printing to stdout is the point of the binary."
)]
fn main() -> Result<(), Report> {
    color_eyre::install().expect("only called once.");
    let args = Args::parse();
    let path = args.path;

    let bytes = read_binary_file(path)?;
    let disasm = disassemble(&mut bytes.iter().copied().enumerate().map(|(addr, byte)| {
        (
            u16::try_from(addr)
                .expect("can't disassemble ROMs with addresses larger than 2^16 bytes."),
            byte,
        )
    }));
    println!("{disasm}");

    Ok(())
}
