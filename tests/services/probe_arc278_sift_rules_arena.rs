//! Arc 278 task #6 ARENA — the `sift-rules-defsvc` macro's RICH-RECORD + SCALE + PAGED RED gate,
//! proven end-to-end on BOTH loci (loci-agnostic is non-negotiable). A 6-rule graph over a
//! genuinely NESTED HTTP/anomaly domain (Geo -> Client -> Event, an enum Method, a 2-level nested
//! `where`-accessor on client.geo.country) with a where-cascade (Event -> Suspect[Lemma] ->
//! {Anomaly,Breach}[Deductions], graded parallel at different thresholds off the same Lemma), a
//! 2nd independent cascade (Event -> Flagged[Lemma] -> Critical[Deduction]), and a direct
//! single-level branch (Event -> Overflow[Deduction], no gate). Floods N=800 Logs cycling 10
//! categories (80 each); the exact per-cycle deduction math (0+0+1+2+0+1+1+0+4+0 = 9) over 80
//! cycles yields EXACTLY 720 terminal Deductions — Lemma types (Suspect/Flagged) never appear in
//! the returned items. Paged at :limit 100 (8 exact pages) via the cursor, accumulated by the
//! caller until exhausted.
//!
//! Phase 0 (grounded): a single write-logs call carrying this rich payload crashes the journal'
//! child on PROCESS locus somewhere between 650-700 rows (bisected in scratchpad/) — a per-call
//! IPC-frame-size ceiling, distinct from the ~1000-row total-store-duplication ruin reproduced
//! separately at N=3600 (still open, not hit here). The PROCESS driver chunks the flood into 2
//! batches of 400 (well under the ceiling); 800 total rows read back exact, no duplication.
//!
//! A second scenario proves the fail-closed guard still holds at this richer graph: a Log whose
//! message type (`:arena::Bogus`) is NOT among `:defs` makes the whole page `::Fatal`.
//!
//! Run: cargo test --release -p wat sift_rules

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

#[test]
fn sift_rules_arena_counts_exact_deductions_paged_on_thread() {
    let world = startup_beside(file!()).expect("startup should succeed");
    let func = world.symbols().get(":user::sift-rules-arena-thread").expect(":user::sift-rules-arena-thread").clone();
    let got = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!()).unwrap_or_else(|e| {
        panic!(
            "sift-rules arena (THREAD) raised: {e:?}. A dial/timeout means grant-before-dial \
             failed somewhere in the mem-store'/journal'/my-sift' chain; a crash inside sift-rules' \
             own op body is now a diagnosable RuntimeError, not a deadlock."
        )
    });
    assert!(
        matches!(got, Value::i64(720)),
        "expected 80 cycles x 9 deductions/cycle = 720 EXACT terminal Deductions (Suspect/Flagged \
         Lemmas excluded — the driver returns -1 if any leaked into the paged items); got {got:?}"
    );
}

#[test]
fn sift_rules_arena_counts_exact_deductions_paged_on_process() {
    let world = startup_beside(file!()).expect("startup should succeed");
    let func = world.symbols().get(":user::sift-rules-arena-process").expect(":user::sift-rules-arena-process").clone();
    let got = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!()).unwrap_or_else(|e| {
        panic!(
            "sift-rules arena (PROCESS) raised: {e:?}. A dial/timeout means grant-before-dial \
             failed somewhere in the mem-store'/journal'/my-sift' chain across the fork, or the \
             chunked flood tripped the Phase-0 per-call IPC-frame-size ceiling."
        )
    });
    assert!(
        matches!(got, Value::i64(720)),
        "loci-agnostic: sift-rules arena on a PROCESS fork must return the SAME 720 Deductions \
         as thread, paged via the cursor; got {got:?}"
    );
}

#[test]
fn sift_rules_arena_fails_closed_on_unknown_message_type_thread() {
    let world = startup_beside(file!()).expect("startup should succeed");
    let func = world.symbols().get(":user::sift-rules-arena-fatal-thread").expect(":user::sift-rules-arena-fatal-thread").clone();
    let got = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .unwrap_or_else(|e| panic!("sift-rules arena fail-closed (THREAD) raised: {e:?}"));
    assert!(
        matches!(got, Value::bool(true)),
        "expected a Log whose message type (:arena::Bogus) is NOT among :defs to make the whole \
         page ::Fatal (fail-closed, never a silent skip) on the richer graph too; got {got:?}"
    );
}

#[test]
fn sift_rules_arena_fails_closed_on_unknown_message_type_process() {
    let world = startup_beside(file!()).expect("startup should succeed");
    let func = world.symbols().get(":user::sift-rules-arena-fatal-process").expect(":user::sift-rules-arena-fatal-process").clone();
    let got = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .unwrap_or_else(|e| panic!("sift-rules arena fail-closed (PROCESS) raised: {e:?}"));
    assert!(
        matches!(got, Value::bool(true)),
        "loci-agnostic: the fail-closed ::Fatal guard must hold across a PROCESS fork too; got {got:?}"
    );
}
