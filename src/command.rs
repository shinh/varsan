use expr;
use expr::Expr;

#[derive(PartialEq, Debug)]
pub enum Command {
    Break (Expr),
    Cont,
    Info,
    Print (Expr),
    Run (Vec<String>),
    Start (Vec<String>),
    StepI,
    X (usize, i32, Expr),
}

fn parse_print(s: &str) -> Result<Command, String> {
    Ok(Command::Print(try!(expr::parse(s))))
}

fn parse_break(s: &str) -> Result<Command, String> {
    Ok(Command::Break(try!(expr::parse(s))))
}

fn parse_x(_: &str, s: &str) -> Result<Command, String> {
    let num = 1;
    let base = 16;
    // TODO: Parse num and base.
    match expr::parse(s) {
        Ok(e) => Ok(Command::X(num, base, e)),
        Err(e) => Err(e)
    }
}

fn parse_run(s: &str) -> Result<Command, String> {
    Ok(Command::Run(s.split_whitespace().map(|a|a.to_string()).collect()))
}

fn parse_start(s: &str) -> Result<Command, String> {
    Ok(Command::Start(s.split_whitespace().map(|a|a.to_string()).collect()))
}

pub fn parse(line: &str) -> Result<Command, String> {
    let line = line.trim();
    if line.len() == 0 {
        return Err("".to_string());
    }
    let (cmd, rest) = match line.find(' ') {
        Some(found) => (&line[..found], &line[found+1..]),
        None => (line, ""),
    };

    let command_names = [
        "break",
        "continue",
        "info",
        "print",
        "run",
        "si",
        "start",
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
        "break" => parse_break(rest),
        "continue" => Ok(Command::Cont),
        "info" => Ok(Command::Info),
        "print" => parse_print(rest),
        "run" => parse_run(rest),
        "si" | "stepi"  => Ok(Command::StepI),
        "start" => parse_start(rest),
        _ => Err(String::from("Shouldn't happen"))
    }
}

#[test]
fn test_cont() {
    assert_eq!(Ok(Command::Cont), parse("cont"));
}

#[test]
fn test_print() {
    assert_eq!(Ok(Command::Print(Expr::Num(42))), parse("p 42"));
}

#[test]
fn test_break() {
    assert_eq!(Ok(Command::Break(Expr::Ident("main".to_string()))),
               parse("b main"));
}

#[test]
fn test_err() {
    assert_eq!(Err("No such command: xxx".to_string()), parse("xxx"));
}

#[test]
fn test_empty() {
    assert_eq!(Err("".to_string()), parse(" "));
}
