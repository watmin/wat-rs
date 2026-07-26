//! Arc 260.1a — the DECLARE side: a `defn` kwargs section `& {fields}` mints a typed record and the
//! fn takes/destructures it. The foundation (no call-site sugar yet — the explicit-record call form).
//!
//! `(:user::connect [host <- :String & [port <- :i64 tls <- :bool]] …)` → `defn` mints
//! `:user::connect::Kwargs` (the rs-1 mint). The kwargs section is an ARGSPEC nested once (same
//! binder-triple syntax as the main params, reusing the existing parse; a nested `&` inside it is
//! disallowed — flat, one level). `defn` reshapes the fn so its last param is that record, and
//! destructures the fields into the body scope (clojure `& {:keys}` binds `port`/`tls`). The body uses
//! `port` + `tls` by name. The call constructs the record explicitly and passes it.
//!
//! Wat source lives in the co-located fixture: probe_arc260_decl_kwargs_minted_record.wat
//! (slurped via startup_beside(file!())).
//!
//! Run: cargo test --release -p wat --test probe_arc260_decl_kwargs_minted_record -- --include-ignored

use wat::freeze::call_beside_value;
use wat::runtime::Value;

// just-eval (rubric): the probe is a zero-arg entry fn in the co-located fixture, driven via
// call_beside_value — no inline wat driver expression.
#[test]
fn decl_kwargs_mints_record_and_destructures() {
    let got = call_beside_value(file!(), ":user::compute")
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(444)),
        "expected 444: & {{port tls}} minted :user::connect::Kwargs; the explicit-record call passed \
         (Kwargs 443 true); the body read port + tls by name (destructured) → 443 + 1; got {got:?}"
    );
}
