extern crate colored;
use colored::*;

pub fn info(msg: String) {
    println!("{}", msg.white());
}
