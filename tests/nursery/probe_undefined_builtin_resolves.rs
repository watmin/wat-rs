//! FM-2-bis disconfirming probe: an undefined call-head under a RESERVED prefix
//! must be caught at CHECK/RESOLVE time — never reach runtime as "unknown function".
//!
//! THE GAP (resolve.rs `is_resolvable_call_head`): `if is_reserved_prefix(head)
//! { return true }` blanket-accepts ANY leaf under `:wat::core::`/`:wat::kernel::`
//! /etc. — the namespace is validated, the leaf is not. The comment hands
//! leaf-validation to "the type checker's concern," but the checker doesn't catch
//! builtin leaves either, so a wrong leaf (`+'2`, `Bogus`) falls through both
//! gates and dies at RUNTIME as "unknown function" — exactly the bug that, behind
//! an `(Err _)` swallow in a spawned thread, cost a 30-minute crawl in wat-lru.
//!
//! This is the `make-*-queue` phantom class generalized: the set of call-heads the
//! front-end ACCEPTS (any reserved-prefix leaf) is strictly larger than the set
//! the runtime can DISPATCH. Every name in the gap is a phantom.
//!
//! RED AT HEAD: `(:wat::core::i64::+'2 1 2)` freezes CLEAN (startup Ok) — the error
//! is deferred to runtime. GREEN AFTER: resolve checks leaf membership against the
//! single dispatchable-builtin source of truth and rejects it at check time.
//!
//! Run un-ignored to confirm RED, then sonnet un-ignores after the fix lands.

use std::sync::Arc;
use wat::freeze::{startup_from_source, StartupError};
use wat::load::InMemoryLoader;

fn check_result(src: &str) -> Result<(), String> {
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => Ok(()),
        Err(StartupError::Check(errs)) => Err(format!("{}", errs)),
        Err(other) => Err(format!("{}", other)),
    }
}

// A renamed-away operator (`+'2` → `+`): a wrong leaf under :wat::core::i64::.
const WRONG_OPERATOR_LEAF: &str = r#"
(:wat::core::defn :user::main [] -> :wat::core::i64
  (:wat::core::i64::+'2 1 2))
"#;

// A bogus leaf under a real namespace (the comment's own example shape).
const BOGUS_LEAF: &str = r#"
(:wat::core::defn :user::main [] -> :wat::core::i64
  (:wat::core::Bogus 1 2))
"#;

// Control: the real, dispatchable operator MUST keep resolving.
const VALID_OPERATOR: &str = r#"
(:wat::core::defn :user::main [] -> :wat::core::i64
  (:wat::core::i64::+ 1 2))
"#;

// RED at HEAD: freezes clean today (deferred to runtime); GREEN after the fix.
#[test]
fn wrong_operator_leaf_is_a_check_error() {
    let result = check_result(WRONG_OPERATOR_LEAF);
    assert!(
        result.is_err(),
        "(:wat::core::i64::+'2 ...) — a renamed-away operator leaf — must be caught \
         at check/resolve time, not deferred to a runtime 'unknown function'. It \
         froze CLEAN (the reserved-prefix blanket-accept gap)."
    );
}

// RED at HEAD.
#[test]
fn bogus_leaf_under_known_namespace_is_a_check_error() {
    let result = check_result(BOGUS_LEAF);
    assert!(
        result.is_err(),
        "(:wat::core::Bogus ...) — a wrong leaf under a real namespace — must be a \
         check/resolve error, not a runtime surprise."
    );
}

// Control — must NOT over-reject: the real operator keeps resolving.
#[test]
fn valid_operator_still_resolves() {
    let result = check_result(VALID_OPERATOR);
    assert!(
        result.is_ok(),
        "(:wat::core::i64::+ ...) is a real dispatchable builtin and must keep \
         type-checking; got: {:?}",
        result
    );
}
