//! FM 2-bis probe for Stone 241.13 — `:wat::core::define-dispatch` HARD CUT.
//!
//! Stone 241.13 retires `:wat::core::define-dispatch` (arc 146's dispatch-by-arity+type
//! entity kind). `:wat::core::defclause` (Stone 237.2 SHIPPED `bdd9eb6c`) is the
//! surviving dispatch entity kind. ALL substrate scaffolding for define-dispatch is
//! DELETED: src/dispatch.rs (445 lines), DispatchRegistry plumbing across check.rs +
//! freeze.rs + runtime.rs + resolve.rs, special_forms.rs entry, freeze.rs walker arms.
//!
//! THE DOCTRINE (per `feedback_hard_cut_admits_no_bypasses`): HARD CUT is total. No
//! "infrastructure stays empty so it's fine" framing. The substrate cannot carry
//! dead-but-live infrastructure for a retired form.
//!
//! Active wat-source callers at HEAD: ZERO (verified — wat/core.wat decls already
//! evacuated to ∀T intrinsics per arc 237.7a/7b/7c). Stone 241.13 makes the
//! retirement enforced.
//!
//! HEAD-disconfirmation map (both contracts FAIL at HEAD):
//! - C01: `:wat::core::define-dispatch` HARD-CUT-rejected at startup
//!        ⇒ FAILS at HEAD (form parses + registers into DispatchRegistry without error)
//! - C02: rejection error carries structured retirement remedy naming `:wat::core::defclause`
//!        ⇒ FAILS at HEAD (no rejection fires; no error to inspect)
//!
//! Post-stone: both contracts PASS.
//!
//! Run: `cargo test --release --test probe_arc241_stone13_define_dispatch_hard_cut`

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

fn try_startup(src: &str) -> Result<(), String> {
    let full = format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        src
    );
    startup_from_source(&full, None, Arc::new(InMemoryLoader::new()))
        .map(|_| ())
        .map_err(|e| format!("{:?}", e))
}

fn try_startup_display(src: &str) -> String {
    let full = format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        src
    );
    match startup_from_source(&full, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => String::from("<startup succeeded — no error to display>"),
        Err(e) => format!("{}", e),
    }
}

// ─── C01: :wat::core::define-dispatch HARD-CUT-rejected at startup ─────────────

#[test]
fn contract_01_define_dispatch_hard_cut_rejected() {
    // A WELL-FORMED define-dispatch decl (the arc 146 mechanism — impl is a
    // keyword-path, NOT a call expression) must be HARD-CUT-rejected at startup
    // post-stone. At HEAD: the form parses + registers into DispatchRegistry
    // without error. Post-stone: rejected with retirement remedy before any
    // dispatch-specific parsing fires.
    //
    // Per `feedback_hard_cut_admits_no_bypasses`: HARD CUT is total. The substrate
    // cannot carry dead-but-live infrastructure for a retired form.
    //
    // Form shape (per arc 146): impl is a bare keyword-path reference to the
    // implementation function; argspec is a list of type-keywords.
    let src = r#"
        (:wat::core::defn :test::desc-i64 [x <- :wat::core::i64] -> :wat::core::String "i64 arm")
        (:wat::core::defn :test::desc-str [x <- :wat::core::String] -> :wat::core::String "str arm")
        (:wat::core::define-dispatch :test::describe
          ((:wat::core::i64) :test::desc-i64)
          ((:wat::core::String) :test::desc-str))
    "#;
    let result = try_startup(src);
    assert!(
        result.is_err(),
        "`:wat::core::define-dispatch` must be HARD-CUT-rejected post-stone; got Ok"
    );
}

// ─── C02: rejection carries structured retirement remedy naming defclause ──────

#[test]
fn contract_02_rejection_remedy_names_defclause() {
    // The HARD-CUT rejection must include structured remedy per Stone 241.10's
    // apparatus, naming `:wat::core::defclause` as the replacement
    // (via 7th RETIREMENT_TABLE entry consumed by remedies_for).
    let src = r#"
        (:wat::core::defn :test::desc-i64 [x <- :wat::core::i64] -> :wat::core::String "i64 arm")
        (:wat::core::define-dispatch :test::describe
          ((:wat::core::i64) :test::desc-i64))
    "#;
    let msg = try_startup_display(src);
    assert!(
        msg.contains(":wat::core::defclause"),
        "retirement remedy must name :wat::core::defclause; got:\n{}",
        msg
    );
    assert!(
        msg.contains("[retirement replacement]"),
        "retirement remedy must carry '[retirement replacement]' annotation per Stone 241.10's apparatus; got:\n{}",
        msg
    );
}
