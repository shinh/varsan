use std;

pub enum Expr {
    Empty,
    Num (i64)
}

pub enum Command {
    Break (Expr),
    Cont,
    Info,
    Print (Expr),
    StepI,
}

fn parse_num_from_result(r: Result<i64, std::num::ParseIntError>,
                         e: &str) -> Result<Expr, String> {
    match r {
        Ok(v) => Ok(Expr::Num(v)),
        Err(_) => Err(format!("Invalid number \"{}\".", e))
    }
}

fn parse_expr(s: &str) -> Result<Expr, String> {
    let s = s.trim();
    if s.len() == 0 {
        return Ok(Expr::Empty);
    }

    if s.starts_with("0x") {
        if s.len() == 2 {
            return Err(format!("Invalid number \"{}\".", s));
        }
        return parse_num_from_result(i64::from_str_radix(&s[2..], 16), s);
    } else if s.starts_with("0") {
        return parse_num_from_result(i64::from_str_radix(&s[1..], 8), s);
    } else {
        return parse_num_from_result(i64::from_str_radix(s, 10), s);
    }
}

fn parse_print(s: &str) -> Result<Command, String> {
    match parse_expr(s) {
        Ok(e) => Ok(Command::Print(e)),
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
    ];
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
