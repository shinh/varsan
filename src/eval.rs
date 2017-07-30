use expr::Expr;

pub fn eval(e: Expr) -> i64 {
    match e {
        // TODO
        Expr::Empty => 0,
        Expr::Num(v) => v,
    }
}
