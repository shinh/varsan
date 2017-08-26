extern crate regex;

use binary;
use breakpoint;
use command;
use eval;
use log;
use ptracer;
use target_desc;
use std::collections::HashMap;

pub struct Context<'a> {
    main_binary: Option<binary::Binary<'a>>,
    interp: Option<binary::Binary<'a>>,

    args: Vec<String>,
    symtab: HashMap<&'a str, u64>,
    ptracer: Option<ptracer::Ptracer>,
    breakpoints: breakpoint::BreakpointManager,
    needs_wait: bool,
    regs: ptracer::Registers,
    target: target_desc::Target,
    cur_breakpoint: i32,

    r_map: u64,
}

impl<'a> Context<'a> {
    pub fn new(args: &Vec<String>) -> Self {
        Self {
            main_binary: None,
            interp: None,
            args: args.iter().map(|a|a.clone()).collect(),
            symtab: HashMap::new(),
            ptracer: None,
            breakpoints: breakpoint::BreakpointManager::new(),
            needs_wait: false,
            regs: ptracer::Registers::empty(),
            target: target_desc::get_target(),
            cur_breakpoint: 0,
            r_map: 0,
        }
    }

    #[allow(dead_code)]
    pub fn ip(&self) -> u64 { self.regs.ip() }

    #[allow(dead_code)]
    pub fn is_running(&self) -> bool { self.ptracer.is_some() }

    #[cfg(test)]
    fn interp(&self) -> Option<&binary::Binary<'a>> {
        return self.interp.as_ref();
    }

    #[cfg(test)]
    fn ptracer(&self) -> &ptracer::Ptracer {
        return self.ptracer.as_ref().unwrap();
    }

    pub fn set_main_binary(&mut self, main_binary: &str)
                           -> Result<String, String> {
        self.symtab.clear();
        let bin = try!(binary::Binary::new(main_binary.to_string()));
        for sym in bin.syms() {
            self.symtab.insert(sym.name, sym.value);
        }

        if let Some(interp) = bin.interp() {
            let interp = try!(binary::Binary::new(interp.to_string()));
            self.interp = Some(interp);
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

    fn handle_breakpoint(&mut self, is_single_step: bool)
                         -> Result<String, String> {
        {
            let ptracer = self.ptracer.as_ref().unwrap();
            self.regs = ptracer.get_regs();
            // TODO: Handle single-step-to-braekpoint case.
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
                    match bp.action() {
                        &Some(breakpoint::Action::UpdateRDebug) => {
                        }

                        &Some(breakpoint::Action::EnterMainBinary) => {
                            log::info(format!("Entering main binary"));
                        }

                        &None => {
                            return Ok(format!("Breakpoint {}, 0x{:x}",
                                              bp.id(), self.regs.ip()));
                        }
                    }
                }
                None => {
                    return Ok("".to_string());
                }
            }
        }

        try!(self.cont());
        return Ok(format!(""));
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
                return self.handle_breakpoint(is_single_step);
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
            if let Some(bp) = self.breakpoints.find_by_id(self.cur_breakpoint) {
                ptracer.poke_byte(bp.addr(), bp.token());
                ptracer.single_step();
                ptracer.wait();
                ptracer.poke_breakpoint(bp.addr());
            }
            self.cur_breakpoint = 0;
        }

        ptracer.cont();
        assert!(!self.needs_wait);
        self.needs_wait = true;
        return Ok("Continuing.".to_string());
    }

    fn set_entry_bias(&mut self, ip: u64) {
        if let Some(ref mut interp) = self.interp {
            let entry = interp.entry();
            interp.set_bias(ip - entry);
        } else if let Some(ref mut main_binary) = self.main_binary {
            let entry = main_binary.entry();
            main_binary.set_bias(ip - entry);
        } else {
            panic!("No binary is set");
        }
    }

    pub fn run(&mut self, args: Vec<String>) -> Result<String, String> {
        let msg = try!(self.start(args));
        try!(self.cont());
        return Ok(msg);
    }

    fn handle_boot_entry(&mut self) {
        if self.interp.is_some() {
            // TODO: Handle PIE.
            let main_binary = match self.main_binary.as_ref() {
                Some(bin) => bin,
                None => panic!("No start binary"),
            };
            self.breakpoints.add(main_binary.entry(), false,
                                 Some(breakpoint::Action::EnterMainBinary),
                                 self.ptracer.as_ref());
            return;
        } else {
            self.read_r_debug();
        }
    }

    fn read_r_debug(&mut self) {
        let bin = {
            match self.interp.as_ref() {
                Some(bin) => bin,
                None => match self.main_binary.as_ref() {
                    Some(bin) => bin,
                    None => panic!("No start binary"),
                }
            }
        };

        for sym in bin.syms() {
            if sym.name == "_r_debug" {
                let r_debug_addr = sym.value + bin.bias();
                self.r_map = r_debug_addr + (self.target.gp_size as u64);
                let ptracer = self.ptracer.as_ref();
                let bp = ptracer.unwrap().peek_word(
                    r_debug_addr + (self.target.gp_size as u64) * 2);
                log::info(format!("r_debug_addr={:x} bp={:x}",
                                  r_debug_addr, bp));
                self.breakpoints.add(bp, false,
                                     Some(breakpoint::Action::UpdateRDebug),
                                     ptracer);
            }
        }
    }

    pub fn start(&mut self, args: Vec<String>) -> Result<String, String> {
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
        // TODO: Stop at main if it exists.
        let ptracer = ptracer::Ptracer::new(&argv);
        let msg = format!("Starting program: {} (pid={})",
                          argv[0], ptracer.pid());

        let regs = ptracer.get_regs();
        self.set_entry_bias(regs.ip());

        self.ptracer = Some(ptracer);
        self.breakpoints.notify_start(&self.ptracer.as_ref().unwrap());

        self.handle_boot_entry();
        return Ok(msg);
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
        let bp = self.breakpoints.add(addr, true, None,
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

            command::Command::Start(args) => {
                return self.start(args);
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

#[cfg(test)]
fn ok_match(pat: &str, result: Result<String, String>, expr: &str) {
    match result {
        Err(err) => {
            panic!("assertion failed: `{}` is an err: {}", expr, err);
        }
        Ok(msg) => {
            let re = regex::Regex::new(pat).unwrap();
            if !re.is_match(&msg) {
                panic!("assertion failed: `{}` (\"{}\"), does not match {}",
                       expr, msg, pat);
            }
        }
    }
}

macro_rules! assert_ok_match {
    ($pat:expr, $result:expr) => {
        ok_match($pat, $result, stringify!($result));
    };
}

#[test]
fn test_start() {
    let args = vec!["test/data/hello".to_string()];
    let mut ctx = Context::new(&args);
    assert!(!ctx.is_running());
    assert!(ctx.set_main_binary(&args[0]).is_ok());
    assert!(ctx.start(vec!()).is_ok());
    assert!(ctx.is_running());
    assert!(ctx.interp().unwrap().bias() != 0);
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
    assert_ok_match!(r"Breakpoint 1 at 0x", ctx.add_breakpoint(addr));
    assert!(!ctx.is_running());

    assert!(ctx.run(vec!()).is_ok());
    assert!(ctx.wait().is_ok());
    assert_ok_match!(r"Breakpoint 1, ", ctx.wait());
    assert!(ctx.is_running());
    assert_eq!(ctx.ip(), addr);
    assert!(ctx.single_step().is_ok());
    assert_eq!(ctx.ip(), addr + 1);
    assert!(ctx.is_running());
    assert!(ctx.cont().is_ok());
    assert_ok_match!(r"Process \d+ exited with code 0", ctx.wait());
    assert!(!ctx.is_running());
}

#[test]
fn test_segv() {
    let args = vec!["test/data/segv".to_string()];
    let mut ctx = Context::new(&args);
    assert!(!ctx.is_running());
    assert!(ctx.set_main_binary(&args[0]).is_ok());
    assert!(ctx.run(vec!()).is_ok());
    assert!(ctx.wait().is_ok());
    assert_ok_match!(r"Process \d+ signaled with code \d+", ctx.wait());
    assert!(!ctx.is_running());
}

#[test]
fn test_neg_one() {
    let args = vec!["test/data/neg_one".to_string()];
    let mut ctx = Context::new(&args);
    assert!(!ctx.is_running());
    assert!(ctx.set_main_binary(&args[0]).is_ok());
    assert!(ctx.start(vec!()).is_ok());

    let addr = ctx.resolve("neg_one").unwrap();
    assert_eq!(-1, ctx.ptracer().peek_word(addr) as i64);
}
