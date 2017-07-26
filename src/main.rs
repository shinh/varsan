#[macro_use]

mod libc_utils;
mod ptracer;
mod target_desc;

extern crate rustyline;
use rustyline::error::ReadlineError;
use rustyline::Editor;

fn main() {
    let args: Vec<_> = std::env::args().collect();

    let target = target_desc::get_target();

    let mut ptracer = ptracer::Ptracer::new(
        args[1..args.len()].iter().collect());

    let mut rl = Editor::<()>::new();
    loop {
        let readline = rl.readline("(vdb) ");
        match readline {
            Ok(line) => {
                // TODO: Improve the command parser.
                let toks: Vec<&str> = line.split(' ').collect();
                if toks.len() == 0 {
                    continue;
                }
                if toks[0] == "si" || toks[0] == "stepi" {
                    ptracer.single_step();
                    let status = ptracer.wait();
                    if !status.is_stopped() {
                        println!("program finished");
                        break;
                    }
                } else if toks[0] == "i" {
                    let regs = ptracer.get_regs(&target);
                    println!("ip={:x} sp={:x} bp={:x}",
                             regs.ip(), regs.sp(), regs.bp());
                } else {
                    println!("Unknown command: {}", toks[0]);
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
}
