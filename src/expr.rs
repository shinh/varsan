use std;

#[derive(PartialEq, Debug)]
pub enum Expr {
    Empty,
    Num (i64),
    Ident (String),
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
    } else if s.chars().nth(0).unwrap().is_digit(10) {
        return parse_num_from_result(i64::from_str_radix(s, 10), s);
    } else {
        return Ok(Expr::Ident(String::from(s)));
    }
}

#[test]
fn test_num() {
    assert_eq!(Ok(Expr::Num(42)), parse("42"));
    assert_eq!(Ok(Expr::Num(0xcc)), parse("0xcc"));
    assert_eq!(Ok(Expr::Num(493)), parse("0755"));
}

#[test]
fn test_ident() {
    assert_eq!(Ok(Expr::Ident("foo".to_string())), parse("foo"));
}
