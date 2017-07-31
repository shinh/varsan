mod binary;
mod breakpoint;
mod command;
mod context;
mod eval;
mod expr;
mod flags;
#[macro_use]
mod libc_utils;
mod ptracer;
mod target_desc;

extern crate colored;
use colored::*;
extern crate rustyline;
use rustyline::error::ReadlineError;
use rustyline::Editor;

fn main() {
    let flags = flags::parse(std::env::args().collect());

    let mut ctx = context::Context::new(&flags.args);
    if let Err(e) = ctx {
        println!("{}", e.red());
        return;
    }
    let mut ctx = ctx.unwrap();

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
                        match ctx.run_command(cmd) {
                            Ok(result) => {
                                if result.len() > 0 {
                                    println!("{}", result);
                                }
                            }
                            Err(err) => {
                                println!("{}", err.red());
                            }
                        }
                        if ctx.is_done() {
                            break;
                        }
                    }
                    Err(e) => {
                        if e.len() > 0 {
                            println!("{}", e.red());
                        }
                    }
                }
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            },
            Err(ReadlineError::Eof) => {
                println!("quit");
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
