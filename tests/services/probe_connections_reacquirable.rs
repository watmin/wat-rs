//! Connections are re-acquirable — Address is soul, Peer is body.
//!
//! Drives `wat-scripts/scratch-pad/probe-redial-from-durable-addr.wat`, which
//! already answered the three unknowns: `:durable` holds an Address; an arm can
//! re-dial and store the fresh Peer; the re-dialed peer works. Nothing today
//! can break a pipe while the peer lives — proving the path *fires* is the
//! chaos stone. This gates that the path is expressible.

use wat::freeze::startup_from_file;
use wat::runtime::{apply_function, Value};

fn field<'a>(s: &'a str, key: &str) -> &'a str {
    for part in s.split(';') {
        if let Some((k, v)) = part.split_once('=') {
            if k == key {
                return v;
            }
        }
    }
    panic!("missing field {key:?}: {s}");
}

#[test]
fn redial_from_durable_addr_works() {
    let world = startup_from_file("wat-scripts/scratch-pad/probe-redial-from-durable-addr.wat")
        .expect("redial probe should freeze");
    let func = world
        .symbols()
        .get(":user::compute")
        .unwrap_or_else(|| panic!(":user::compute not registered"))
        .clone();
    let out = match apply_function(func, vec![], world.symbols(), wat::rust_caller_span!()) {
        Ok(Value::String(s)) => (*s).clone(),
        Ok(other) => panic!(":user::compute returned non-String: {other:?}"),
        Err(e) => panic!(":user::compute raised: {e:?}"),
    };
    assert_eq!(
        field(&out, "durable-addr"),
        "ok",
        ":durable must accept an Address; got {out}"
    );
    assert_eq!(
        field(&out, "before"),
        "yes",
        "hit through the init-dialed peer must work; got {out}"
    );
    assert_eq!(
        field(&out, "redial"),
        "yes",
        "an arm must be able to re-dial from the durable Address; got {out}"
    );
    assert_eq!(
        field(&out, "after"),
        "yes",
        "the re-dialed Peer must work; got {out}"
    );
}
