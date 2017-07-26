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
    gps: Vec<i64>,
    ip: i64,
    sp: i64,
    bp: i64,
}

impl Registers {
    pub fn ip(&self) -> i64 { self.ip }
    pub fn sp(&self) -> i64 { self.sp }
    pub fn bp(&self) -> i64 { self.bp }
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
        }, stringify!($e)));
}

pub struct Ptracer {
    pid: libc::pid_t
}

impl Ptracer {
    pub fn new(args: Vec<&String>) -> Self {
        let pid: libc::pid_t;
        unsafe {
            pid = libc::fork();
        }
        check_libc(pid, "fork");

        if pid == 0 {
            let mut args: Vec<*const libc::c_char> =
                args.iter().map(|a|a.as_bytes().as_ptr()
                                     as *const libc::c_char).collect();
            unsafe {
                check_libc!(libc::ptrace(libc::PTRACE_TRACEME, 0, 0, 0) as i32);
                check_libc!(libc::execv(args[0], args.as_mut_ptr()));
                libc::_exit(1);
            }
        }

        let mut ptracer = Ptracer {
            pid: pid
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

    pub fn get_regs(&self, target: &target_desc::Target) -> Registers {
        let buf = vec![0 as u8; target.user_size];
        check_ptrace!(libc::PTRACE_GETREGS, self.pid, 0, buf.as_ptr());

        let mut gps = vec![0; target.gp_names.len()];
        let gp_ptr = unsafe {
            (buf.as_ptr() as *const u8).offset(target.gp_off)
        };
        for i in 0..gps.len() {
            let mut r: i64 = 0xdeadbeef;
            if target.gp_size == 8 {
                r = unsafe {
                    *(gp_ptr.offset((target.gp_size * i) as isize)
                      as *const i64)
                }
            } else {
                assert!(false);
            }
            gps[i] = r;
        }

        return Registers {
            ip: gps[target.ip_index],
            sp: gps[target.sp_index],
            bp: gps[target.bp_index],
            gps: gps,
        }
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
