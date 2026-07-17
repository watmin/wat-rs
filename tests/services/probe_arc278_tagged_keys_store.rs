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
    let scanned = call_str(&world, ":user::scan-order");

    // Put was out-of-order (late, early, mid). assert_edn_eq! parses both sides as EDN and compares
    // the ORDERED vectors — proving chronological order (early < mid < late) AND that a
    // constant-width #inst sk sorts a second-boundary instant correctly against a sub-second one.
    wat::assert_edn_eq!(
        scanned,
        include_str!("probe_arc278_tagged_keys_store__scan_order.edn"),
        "constant-width #inst sk sorts chronologically"
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
