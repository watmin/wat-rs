//! Reads schema-validated EDN from a Clojure consumer that loaded
//! shared.wat. Confirms wat-edn parses the typed output cleanly.

use std::io::Read;
use wat_edn::{parse, Value};

fn main() {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    println!("─── Clojure-emitted EDN ───");
    println!("{}", input.trim());

    let v = parse(&input).expect("wat-edn parse");
    let (tag, body) = v.as_tagged().expect("Tagged");
    assert_eq!(tag.namespace(), "enterprise.config");
    assert_eq!(tag.name(), "SizeAdjust");

    let map = body.as_map().expect("Map");
    let asset = map
        .iter()
        .find_map(|(k, v)| {
            k.as_keyword()
                .filter(|kw| kw.namespace().is_none() && kw.name() == "asset")
                .map(|_| v)
        })
        .unwrap();
    assert_eq!(asset.as_keyword().unwrap().name(), "BTC");

    let factor = map
        .iter()
        .find_map(|(k, v)| {
            k.as_keyword()
                .filter(|kw| kw.name() == "factor")
                .map(|_| v)
        })
        .unwrap();
    assert_eq!(factor.as_f64().unwrap(), 1.5);

    let reason = map
        .iter()
        .find_map(|(k, v)| {
            k.as_keyword()
                .filter(|kw| kw.name() == "reason")
                .map(|_| v)
        })
        .unwrap();
    assert!(reason.as_str().unwrap().contains("drawdown"));

    println!();
    println!("─── parsed by wat-edn ───");
    println!("tag:    {}/{}", tag.namespace(), tag.name());
    println!("asset:  {}", asset.as_keyword().unwrap());
    println!("factor: {}", factor.as_f64().unwrap());
    println!("reason: {:?}", reason.as_str().unwrap());
    println!();
    println!("✓ Schema-driven Clojure → wat-edn pipeline works.");
    println!("✓ shared.wat is the SINGLE SOURCE OF TRUTH for both sides.");
}
