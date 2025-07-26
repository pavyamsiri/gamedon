use clap::Parser as ClapParser;
use color_eyre::Report;
use gamedon_bus::{BusReader, MemoryBus};
use gamedon_cpu::{BootRom, BreakPointCondition, Cpu};
use gamedon_cpu::{ExecutionStatus, Instruction};
use gamedon_debugger::{Command, Parser};
use gamedon_file::read_binary_file;
use owo_colors::{OwoColorize, Stream, Style};
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use std::path::PathBuf;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

/// Style to use when pretty printing a keyword.
const KEYWORD_STYLE: Style = Style::new().red();
/// Style to use when pretty printing a value.
const VALUE_STYLE: Style = Style::new().magenta();
/// Style to use when pretty printing an address.
const ADDRESS_STYLE: Style = Style::new().bright_magenta();
/// Style to use when pretty printing a flag.
const FLAG_STYLE: Style = Style::new().yellow();

/// Apply a colour to any displayable token only if stdout supports colour output.
macro_rules! apply_colour {
    ($token:expr, $style:expr) => {
        $token.if_supports_color(Stream::Stdout, |text| text.style($style))
    };
}

/// The command line arguments.
#[derive(ClapParser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The path to the ROM.
    path: PathBuf,
    /// Set this flag to turn on the debugger.
    #[arg(short, long)]
    debug: bool,
}

#[expect(
    clippy::print_stdout,
    reason = "Need to print to stdout for certain debugging uses."
)]
fn main() -> Result<(), Report> {
    color_eyre::install().expect("only called once.");
    let filter = EnvFilter::builder()
        .from_env()?
        .add_directive("rustyline=info".parse()?);
    tracing_subscriber::registry()
        .with(fmt::layer().without_time())
        .with(filter)
        .init();

    let args = Args::parse();
    let path = args.path;

    let bytes = read_binary_file(path).expect("can't read binary file.");

    let mut cpu = Cpu::default();
    let mut bus = MemoryBus::load_rom(&bytes)?;

    cpu.boot(BootRom::Doctor);

    let debug_flag = args.debug;
    let mut paused = debug_flag;
    let mut pause_scheduled = false;
    let mut rl = DefaultEditor::new().unwrap();
    loop {
        if paused && debug_flag {
            if handle_user_input(&mut rl, &mut cpu, &mut bus) {
                break;
            }
            paused = false;
        } else if paused && !debug_flag {
            break;
        } else {
            let (step_result, should_pause) = step(&mut cpu, &mut bus, false);
            pause_scheduled |= should_pause;

            if let ExecutionStatus::Completed(info) = step_result {
                if pause_scheduled {
                    paused = true;
                    pause_scheduled = false;
                }

                if !debug_flag && !matches!(Instruction::decode(info.to_u16()), Instruction::Prefix)
                {
                    println!("{}", cpu.print_doctor_status(&bus)?);
                }
            }
        }
    }

    Ok(())
}

/// Step through one M-cycle of the emulated Gameboy either skipping or respecting breakpoints
/// depending on the flag.
///
/// Returns the execution status of the CPU and whether it has hit a breakpoint.
fn step(cpu: &mut Cpu, bus: &mut MemoryBus, skip_breakpoints: bool) -> (ExecutionStatus, bool) {
    let outcome = match cpu.step(bus, skip_breakpoints) {
        Ok(res) => res,
        Err(err) => panic!("{err}"),
    };
    let should_run = !matches!(outcome.status, ExecutionStatus::Nop);

    let mut should_pause = outcome.hit_breakpoint;

    if should_run {
        bus.timer_tick();
        bus.serial_tick();
        bus.update_interrupt_requests();
        if let Some(output) = bus.has_new_serial_output() {
            let serial_string = String::from_utf8_lossy(output);

            if serial_string.contains("Failed") {
                should_pause |= true;
            }
            tracing::info!("Serial = {}", serial_string);
        }
    }

    (outcome.status, should_pause)
}

/// Display the debug command prompt and handle user input.
#[expect(
    clippy::print_stdout,
    reason = "This function is supposed to act as the command line interface."
)]
fn handle_user_input(rl: &mut DefaultEditor, cpu: &mut Cpu, bus: &mut MemoryBus) -> bool {
    print_status(cpu);
    let prompt = apply_colour!("> ", Style::new().red()).to_string();
    loop {
        let readline = rl.readline(prompt.as_str());

        match readline {
            Ok(mut line) => {
                line.make_ascii_lowercase();
                let line_ref = line.as_str();
                if !line_ref.is_empty() {
                    rl.add_history_entry(line_ref).unwrap();
                }

                match Parser::new(line_ref).parse() {
                    Command::Reask => continue,
                    Command::Retry(s) => {
                        println!("{s}");
                        println!("Please try again.");
                    }
                    Command::Exit => return true,
                    Command::Resume => return false,
                    Command::Step => {
                        let info = 'inst: loop {
                            if let (ExecutionStatus::Completed(inst), _) = step(cpu, bus, true) {
                                break 'inst inst;
                            }
                        };
                        let inst = Instruction::decode(info.to_u16());
                        let mut it = bus.iter_from(info.address + 1);
                        let mut display_str =
                            format!("{:#06X}: ", apply_colour!(info.address, ADDRESS_STYLE));
                        inst.format(&mut display_str, &mut it, info.opcode.to_le_bytes()[0])
                            .unwrap();
                        println!("Executed");
                        println!("{display_str}");
                    }
                    Command::RunInto(addr) => {
                        println!(
                            "Adding runto breakpoint at {:#06X}",
                            apply_colour!(addr, ADDRESS_STYLE),
                        );
                        cpu.add_breakpoint(addr, BreakPointCondition::RunInto);
                    }
                    Command::RunIntoWhen { address, opcode } => {
                        println!(
                            "Adding runto breakpoint at {:#06X} when opcode is {:#06X}",
                            apply_colour!(address, ADDRESS_STYLE),
                            apply_colour!(opcode, VALUE_STYLE),
                        );
                        cpu.add_breakpoint(address, BreakPointCondition::WhenOp(opcode));
                    }
                    Command::Doctor => match cpu.print_doctor_status(bus) {
                        Ok(st) => println!("{st}"),
                        Err(err) => println!("{err}"),
                    },
                    Command::Status => {
                        print_status(cpu);
                    }
                    Command::Read(addr) => match bus.read_byte(addr) {
                        Ok(byte) => {
                            println!(
                                "{:#06X}: {:#04X}",
                                apply_colour!(addr, ADDRESS_STYLE),
                                apply_colour!(byte, VALUE_STYLE),
                            );
                        }
                        Err(err) => {
                            println!("{err}");
                        }
                    },
                    Command::BreakRead { address } => {
                        println!(
                            "Adding {} breakpoint at {:#06X}",
                            apply_colour!("read", KEYWORD_STYLE),
                            apply_colour!(address, ADDRESS_STYLE)
                        );
                        cpu.add_breakpoint(address, BreakPointCondition::ReadBus);
                    }
                    Command::BreakWrite { address, value } => {
                        if let Some(value) = value {
                            println!(
                                "Adding {} breakpoint at {:#06X} on writes equal to {:#04X}",
                                apply_colour!("write", KEYWORD_STYLE),
                                apply_colour!(address, ADDRESS_STYLE),
                                apply_colour!(value, VALUE_STYLE)
                            );
                        } else {
                            println!(
                                "Adding {} breakpoint at {:#06X}",
                                apply_colour!("write", KEYWORD_STYLE),
                                apply_colour!(address, ADDRESS_STYLE)
                            );
                        }
                        cpu.add_breakpoint(address, BreakPointCondition::WriteBus { value });
                    }
                    Command::LastOp => {
                        let info = cpu.get_last_op();
                        let inst = Instruction::decode(info.to_u16());
                        let mut it = bus.iter_from(info.address + 1);
                        let mut display_str =
                            format!("{:#06X}: ", apply_colour!(info.address, ADDRESS_STYLE));
                        inst.format(&mut display_str, &mut it, info.opcode).unwrap();
                        println!("Last opcode");
                        println!("{display_str}");
                    }
                    Command::Stack => {
                        let lo = bus.read_byte(cpu.registers.get_sp());
                        let hi = bus.read_byte(cpu.registers.get_sp().wrapping_add(1));
                        let word = match (lo, hi) {
                            (Ok(lo), Ok(hi)) => u16::from_le_bytes([lo, hi]),
                            (Ok(_), Err(err)) => {
                                println!("{err}");
                                continue;
                            }
                            (Err(err), _) => {
                                println!("{err}");
                                continue;
                            }
                        };
                        println!(
                            "Stack pointer = {:#06X}",
                            apply_colour!(cpu.registers.get_sp(), ADDRESS_STYLE)
                        );
                        println!("Word at SP = {:#06X}", apply_colour!(word, VALUE_STYLE));
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

#[expect(
    clippy::print_stdout,
    reason = "This function is supposed to act as the command line interface."
)]
fn print_status(cpu: &Cpu) {
    const INDENT: &str = "  ";
    let af = cpu.registers.get_af();
    let bc = cpu.registers.get_bc();
    let de = cpu.registers.get_de();
    let hl = cpu.registers.get_hl();
    let sp = cpu.registers.get_sp();
    let pc = cpu.registers.get_pc();
    let addr = cpu.addr;
    let tmp1 = cpu.tmp1;

    let zero = if cpu.registers.get_zero_flag() {
        "Z"
    } else {
        "-"
    };
    let sub = if cpu.registers.get_subtraction_flag() {
        "S"
    } else {
        "-"
    };
    let half = if cpu.registers.get_half_carry_flag() {
        "H"
    } else {
        "-"
    };
    let carry = if cpu.registers.get_carry_flag() {
        "C"
    } else {
        "-"
    };

    println!("Registers:");
    println!("{INDENT}AF: {:#06X}", apply_colour!(af, VALUE_STYLE),);
    println!("{INDENT}BC: {:#06X}", apply_colour!(bc, VALUE_STYLE),);
    println!("{INDENT}DE: {:#06X}", apply_colour!(de, VALUE_STYLE),);
    println!("{INDENT}HL: {:#06X}", apply_colour!(hl, VALUE_STYLE),);
    println!("{INDENT}SP: {:#06X}", apply_colour!(sp, VALUE_STYLE),);
    println!("{INDENT}PC: {:#06X}", apply_colour!(pc, VALUE_STYLE),);
    println!("{INDENT}$A: {:#06X}", apply_colour!(addr, VALUE_STYLE),);
    println!("{INDENT}#T: {:#06X}", apply_colour!(tmp1, VALUE_STYLE),);
    println!(
        "{INDENT}Fl: {}{}{}{}",
        apply_colour!(zero, FLAG_STYLE),
        apply_colour!(sub, FLAG_STYLE),
        apply_colour!(half, FLAG_STYLE),
        apply_colour!(carry, FLAG_STYLE),
    );
}
