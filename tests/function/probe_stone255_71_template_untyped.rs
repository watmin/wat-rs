//! Arc 255 STONE 71 (THE WALL) — a macro's EXPANSION is checked like any other code, so an
//! untyped constructor written inside a `defmacro` TEMPLATE is refused at every use. See the
//! co-located `.wat.bad`'s header; `probe_stone255_71_template_typed.wat` is the typed twin
//! (passes once the template itself names the type).
//!
//! CORRECTED (measured, not the first draft's assumption): the span does NOT land on the
//! template's own definition line here. `untyped_constructor_span` (src/check.rs) prefers the
//! call's OWN first argument's span over the coarse head span — and this template's first
//! argument, `~a`, is itself wholly an unquote-substitution: at THIS expansion its span is
//! exactly the call site's own literal `1` (line 15 of the fixture), because the substituted AST
//! fragment carries the span it had where the CALLER wrote it, not where the template quoted it.
//! Measured directly: `wat --check` on the fixture reports `:line 15 :col 35`. This still
//! satisfies the stone's requirement ("the error... points at the call") — it just does not reach
//! the BONUS half ("also name the template's own source position if the span allows it") for a
//! template whose first arg is purely a substitution; the brief's own "if the span allows it"
//! hedges exactly this case.

use wat::check::error::{CheckErrorKind, CheckErrors};
use wat::freeze::{startup_from_file, StartupError};

#[test]
fn untyped_constructor_in_a_macro_template_is_refused_at_its_use() {
    let err = startup_from_file("tests/function/probe_stone255_71_template_untyped.wat.bad")
        .expect_err("a macro expanding to an untyped constructor call must fail check");
    let StartupError::Check(CheckErrors(errs)) = &err else {
        panic!("expected a type-check error, got {err:?}");
    };
    // Measured: the span is the call site's own `1` literal (line 15, col 35) — the value
    // substituted for the template's `~a`, which is this call's own first argument — not the
    // template's static definition line (12). See the module doc comment above.
    let found = errs.iter().any(|e| {
        matches!(&e.kind, CheckErrorKind::MalformedForm { head, reason, .. }
            if head == ":wat::core::Tuple"
            && reason.contains("Tuple")
            && reason.contains("wat.type/Tuple :- [T"))
            && e.span.line == 15
    });
    assert!(found, "expected a MalformedForm naming :wat::core::Tuple at the call site's line 15 (the `1` substituted for the template's `~a`); errors were:\n{:?}", CheckErrors(errs.clone()));
}
