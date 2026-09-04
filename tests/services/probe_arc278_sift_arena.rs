//! Arc 278 sift-arena, Part B — the two-universe flood-and-sift RED gate, PROCESS-tier.
//!
//! A `:prod::producer` service (its `:messages` carry the producer's OWN log-payload universe —
//! `:prod::Alert`/`:prod::Flow`/`:prod::Query`, arbitrary domain records) floods a shared
//! `journal'` with N=240 Logs cycling 4 shapes. A `:cons::consumer` service — which never
//! `:peers`/`:satisfies` anything Producer-shaped, never defines `:prod::*` — pages the journal
//! via `Journal/sift-logs` with a class-guarded FOREIGN predicate (`read-foreign` +
//! `ForeignRecord/class`/`get`). All four services (mem-store' + journal' + producer' +
//! consumer') run on PROCESS — the guarantee that the consumer can't hold `:prod::*` is a
//! process property (separate registries per fork).
//!
//! arc 255 Stone 1c-g — the consumer's predicate compares a foreign `Value`
//! (`ForeignRecord/get`'s `severity` result, no `Value`→`String` coercion exists) to a string
//! literal via `(:wat::core::= s "high")`. `=`/`not=` are now registered `@Totality Partial`
//! with no `Value` arm, so sift's fence REFUSES this predicate outright: every page of
//! `Journal/sift-logs` returns `::Fatal`/`Fault`, never `::Success`. Before this stone `=`/`not=`
//! were unregistered and the fence never fired, so the fixture counted exactly 60 Alert-high
//! survivors; that outcome is no longer reachable — sift's fence being correct about a predicate
//! that is partial by type and safe only by an argument the type system cannot hold is the
//! consequence being tested now, not a wire-breach or a regression.
//!
//! Run: cargo test --release -p wat sift_arena

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

#[test]
fn sift_arena_foreign_reader_predicate_is_refused_for_comparing_a_value() {
    let world = startup_beside(file!()).expect("startup should succeed");
    let func = world.symbols().get(":user::compute").expect(":user::compute").clone();
    let got = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!()).unwrap_or_else(|e| {
        panic!(
            "sift-arena (flood + foreign-reader sift across a process fork) raised: {e:?}. A \
             dial/timeout means grant-before-dial failed somewhere in the mem-store'/journal'/ \
             producer'/consumer' chain; a type error building `:prod::*` in producer' means the \
             surface-forms carrier did not ship the producer's own :messages types to its child \
             (STOP-1); a crash inside the sieve predicate on a non-Alert row means the `if` did \
             not short-circuit (STOP-2)."
        )
    });
    match got {
        Value::String(message) => assert_eq!(
            message.as_str(), "sift-logs: predicate must be pure, deterministic, and total",
            "the consumer's `.wat` reads this exact text off the `::Fatal`'s `Fault` \
             (`wat::query::Fault/message`, `wat::query::Fatal/reason`) — a different message \
             means sift-logs's own rejection wording moved; got {message:?}"
        ),
        other => panic!(
            "expected `:user::compute` to return the sift fence's refusal message as a \
             `:wat::core::String` (arc 255 Stone 1c-g — `(= s \"high\")` compares a `Value`, \
             which `=` has no arm for and is not registered against, so sift must refuse the \
             predicate rather than return a survivor count); got {other:?}"
        ),
    }
}
