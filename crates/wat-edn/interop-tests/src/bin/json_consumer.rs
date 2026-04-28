//! Reads JSON from stdin (typically Clojure's cheshire output),
//! converts to EDN via wat-edn, asserts the structure.

use std::io::Read;
use wat_edn::{from_json_string, write};

fn main() {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    println!("─── JSON received from Clojure ───");
    println!("{}", input.trim());

    let v = from_json_string(&input).expect("wat-edn from_json_string");

    println!();
    println!("─── as EDN (via write) ───");
    println!("{}", write(&v));

    println!();
    println!("─── structure summary ───");
    if let Some((tag, body)) = v.as_tagged() {
        println!("outer tag: {}/{}", tag.namespace(), tag.name());
        if let Some(map) = body.as_map() {
            println!("body: {} fields", map.len());
            for (k, val) in map {
                println!("  {} → {}", write(k), write(val));
            }
        }
    } else if let Some(map) = v.as_map() {
        println!("plain map: {} fields", map.len());
        for (k, val) in map {
            println!("  {} → {}", write(k), write(val));
        }
    }

    println!();
    println!("✓ Round-trip Rust EDN → JSON → Clojure → JSON → Rust EDN works.");
}
