#[macro_use]

mod libc_utils;
mod ptracer;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let mut ptracer = ptracer::Ptracer::new(
        args[1..args.len()].iter().collect());
    while true {
        ptracer.single_step();
        let status = ptracer.wait();
        if !status.is_stopped() {
            break;
        }
    }
}
