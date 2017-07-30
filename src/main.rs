mod binary;
mod command;
mod context;
mod eval;
mod expr;
#[macro_use]
mod libc_utils;
mod ptracer;
mod target_desc;

use std::collections::HashMap;

extern crate rustyline;
use rustyline::error::ReadlineError;
use rustyline::Editor;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let ctx = context::Context::new(&args[1]);
    if let Err(e) = ctx {
        println!("{}", e);
        return;
    }
    let ctx = ctx.unwrap();

    let mut ptracer = ptracer::Ptracer::new(
        args[1..args.len()].iter().collect());

    let mut breakpoints = HashMap::new();

    let mut rl = Editor::<()>::new();
    match std::env::home_dir() {
        Some(mut path) => {
            path.push(".vdb_history");
            if let Err(_) = rl.load_history(&path) {
                if path.exists() {
                    println!("Failed to load history file: {:?}", path);
                }
            }
        }
        None => println!("Impossible to get your home dir!"),
    }

    loop {
        let readline = rl.readline("(vdb) ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(&line);
                match command::parse(&line) {
                    Ok(cmd) => {
                        match cmd {
                            command::Command::Break(addr) => {
                                let addr = eval::eval(&ctx, addr);
                                let token = ptracer.poke_breakpoint(addr);
                                breakpoints.insert(addr, token);
                            }

                            command::Command::Cont => {
                                ptracer.cont();
                                let status = ptracer.wait();
                                if !status.is_stopped() {
                                    println!("program finished");
                                    break;
                                }
                            }

                            command::Command::Info => {
                                let regs = ptracer.get_regs();
                                println!("ip={:x} sp={:x} bp={:x}",
                                         regs.ip(), regs.sp(), regs.bp());
                            }

                            command::Command::Print(val) => {
                                println!("{}", eval::eval(&ctx, val));
                            }

                            command::Command::StepI => {
                                ptracer.single_step();
                                let status = ptracer.wait();
                                if !status.is_stopped() {
                                    println!("program finished");
                                    break;
                                }
                            }

                            command::Command::X(num, base, addr) => {
                                let addr = eval::eval(&ctx, addr);
                                for i in 0..num {
                                    let addr = addr + (i * 4) as u64;
                                    let data = ptracer.peek_word(addr);
                                    println!("{:x}: {:x}", addr, data as i32);
                                }
                            }

                        }
                    }
                    Err(e) => {
                        if e.len() > 0 {
                            println!("{}", e);
                        }
                    }
                }
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    match std::env::home_dir() {
        Some(mut path) => {
            path.push(".vdb_history");
            if let Err(_) = rl.save_history(&path) {
                println!("Failed to save history file: {:?}", path);
            }
        }
        None => println!("Impossible to get your home dir!"),
    }
}
