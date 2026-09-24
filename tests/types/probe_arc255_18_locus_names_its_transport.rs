//! Stone 255.18 (C-b1a) — the locus names its transport (arc 255).
//!
//! WHY: `Locus/launch` returned a 4-argument `(Launched :- [S R Sh Lu])`, saying nothing about
//! the transport; it passed only through the checker's missing-slot arms. Measured on the
//! pre-stone binary: a THREAD launch ascribed `(Launched :- [… Wire])` type-checked (rc=0) — a
//! shared-memory address claimed portable. `:wat::spawn::Locus` is now the parametric surface
//! `(Locus :- [T])`: ThreadOpts binds `Shared`, ProcessOpts binds `Wire` (in `wat/spawn.wat`
//! and `wat/bracket.wat`), and `launch` returns `(Launched :- [S R Sh Lu T])`, T flowing from
//! the locus's own binding (255.15's inference).
//!
//! Rows:
//! - `thread_yields_wire` — a thread locus cannot yield a Wire `Launched` → `ReturnTypeMismatch`
//!   (Shared vs Wire). Pre-stone: rc=0.
//! - `process_yields_wire` — the opposite transport, same shape → accepted.
//! - `generic` — one consumer `:- [T] (Locus :- [T]) -> (Launched :- [… T])` binds Wire from a
//!   process and Shared from a thread. Pre-stone: rc=1 (the surface was not parametric).
//! - `generic_thread_wire` — that consumer given a thread, claimed Wire → `ReturnTypeMismatch`.
//! - `generic_reads_its_count` — was `generic_narrowing`, the MEASURED GAP (a generic
//!   `(Locus :- [T])` could not narrow to `runner-count`, a defclause keyed on the concrete
//!   loci → `NoMatchingClauseAtCallSite`). 255.19 closed it by REMOVAL: `runner-count` is a
//!   `Locus` surface method, so the generic reader is accepted. Its non-vacuity row (the count
//!   is a declared i64) is 255.19's `generic_count_claimed_string`.

use wat::check::error::{CheckErrorKind, CheckErrors};
use wat::freeze::{startup_from_file, StartupError};

const DIR: &str = "tests/types/probe_arc255_18_locus_names_its_transport";

// rune:lint(no-inlined-wat) — the `(:wat::spawn::Launched :- [...])` literals below are golden
// rendered TYPE NAMES the checker prints, compared by equality; not wat source that is evaluated.
const SHARED: &str = "(:wat::spawn::Launched :- [:wat::core::i64 :wat::core::i64 :wat::core::i64 :wat::core::i64 :wat::kernel::Transport.Shared])";
const WIRE: &str = "(:wat::spawn::Launched :- [:wat::core::i64 :wat::core::i64 :wat::core::i64 :wat::core::i64 :wat::kernel::Transport.Wire])";

fn check_errors(suffix: &str) -> Vec<wat::check::error::CheckError> {
    let path = format!("{DIR}_{suffix}.wat.bad");
    let err = startup_from_file(&path).expect_err(&format!("{path} must fail check"));
    let StartupError::Check(CheckErrors(errs)) = err else {
        panic!("{path}: expected a type-check error, got {err:?}");
    };
    errs
}

fn accepted(suffix: &str) {
    let path = format!("{DIR}_{suffix}.wat");
    startup_from_file(&path).unwrap_or_else(|e| panic!("{path} must be accepted: {e:?}"));
}

#[test]
fn a_thread_locus_cannot_yield_a_wire_launched() {
    let errs = check_errors("thread_yields_wire");
    assert_eq!(errs.len(), 1, "{errs:?}");
    wat::assert_check_error_present!(errs,
        CheckErrorKind::ReturnTypeMismatch { function, expected, got, .. }
            if function == ":probe::launch-thread" && expected == WIRE && got == SHARED);
}

#[test]
fn a_process_locus_yields_a_wire_launched() {
    accepted("process_yields_wire");
}

#[test]
fn a_generic_consumer_binds_t_from_the_locus() {
    accepted("generic");
}

#[test]
fn a_generic_consumer_given_a_thread_cannot_be_claimed_wire() {
    let errs = check_errors("generic_thread_wire");
    assert_eq!(errs.len(), 1, "{errs:?}");
    wat::assert_check_error_present!(errs,
        CheckErrorKind::ReturnTypeMismatch { function, expected, got, .. }
            if function == ":probe::via-thread" && expected == WIRE && got == SHARED);
}

#[test]
fn a_generic_locus_reads_its_runner_count() {
    accepted("generic_reads_its_count");
}
