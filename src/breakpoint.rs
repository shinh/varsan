use ptracer;

pub struct Breakpoint {
    id: i32,
    addr: u64,
    token: u8,
    is_active: bool,
}

impl Breakpoint {
    pub fn id(&self) -> i32 { self.id }
    pub fn addr(&self) -> u64 { self.addr }
}

pub struct BreakpointManager {
    breakpoints: Vec<Breakpoint>,
    next_id: i32,
}

impl BreakpointManager {
    pub fn new() -> Self {
        Self {
            breakpoints: vec!(),
            next_id: 0,
        }
    }

    pub fn notify_start(&mut self, ptracer: &ptracer::Ptracer) {
        for bp in &mut self.breakpoints {
            assert!(!bp.is_active);
            bp.token = ptracer.poke_breakpoint(bp.addr);
            bp.is_active = true;
        }
    }

    pub fn notify_finish(&mut self) {
        for bp in &mut self.breakpoints {
            assert!(bp.is_active);
            bp.token = 0;
            bp.is_active = false;
        }
    }

    pub fn add(&mut self, addr: u64, by_user: bool,
               ptracer: Option<&ptracer::Ptracer>) -> &Breakpoint {
        let id = if by_user {
            self.next_id += 1;
            self.next_id
        } else {
            0
        };
        match ptracer {
            Some(ptracer) => {
                let token = ptracer.poke_breakpoint(addr);
                let bp = Breakpoint {
                    id: id,
                    addr: addr,
                    token: token,
                    is_active: true,
                };
                self.breakpoints.push(bp);
                return &self.breakpoints[self.breakpoints.len()-1];
            }
            None => {
                let bp = Breakpoint {
                    id: id,
                    addr: addr,
                    token: 0,
                    is_active: false,
                };
                self.breakpoints.push(bp);
                return &self.breakpoints[self.breakpoints.len()-1];
            }
        }
    }

    pub fn find_by_addr(&self, addr: u64) -> Option<&Breakpoint> {
        for bp in &self.breakpoints {
            if bp.addr == addr {
                return Some(bp);
            }
        }
        return None;
    }
}
