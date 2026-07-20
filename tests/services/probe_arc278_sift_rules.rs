//! Arc 278 task #6 — the `sift-rules-defsvc` macro's RED gate, proven end-to-end on BOTH loci
//! (loci-agnostic is non-negotiable — thread-only would be a failure). A producer floods N=240
//! Logs whose messages are `:usr::Temp` facts (30 HOT c>50, 210 cold); `sift-rules-defsvc` compiles
//! two rules over `:usr::Temp` (hot -> Hot, hot -> Warn), so each hot Temp fires BOTH — one seed,
//! two deductions. Expect EXACTLY 60 (30 hot x 2 rules; the count EXCEEDS the 30 hot inputs —
//! inference, not selection). A second scenario proves the fail-closed guard: a Log whose message
//! type is NOT among the macro's `:defs` makes the whole page `::Fatal`.
//!
//! Run: cargo test --release -p wat sift_rules

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

#[test]
fn sift_rules_defsvc_counts_exact_deductions_on_thread() {
    let world = startup_beside(file!()).expect("startup should succeed");
    let func = world.symbols().get(":user::sift-rules-thread").expect(":user::sift-rules-thread").clone();
    let got = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!()).unwrap_or_else(|e| {
        panic!(
            "sift-rules-defsvc (THREAD) raised: {e:?}. A dial/timeout means grant-before-dial \
             failed somewhere in the mem-store'/journal'/my-sift' chain; a crash inside sift-rules' \
             own op body is now a diagnosable RuntimeError, not a deadlock."
        )
    });
    assert!(
        matches!(got, Value::i64(60)),
        "expected 30 hot Temps x 2 rules = 60 deductions (inference EXCEEDS the 30 hot inputs); \
         got {got:?}"
    );
}

#[test]
fn sift_rules_defsvc_counts_exact_deductions_on_process() {
    let world = startup_beside(file!()).expect("startup should succeed");
    let func = world.symbols().get(":user::sift-rules-process").expect(":user::sift-rules-process").clone();
    let got = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!()).unwrap_or_else(|e| {
        panic!(
            "sift-rules-defsvc (PROCESS) raised: {e:?}. A dial/timeout means grant-before-dial \
             failed somewhere in the mem-store'/journal'/my-sift' chain across the fork."
        )
    });
    assert!(
        matches!(got, Value::i64(60)),
        "loci-agnostic: sift-rules-defsvc on a PROCESS fork must return the SAME 60 deductions \
         as thread (30 hot Temps x 2 rules); got {got:?}"
    );
}

#[test]
fn sift_rules_defsvc_fails_closed_on_unknown_message_type_thread() {
    let world = startup_beside(file!()).expect("startup should succeed");
    let func = world.symbols().get(":user::sift-rules-fatal-thread").expect(":user::sift-rules-fatal-thread").clone();
    let got = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .unwrap_or_else(|e| panic!("sift-rules-defsvc fail-closed (THREAD) raised: {e:?}"));
    assert!(
        matches!(got, Value::bool(true)),
        "expected a Log whose message type (:usr::Other) is NOT among :defs to make the whole \
         page ::Fatal (fail-closed, never a silent skip); got {got:?}"
    );
}

#[test]
fn sift_rules_defsvc_fails_closed_on_unknown_message_type_process() {
    let world = startup_beside(file!()).expect("startup should succeed");
    let func = world.symbols().get(":user::sift-rules-fatal-process").expect(":user::sift-rules-fatal-process").clone();
    let got = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .unwrap_or_else(|e| panic!("sift-rules-defsvc fail-closed (PROCESS) raised: {e:?}"));
    assert!(
        matches!(got, Value::bool(true)),
        "loci-agnostic: the fail-closed ::Fatal guard must hold across a PROCESS fork too; got {got:?}"
    );
}
