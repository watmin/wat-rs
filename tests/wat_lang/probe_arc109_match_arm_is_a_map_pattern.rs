//! PROBE — arc 109 / 296: a match arm is a BRACKET CLAUSE carrying a MAP PATTERN.
//!
//! Ruled by the builder 2026-09-06 (see `109/NOTE-match-cond-clause-brackets.md`'s amendment of
//! that date, which RETIRES the flat positional clause `[<Variant> [d0 d1 …] <body>]`):
//!
//! ```clojure
//! #wat.core/Option.Some {:val 42}                       the wire
//! [wat.core/Option.Some {:val val} (wat.core/+ 0 val)]   the arm — nearly the wire itself
//! ```
//!
//! ★ ROW 2 IS THE STONE. The two readings of a map pattern produce DIFFERENT, CHECKABLE NUMBERS.
//! `Pair.Two` declares `[a b]`; the fixture constructs `(Two 1 2)` and the arm writes its keys in
//! REVERSE order, `{:b b :a a}`, then evaluates `a - b`:
//!
//! ```
//! bound BY NAME      a=1 b=2  ->  -1      the ruled behaviour
//! bound BY POSITION  b=1 a=2  ->  +1      what the retired positional clause would give
//! ```
//!
//! No positional reading can pass row 2, which is why it is the row and not the others.
//!
//! ROW 4 GUARDS THE HARD CUT. The retired `(pattern body)` clause must be REFUSED once this
//! lands — one grammar, not two. It PASSES at HEAD (the old form works today) and must go to a
//! non-zero exit, so it is the inverse of the others: RED here means "the old form still works".
//!
//! The fixtures were verified clean independently: the same declarations under today's positional
//! form run and print `-1`, so a red on rows 1-3 is the ARM GRAMMAR and nothing else. At HEAD all
//! three report `malformed :wat::core::match form: arm #N must be `(pattern body)`` (the
//! non-exhaustive error alongside it is that refusal's consequence — no arm was recognised — not a
//! second defect).
//!
//! Un-ignored by this stone.

use std::path::PathBuf;
use std::process::{Command, Stdio};

fn run(fixture: &str) -> (i32, String) {
    let path: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/wat_lang")
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
fn a_map_pattern_arm_binds_its_declared_keys() {
    let (code, out) = run("probe_arc109_match_arm__declared_order.wat");
    assert_eq!(code, 0, "the bracket clause + map pattern must be accepted; got {out:?}");
    assert_eq!(out, "-1", "a=1 b=2 bound by name, so a - b is -1");
}

#[test]
fn a_map_pattern_binds_by_name_not_by_position() {
    let (code, out) = run("probe_arc109_match_arm__reversed_keys.wat");
    assert_eq!(code, 0, "keys written out of declaration order must still be accepted; got {out:?}");
    assert_eq!(
        out, "-1",
        "THE ROW: the pattern writes {{:b b :a a}} while the variant declares [a b]. Bound BY NAME \
         a=1/b=2 gives -1; bound BY POSITION b=1/a=2 gives +1. A positional reading CANNOT reach -1"
    );
}

#[test]
fn a_unit_variant_arm_is_an_empty_map() {
    let (code, out) = run("probe_arc109_match_arm__unit_empty_map.wat");
    assert_eq!(code, 0, "a unit variant's arm is `{{}}`, mirroring its `{{}}` wire; got {out:?}");
    assert_eq!(out, "99");
}

#[test]
fn the_retired_positional_clause_is_refused() {
    let (code, _) = run("probe_arc109_match_arm__positional_control.wat");
    assert_ne!(
        code, 0,
        "ONE GRAMMAR, NOT TWO: the retired `(pattern body)` clause must be refused once the bracket \
         clause lands. This row PASSES at HEAD and inverts — a red here means the old form still \
         works and the corpus can silently sit in both grammars"
    );
}
