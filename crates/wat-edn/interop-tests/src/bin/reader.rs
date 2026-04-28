//! Cross-tool handshake: reverse direction.
//!
//! Reads EDN from stdin (produced by Clojure's pr-str), parses it
//! with wat-edn, and asserts the structure matches what we expect.

use std::io::Read;
use wat_edn::{parse, Value};

fn main() {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();

    println!("─── Clojure-emitted EDN (raw bytes) ───");
    println!("{}", input.trim());
    println!();

    println!("─── parsed by wat-edn ───");
    let v = parse(&input).expect("wat-edn should parse Clojure's output");
    println!("type: {}", v.type_name());

    let (tag, body) = v.as_tagged().expect("outer form should be Tagged");
    println!("outer tag: {}/{}", tag.namespace(), tag.name());
    assert_eq!(tag.namespace(), "enterprise.config");
    assert_eq!(tag.name(), "SizeAdjust");

    let map = body.as_map().expect("body should be Map");
    println!("body entries: {}", map.len());
    assert_eq!(map.len(), 6);

    // Look up specific fields by walking the entries.
    fn find<'a, 'b>(map: &'a [(Value<'b>, Value<'b>)], key: &str) -> Option<&'a Value<'b>> {
        map.iter().find_map(|(k, v)| {
            if let Some(kw) = k.as_keyword() {
                if kw.namespace().is_none() && kw.name() == key {
                    return Some(v);
                }
            }
            None
        })
    }

    let asset = find(map, "asset").unwrap();
    println!("asset:  {} ({})", write_short(asset), asset.type_name());
    assert_eq!(asset.as_keyword().unwrap().name(), "BTC");

    let factor = find(map, "factor").unwrap();
    println!("factor: {}", write_short(factor));
    assert_eq!(factor.as_f64().unwrap(), 1.5);

    let reason = find(map, "reason").unwrap();
    println!("reason: {:?}", reason.as_str().unwrap());

    let issued_at = find(map, "issued-at").unwrap();
    let inst = issued_at.as_inst().unwrap();
    println!("issued-at: {}  (type: {})", inst, issued_at.type_name());

    let ticket = find(map, "ticket").unwrap();
    let uuid = ticket.as_uuid().unwrap();
    println!("ticket: {}  (type: {})", uuid, ticket.type_name());

    let nested = find(map, "nested").unwrap();
    let (ntag, nbody) = nested.as_tagged().unwrap();
    println!(
        "nested: tag={}/{} body={} elements",
        ntag.namespace(),
        ntag.name(),
        nbody.as_vector().unwrap().len()
    );
    assert_eq!(ntag.namespace(), "wat.core");
    assert_eq!(ntag.name(), "Vec<wat.holon.HolonAST>");
    let inner = nbody.as_vector().unwrap();
    assert_eq!(inner.len(), 2);
    for item in inner {
        let (atag, abody) = item.as_tagged().unwrap();
        assert_eq!(atag.namespace(), "wat.holon");
        assert_eq!(atag.name(), "Atom");
        println!("  - #wat.holon/Atom {}", write_short(abody));
    }

    println!();
    println!("✓ wat-edn parsed Clojure-emitted EDN cleanly.");
    println!("✓ Built-in #inst → DateTime<Utc>; #uuid → uuid::Uuid.");
    println!("✓ User tags (enterprise.config/SizeAdjust, wat.core/Vec<...>,");
    println!("             wat.holon/Atom) preserved with full namespace/name.");
}

fn write_short(v: &Value) -> String {
    wat_edn::write(v)
}
