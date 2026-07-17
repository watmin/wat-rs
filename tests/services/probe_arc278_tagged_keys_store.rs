//! arc 278 T1b.2 key-design gate — the `:wat::query::Store` round-trips + indexes correctly with
//! the TAGGED-EDN key shapes journal' will produce, isolated from journal':
//!   pk  = #wat.telemetry'/PartitionKey {:namespace … :kind …}
//!   sk  = #inst "<constant-width iso8601-nanos>"   (:wat::time::to-iso8601 … 9)
//!   gsi = #uuid "8-4-4-4-12"
//!
//! The load-bearing property is sk sort-safety: a constant-width #inst sorts lexicographically =
//! chronologically, so mem-store''s `sort-by Row/sk` returns time order — even for a second-boundary
//! instant (the case the generic EDN writer's variable-width AutoSi render would sort WRONG).

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

fn call_str(world: &wat::freeze::FrozenWorld, name: &str) -> String {
    let func = world
        .symbols()
        .get(name)
        .unwrap_or_else(|| panic!("{name} not registered"))
        .clone();
    match apply_function(func, vec![], world.symbols(), wat::rust_caller_span!()) {
        Ok(Value::String(s)) => (*s).clone(),
        Ok(other) => panic!("{name} returned non-String: {other:?}"),
        Err(e) => panic!("{name} raised: {e:?}"),
    }
}

fn call_i64(world: &wat::freeze::FrozenWorld, name: &str) -> i64 {
    let func = world
        .symbols()
        .get(name)
        .unwrap_or_else(|| panic!("{name} not registered"))
        .clone();
    match apply_function(func, vec![], world.symbols(), wat::rust_caller_span!()) {
        Ok(Value::i64(n)) => n,
        Ok(other) => panic!("{name} returned non-i64: {other:?}"),
        Err(e) => panic!("{name} raised: {e:?}"),
    }
}

#[test]
fn constant_width_inst_sk_sorts_chronologically() {
    let world = startup_beside(file!())
        .expect("startup should succeed (telemetry PartitionKey/Kind baked; mem-store' baked)");
    let joined = call_str(&world, ":user::scan-order");

    // The three iso8601 timestamps in CHRONOLOGICAL order: 01.0 < 01.000000001 < 02.0. Put was
    // out-of-order (late, early, mid); a sort-safe sk brings them back in this order.
    let early = "1970-01-01T00:00:01.000000000Z";
    let mid = "1970-01-01T00:00:01.000000001Z";
    let late = "1970-01-01T00:00:02.000000000Z";
    let (pe, pm, pl) = (
        joined.find(early),
        joined.find(mid),
        joined.find(late),
    );
    assert!(
        pe.is_some() && pm.is_some() && pl.is_some(),
        "all three #inst sks should round-trip through scan; got: {joined}"
    );
    assert!(
        pe < pm && pm < pl,
        "scan must return #inst sks in chronological order (early < mid < late) — a second-boundary \
         instant must NOT sort after a sub-second one; got: {joined}"
    );
    // And the stored form is the tagged #inst, round-trippable back to an Instant.
    assert!(
        joined.contains("#inst \""),
        "sk should be stored as a tagged #inst form; got: {joined}"
    );
}

#[test]
fn uuid_gsi_scan_index_round_trips() {
    let world = startup_beside(file!())
        .expect("startup should succeed (telemetry PartitionKey/Kind baked; mem-store' baked)");
    let n = call_i64(&world, ":user::index-count");
    assert_eq!(
        n, 2,
        "scan-index by the #uuid GSI u1 must return exactly the 2 rows projecting it (u2's row \
         excluded); got {n}"
    );
}
