//! Probe (excursus 003, stone C) — **`Lru/put` ON AN UNHASHABLE KEY IS A WAT VALUE; `Lru/get` MISSES.**
//!
//! Before this stone, a well-typed program that used an `Lru` handle as another `Lru`'s key got a
//! Rust `panic!` from `src/rust_deps/cache.rs`, direct AND through a generic `K`:
//!
//! ```text
//!   thread 'main' panicked at src/rust_deps/cache.rs:153:13:
//!   :rust::cache::Lru/put: key must be a hashable value; got :rust::cache::Lru
//!   note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
//! ```
//!
//! (`:165:13` and `Lru/get` for the other verb.) The checker admits an opaque handle as a `K`, so
//! the arc-109 NOTE's defence of the panic — *"the checker already rejects an opaque-typed key at
//! most call sites"* — measured false for both shapes. The builder's ruling (2026-09-23): *"the
//! least amount of panics possible"*. The design is
//! `docs/excursus/2026/09/003-the-little-wat-findings/DESIGN-stone-C-put-get-refuse-as-values.md`.
//!
//! ## The cure these fixtures pin
//!
//! | fixture | verb | answer |
//! |---|---|---|
//! | `put_direct` | `Lru/put`, key type written at the call site | `Err` — a `:wat::cache::Fault` naming `:wat::cache::Lru/put` |
//! | `put_generic` | `Lru/put`, key laundered through `:user::put-any :- [K]` | the same `Err` |
//! | `get_direct` | `Lru/get`, direct | a miss |
//! | `get_generic` | `Lru/get`, through `:user::get-any :- [K]` | a miss |
//!
//! `get` MISSES rather than erring because its return type is already `Option` and a key `put`
//! refuses to store cannot be present — the answer is total. Precedent: `HashMap`'s
//! `contains-key?` answers `false` for an unhashable key (`src/collection/eval.rs`,
//! `hashmap_contains_key_q_inner`).
//!
//! ⭐ Each test pins the WHOLE observable: exit code, exact stdout, and an EMPTY stderr. The empty
//! stderr is the no-panic assertion — a panic writes its banner and `RUST_BACKTRACE` note there,
//! so an exact `""` refuses both without a loose `contains`.
//!
//! ## ⛔ The driver is the binary — the same contract decision as the sibling probes
//!
//! `probe_ex003_silent_failure_pair.rs`'s header states it in full: a the-little-wat finding is a
//! claim about what a user experiences running `wat foo.wat`, and this defect lives at RUN time.
//!
//! ## Mutation (this gate has been driven RED, per verb)
//!
//! With the new signatures left intact, each `panic!` was restored in turn in
//! `src/rust_deps/cache.rs`:
//! - `put`'s guard `panic!`s again → the two `put_*` tests go RED, the two `get_*` stay green.
//! - `get`'s guard `panic!`s again → the two `get_*` tests go RED, the two `put_*` stay green.
//!
//! ## ⚠ What this gate does NOT cover
//!
//! The guard is SHALLOW (`value_is_hashable` reads the key's own variant). A hashable container
//! holding a handle — `(Option.Some <handle>)` as the key — passes it and panics in
//! `impl Hash for Value`'s `unreachable!()` (`src/value/value.rs`). Measured, recorded in
//! `src/rust_deps/cache.rs`'s module doc; not cured here.

use std::path::PathBuf;
use std::process::{Command, Stdio};

fn fixture(case: &str) -> PathBuf {
    let p: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/diagnostics")
        .join(format!(
            "probe_ex003_lru_put_get_refuse_as_values__{case}.wat"
        ));
    assert!(p.exists(), "fixture missing: {}", p.display());
    p
}

/// One invocation of the binary against a fixture. Returns `(exit code, stdout, stderr)`.
fn run(case: &str) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_wat"))
        .arg(fixture(case))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn wat");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

/// The `Err` a `put` fixture prints: `diagnostic | message`. The `diagnostic` column names the
/// verb the USER typed. ⚠ The message's `got :rust::cache::Lru` is the key VALUE's type name —
/// the backing opaque type, which is also how the checker spells it — not the shim's verb.
const PUT_REFUSAL: &str =
    "\":wat::cache::Lru/put | key must be a hashable value; got :rust::cache::Lru\"\n";

const MISS: &str = "\"MISS\"\n";

#[test]
fn put_on_an_opaque_key_direct_is_an_err_naming_the_user_facing_verb() {
    assert_eq!(
        run("put_direct"),
        (0, PUT_REFUSAL.to_string(), String::new()),
        "Lru/put on an opaque key must be an Err value naming :wat::cache::Lru/put, \
         with an exit of 0 and nothing on stderr (no panic banner, no RUST_BACKTRACE note)"
    );
}

#[test]
fn put_on_an_opaque_key_through_a_generic_k_is_the_same_err() {
    assert_eq!(
        run("put_generic"),
        (0, PUT_REFUSAL.to_string(), String::new()),
        "Lru/put through a generic K must refuse exactly as the direct call does"
    );
}

#[test]
fn get_on_an_opaque_key_direct_is_a_miss() {
    assert_eq!(
        run("get_direct"),
        (0, MISS.to_string(), String::new()),
        "Lru/get on an opaque key must miss — it cannot have been stored — with nothing on stderr"
    );
}

#[test]
fn get_on_an_opaque_key_through_a_generic_k_is_a_miss() {
    assert_eq!(
        run("get_generic"),
        (0, MISS.to_string(), String::new()),
        "Lru/get through a generic K must miss exactly as the direct call does"
    );
}
