//! Arc 209 Stone C.1 — `defservice` emits the op enum (the skeleton's first generated form).
//!
//! Stone C mints `:wat::service::defservice` (a PURE-WAT defmacro in `wat/service.wat`) that
//! generates a complete service — the op enum + the `poll'` dispatch loop + the client wrappers
//! — from a flat `:state` + `:ops` surface. **C.1 is the first sub-stone: the macro skeleton +
//! the OP ENUM only.** C.2 adds the dispatch loop; C.3 the client wrappers + start fn.
//!
//! THE SURFACE (settled 2026-06-13, four-questions → option A): each op is a self-contained
//! List `(OpName [s <- :State ...client-args] -> RetType body)` — bodies INLINE, the whole
//! service in one form. The macro walks `:ops` WatAST-native (`ast->children` per op-List), and
//! for the op enum it takes OpName + the arg-vec MINUS the leading `s <- :State` self-arg →
//! the variant's client-arg fields. (The `->`/RetType/body are consumed by C.2, not C.1.)
//!
//! C.1's ONLY claim: `(defservice :my::counter :state :i64 :ops [...])` emits an op enum
//! `:my::counter::Op` with one variant per op, each carrying the op's CLIENT args:
//!   - `Get` handler args = `[s <- :State]` → only the self-arg → a BARE (fieldless) variant.
//!   - `Increment` handler args = `[s <- :State n <- :i64]` → drop `s` → variant field `n <- :i64`.
//!
//! THE PROOF: construct `(:my::counter::Op::Increment 5)` and `match` it — the bare `:Get`
//! arm compiles (proving the fieldless variant exists), the `:Increment` arm extracts `n == 5`
//! (proving the client arg became a variant field, and the `s <- :State` self-arg was dropped).
//!
//! RED at HEAD: `:wat::service::defservice` is an unknown macro — `startup_from_source` fails
//! to expand the top-level form, so the world never builds (and `:my::counter::Op::Increment`
//! never resolves). Deterministically GREEN once C.1 ships the defservice skeleton + op enum.
//!
//! Run: cargo test --release -p wat --test probe_arc209_c1_defservice_op_enum

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// The counter as ONE defservice (surface A — inline bodies). C.1 reads the op surface and
// emits the op enum; the bodies/ret are inert until C.2 builds the loop. `probe-op` exercises
// ONLY the generated enum: construct the payload variant, match both arms, return `n`.
const PROGRAM: &str = r#"
(:wat::service::defservice :my::counter
  :state :wat::core::i64
  :ops
  [(:Get [s <- :State]
         -> (:wat::core::Tuple :State :wat::core::i64)
     (:wat::core::Tuple s s))

   (:Increment [s <- :State n <- :wat::core::i64]
               -> (:wat::core::Tuple :State :wat::core::i64)
     (:wat::core::let [s' (:wat::core::i64::+ s n)]
       (:wat::core::Tuple s' s')))])

;; Exercise the GENERATED op enum: build :Increment with a client arg, match both variants.
;; The bare `:Get` arm compiles only if Get is a real fieldless variant; the `:Increment`
;; arm extracts the client field `n`.
(:wat::core::defn :user::probe-op [] -> :wat::core::i64
  (:wat::core::let [op (:my::counter::Op::Increment 5)]
    (:wat::core::match op -> :wat::core::i64
      (:my::counter::Op::Get 0)
      ((:my::counter::Op::Increment n) n))))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
fn defservice_emits_op_enum_with_client_arg_fields() {
    let world = startup_from_source(PROGRAM, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed (Stone C.1: defservice emits the op enum)");
    let ast = wat::parse_one!("(:user::probe-op)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("probe-op raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(5)),
        "expected 5: defservice :my::counter must emit `:my::counter::Op` with a bare \
         (fieldless) `:Get` variant and an `:Increment` variant carrying the client arg \
         `n <- :i64` (the leading `s <- :State` self-arg dropped); constructing \
         `(:my::counter::Op::Increment 5)` + matching extracts n=5; got {got:?}"
    );
}
