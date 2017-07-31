pub struct Flags {
    pub args: Vec<String>,
    pub core: Option<String>,
}

pub fn parse(argv: Vec<String>) -> Flags {
    let mut exec: Option<String> = None;
    let mut core: Option<String> = None;
    let mut args: Vec<String> = vec!();

    let mut i = 1;
    while i < argv.len() {
        let arg = &argv[i];
        if arg == "--args" {
            i += 1;
            while i < args.len() {
                args.push(argv[i].clone());
                i += 1;
            }
        } else if arg.starts_with('-') {
            panic!("Unknown flag: {}", arg);
        } else {
            if exec.is_none() {
                exec = Some(arg.to_string());
            } else if core.is_none() {
                core = Some(arg.to_string());
            } else {
                panic!("Excess command line argument: {}", arg);
            }
        }
        i += 1;
    }

    if args.is_empty() {
        if let Some(exec) = exec {
            args.push(exec);
        }
    }

    Flags {
        args: args,
        core: core,
    }
}
