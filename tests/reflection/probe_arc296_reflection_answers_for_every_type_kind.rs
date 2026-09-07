//! PROBE — reflection must answer for EVERY `TypeDef` kind, at runtime.
//!
//! Builder, 2026-09-07: *"we've built a pretty competent reflection subsystem… you hitting this
//! failure is disappointing… reflection has been grown organically… we must be able to do this
//! work at runtime."*
//!
//! ## THE GAP, MEASURED
//!
//! ```text
//! TypeDef kinds        Aggregate · Enum · Newtype · Alias · Union · Surface     6
//! reflection answers   field-names-of / field-types-of, Aggregate only          1
//! ```
//!
//! Two questions about one of six kinds — and even for `Aggregate` it cannot report `nature`
//! (Struct/Record/HolonRecord), `type_params`, or restrictions. An `Enum` is fully opaque: no
//! variants, no per-variant field names, no purity, no params. Measured at HEAD:
//!
//! ```text
//! field-names-of :probe::Rec        -> [:alpha]                          ok
//! field-names-of :probe::Box        -> "is not a struct/record type"     refused
//! field-names-of :probe::Box::Full  -> "unknown type"                    refused
//! type-of        :probe::Box        -> UnknownFunction                   does not exist
//! ```
//!
//! ## WHAT THIS COST
//!
//! The match-arm codemod needed one fact — *what are this variant's declared field names* — and
//! could not ask. It guessed the binder name instead, wrote `{:_cur _cur}` where `_cur` is a
//! binder and not a field, and that guess reached the corpus.
//!
//! ★ The dangerous half is not the loud failure. `:_cur` errored only because no such field
//! exists. Had the variant HAD a field named `_cur`, the arm would have bound the WRONG FIELD
//! SILENTLY. A reflection hole does not merely block tooling; it makes tooling guess.
//!
//! Un-ignored by the type-of stone.

use std::path::PathBuf;
use std::process::{Command, Stdio};

fn run(fixture: &str) -> (i32, String) {
    let path: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/reflection")
        .join(fixture);
    let out = Command::new(env!("CARGO_BIN_EXE_wat"))
        .arg(&path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn wat");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).trim().to_owned(),
    )
}

#[test]
fn reflection_answers_for_an_enum() {
    let (code, out) = run("probe_arc296_reflection_answers_for_every_type_kind.wat");
    assert_eq!(
        code, 0,
        "a wat program must be able to interrogate a declared ENUM at runtime — its variants, \
         and each variant's declared field names. Today the question has no verb; got:\n{out}"
    );
    // ⛔ DELIBERATELY NOT ASSERTED HERE: that the row NAMES the variant and its declared field.
    // A `contains` check is a loose assertion (`no_loose_string_assert` refused the first draft,
    // correctly — it passes on reordered fields and appended garbage), and an exact golden cannot
    // be captured for output that does not exist yet. Pinning the row's SHAPE here would also
    // pre-empt the stone's own contract decision. So this row asserts only the non-negotiable —
    // THE QUESTION IS ANSWERABLE AT ALL — and the field-name requirement rides EXPECTATIONS row 4,
    // verified per-kind in the SCORE against the shape the strike actually lands.
    assert!(!out.is_empty(), "type-of must answer with a value, not silence");
}

#[test]
fn reflection_answers_for_all_six_kinds_and_variant_field_order() {
    let (code, out) = run("probe_arc296_type_of_six_kinds.wat");
    assert_eq!(code, 0, "six-kind fixture must run; got:\n{out}");
    // Exact stdout — not a contains check. One fact per line, in this order:
    // six kinds, Pair's declared fields (declaration order), Rec nature,
    // field-names-of Rec (not retired), Option type-params.
    assert_eq!(
        out,
        "\"Aggregate\"\n\"Enum\"\n\"Newtype\"\n\"Alias\"\n\"Union\"\n\"Surface\"\n\":left\"\n\":right\"\n\"Record\"\n\":alpha\"\n\"T\""
    );
}
