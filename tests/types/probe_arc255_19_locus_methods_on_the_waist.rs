//! Stone 255.19 — per-locus behaviour lives on the waist (arc 255).
//!
//! WHY: `:wat::spawn::runner-count` and `:wat::spawn::with-label` were defclauses keyed on the
//! concrete loci — a per-locus list outside the `(Locus :- [T])` surface. A generic locus could
//! not narrow into them, and `with-label` returned the BARE `Locus`, which is accepted as any
//! `(Locus :- [T])`: measured on the pre-stone binary, a PROCESS locus through `with-label`
//! type-checked as a Shared launch (rc=0). Both are now `Locus` surface methods (retired
//! defclauses, no alias); `with-label` returns `(Locus :- [T])`, keeping the transport.
//!
//! Rows:
//! - `with_label_process_claimed_shared` — the erasure is refused: `ReturnTypeMismatch`
//!   (Wire produced, Shared declared).
//! - `with_label_process_claimed_wire` — the positive twin, same shape claiming Wire → accepted.
//! - `generic_count_claimed_string` — `runner-count` is a DECLARED surface method returning i64,
//!   so a generic reader claiming String is refused. The non-vacuity row for 255.18's
//!   `generic_reads_its_count`: a call to an UNDECLARED `Locus/<name>` checks with a free result
//!   (on the pre-stone binary this file is accepted, rc=0).
//! - `start_process_claimed_shared` — defservice's ABSTRACT `start$impl` (a locus held in a
//!   symbol) now takes `(Locus :- [T])` with T the Handle's own transport letter, so a process
//!   locus cannot yield a Shared Handle → `ReturnTypeMismatch`. Pre-stone: rc=0 (bare locus,
//!   free T).
//! - `start_process_claimed_wire` — the positive twin → accepted.

use wat::check::error::{CheckErrorKind, CheckErrors};
use wat::freeze::{startup_from_file, StartupError};

const DIR: &str = "tests/types/probe_arc255_19_locus_methods_on_the_waist";

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

#[test]
fn a_process_locus_through_with_label_cannot_be_claimed_shared() {
    let errs = check_errors("with_label_process_claimed_shared");
    assert_eq!(errs.len(), 1, "{errs:?}");
    wat::assert_check_error_present!(errs,
        CheckErrorKind::ReturnTypeMismatch { function, expected, got, .. }
            if function == ":probe::mislabeled-launch" && expected == SHARED && got == WIRE);
}

#[test]
fn a_process_locus_through_with_label_is_wire() {
    let path = format!("{DIR}_with_label_process_claimed_wire.wat");
    startup_from_file(&path).unwrap_or_else(|e| panic!("{path} must be accepted: {e:?}"));
}

#[test]
fn a_generic_runner_count_is_a_declared_i64() {
    let errs = check_errors("generic_count_claimed_string");
    assert_eq!(errs.len(), 1, "{errs:?}");
    wat::assert_check_error_present!(errs,
        CheckErrorKind::ReturnTypeMismatch { function, expected, got, .. }
            if function == ":probe::count" && expected == ":wat::core::String" && got == ":wat::core::i64");
}

// rune:lint(no-inlined-wat) — golden rendered TYPE NAMES the checker prints, compared by equality.
const KV_SHARED: &str = "(:probe::kv::Handle :- [:wat::kernel::Transport.Shared])";
const KV_WIRE: &str = "(:probe::kv::Handle :- [:wat::kernel::Transport.Wire])";

#[test]
fn an_abstract_start_of_a_process_locus_cannot_be_claimed_shared() {
    let errs = check_errors("start_process_claimed_shared");
    assert_eq!(errs.len(), 1, "{errs:?}");
    wat::assert_check_error_present!(errs,
        CheckErrorKind::ReturnTypeMismatch { function, expected, got, .. }
            if function == ":probe::go" && expected == KV_SHARED && got == KV_WIRE);
}

#[test]
fn an_abstract_start_of_a_process_locus_is_wire() {
    let path = format!("{DIR}_start_process_claimed_wire.wat");
    startup_from_file(&path).unwrap_or_else(|e| panic!("{path} must be accepted: {e:?}"));
}
