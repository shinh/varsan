#[macro_use]

mod libc_utils;
mod ptracer;
mod target_desc;

fn main() {
    let args: Vec<_> = std::env::args().collect();

    let target = target_desc::get_target();

    let mut ptracer = ptracer::Ptracer::new(
        args[1..args.len()].iter().collect());
    while true {
        let regs = ptracer.get_regs(&target);
        println!("ip={}", regs.ip());

        ptracer.single_step();
        let status = ptracer.wait();
        if !status.is_stopped() {
            break;
        }
    }
}
