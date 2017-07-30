use std;

pub enum Expr {
    Empty,
    Num (i64)
}

fn parse_num_from_result(r: Result<i64, std::num::ParseIntError>,
                         e: &str) -> Result<Expr, String> {
    match r {
        Ok(v) => Ok(Expr::Num(v)),
        Err(_) => Err(format!("Invalid number \"{}\".", e))
    }
}

pub fn parse(s: &str) -> Result<Expr, String> {
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

