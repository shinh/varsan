use context;
use expr::Expr;

pub fn eval(ctx: &context::Context, e: Expr) -> u64 {
    match e {
        // TODO
        Expr::Empty => 0,
        Expr::Num(v) => v as u64,
        Expr::Ident(v) => {
            match ctx.resolve(&v) {
                Some(v) => v,
                None => 0,
            }
        }
    }
}
