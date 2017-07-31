use binary;
use breakpoint;
use command;
use eval;
use ptracer;
use std::collections::HashMap;

pub struct Context<'a> {
    main_binary: Option<binary::Binary<'a>>,
    args: Vec<String>,
    symtab: HashMap<&'a str, u64>,
    ptracer: Option<ptracer::Ptracer>,
    breakpoints: breakpoint::BreakpointManager,
}

impl<'a> Context<'a> {
    pub fn new(args: &Vec<String>) -> Self {
        Self {
            main_binary: None,
            args: args.iter().map(|a|a.clone()).collect(),
            symtab: HashMap::new(),
            ptracer: None,
            breakpoints: breakpoint::BreakpointManager::new(),
        }
    }

    pub fn set_main_binary(&mut self, main_binary: &str)
                           -> Result<String, String> {
        self.symtab.clear();
        let bin = try!(binary::Binary::new(main_binary.to_string()));
        for sym in bin.syms() {
            self.symtab.insert(sym.name, sym.value);
        }
        self.main_binary = Some(bin);
        return Ok(format!("Reading symbols from {}...done.", main_binary));
    }

    pub fn resolve(&self, name: &str) -> Option<&u64> {
        return self.symtab.get(name);
    }

    fn check_status(&mut self, status: ptracer::ProcessState)
                    -> Result<String, String> {
        if !status.is_stopped() {
            self.breakpoints.notify_finish();
            self.ptracer = None;
            return Ok("program finished".to_string());
        }
        return Ok("".to_string());
    }

    pub fn cont(&mut self) -> Result<String, String> {
        if self.ptracer.is_none() {
            return Err("The program is not being run.".to_string());
        }

        let status = {
            let ptracer = self.ptracer.as_mut().unwrap();
            ptracer.cont();
            ptracer.wait()
        };
        return self.check_status(status);
    }

    pub fn run(&mut self, args: Vec<String>) -> Result<String, String> {
        let mut argv = vec![];
        {
            let main_binary = try!(self.main_binary.as_mut().ok_or(
                "No executable specified.".to_string()));
            argv.push(main_binary.filename().clone());
        }
        if args.len() > 0 {
            argv.extend(args);
        } else {
            argv.extend(self.args.iter().cloned());
        }
        self.ptracer = Some(ptracer::Ptracer::new(&argv));
        self.breakpoints.notify_start(&self.ptracer.as_ref().unwrap());
        return self.cont();
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
                return self.cont();
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

            command::Command::Run(args) => {
                return self.run(args);
            }

            command::Command::StepI => {
                if self.ptracer.is_none() {
                    return Err("The program is not being run.".to_string());
                }
                let status = {
                    let ptracer = self.ptracer.as_mut().unwrap();
                    ptracer.single_step();
                    ptracer.wait()
                };
                return self.check_status(status);
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
