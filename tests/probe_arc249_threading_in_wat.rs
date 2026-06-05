//! FM-2-bis PROBE-LED diagnostic for Arc 249 Stone 249.3 — can the total-pure
//! macro engine (249.2b) express threading `->`/`->>` as WAT CODE, replacing the
//! Rust `thread_desugar` (src/macros/expand.rs:215)?
//!
//! THE QUESTION (probe-led, NOT conviction-led — per REALIZATIONS §"the
//! practitioner is the failure domain"): a prior self GROUNDED, by reading
//! src/runtime.rs:10414, that eval-time `walk_quasiquote` lacks `~@`-splice, and
//! that no `Vec<WatAST> → List`-form constructor exists. That is a from-INSIDE
//! verdict. This probe replaces the verdict with ground truth: it ATTEMPTS the
//! natural Clojure-faithful encoding and lets the substrate name the real gap.
//!
//! Threading is HARDER than the engine's gate-D fold (probe_arc249_macro_engine
//! `mint_program_body_fold`): gate D injects two scalars into a FIXED 2-arg
//! template each iteration (`` `(i64::+ ~acc ~n) ``). Threading must inject `acc`
//! into an EXISTING, VARIABLE-ARITY step form — `(f a b …)` → `(f a b … acc)`.
//! That is precisely where splice / decomposition is needed.
//!
//! ENCODING UNDER TEST (thread-last): a variadic macro whose body folds over the
//! steps, seeding from the first part, splicing each step's children and
//! appending the accumulator:
//!
//!   (defmacro :test::thread-last [& parts] -> :AST<…>
//!     (foldl (fn [a step] `(~@step ~a))
//!            (first parts)
//!            (rest parts)))
//!
//! ROW STATUS (all `#[ignore]`'d — this is a diagnostic, run explicitly):
//!   - A: thread-last single step. If `~@`-splice works → GREEN (my conviction
//!        is WRONG, threading is pure wat, no substrate change). If it errors →
//!        the error NAMES the gap (splice unimplemented / refused / type
//!        mismatch), earning the right to the 249.3 substrate design.
//!   - B: thread-last two-step pipeline (variable injection across steps).
//!
//! Run the diagnostic:
//!   cargo test --release --test probe_arc249_threading_in_wat -- --ignored --nocapture
//!
//! This file is DESIGN SUBSTRATE for Stone 249.3, not a contract. The contract
//! is tests/probe_arc249_threading.rs (the 5 threading mints). When 249.3 ships,
//! THIS probe is deleted or folded into the contract; it exists to reveal the gap.

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

/// True if startup (parse + macro-expand + check) ACCEPTS the program.
fn startup_ok(src: &str) -> bool {
    startup_from_source(&with_nil_main(src), None, Arc::new(InMemoryLoader::new())).is_ok()
}

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        src
    )
}

/// Eval a bool-returning `:user::compute` with body `body`, after sibling `decls`.
/// Returns the Value or a stringified error — at HEAD the threading-macro body is
/// expected to error somewhere (startup / expand / eval), and `.unwrap_err()` is
/// the diagnostic we read.
fn eval_bool_with(decls: &str, body: &str) -> Result<Value, String> {
    let src = format!("{decls}\n(:wat::core::defn :user::compute [] -> :wat::core::bool {body})");
    let full = with_nil_main(&src);
    let world = startup_from_source(&full, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup: {:?}", e))?;
    let ast = wat::parse_one!("(:user::compute)").map_err(|e| format!("parse: {:?}", e))?;
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env)
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {:?}", e))
}

const INC: &str =
    "(:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::+ x 1))";
const GT2: &str =
    "(:wat::core::fn [x <- :wat::core::i64] -> :wat::core::bool (:wat::core::i64::> x 2))";

/// The thread-last macro under test — the faithful Clojure shape: a FIXED first
/// param `acc` + variadic `& steps`, folding `acc` through each step via a
/// `~@`-splice of the step's children plus a `~acc` tail.
///
/// PEEL 1 (from diag run 1): all-variadic `[& parts]` + `(first parts)` fed an
/// `Option<WatAST>` (first is a projective Vector<T>→Option<T> intrinsic) into
/// the accumulator and the `~acc` unquote rejected the Option. Mixed `[acc &
/// steps]` binds `acc` directly as a WatAST and sidesteps the Option — and tests
/// whether the macro param parser supports fixed+variadic.
const THREAD_LAST_MACRO: &str = "(:wat::core::defmacro :test::thread-last \
     [acc <- :wat::holon::HolonAST & steps <- :AST<wat::holon::Holons>] \
     -> :AST<wat::holon::HolonAST> \
     (:wat::core::foldl \
        (:wat::core::fn [a <- :wat::holon::HolonAST step <- :wat::holon::HolonAST] \
           -> :wat::holon::HolonAST `(~@step ~a)) \
        acc \
        steps))";

// ═══════════════════════════════════════════════════════════════════════════
// A — thread-last, single step. `(:test::thread-last [1 2 3] (map INC))` should
// expand to `(map INC [1 2 3])` → [2 3 4]. Tests `~@`-splice of one list step.
// ═══════════════════════════════════════════════════════════════════════════
#[test]
#[ignore = "249.3 diagnostic — run with --ignored to read the gap"]
fn diag_thread_last_single_step() {
    let decls = THREAD_LAST_MACRO;
    let body = format!(
        "(:wat::core::= (:test::thread-last [1 2 3] (:wat::core::map {INC})) [2 3 4])"
    );
    let result = eval_bool_with(decls, &body);
    println!("\n=== diag_thread_last_single_step ===\n{:#?}\n", result);
    assert_eq!(result.unwrap(), Value::bool(true));
}

// ═══════════════════════════════════════════════════════════════════════════
// B — thread-last, two-step pipeline. `(:test::thread-last [1 2 3] (map INC)
// (filter GT2))` → `(filter GT2 (map INC [1 2 3]))` → [3 4]. The real test:
// variable injection across two steps via the fold.
// ═══════════════════════════════════════════════════════════════════════════
#[test]
#[ignore = "249.3 diagnostic — run with --ignored to read the gap"]
fn diag_thread_last_pipeline() {
    let decls = THREAD_LAST_MACRO;
    let body = format!(
        "(:wat::core::= (:test::thread-last [1 2 3] (:wat::core::map {INC}) \
         (:wat::core::filter {GT2})) [3 4])"
    );
    let result = eval_bool_with(decls, &body);
    println!("\n=== diag_thread_last_pipeline ===\n{:#?}\n", result);
    assert_eq!(result.unwrap(), Value::bool(true));
}

// ═══════════════════════════════════════════════════════════════════════════
// C — FORM-INTROSPECTION: does `:wat::holon::is-List?` work over a macro-bound
// form-value (a `wat__WatAST::List`)? The threading macro must branch on step
// shape (list step → splice; bare symbol → wrap). `(:test::is-list (i64::+ 1 2))`
// → the arg is a quoted LIST form → if is-List? works, the body picks `1`.
// `(:test::is-list 5)` → IntLit, not List → `0`.
// ═══════════════════════════════════════════════════════════════════════════
const IS_LIST_MACRO: &str = "(:wat::core::defmacro :test::is-list \
     [form <- :wat::holon::HolonAST] -> :AST<wat::holon::HolonAST> \
     (:wat::core::if (:wat::holon::is-List? form) -> :AST<wat::holon::HolonAST> `1 `0))";

#[test]
#[ignore = "249.3 diagnostic — run with --ignored to read the gap"]
fn diag_is_list_over_form() {
    let yes = eval_bool_with(
        IS_LIST_MACRO,
        "(:wat::core::= (:test::is-list (:wat::core::i64::+ 1 2)) 1)",
    );
    let no = eval_bool_with(IS_LIST_MACRO, "(:wat::core::= (:test::is-list 5) 0)");
    println!("\n=== diag_is_list_over_form ===\nLIST→1: {:#?}\nINT→0:  {:#?}\n", yes, no);
    assert_eq!(yes.unwrap(), Value::bool(true), "is-List? must be true for a list form");
    assert_eq!(no.unwrap(), Value::bool(true), "is-List? must be false for an int form");
}

// ═══════════════════════════════════════════════════════════════════════════
// D — FORM-DECOMPOSITION: do `:wat::core::first` / `:wat::core::rest` operate
// over a form-value (decompose a `wat__WatAST::List` into head + tail forms)?
// Thread-FIRST `(f a b)` → `(f acc a b)` needs head + rest, not just splice.
// This row reads the diagnostic — it expands `~(Option/expect (first form) …)`,
// so if first-over-form yields the head as a form-value, the unquote splices it.
// We only read the error/Value shape, not a clean assertion.
// ═══════════════════════════════════════════════════════════════════════════
const HEAD_MACRO: &str = "(:wat::core::defmacro :test::head \
     [form <- :wat::holon::HolonAST] -> :AST<wat::holon::HolonAST> \
     `(~(:wat::core::Option/expect -> :wat::holon::HolonAST (:wat::core::first form) \"nonempty\")))";

#[test]
#[ignore = "249.3 diagnostic — run with --ignored to read the gap"]
fn diag_first_over_form() {
    // `(:test::head (5))` — arg is a 1-element list form `(5)`; first → 5 (IntLit
    // form); `~5` splices → expands to `5`. If first-over-form works, compute = 5.
    let result = eval_bool_with(HEAD_MACRO, "(:wat::core::= (:test::head (5)) 5)");
    println!("\n=== diag_first_over_form ===\n{:#?}\n", result);
    // Diagnostic only — read the shape; do not gate on it.
    let _ = result;
}

// ═══════════════════════════════════════════════════════════════════════════
// E — PURITY of the eval-time quasiquote path. A PROGRAM body (an `if`) returns
// a quasiquote whose computed unquote is IMPURE (`~(:wat::kernel::stopped?)`).
//
// The bare-quasiquote path is fenced (gate B of probe_arc249_macro_engine, via
// expand-time walk_template→macro_eval). But a PROGRAM body runs through
// runtime::eval, and its inner quasiquote is walked by eval-time
// `walk_quasiquote` (runtime.rs:10380) whose unquote uses raw `eval_inner`
// (10402), NOT the fenced `macro_eval` — and `validate_pure_total` SKIPS
// quasiquote contents (eval.rs:99). So this path may run the impure unquote
// UNFENCED — an F5-redux.
//
// EXPECTED IF FENCED: startup_ok == false (refused). EXPECTED IF HOLE:
// startup_ok == true (the kernel call ran at expand time). This row asserts the
// SAFE expectation; if it FAILS, 249.3a must close the eval-time purity hole
// (route eval-time quasiquote unquote/splice through macro_eval in macro
// context), not merely add splice.
// ═══════════════════════════════════════════════════════════════════════════
#[test]
#[ignore = "249.3 diagnostic — run with --ignored to read the gap"]
fn diag_program_body_quasiquote_impure_unquote_fenced() {
    // TYPE-COMPATIBLE impure unquote (grounds against PURITY, not a type
    // coincidence): `stopped?` returns bool; `(not ~bool)` is bool; the probe's
    // return type is bool. If `stopped?` runs at expand time, the expansion
    // type-checks and startup SUCCEEDS → the kernel call ran UNFENCED → HOLE.
    // (Run 1 used `(i64::+ ~bool 1)` and got a TypeMismatch — a FALSE refusal:
    // the impurity ran; the type error was a coincidence, not a purity fence.
    // "Ground against the right target" — feedback_ground_against_right_target.)
    let src = "(:wat::core::defmacro :test::impure-prog [] -> :AST<wat::holon::HolonAST> \
                 (:wat::core::if (:wat::core::= 1 1) -> :AST<wat::holon::HolonAST> \
                   `(:wat::core::not ~(:wat::kernel::stopped?)) \
                   `false))\n\
               (:wat::core::defn :user::probe [] -> :wat::core::bool (:test::impure-prog))";
    let err = startup_from_source(&with_nil_main(src), None, Arc::new(InMemoryLoader::new()))
        .err()
        .map(|e| format!("{:?}", e));
    let accepted = err.is_none();
    println!(
        "\n=== diag_program_body_quasiquote_impure_unquote_fenced ===\nstartup_ok = {} (false = fenced/safe, true = F5-redux HOLE)\nrefusal mechanism: {:#?}\n",
        accepted, err
    );
    assert!(
        !accepted,
        "a program-body quasiquote with an impure computed unquote MUST be refused \
         (eval-time quasiquote purity); if accepted, the eval-time path is an F5-redux hole"
    );
}
