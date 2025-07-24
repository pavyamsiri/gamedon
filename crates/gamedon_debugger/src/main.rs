use color_eyre::Report;
use gamedon_bus::{BusReader, MemoryBus};
use gamedon_cpu::{BootRom, BreakPointCondition, Cpu};
use owo_colors::{OwoColorize, Stream, Style};
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use std::env;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

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

fn main() -> Result<(), Report> {
    color_eyre::install().expect("only called once.");
    let filter = EnvFilter::builder()
        .from_env()?
        .add_directive("rustyline=info".parse()?);
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();

    let args: Vec<_> = env::args().collect();
    let path = args.get(1).expect("missing path.");

    let bytes = read_binary_file(path).expect("can't read binary file.");

    let mut cpu = Cpu::default();
    let mut bus = MemoryBus::load_rom(&bytes)?;

    cpu.boot(BootRom::Doctor);
    cpu.debug_flag = false;
    let MAX_INSTRUCTIONS: usize = if cpu.debug_flag { 900_000 } else { usize::MAX };

    cpu.add_breakpoint(0x0101, BreakPointCondition::RunInto);
    cpu.pause();

    let mut rl = DefaultEditor::new().unwrap();

    for i in 0..MAX_INSTRUCTIONS {
        tracing::debug!("Instruction {i}");
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

        if cpu.is_paused() {
            if handle_user_input(&mut rl, &mut cpu, &bus) {
                break;
            }
            cpu.resume();
        }
    }

    Ok(())
}

fn handle_user_input(rl: &mut DefaultEditor, cpu: &mut Cpu, bus: &MemoryBus) -> bool {
    loop {
        let readline = rl.readline("> ");

        match readline {
            Ok(mut line) => {
                line.make_ascii_lowercase();
                let line_ref = line.as_str();
                rl.add_history_entry(line_ref).unwrap();

                let mut tokens = line.split_whitespace();
                let Some(command) = tokens.next() else {
                    return false;
                };

                match command {
                    "exit" => {
                        return true;
                    }
                    "step" => {
                        return false;
                    }
                    "serial" => {
                        let output = bus.get_serial_output();
                        let serial_string = String::from_utf8_lossy(output);
                        if serial_string.is_empty() {
                            tracing::info!(
                                "{}",
                                "`<EMPTY>`".if_supports_color(Stream::Stdout, |text| text
                                    .style(Style::new().yellow()))
                            );
                        } else {
                            tracing::info!(
                                "{}",
                                serial_string.if_supports_color(Stream::Stdout, |text| text
                                    .style(Style::new().blue()))
                            );
                        }
                        return false;
                    }
                    "runto" => {
                        let Some(operand) = tokens.next() else {
                            tracing::error!("`runto` requires an address!");
                            continue;
                        };
                        let Ok(operand) = u16::from_str_radix(operand.trim_start_matches("0x"), 16)
                        else {
                            tracing::error!("`runto` requires a hexadecimal address!");
                            continue;
                        };
                        cpu.add_breakpoint(operand, BreakPointCondition::RunInto);
                        return false;
                    }
                    "read" => {
                        let Some(operand) = tokens.next() else {
                            tracing::error!("`read` requires an address!");
                            continue;
                        };
                        let Ok(operand) = u16::from_str_radix(operand.trim_start_matches("0x"), 16)
                        else {
                            tracing::error!("`read` requires a hexadecimal address!");
                            continue;
                        };
                        let byte = bus.read_byte(operand);
                        match byte {
                            Ok(value) => {
                                tracing::info!("{operand:#06X} = {value:#04X}");
                            }
                            Err(err) => {
                                tracing::info!("{operand:#06X}: {err}");
                            }
                        }

                        return false;
                    }
                    _ => {
                        tracing::info!("Got {line_ref}");
                    }
                }
            }
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => return true,
            Err(err) => {
                tracing::error!("Command line reader errored: {err}");
                return true;
            }
        }
    }
}
