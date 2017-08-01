use binary;
use breakpoint;
use command;
use eval;
use ptracer;
use target_desc;
use std::collections::HashMap;

pub struct Context<'a> {
    main_binary: Option<binary::Binary<'a>>,
    args: Vec<String>,
    symtab: HashMap<&'a str, u64>,
    ptracer: Option<ptracer::Ptracer>,
    breakpoints: breakpoint::BreakpointManager,
    needs_wait: bool,
    regs: ptracer::Registers,
    target: target_desc::Target,
    cur_breakpoint: i32,
}

impl<'a> Context<'a> {
    pub fn new(args: &Vec<String>) -> Self {
        Self {
            main_binary: None,
            args: args.iter().map(|a|a.clone()).collect(),
            symtab: HashMap::new(),
            ptracer: None,
            breakpoints: breakpoint::BreakpointManager::new(),
            needs_wait: false,
            regs: ptracer::Registers::empty(),
            target: target_desc::get_target(),
            cur_breakpoint: 0,
        }
    }

    #[allow(dead_code)]
    pub fn ip(&self) -> u64 { self.regs.ip() }

    #[allow(dead_code)]
    pub fn is_running(&self) -> bool { self.ptracer.is_some() }

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

    pub fn needs_wait(&self) -> bool { self.needs_wait }

    pub fn resolve(&self, name: &str) -> Option<u64> {
        return self.symtab.get(name).map(|v|*v);
    }

    fn pid(&self) -> i32 {
        assert!(self.ptracer.is_some());
        return self.ptracer.as_ref().unwrap().pid() as i32;
    }

    pub fn wait(&mut self) -> Result<String, String> {
        assert!(self.needs_wait);
        return self.wait_impl(false);
    }

    fn wait_impl(&mut self, is_single_step: bool) -> Result<String, String> {
        assert!(self.ptracer.is_some());
        self.needs_wait = false;
        let status = {
            let ptracer = self.ptracer.as_mut().unwrap();
            ptracer.wait()
        };

        match status {
            ptracer::ProcessState::Stop(_) => {
                let ptracer = self.ptracer.as_ref().unwrap();
                self.regs = ptracer.get_regs();
                if is_single_step {
                    return Ok("".to_string());
                }

                let ip = self.regs.ip() - self.target.breakpoint_size as u64;
                match self.breakpoints.find_by_addr(ip) {
                    Some(bp) => {
                        self.regs.update_ip(ip, &self.target);
                        ptracer.set_regs(&self.regs);
                        ptracer.poke_byte(ip, bp.token());
                        self.cur_breakpoint = bp.id();
                        return Ok(format!("Breakpoint {}, 0x{:x}",
                                          bp.id(), self.regs.ip()));
                    }
                    None => {
                        return Ok("".to_string());
                    }
                }
            }
            ptracer::ProcessState::Exit(st) => {
                let pid = self.pid();
                self.breakpoints.notify_finish();
                self.ptracer = None;
                return Ok(format!("Process {} exited with code {}",
                                  pid, st));
            }
            ptracer::ProcessState::Signal(sig) => {
                let pid = self.pid();
                self.breakpoints.notify_finish();
                self.ptracer = None;
                return Ok(format!("Process {} signaled with code {}",
                                  pid, sig));
            }
        }
    }

    pub fn cont(&mut self) -> Result<String, String> {
        if self.ptracer.is_none() {
            return Err("The program is not being run.".to_string());
        }
        let ptracer = self.ptracer.as_mut().unwrap();

        if self.cur_breakpoint != 0 {
            if self.breakpoints.find_by_id(self.cur_breakpoint).is_some() {
                ptracer.single_step();
                ptracer.wait();
            }
            self.cur_breakpoint = 0;
        }

        ptracer.cont();
        assert!(!self.needs_wait);
        self.needs_wait = true;
        return Ok("Continuing.".to_string());
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
        try!(self.cont());
        return Ok(format!("Starting program: {}", argv[0]));
    }

    pub fn single_step(&mut self) -> Result<String, String> {
        if self.ptracer.is_none() {
            return Err("The program is not being run.".to_string());
        }
        {
            let ptracer = self.ptracer.as_mut().unwrap();
            ptracer.single_step();
        }
        try!(self.wait_impl(true));
        return Ok("".to_string());
    }

    pub fn add_breakpoint(&mut self, addr: u64) -> Result<String, String> {
        let bp = self.breakpoints.add(addr, true,
                                      self.ptracer.as_ref());
        return Ok(format!("Breakpoint {} at 0x{:x}", bp.id(), bp.addr()));
    }

    pub fn run_command(&mut self, cmd: command::Command)
                       -> Result<String, String> {
        match cmd {
            command::Command::Break(addr) => {
                let addr = eval::eval(self, addr);
                return self.add_breakpoint(addr);
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
                return self.single_step();
            }

            command::Command::X(num, _, addr) => {
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

#[test]
fn test_hello() {
    let args = vec!["test/data/hello".to_string()];
    let mut ctx = Context::new(&args);
    assert!(!ctx.is_running());
    assert!(ctx.set_main_binary(&args[0]).is_ok());
    let addr = ctx.resolve("main");
    assert!(addr.is_some());
    let addr = addr.unwrap();
    assert!(ctx.add_breakpoint(addr).is_ok());
    assert!(!ctx.is_running());
    assert!(ctx.run(vec!()).is_ok());
    assert!(ctx.wait().is_ok());
    assert!(ctx.is_running());
    assert_eq!(ctx.ip(), addr);
    assert!(ctx.single_step().is_ok());
    assert_eq!(ctx.ip(), addr + 1);
    assert!(ctx.is_running());
    assert!(ctx.cont().is_ok());
    assert!(ctx.wait().is_ok());
    assert!(!ctx.is_running());
}
