use expr;
use expr::Expr;

pub enum Command {
    Break (Expr),
    Cont,
    Info,
    Print (Expr),
    StepI,
    X (usize, i32, Expr),
}

fn parse_print(s: &str) -> Result<Command, String> {
    match expr::parse(s) {
        Ok(e) => Ok(Command::Print(e)),
        Err(e) => Err(e)
    }
}

fn parse_x(cmd: &str, s: &str) -> Result<Command, String> {
    let mut num = 1;
    let mut base = 16;
    // TODO: Parse num and base.
    match expr::parse(s) {
        Ok(e) => Ok(Command::X(num, base, e)),
        Err(e) => Err(e)
    }
}

pub fn parse(line: &str) -> Result<Command, String> {
    let line = line.trim();
    let found = line.find(' ');
    if found.is_none() {
        return Err(String::from(""));
    }
    let cmd = &line[..found.unwrap()];
    let rest = &line[found.unwrap() + 1..];

    let command_names = [
        "break",
        "continue",
        "info",
        "print",
        "si",
        "stepi",
        "x",
    ];

    if cmd.starts_with("x") && cmd.len() == 1 || cmd.starts_with("x/") {
        return parse_x(cmd, rest);
    }

    let mut cands = Vec::new();
    for name in command_names.iter() {
        if name.starts_with(cmd) {
            cands.push(name);
        }
    }
    if cands.len() == 0 {
        return Err(format!("No such command: {}", cmd));
    }
    if cands.len() > 1 {
        return Err(format!("Multiple candidates for `{}': {:?}", cmd, cands));
    }

    match *cands[0] {
        "break" => Ok(Command::Break(Expr::Num(42))),
        "continue" => Ok(Command::Cont),
        "info" => Ok(Command::Info),
        "print" => parse_print(rest),
        "si" | "stepi"  => Ok(Command::StepI),
        _ => Err(String::from("Shouldn't happen"))
    }
}
