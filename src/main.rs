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

    let mut ptracer = ptracer::Ptracer::new(
        args[1..args.len()].iter().collect());
    let mut ctx = context::Context::new(&args[1], ptracer);
    if let Err(e) = ctx {
        println!("{}", e);
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
                        let result = ctx.run_command(cmd);
                        if result.len() > 0 {
                            println!("{}", result);
                        }
                        if ctx.is_done() {
                            break;
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
