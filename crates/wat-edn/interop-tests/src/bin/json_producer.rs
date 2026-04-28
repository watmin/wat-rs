//! Reads EDN from stdin via wat-edn, converts to JSON, writes to
//! stdout. Pipes into clojure cheshire for cross-language JSON
//! verification.

use std::io::Read;
use wat_edn::{parse, to_json_string};

fn main() {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    let v = parse(&input).expect("wat-edn parse").into_owned();
    println!("{}", to_json_string(&v));
}
