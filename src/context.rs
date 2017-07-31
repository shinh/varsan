use binary;
use breakpoint;
use command;
use eval;
use ptracer;
use std::collections::HashMap;

pub struct Context<'a> {
    main_binary: binary::Binary<'a>,
    symtab: HashMap<&'a str, u64>,
    ptracer: Option<ptracer::Ptracer>,
    breakpoints: breakpoint::BreakpointManager,
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
            ptracer: Some(ptracer),
            breakpoints: breakpoint::BreakpointManager::new(),
            done: false,
        })
    }

    pub fn is_done(&self) -> bool { self.done }

    pub fn resolve(&self, name: &str) -> Option<&u64> {
        return self.symtab.get(name);
    }

    pub fn run_command(&mut self, cmd: command::Command)
                       -> Result<String, String> {
        match cmd {
            command::Command::Break(addr) => {
                let addr = eval::eval(self, addr);
                let bp = self.breakpoints.add(addr, true,
                                              self.ptracer.as_ref());
                return Ok(format!("Breakpoint {} at 0x{:x}",
                                  bp.id(), bp.addr()));
            }

            command::Command::Cont => {
                if self.ptracer.is_none() {
                    return Err("The program is not being run.".to_string());
                }
                let ptracer = self.ptracer.as_mut().unwrap();

                ptracer.cont();
                let status = ptracer.wait();
                if !status.is_stopped() {
                    println!("program finished");
                    self.done = true;
                }
            }

            command::Command::Info => {
                if self.ptracer.is_none() {
                    return Err("The program is not being run.".to_string());
                }
                let ptracer = self.ptracer.as_mut().unwrap();

                let regs = ptracer.get_regs();
                println!("ip={:x} sp={:x} bp={:x}",
                         regs.ip(), regs.sp(), regs.bp());
            }

            command::Command::Print(val) => {
                println!("{}", eval::eval(self, val));
            }

            command::Command::StepI => {
                if self.ptracer.is_none() {
                    return Err("The program is not being run.".to_string());
                }
                let ptracer = self.ptracer.as_mut().unwrap();

                ptracer.single_step();
                let status = ptracer.wait();
                if !status.is_stopped() {
                    println!("program finished");
                    self.done = true;
                }
            }

            command::Command::X(num, base, addr) => {
                if self.ptracer.is_none() {
                    return Err("The program is not being run.".to_string());
                }
                let ptracer = self.ptracer.as_ref().unwrap();

                let addr = eval::eval(self, addr);
                for i in 0..num {
                    let addr = addr + (i * 4) as u64;
                    let data = ptracer.peek_word(addr);
                    println!("{:x}: {:x}", addr, data as i32);
                }
            }

        }
        return Ok("".to_string());
    }
}
