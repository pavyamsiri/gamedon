use gamedon_bus::{BusReader, BusWriter, MemoryBus};
use gamedon_cpu::{BootRom, Cpu};
use std::env;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use tracing::level_filters::LevelFilter;
use tracing::trace;

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
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .init();

    let args: Vec<_> = env::args().collect();
    let path = args.get(1).expect("missing path.");

    let bytes = read_binary_file(path).expect("can't read binary file.");

    let mut cpu = Cpu::default();
    let mut bus = MemoryBus::default();
    for (index, byte) in bytes.iter().copied().enumerate() {
        let Ok(index) = u16::try_from(index) else {
            break;
        };
        bus.write_byte_raw(index, byte).unwrap();
    }

    cpu.boot(BootRom::Doctor);
    cpu.debug_flag = false;
    let MAX_INSTRUCTIONS: usize = if cpu.debug_flag { 900_000 } else { usize::MAX };

    for i in 0..MAX_INSTRUCTIONS {
        tracing::trace!("INSTRUCTION = {i}");
        let num_m_cycles = cpu.step(&mut bus).expect("not expecting errors.");
        bus.timer_tick(num_m_cycles);
        bus.serial_tick(num_m_cycles);

        bus.update_interrupt_requests();

        if let Some(output) = bus.has_new_serial_output()
            && !cpu.debug_flag
        {
            let serial_string = String::from_utf8_lossy(output);
            tracing::debug!("Serial = {}", serial_string);
        }

        if cpu.paused() {
            break;
        }
    }
}
