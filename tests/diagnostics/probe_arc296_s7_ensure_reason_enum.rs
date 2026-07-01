//! Arc 296 S7 probe — `EnsureFnInvalid.reason` is a structural enum, never prose.
//!
//! `CheckErrorKind::EnsureFnInvalid { …, reason }` today carries a `String` that is a
//! DISCRIMINANT-AS-PROSE: the 7 construction sites emit one of 5 fixed failure modes, and
//! three of them `format!()` structured data (a count, a type pair, a type) into the string.
//! S7 replaces `reason: String` with a `#[derive(ToEdn)]` enum `EnsureFnInvalidReason`, so the
//! CheckError derive emits `:reason` as a structural tagged value instead of a prose blob.
//!
//! This is a BEHAVIORAL RED — it drives a real bad `:ensure :fn` through startup and reads the
//! emitted `:reason` off the wire. It COMPILES at HEAD (it names no new type) and is RED at HEAD
//! because `:reason` is an `OwnedValue::String` today; it turns GREEN when `:reason` becomes a
//! `#wat.kernel/ArgTypeMismatch {:arg-type … :clause-return-type …}` tagged value.
//!
//! Committed `#[ignore]`'d (RED at HEAD, keeps the floor green); the S7 strike un-ignores it.

use wat::freeze::startup_from_file;
use wat_edn::OwnedValue;

/// Pull the `#wat.kernel/<Tag>` name off a tagged value (else "").
fn tag_name(v: &OwnedValue) -> &str {
    match v {
        OwnedValue::Tagged(tag, _) => tag.name(),
        _ => "",
    }
}

/// Read a keyword-keyed field out of a `#wat.kernel/<Tag> {…}` tagged map.
fn field<'a>(v: &'a OwnedValue, key: &str) -> Option<&'a OwnedValue> {
    if let OwnedValue::Tagged(_, body) = v {
        if let OwnedValue::Map(fields) = body.as_ref() {
            return fields
                .iter()
                .find(|(k, _)| matches!(k, OwnedValue::Keyword(kw) if kw.name() == key))
                .map(|(_, val)| val);
        }
    }
    None
}

#[test]
#[ignore = "296 S7 RED — reason is prose String at HEAD; un-ignore when the enum lands"]
fn ensure_fn_invalid_reason_is_structural_not_prose() {
    let err = startup_from_file("tests/diagnostics/probe_arc296_s7_ensure_reason_enum.wat")
        .expect_err("a defclause with a mismatched :ensure :fn arg type must fail startup");

    // Each CheckError → its structured EDN.
    let edns = err.to_edn_values();
    let ensure = edns
        .iter()
        .find(|v| tag_name(v) == "EnsureFnInvalid")
        .unwrap_or_else(|| {
            let all: Vec<String> = edns.iter().map(wat_edn::write).collect();
            panic!("expected an #wat.kernel/EnsureFnInvalid error; got: {all:?}")
        });

    let reason = field(ensure, "reason")
        .unwrap_or_else(|| panic!("EnsureFnInvalid must carry a :reason field; got: {}", wat_edn::write(ensure)));

    // THE CONTRACT: :reason is a structural tagged value, never a prose String.
    assert!(
        matches!(reason, OwnedValue::Tagged(..)),
        "S7: :reason must be a structural tagged value (e.g. #wat.kernel/ArgTypeMismatch \
         {{:arg-type … :clause-return-type …}}), NOT a prose String; got: {}",
        wat_edn::write(reason)
    );

    // And for this fixture specifically it is the type-pair variant, carrying BOTH types
    // as separate fields (no format!'d prose).
    assert_eq!(
        tag_name(reason), "ArgTypeMismatch",
        "the arg-type≠clause-return mismatch must be the ArgTypeMismatch reason; got: {}",
        wat_edn::write(reason)
    );
    assert!(
        field(reason, "arg-type").is_some() && field(reason, "clause-return-type").is_some(),
        "ArgTypeMismatch must carry :arg-type and :clause-return-type as separate fields; got: {}",
        wat_edn::write(reason)
    );

    // The whole thing stays valid EDN.
    wat_edn::parse_owned(&wat_edn::write(ensure)).expect("must be valid EDN");
}
