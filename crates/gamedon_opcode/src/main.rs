use gamedon_opcode::disassemble;
use std::env;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

fn read_binary_file(path: impl AsRef<Path>) -> Result<Vec<u8>, String> {
    fn read(path: &Path) -> Result<Vec<u8>, String> {
        let file = File::open(path).map_err(|_| "can't open file.")?;
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();
        reader
            .read_to_end(&mut buffer)
            .map_err(|_| "can't read all bytes.")?;
        Ok(buffer)
    }
    read(path.as_ref())
}

fn main() {
    let args: Vec<_> = env::args().collect();
    let path = args.get(1).expect("missing path.");

    let bytes = read_binary_file(path).expect("can't read binary file.");
    let disasm = disassemble(&bytes);
    println!("{disasm}");
}
