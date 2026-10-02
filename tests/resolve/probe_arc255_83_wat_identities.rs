//! 255.83 amend 2 — identity heads on the wat side.
//!
//! Each listed compare goes through `:wat::core::canonical-identity`. The
//! keyword spelling, the symbol spelling, and the dotted keyword of that same
//! identity agree. Markers (`->`, `:from`) stay text. The programs live in the
//! co-located fixture.

use wat::freeze::{startup_from_file, FrozenWorld};
use wat::runtime::{apply_function, Value};

fn world() -> FrozenWorld {
    startup_from_file("tests/resolve/probe_arc255_83_wat_identities.wat")
        .unwrap_or_else(|e| panic!("identities define: {e:?}"))
}

fn call_string(world: &FrozenWorld, name: &str, args: Vec<Value>) -> String {
    let func = world.symbols().get(name).unwrap_or_else(|| panic!("missing {name}")).clone();
    match apply_function(func, args, world.symbols(), wat::rust_caller_span!()) {
        Ok(Value::String(s)) => s.to_string(),
        Ok(other) => panic!("{name}: expected a string, got {other:?}"),
        Err(e) => panic!("{name}: {e:?}"),
    }
}

fn call_i64(world: &FrozenWorld, name: &str) -> i64 {
    let func = world.symbols().get(name).unwrap_or_else(|| panic!("missing {name}")).clone();
    match apply_function(func, vec![], world.symbols(), wat::rust_caller_span!()) {
        Ok(Value::i64(n)) => n,
        Ok(other) => panic!("{name}: expected i64, got {other:?}"),
        Err(e) => panic!("{name}: {e:?}"),
    }
}

#[test]
fn spelling_25583_wat_identities_agree() {
    let world = world();
    assert_eq!(
        call_string(&world, ":user::probe", vec![]),
        "E1e1e.1A1a1a.1X0P1p1p.1U1u1F1f1f.1"
    );
    assert_eq!(call_string(&world, ":user::kw-metric", vec![]), "ns");
    assert_eq!(call_string(&world, ":user::sym-metric", vec![]), "ns");
    assert_eq!(call_string(&world, ":user::hkw-metric", vec![]), "ns");
    assert_eq!(call_string(&world, ":user::hsym-metric", vec![]), "ns");
    assert_eq!(call_i64(&world, ":user::kw-agg"), 1);
    assert_eq!(call_i64(&world, ":user::sym-agg"), 1);
    assert_eq!(call_i64(&world, ":user::dot-agg"), 1);
    let kw = call_string(&world, ":user::consumes-kw", vec![]);
    let sym = call_string(&world, ":user::consumes-sym", vec![]);
    assert_eq!(kw, sym);
    assert_eq!(kw, "user::T");
}
