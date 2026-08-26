//! Arc 296 — non-Macro `StartupError` variants emit STRUCTURED tagged EDN,
//! not a `:detail` prose blob.
//!
//! ## What changed (296.3 + 296.5 completion)
//!
//! Before: `process_died_error_startup_value(format!("{}", e))` — a Display
//! string crossed the process boundary.
//!
//! After 296.2/296.3: `process_died_error_startup_value(wat_edn::write(&e.to_edn()))`
//! — tagged EDN text crosses. But the INTERIM serializer still wrapped most
//! variants as `#wat.kernel/StartupPhaseError {:phase :x :detail "<Display>"}`
//! — structure smuggled in a `:detail` string.
//!
//! After 296.5 completion: every variant that carries a structured underlying
//! error (Parse, Config, Load, Type, Resolve, Check, Stdlib) delegates to that
//! error's own `ToEdn` impl — fully navigable (span + kind + fields), no
//! `:detail` blob. ONLY `SigmaFn(String)` keeps a `:detail` (it is a genuinely
//! flat human message — no span, no kind, no structured fields).
//!
//! This probe verifies the PATTERN at the unit level (no subprocess needed).

use std::sync::Arc;
use wat::check::error::{CheckError, CheckErrorKind, CheckErrors};
use wat::config::{ConfigError, ConfigErrorKind};
use wat::freeze::StartupError;
use wat::load::loader::{LoadError, LoadErrorKind};
use wat::macros::{MacroError, MacroErrorKind};
use wat::resolve::ResolveError;
use wat::runtime::{RuntimeError, RuntimeErrorKind};
use wat::span::Span;
use wat::edn::contract::ToEdn;
use wat::types::error::{TypeError, TypeErrorKind};

// ─── Probe 1 — Parse error: structured tagged EDN, NOT a :detail blob ────────

#[test]
fn probe_1_parse_startup_error_to_edn_is_structured_not_detail() {
    // Construct a StartupError::Parse the same way a child process would see one.
    // rune:lint(no-inlined-edn) — input under test: malformed wat source with an unclosed paren, fed to the parser
    let bad_source = "(defn :user::main [] (unclosed";
    let parse_err = match wat::parser::parse_all_with_file(bad_source, "test.wat") {
        Err(e) => e,
        Ok(_) => panic!("expected parse to fail for probe source"),
    };
    let startup_err = StartupError::Parse(parse_err);

    let display_str = format!("{}", startup_err);
    let edn_str = wat_edn::write(&startup_err.to_edn());

    // Must be a tagged form in the wat.parse namespace (delegates to
    // ParseError::to_edn → #wat.parse/<ParseErrorVariant>); NO :detail blob.
    wat::assert_edn_matches_file!(edn_str.clone(), "probe_arc296_3_holdout_edn__parse_unclosed_paren.edn", "Parse startup error must produce exact structured #wat.parse/ EDN");

    // Must differ from the Display string.
    assert_ne!(
        edn_str, display_str,
        "to_edn() must produce structured EDN, not the Display string"
    );

    // Must be valid EDN.
    wat_edn::parse_owned(&edn_str).expect("Parse startup error must produce valid EDN");
}

// ─── Probe 2 — SigmaFn: the ONE honest :detail (bare message, no structure) ──

#[test]
fn probe_2_sigmafn_startup_error_keeps_honest_detail() {
    // SigmaFn carries a plain String message — no span, no kind, no structured
    // fields. `:detail` is the honest serialization here, NOT a deferral.
    let startup_err =
        StartupError::SigmaFn("sigma fn registration failed: bad config".into());

    let display_str = format!("{}", startup_err);
    let edn_str = wat_edn::write(&startup_err.to_edn());

    // SigmaFn's :detail is the honest, deliberate exception (bare message).
    wat::assert_edn_matches_file!(edn_str.clone(), "probe_arc296_3_holdout_edn__sigmafn_detail.edn", "SigmaFn must produce exact #wat.macro/SigmaFnError tagged EDN with :detail");
    assert_ne!(edn_str, display_str, "to_edn() must differ from Display string");
    wat_edn::parse_owned(&edn_str).expect("must be valid EDN");
}

// ─── Probe 3 — Span.to_edn() round-trip stays stable (regression) ───────────

#[test]
fn probe_3_span_to_edn_round_trip_stable() {
    let span = Span::new(Arc::new("src/user.wat".to_string()), 7, 2);
    let edn = span.to_edn();
    let s = wat_edn::write(&edn);
    let parsed = wat_edn::parse_owned(&s).expect("valid EDN");
    assert_eq!(
        wat_edn::write(&parsed),
        s,
        "Span.to_edn() must round-trip via write+parse"
    );
}

// ─── Probe 4 — Check: the worst offender, now a navigable vector of errors ───

#[test]
fn probe_4_check_startup_error_emits_structured_vector_not_detail() {
    // A Check failure is a COLLECTION of CheckErrors. Before the 296.5
    // completion, the Check arm stringified the whole collection as
    // `:detail (e.to_string())` — even though `check_error_to_edn` / the
    // `impl ToEdn for CheckError` serializer was already built and wired
    // NOWHERE. This probe asserts the serializer is now USED.
    let span = Span::new(Arc::new("user.wat".to_string()), 8, 3);
    let errors = CheckErrors(vec![
        CheckError {
            span: span.clone(),
            kind: CheckErrorKind::UnknownCallee {
                callee: ":user::do-thing".into(),
            },
        },
        CheckError {
            span,
            kind: CheckErrorKind::CommCallOutOfPosition {
                callee: ":wat::kernel::send".into(),
            },
        },
    ]);
    let startup_err = StartupError::Check(errors);

    let display_str = format!("{}", startup_err);
    let edn_str = wat_edn::write(&startup_err.to_edn());

    // Must be the structured collection envelope with navigable inner CheckErrors; NO :detail.
    wat::assert_edn_matches_file!(edn_str.clone(), "probe_arc296_3_holdout_edn__check_errors_structured.edn", "Check startup error must produce exact structured #wat.check/CheckErrors EDN");
    // Must differ from Display.
    assert_ne!(edn_str, display_str, "to_edn() must produce structured EDN, not Display");
    wat_edn::parse_owned(&edn_str).expect("Check startup error must produce valid EDN");
}

// ─── Probe 5 — the WALL: EVERY StartupError variant is structured, never a ──
//                bare OwnedValue::String. The boundary (process_died_error_*
//                builders + to_wire_edn) is generic over ToEdn; this probe
//                proves the conversion never degrades to a stringly payload.

/// Assert a StartupError's `to_edn()` is a structured tagged envelope, NOT a
/// bare `OwnedValue::String`. Returns the variant tag for the caller to log.
fn assert_structured(label: &str, e: StartupError) {
    let edn = e.to_edn();
    eprintln!("=== {label}: {}", wat_edn::write(&edn));
    match edn {
        wat_edn::OwnedValue::Tagged(_, _) => { /* structured envelope — good */ }
        wat_edn::OwnedValue::Map(_) => { /* structured map — also acceptable */ }
        other => panic!(
            "{label}: StartupError::to_edn() must be a structured tagged/map value, \
             never a bare String; got: {:?}",
            other
        ),
    }
}

#[test]
fn probe_5_every_startup_variant_is_structured_not_stringly() {
    let span = || Span::new(Arc::new("probe.wat".to_string()), 1, 1);

    // Parse — via a real parse failure.
    // rune:lint(no-inlined-edn) — input under test: malformed wat source fed to the parser
    let parse_err = wat::parser::parse_all_with_file("(unclosed", "probe.wat")
        .expect_err("must fail to parse");
    assert_structured("Parse", StartupError::Parse(parse_err));

    // Config — Pattern A.
    assert_structured(
        "Config",
        StartupError::Config(ConfigError {
            span: span(),
            kind: ConfigErrorKind::MalformedSetter,
        }),
    );

    // Load — Pattern A.
    assert_structured(
        "Load",
        StartupError::Load(LoadError::new(
            span(),
            LoadErrorKind::DuplicateLoad { path: "a.wat".into() },
        )),
    );

    // Macro — full typed cause chain.
    assert_structured(
        "Macro",
        StartupError::Macro(MacroError {
            span: span(),
            kind: MacroErrorKind::DuplicateMacro("m".into()),
        }),
    );

    // Type — Pattern A.
    assert_structured(
        "Type",
        StartupError::Type(TypeError::new(
            span(),
            TypeErrorKind::DuplicateType { name: ":user::T".into() },
        )),
    );

    // Resolve — vector of structured references.
    assert_structured(
        "Resolve",
        StartupError::Resolve(ResolveError::UnresolvedReferences(vec![])),
    );

    // Check — vector of structured errors.
    assert_structured(
        "Check",
        StartupError::Check(CheckErrors(vec![CheckError {
            span: span(),
            kind: CheckErrorKind::UnknownCallee { callee: ":user::x".into() },
        }])),
    );

    // Runtime — arc 233 serializer.
    assert_structured(
        "Runtime",
        StartupError::Runtime(Box::new(RuntimeError::new(span(), RuntimeErrorKind::UnboundSymbol("x".into())))),
    );

    // Stdlib — omitted here: `StdlibError` lives in the `pub(crate) mod stdlib`
    // module and is not constructible from an integration test. Its
    // `impl ToEdn` (src/stdlib.rs) is wired through `startup_error_to_edn`'s
    // `Stdlib` arm and exercised by the crate-internal startup path.

    // SigmaFn — the ONE message-only variant. The string IS the datum, but it
    // is STILL wrapped in a tagged envelope (#wat.kernel/SigmaFnError) — never
    // a bare OwnedValue::String. This is the explicitly-justified exception.
    assert_structured(
        "SigmaFn",
        StartupError::SigmaFn("sigma registration failed".into()),
    );
}
