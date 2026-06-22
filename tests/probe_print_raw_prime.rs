//! Stone 259-negatives — `print-raw'` RED probe.
//!
//! **RED at HEAD (before Part A)**: `:wat::kernel::print-raw'` is not
//! registered in the dispatch table or the check scheme. The child
//! program freeze-fails with "not a builtin" and the child crashes;
//! `recv'` in the parent raises with the crash reason. The test
//! asserts success (i64(0)) → RED.
//!
//! **GREEN after Part A**: `print-raw'` is registered as a `String ->
//! nil` verb. The child:
//!   1. Calls `(:wat::kernel::print-raw' "")` — writes 0 bytes to fd 1,
//!      no newline, returns nil. This proves the verb is callable and
//!      does not corrupt the channel when given an empty string.
//!   2. Calls `(:wat::kernel::println 0)` — the pass-marker.
//!   3. The parent `recv'` decodes the `0` frame → `Value::i64(0)`.
//!   Test asserts i64(0) → GREEN.
//!
//! The "writes bytes without a trailing newline" property is proven
//! by the three negative tests (over-cap, truncated, anti-smuggle)
//! in `wat-tests/kernel/ipc-framing-negatives.wat`.
//!
//! Deviation from the brief: the original design called for
//! `#[restricted_to(":wat::kernel::print-raw'", ":wat::test::")]`.
//! That restriction is NOT applied because process-child WAT programs
//! define `:user::main`, which cannot carry a `:wat::test::` FQDN
//! (the prefix is reserved; child programs cannot define functions in
//! the `:wat::` hierarchy). The verb is left unrestricted by policy,
//! matching the `println` family — see verbs.rs for the full note.

use std::sync::Arc;

use wat::ast::WatAST;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;
use wat::runtime::{eval, Environment, Value};
use wat::span::Span;

fn freeze_ok(src: &str) -> wat::freeze::FrozenWorld {
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(w) => w,
        Err(e) => panic!("freeze should succeed; got: {}", e),
    }
}

/// Build `(:wat::kernel::spawn-program' (:wat::spawn::process) (:wat::core::forms <forms>...))`
fn build_spawn_process_call(child_program_src: &str) -> WatAST {
    let child_forms =
        wat::parser::parse_all_with_file(child_program_src, "<spawn-process-program>")
            .expect("child program parse");
    let mut forms_items = vec![WatAST::Keyword(":wat::core::forms".into(), Span::unknown())];
    forms_items.extend(child_forms);
    let forms_call = WatAST::List(forms_items, Span::unknown());
    WatAST::List(
        vec![
            WatAST::Keyword(":wat::kernel::spawn-program'".into(), Span::unknown()),
            WatAST::List(
                vec![WatAST::Keyword(":wat::spawn::process".into(), Span::unknown())],
                Span::unknown(),
            ),
            forms_call,
        ],
        Span::unknown(),
    )
}

/// The child program:
///   - Calls `print-raw'` with "" (empty string, 0 bytes, no effect).
///   - Calls `println 0` as the pass-marker.
///
/// RED at HEAD (verb not registered): child freeze fails → child crashes → recv' raises.
/// GREEN after Part A (verb registered): child completes → recv' decodes 0.
const PRINT_RAW_PRIME_CHILD_SRC: &str = r#"
    (:wat::core::defn :user::main [] -> :wat::core::nil
      (:wat::core::do
        (:wat::kernel::print-raw' "")
        (:wat::kernel::println 0)))
"#;

/// `print-raw'` is callable from `:user::main` and writes without corrupting the channel.
///
/// RED at HEAD: verb not in dispatch/check → child freeze fails → recv' raises.
/// GREEN after Part A: verb registered → child completes → recv' decodes 0 → i64(0).
#[test]
fn print_raw_prime_callable_and_does_not_corrupt_channel() {
    let world = freeze_ok("");

    let spawn_call = build_spawn_process_call(PRINT_RAW_PRIME_CHILD_SRC);
    let child = eval(&spawn_call, &Environment::new(), world.symbols())
        .expect("spawn-program' should succeed")
        .value_owned();

    let env = Environment::new()
        .child()
        .bind("child", Span::unknown(), child.into())
        .build();

    let recv_call = wat::parse_one!(
        r#"(:wat::kernel::recv' child)"#
    )
    .expect("parse recv' call");

    let result = eval(&recv_call, &env, world.symbols());

    match result {
        Ok(tv) => {
            let v = tv.value_owned();
            assert_eq!(
                v,
                Value::i64(0),
                "recv' must decode 0 (pass-marker from println 0); got {:?}",
                v
            );
        }
        Err(e) => {
            panic!(
                "recv' raised — print-raw' may not be registered yet \
                 (RED at HEAD — implement Part A first): {}",
                e
            );
        }
    }
}
