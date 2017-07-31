extern crate libc;

use std;

use libc_utils::*;
use target_desc;

pub struct ProcessState {
    status: i32
}

impl ProcessState {
    pub fn is_stopped(&self) -> bool {
        unsafe {
            libc::WIFSTOPPED(self.status)
        }
    }
}

pub struct Registers {
    gps: Vec<u64>,
    ip: u64,
    sp: u64,
    bp: u64,
}

impl Registers {
    pub fn ip(&self) -> u64 { self.ip }
    pub fn sp(&self) -> u64 { self.sp }
    pub fn bp(&self) -> u64 { self.bp }
}

fn check_ptrace<'a>(retval: i64, msg: &'a str) -> i64 {
    if retval < 0 {
        abort_libc(msg);
    }
    return retval;
}

#[macro_export]
macro_rules! check_ptrace {
    ($r:expr, $e1:expr, $e2:expr, $e3:expr) =>
        (check_ptrace(unsafe {
            libc::ptrace($r, $e1, $e2, $e3)
        }, stringify!($r)));
}

pub struct Ptracer {
    pid: libc::pid_t,
    target: target_desc::Target,
}

impl Ptracer {
    pub fn new(args: &Vec<String>) -> Self {
        let pid: libc::pid_t;
        unsafe {
            pid = libc::fork();
        }
        check_libc(pid, "fork");

        if pid == 0 {
            // Ensure args are null terminated.
            let args: Vec<String> =
                args.iter().map(|a|format!("{}\0", a)).collect();
            let mut args: Vec<*const libc::c_char> =
                args.iter().map(|a|a.as_bytes().as_ptr()
                                as *const libc::c_char).collect();
            args.push(std::ptr::null());
            check_ptrace!(libc::PTRACE_TRACEME, 0, 0, 0);
            unsafe {
                check_libc!(libc::execv(args[0], args.as_mut_ptr()));
                libc::_exit(1);
            }
        }

        let mut ptracer = Ptracer {
            pid: pid,
            target: target_desc::get_target(),
        };

        let status = ptracer.wait();
        if !status.is_stopped() {
            // TODO: Handle error properly.
            println!("Starting a child process ({}) failed", args[0]);
            std::process::exit(-1);
        }

        return ptracer;
    }

    pub fn single_step(&self) {
        check_ptrace!(libc::PTRACE_SINGLESTEP, self.pid, 0, 0);
    }

    pub fn get_regs(&self) -> Registers {
        let buf = vec![0 as u8; self.target.user_size];
        check_ptrace!(libc::PTRACE_GETREGS, self.pid, 0, buf.as_ptr());

        let mut gps = vec![0; self.target.gp_names.len()];
        let gp_ptr = unsafe {
            (buf.as_ptr() as *const u8).offset(self.target.gp_off)
        };
        for i in 0..gps.len() {
            let mut r: u64 = 0xdeadbeef;
            if self.target.gp_size == 8 {
                r = unsafe {
                    *(gp_ptr.offset((self.target.gp_size * i) as isize)
                      as *const u64)
                }
            } else {
                assert!(false);
            }
            gps[i] = r;
        }

        return Registers {
            ip: gps[self.target.ip_index],
            sp: gps[self.target.sp_index],
            bp: gps[self.target.bp_index],
            gps: gps,
        }
    }

    pub fn peek_word(&self, addr: u64) -> u64 {
        return check_ptrace!(libc::PTRACE_PEEKDATA, self.pid, addr, 0) as u64;
    }

    pub fn poke_word(&self, addr: u64, data: u64) {
        check_ptrace!(libc::PTRACE_POKEDATA, self.pid, addr, data) as u64;
    }

    pub fn poke_byte(&self, addr: u64, data: u8) -> u8 {
        assert!(self.target.le);
        let orig = self.peek_word(addr);
        let word = (orig & !0xff) | (data as u64);
        self.poke_word(addr, word);
        return (orig & 0xff) as u8;
    }

    pub fn poke_breakpoint(&self, addr: u64) -> u8 {
        assert_eq!(self.target.breakpoint_size, 1);
        return self.poke_byte(addr, self.target.breakpoint_op as u8);
    }

    pub fn cont(&self) {
        check_ptrace!(libc::PTRACE_CONT, self.pid, 0, 0);
    }

    pub fn wait(&mut self) -> ProcessState {
        let mut status: i32 = -1;
        unsafe {
            check_libc!(libc::wait(&mut status));
        }
        return ProcessState {
            status: status
        };
    }
}
