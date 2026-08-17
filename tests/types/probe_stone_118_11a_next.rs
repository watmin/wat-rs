//! Stone 118.11a — `:wat::stream::next` + `:wat::stream::NextOutcome<T>`.
//!
//! Additive: mint the parametric outcome enum + the one native verb that forces exactly one
//! `Stream` cell and returns both halves. Nothing existing moves; the `forced: OnceLock` memo in
//! `src/stream/mod.rs` is untouched (see `git diff src/stream/mod.rs` in the strike's own report —
//! this file cannot assert an absence of a diff, only what the verb itself does).
//!
//! Rows 1/2/4 (`DESIGN-STONE-118.11a` / `BRIEF-STONE-118.11a` / `EXPECTATIONS-STONE-118.11a`) are
//! plain value-returning fns in the co-located fixture, driven in-process via `call_beside_value`
//! and inspected directly as `Value::Enum`.
//!
//! Row 3 — ★★ the whole stone — needs REAL stdout: a printing `f` inside `(map f v)`, one `next`
//! call, must print exactly one line. `println` requires the primed stdio services a running
//! program provides, so this row spawns the real `wat` binary as a subprocess (the established
//! pattern in `tests/cli/wat_cli.rs`) and counts actual OS-level stdout lines.

use std::process::Command;

use wat::freeze::call_beside_value;
use wat::runtime::Value;

const FIXTURE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/types/probe_stone_118_11a_next.wat");

/// Row 1 — `(next <3-element stream>)` -> `Item`, `value` = first element (1).
#[test]
fn row1_next_on_three_element_stream_is_item_with_first_value() {
    let got = call_beside_value(file!(), ":probe::row1")
        .expect(":probe::row1 should evaluate without raising");
    match &got {
        Value::Enum(e) => {
            assert_eq!(
                e.type_path.trim_start_matches(':'),
                "wat::stream::NextOutcome",
                "row1: wrong enum type_path: {got:?}"
            );
            assert_eq!(e.variant_name, "Item", "row1: expected Item, got: {got:?}");
            match e.fields.first() {
                Some(Value::i64(1)) => {}
                other => panic!("row1: expected value=1, got fields[0]={other:?} (full: {got:?})"),
            }
        }
        other => panic!("row1: expected Value::Enum, got: {other:?}"),
    }
}

/// Row 2 — `(next <exhausted stream>)` -> `Exhausted`.
#[test]
fn row2_next_on_empty_stream_is_exhausted() {
    let got = call_beside_value(file!(), ":probe::row2")
        .expect(":probe::row2 should evaluate without raising");
    match &got {
        Value::Enum(e) => {
            assert_eq!(
                e.type_path.trim_start_matches(':'),
                "wat::stream::NextOutcome",
                "row2: wrong enum type_path: {got:?}"
            );
            assert_eq!(e.variant_name, "Exhausted", "row2: expected Exhausted, got: {got:?}");
            assert!(e.fields.is_empty(), "row2: Exhausted must carry no fields, got: {got:?}");
        }
        other => panic!("row2: expected Value::Enum, got: {other:?}"),
    }
}

/// Row 4 — pulling `rest` out of row 1's `Item` and calling `next` again yields the SECOND
/// element (2) — proving `rest` actually advances, not just that `next` decodes a Cons once.
#[test]
fn row4_next_on_rest_yields_second_element() {
    let got = call_beside_value(file!(), ":probe::row4")
        .expect(":probe::row4 should evaluate without raising");
    match &got {
        Value::Enum(e) => {
            assert_eq!(e.variant_name, "Item", "row4: expected Item, got: {got:?}");
            match e.fields.first() {
                Some(Value::i64(2)) => {}
                other => panic!("row4: expected value=2, got fields[0]={other:?} (full: {got:?})"),
            }
        }
        other => panic!("row4: expected Value::Enum, got: {other:?}"),
    }
}

/// Row 3 — ★★ THE STONE. With a printing `f`, a SINGLE `next` on `(map f v)` must print exactly
/// one line. `realize` (src/stream/mod.rs:158) already stops at the first `Empty|Cons`; this row
/// measures that `next` doesn't add a second force on top of it, rather than assuming it. Real
/// OS stdout, real subprocess — `println` needs the primed stdio a running program provides.
#[test]
fn row3_one_next_on_mapped_stream_prints_exactly_one_line() {
    let bin = env!("CARGO_BIN_EXE_wat");
    let output = Command::new(bin)
        .arg(FIXTURE)
        .output()
        .expect("spawn wat");

    assert!(
        output.status.success(),
        "row3: wat exit {:?}, stderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines.len(),
        1,
        "STOP-1: next must force exactly ONE cell — one next on (map f v) printed {} line(s), \
         not 1. stdout: {:?}, stderr: {}",
        lines.len(),
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        lines[0], "\"CALLED\"",
        "row3: the one printed line should be f's own marker (EDN-quoted by println); got: {:?}",
        lines[0]
    );
}
