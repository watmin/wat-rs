//! Excursus 003 strike D3, GD2a — the activation census is a standing gate.
//!
//! BRIEF-shape-strike-D3-the-activation-census-is-a-gate.md. Finishes
//! BRIEF-shape-strike-D2-every-raise-names-its-activation.md items 2 and 3. Drives
//! every reachable `RuntimeErrorKind` variant through its REAL producer — a
//! minimal wat program through the real entry point (`call_beside_value`,
//! `startup_from_file`, or `invoke_user_main`), never a hand-constructed
//! `RuntimeError` — and asserts the raised error's innermost Rust frame is PRESENT
//! and named as expected. Presence is the point: an omission goes RED.
//!
//! Co-located fixture: probe_excursus003_d3_gd2a_census.wat (the runtime-dispatch
//! kinds). Freeze-time/registration kinds each get their own small fixture beside
//! this one — a kind that makes FREEZING ITSELF fail cannot sit in the shared
//! fixture every other probe here also needs to freeze cleanly.
//!
//! Four producer shapes, normalized into one `Raised { class, frame_fns }`:
//! (A) `call_beside_value` -> `Err(RuntimeError)` directly;
//! (B) `call_beside_value` -> `Ok(Value::Result(Err(Failure)))` — every kind reached
//!     through `:wat::eval-ast!`/`:wat::eval-step!`/`:wat::eval-digest-string!`,
//!     whose `wrap_as_eval_result` catches the raise and carries the real
//!     `:wat::kernel::Failure` (excursus 003 strike E);
//! (C) `startup_from_file` -> `Err(StartupError::Runtime(RuntimeError))` — a
//!     registration-time (freeze step 6) or freeze-step-9 producer;
//! (D) `startup_from_source` (Ok) then `invoke_user_main` -> `Err(RuntimeError)`.
//!
//! The no-real-producer and measured-not-driven kinds (`ParamShadowsBuiltin`,
//! `NoEncodingCtx`, `NoSourceLoader`, `NoMacroRegistry`, `ChannelDisconnected`,
//! `AssertionFailed`, `SandboxScopeLeak`, `ReteCeiling`, `EdnCoerceMismatch`,
//! `WriteStopped`) carry no test here — see the strike report for the measured
//! reason on each; a hand-constructed `RuntimeError` would bypass the very
//! dispatch this census exists to prove, so they are not faked into this file.

use wat::edn::contract::ToEdn;
use wat::freeze::{call_beside_value, invoke_user_main, startup_from_file};
use wat::runtime::Value;
use wat::StartupError;
use wat_edn::OwnedValue;

/// Normalized result of driving one real producer: the declared record class
/// (`"wat::runtime::<Kind>"`) and the frame `:fn` names, innermost first.
struct Raised {
    class: String,
    frame_fns: Vec<String>,
}

fn get_field<'a>(pairs: &'a [(OwnedValue, OwnedValue)], key: &str) -> Option<&'a OwnedValue> {
    pairs.iter().find_map(|(k, v)| {
        let kw = k.as_keyword()?;
        (kw.name() == key && kw.namespace().is_none()).then_some(v)
    })
}

fn frame_fn(frame: &OwnedValue) -> String {
    let (tag, body) = frame.as_tagged().expect("frame is a tagged Frame");
    assert_eq!(tag.namespace(), "wat.kernel", "frame tag namespace: {tag}");
    assert_eq!(tag.name(), "Frame", "frame tag name: {tag}");
    let pairs = body.as_map().expect("Frame body is a map");
    get_field(pairs, "fn")
        .and_then(|v| v.as_str())
        .expect(":fn is a String")
        .to_string()
}

/// Shape (A)/(C)/(D): a `RuntimeError`'s own `to_edn()` (the derive-generated
/// `{:span :frames :frames-elided ...}` shape, NOT `error_edn()`/the wire — step 3c's
/// gate_b proved the wire carries no `:frames` at all).
fn from_runtime_error(e: &wat::runtime::RuntimeError) -> Raised {
    let edn = e.to_edn();
    let (tag, body) = edn.as_tagged().expect("RuntimeError EDN is tagged");
    assert_eq!(tag.namespace(), "wat.runtime", "class namespace: {tag}");
    let pairs = body.as_map().expect("RuntimeError EDN body is a map");
    let frames = get_field(pairs, "frames")
        .and_then(|v| v.as_vector())
        .expect(":frames is a vector");
    Raised {
        // rune:lint(one-variant-separator, namespace) — `tag.name()` is a bare
        // `defrecord` class name (e.g. "DivisionByZero"); this prepends the
        // `wat::runtime` NAMESPACE prefix to it, never an enum/variant pair —
        // there is no enum here to compose against.
        class: format!("wat::runtime::{}", tag.name()),
        frame_fns: frames.iter().map(frame_fn).collect(),
    }
}

/// Shape (B): `Value::Result(Err(Value::Aggregate(Failure{error frames
/// frames-elided})))` — the eval family's real carried error (excursus 003 strike
/// E). `error`'s class is its own declared `:wat::runtime::<Kind>` record; `frames`
/// is a `Vec<Value::Aggregate(Frame{fn at tail-elided})>`.
fn from_eval_failure(v: &Value) -> Raised {
    match v {
        Value::Result(r) => match &**r {
            Err(Value::Aggregate(failure)) => {
                assert_eq!(failure.class.as_ref(), "wat::kernel::Failure");
                let class = match &failure.fields[0] {
                    Value::Aggregate(error) => error.class.to_string(),
                    other => panic!("Failure.error not Aggregate; got {other:?}"),
                };
                let frames = match &failure.fields[1] {
                    Value::Vec(fr) => fr,
                    other => panic!("Failure.frames not Vec; got {other:?}"),
                };
                let frame_fns = frames
                    .iter()
                    .map(|f| match f {
                        Value::Aggregate(frame) => match &frame.fields[0] {
                            Value::String(s) => (**s).clone(),
                            other => panic!("Frame.fn not String; got {other:?}"),
                        },
                        other => panic!("frame not Aggregate; got {other:?}"),
                    })
                    .collect();
                Raised { class, frame_fns }
            }
            other => panic!("expected Err(Record(Failure)); got {other:?}"),
        },
        other => panic!("expected Value::Result; got {other:?}"),
    }
}

/// Assert presence (the point of GD2a) and the expected innermost activation name.
fn assert_named(kind: &str, raised: &Raised, expected_class: &str, expected_activation: &str) {
    assert_eq!(
        raised.class, expected_class,
        "{kind}: wrong class produced — the fixture drives the wrong kind"
    );
    assert!(
        !raised.frame_fns.is_empty(),
        "{kind}: frames must not be empty — the activation that raised must be named, \
         never omitted (an omission is exactly what this census exists to catch)"
    );
    assert_eq!(
        raised.frame_fns[0], expected_activation,
        "{kind}: innermost frame must name {expected_activation:?}; got {:?}",
        raised.frame_fns
    );
}

// ─── Shape (A): call_beside_value -> Err(RuntimeError) directly ──────────────────

#[test]
fn census_notcallable() {
    let e = call_beside_value(file!(), ":t::probe-notcallable").expect_err("must raise");
    assert_named("NotCallable", &from_runtime_error(&e), "wat::runtime::NotCallable", "f");
}

#[test]
fn census_servicenotrunning() {
    let e = call_beside_value(file!(), ":t::probe-servicenotrunning").expect_err("must raise");
    assert_named(
        "ServiceNotRunning",
        &from_runtime_error(&e),
        "wat::runtime::ServiceNotRunning",
        ":wat::kernel::println",
    );
}

#[test]
fn census_nomatchingclause() {
    let e = call_beside_value(file!(), ":t::probe-nomatchingclause").expect_err("must raise");
    assert_named(
        "NoMatchingClause",
        &from_runtime_error(&e),
        "wat::runtime::NoMatchingClause",
        ":t::pick02",
    );
}

#[test]
fn census_postconditionfailed() {
    let e = call_beside_value(file!(), ":t::probe-postconditionfailed").expect_err("must raise");
    assert_named(
        "PostconditionFailed",
        &from_runtime_error(&e),
        "wat::runtime::PostconditionFailed",
        ":t::pick07",
    );
}

// MacroAbort deliberately has no test here — measured, not driven. See the
// long comment at the matching spot in probe_excursus003_d3_gd2a_census.wat.

#[test]
fn census_macroexpansionfailed() {
    let e = call_beside_value(file!(), ":t::probe-macroexpansionfailed").expect_err("must raise");
    assert_named(
        "MacroExpansionFailed",
        &from_runtime_error(&e),
        "wat::runtime::MacroExpansionFailed",
        ":wat::core::macroexpand",
    );
}

// ─── Shape (B): call_beside_value -> Ok(Value::Result(Err(Failure))) ─────────────

fn census_eval_family_case(fn_name: &str, kind: &str, expected_class: &str, expected_activation: &str) {
    let v = call_beside_value(file!(), fn_name).expect("eval-family wrap always returns Ok(Value::Result(..))");
    assert_named(kind, &from_eval_failure(&v), expected_class, expected_activation);
}

#[test]
fn census_unboundsymbol() {
    census_eval_family_case(
        ":t::probe-unboundsymbol",
        "UnboundSymbol",
        "wat::runtime::UnboundSymbol",
        "zzz-nowhere-bound",
    );
}

#[test]
fn census_unknownfunction() {
    census_eval_family_case(
        ":t::probe-unknownfunction",
        "UnknownFunction",
        "wat::runtime::UnknownFunction",
        ":wat::core::apply",
    );
}

#[test]
fn census_notvaluedispatchable() {
    census_eval_family_case(
        ":t::probe-notvaluedispatchable",
        "NotValueDispatchable",
        "wat::runtime::NotValueDispatchable",
        ":wat::core::apply",
    );
}

#[test]
fn census_typemismatch() {
    census_eval_family_case(
        ":t::probe-typemismatch",
        "TypeMismatch",
        "wat::runtime::TypeMismatch",
        ":wat::i64::+",
    );
}

#[test]
fn census_aritymismatch() {
    census_eval_family_case(
        ":t::probe-aritymismatch",
        "ArityMismatch",
        "wat::runtime::ArityMismatch",
        ":wat::i64::+",
    );
}

/// GD2a's writer-4 row: `apply_function`'s OWN guard, superseding writer 3's
/// symbol-as-written name the instant the symbol resolves to a real function.
/// See the long comment beside `:t::probe-aritymismatch-via-apply` in the `.wat`
/// fixture for why a keyword-headed call doesn't isolate writer 4 (writer 1
/// already set the same name first).
#[test]
fn census_aritymismatch_via_apply_function() {
    census_eval_family_case(
        ":t::probe-aritymismatch-via-apply",
        "ArityMismatch (writer 4)",
        "wat::runtime::ArityMismatch",
        ":wat::core::Fn",
    );
}

#[test]
fn census_badcondition() {
    census_eval_family_case(
        ":t::probe-badcondition",
        "BadCondition",
        "wat::runtime::BadCondition",
        ":wat::core::if",
    );
}

#[test]
fn census_malformedform() {
    census_eval_family_case(
        ":t::probe-malformedform",
        "MalformedForm",
        "wat::runtime::MalformedForm",
        ":wat::core::if",
    );
}

#[test]
fn census_divisionbyzero() {
    census_eval_family_case(
        ":t::probe-divisionbyzero",
        "DivisionByZero",
        "wat::runtime::DivisionByZero",
        ":wat::i64::/",
    );
}

#[test]
fn census_integeroverflow() {
    census_eval_family_case(
        ":t::probe-integeroverflow",
        "IntegerOverflow",
        "wat::runtime::IntegerOverflow",
        ":wat::i64::+",
    );
}

#[test]
fn census_declarationinexpressionposition() {
    census_eval_family_case(
        ":t::probe-declarationinexpressionposition",
        "DeclarationInExpressionPosition",
        "wat::runtime::DeclarationInExpressionPosition",
        ":wat::core::defalias",
    );
}

#[test]
fn census_evalforbidsmutationform() {
    census_eval_family_case(
        ":t::probe-evalforbidsmutationform",
        "EvalForbidsMutationForm",
        "wat::runtime::EvalForbidsMutationForm",
        ":wat::eval-ast!",
    );
}

#[test]
fn census_evalverificationfailed() {
    census_eval_family_case(
        ":t::probe-evalverificationfailed",
        "EvalVerificationFailed",
        "wat::runtime::EvalVerificationFailed",
        ":wat::eval-digest-string!",
    );
}

#[test]
fn census_patternmatchfailed() {
    census_eval_family_case(
        ":t::probe-patternmatchfailed",
        "PatternMatchFailed",
        "wat::runtime::PatternMatchFailed",
        ":wat::core::match",
    );
}

#[test]
fn census_effectfulinstep() {
    census_eval_family_case(
        ":t::probe-effectfulinstep",
        "EffectfulInStep",
        "wat::runtime::EffectfulInStep",
        ":wat::eval-step!",
    );
}

#[test]
fn census_nosteprule() {
    census_eval_family_case(
        ":t::probe-nosteprule",
        "NoStepRule",
        "wat::runtime::NoStepRule",
        ":wat::eval-step!",
    );
}

#[test]
fn census_unknownfield() {
    census_eval_family_case(
        ":t::probe-unknownfield",
        "UnknownField",
        "wat::runtime::UnknownField",
        ":bogus-field",
    );
}

// ─── Shape (C): startup_from_file -> Err(StartupError::Runtime(RuntimeError)) ────

fn census_startup_runtime_case(path: &str, kind: &str, expected_class: &str, expected_activation: &str) {
    let result = startup_from_file(path);
    match result {
        Err(StartupError::Runtime(e)) => {
            assert_named(kind, &from_runtime_error(&e), expected_class, expected_activation);
        }
        other => panic!("{kind}: expected Err(StartupError::Runtime(_)); got {other:?}"),
    }
}

#[test]
fn census_reservedprefix() {
    census_startup_runtime_case(
        "tests/diagnostics/probe_excursus003_d3_gd2a_reserved_prefix.wat.bad",
        "ReservedPrefix",
        "wat::runtime::ReservedPrefix",
        "6-register-defines",
    );
}

#[test]
fn census_unnamespacedname() {
    census_startup_runtime_case(
        "tests/diagnostics/probe_excursus003_d3_gd2a_unnamespaced.wat.bad",
        "UnnamespacedName",
        "wat::runtime::UnnamespacedName",
        "6-register-defines",
    );
}

#[test]
fn census_dottedname() {
    census_startup_runtime_case(
        "tests/diagnostics/probe_excursus003_d3_gd2a_dotted_name.wat.bad",
        "DottedName",
        "wat::runtime::DottedName",
        "6-register-defines",
    );
}

#[test]
fn census_unreachableclause() {
    census_startup_runtime_case(
        "tests/types/probe_stone_118_b2c_unreachable_arm_refused_neg.wat.bad",
        "UnreachableClause",
        "wat::runtime::UnreachableClause",
        "6-register-defines",
    );
}

#[test]
fn census_duplicatedefine() {
    // Empirically measured, not guessed: the extend-type member collision fires
    // during `resolve_references` (freeze step 7), not step 9 — `resolve::register`'s
    // gate runs at the SAME pre-registration point as the method key is derived,
    // which this specific surface-impl path reaches before `register_runtime_defs`
    // (step 9) ever does.
    census_startup_runtime_case(
        "tests/types/probe_arc293_4c_extend_type_adapter_dup.wat.bad",
        "DuplicateDefine",
        "wat::runtime::DuplicateDefine",
        "7-resolve-references",
    );
}

#[test]
fn census_retedefnaxisviolation() {
    census_startup_runtime_case(
        "tests/diagnostics/probe_excursus003_d3_gd2a_rete_defn_axis_violation.wat.bad",
        "ReteDefnAxisViolation",
        "wat::runtime::ReteDefnAxisViolation",
        "9-freeze",
    );
}

#[test]
fn census_retedefnrecursive() {
    census_startup_runtime_case(
        "tests/diagnostics/probe_excursus003_d3_gd2a_rete_defn_recursive.wat.bad",
        "ReteDefnRecursive",
        "wat::runtime::ReteDefnRecursive",
        "9-freeze",
    );
}

// ─── Shape (D): startup_from_source (Ok) then invoke_user_main -> Err ────────────

#[test]
fn census_usermainmissing() {
    let world = startup_from_file("tests/diagnostics/probe_excursus003_d3_gd2a_user_main_missing.wat")
        .expect("a program with no :user::main freezes cleanly (startup_bare)");
    let e = invoke_user_main(&world, Vec::new()).expect_err("no :user::main must raise");
    assert_named(
        "UserMainMissing",
        &from_runtime_error(&e),
        "wat::runtime::UserMainMissing",
        "9-freeze",
    );
}
