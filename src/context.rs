use binary;
use command;
use eval;
use ptracer;
use std::collections::HashMap;

pub struct Context<'a> {
    main_binary: binary::Binary<'a>,
    symtab: HashMap<&'a str, u64>,
    ptracer: ptracer::Ptracer,
    breakpoints: HashMap<u64, u8>,
    done: bool,
}

impl<'a> Context<'a> {
    pub fn new(main_binary: &str, ptracer: ptracer::Ptracer)
               -> Result<Self, String> {
        let bin = try!(binary::Binary::new(main_binary.to_string()));
        let mut symtab = HashMap::new();
        for sym in bin.syms() {
            symtab.insert(sym.name, sym.value);
        }
        Ok(Self {
            main_binary: bin,
            symtab: symtab,
            ptracer: ptracer,
            breakpoints: HashMap::new(),
            done: false,
        })
    }

    pub fn is_done(&self) -> bool { self.done }

    pub fn resolve(&self, name: &str) -> Option<&u64> {
        return self.symtab.get(name);
    }

    pub fn run_command(&mut self, cmd: command::Command) -> String {
        match cmd {
            command::Command::Break(addr) => {
                let addr = eval::eval(self, addr);
                let token = self.ptracer.poke_breakpoint(addr);
                self.breakpoints.insert(addr, token);
            }

            command::Command::Cont => {
                self.ptracer.cont();
                let status = self.ptracer.wait();
                if !status.is_stopped() {
                    println!("program finished");
                    self.done = true;
                }
            }

            command::Command::Info => {
                let regs = self.ptracer.get_regs();
                println!("ip={:x} sp={:x} bp={:x}",
                         regs.ip(), regs.sp(), regs.bp());
            }

            command::Command::Print(val) => {
                println!("{}", eval::eval(self, val));
            }

            command::Command::StepI => {
                self.ptracer.single_step();
                let status = self.ptracer.wait();
                if !status.is_stopped() {
                    println!("program finished");
                    self.done = true;
                }
            }

            command::Command::X(num, base, addr) => {
                let addr = eval::eval(self, addr);
                for i in 0..num {
                    let addr = addr + (i * 4) as u64;
                    let data = self.ptracer.peek_word(addr);
                    println!("{:x}: {:x}", addr, data as i32);
                }
            }

        }
        return "".to_string();
    }
}
