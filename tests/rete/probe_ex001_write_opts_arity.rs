//! Excursus 001 WO-OPT — 0-arg and 3-arg `write-json` / `write-json-natural`
//! are type errors. The arity guard lives in `infer_edn_write_json*`
//! (`src/check.rs`), not in an Exact registry row.
//!
//! The 1-arg ≡ 2-arg-with-default property is a runtime gate in
//! `wat-tests/edn/write-opts.wat`. These fixtures cover the checker bounds.

use wat::check::error::{CheckErrorKind, CheckErrors};
use wat::freeze::{startup_from_file, StartupError};

#[test]
fn write_json_zero_args_is_a_check_error() {
    let err = startup_from_file("tests/rete/probe_ex001_write_opts_arity__zero.wat.bad")
        .expect_err("0-arg write-json must fail check");
    let StartupError::Check(CheckErrors(errs)) = &err else {
        panic!("expected a type-check error, got {err:?}");
    };
    wat::assert_check_error_present!(errs,
        CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::edn::write-json"
            && reason.as_str() == "expected 1 or 2 args (value [opts :wat::edn::WriteOpts]); got 0"
    );
}

#[test]
fn write_json_three_args_is_a_check_error() {
    let err = startup_from_file("tests/rete/probe_ex001_write_opts_arity__three.wat.bad")
        .expect_err("3-arg write-json must fail check");
    let StartupError::Check(CheckErrors(errs)) = &err else {
        panic!("expected a type-check error, got {err:?}");
    };
    wat::assert_check_error_present!(errs,
        CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::edn::write-json"
            && reason.as_str() == "expected 1 or 2 args (value [opts :wat::edn::WriteOpts]); got 3"
    );
}

#[test]
fn write_json_natural_zero_args_is_a_check_error() {
    let err = startup_from_file("tests/rete/probe_ex001_write_opts_arity__natural_zero.wat.bad")
        .expect_err("0-arg write-json-natural must fail check");
    let StartupError::Check(CheckErrors(errs)) = &err else {
        panic!("expected a type-check error, got {err:?}");
    };
    wat::assert_check_error_present!(errs,
        CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::edn::write-json-natural"
            && reason.as_str() == "expected 1 or 2 args (value [opts :wat::edn::WriteOpts]); got 0"
    );
}

#[test]
fn write_json_natural_three_args_is_a_check_error() {
    let err = startup_from_file("tests/rete/probe_ex001_write_opts_arity__natural_three.wat.bad")
        .expect_err("3-arg write-json-natural must fail check");
    let StartupError::Check(CheckErrors(errs)) = &err else {
        panic!("expected a type-check error, got {err:?}");
    };
    wat::assert_check_error_present!(errs,
        CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::edn::write-json-natural"
            && reason.as_str() == "expected 1 or 2 args (value [opts :wat::edn::WriteOpts]); got 3"
    );
}
