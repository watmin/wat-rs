//! Arc 255 Stone 255.14 — **a namespace is not a type.**
//!
//! The builder's ruling: *a non-type parent uses the NAMESPACE join; respell `/` → `::` in
//! the DECLARATION; the faithful form does not change.* `wat/spawn.wat`'s per-env builder
//! constructors (`thread::init`, `process::post-spawn`, `process::runner-count`, …) spelled
//! a namespace as if it were a type. 251.8d-ii's eighth draw proved that a `/` join at a
//! NON-TYPE parent is UNSPELLABLE in the faithful surface: `wat.spawn.process/post-spawn` is
//! the image of BOTH `:wat::spawn::process/post-spawn` and `:wat::spawn::process::post-spawn`,
//! and only `freeze::env::rekey_type_member_functions` can tell them apart — which it can do
//! only when the parent names a TYPE.
//!
//! ⭐ **THE NAMES DID NOT CHANGE.** Rows 1 and 2 are the SAME function reached through the
//! two surfaces, and row 2's spelling is byte-identical to what it was before the migration.
//! That is the claim this file pins, and it is why the respelling is not a rename.
//!
//! ⛔ **What must NOT have moved**, each its own row: a legitimate `Type/member` join
//! (row 3), the refusal of the retired `/` spelling (row 4 — the side that DID move), the
//! refusal of an unknown member under the new join (row 5), and the refusal of an unknown
//! member under a TYPE parent, still `/`-joined (row 6).
//!
//! Recorded migration: `wat-scripts/fixes/spawn-builder-namespace-join.wat`.
//!
//! Run: cargo test --release -p wat --test services -- arc255_14_namespace_join

use wat::freeze::{startup_from_file, StartupError};
use wat::runtime::Value;

const FIXTURE: &str = "tests/services/probe_arc255_14_namespace_join.wat";

/// A `.wat.bad` fixture that must fail startup, and the name its refusal has to carry.
const REFUSALS: &[(&str, &str, &str)] = &[
    (
        "tests/services/probe_arc255_14_namespace_join_old_join.wat.bad",
        ":wat::spawn::process/runner-count",
        "THE SIDE THAT MOVED — the retired `/` join at a non-type parent must be refused, by name",
    ),
    (
        "tests/services/probe_arc255_14_namespace_join_unknown_member.wat.bad",
        ":wat::spawn::process::nope",
        "the wall is on the NAME — the join widened, the population did not",
    ),
    (
        "tests/services/probe_arc255_14_namespace_join_unknown_type_member.wat.bad",
        ":wat::spawn::ProcessOpts/nope",
        "the TYPE-parent wall is untouched, and still reports the `/` join",
    ),
];

/// Freeze one fixture and apply one zero-arg entry from it.
///
/// ⛔ NOT `call_beside_value`: that helper PANICS on a freeze failure, which would take every
/// later row of this test with it. A `Result` keeps all rows readable on ANY binary.
fn entry_value(path: &str, entry: &str) -> Result<Value, StartupError> {
    let world = startup_from_file(path)?;
    // A missing entry is a BROKEN FIXTURE, not a result this test reports: panic rather than
    // mint a `Result` arm no row discriminates (arc 296 Stone M's rubric).
    let func = world
        .symbols()
        .get(entry)
        .unwrap_or_else(|| panic!("fixture {path} no longer declares {entry}"))
        .clone();
    wat::runtime::apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .map_err(|e| StartupError::Runtime(Box::new(e)))
}

#[test]
fn a_namespace_member_answers_to_both_surfaces_and_only_the_new_join() {
    let mut wrong: Vec<String> = Vec::new();

    // ⭐ ROW 1 — the respelled name in the KEYWORD surface.
    match entry_value(FIXTURE, ":user::kw-spelled") {
        Ok(Value::i64(3)) => {}
        other => wrong.push(format!(
            "  :user::kw-spelled -> {other:?}, want i64(3) — the KEYWORD spelling \
             `:wat::spawn::process::runner-count`"
        )),
    }

    // ⭐ ROW 2 — the SAME name in the FAITHFUL surface, `wat.spawn.process/runner-count`,
    // byte-identical to its pre-migration spelling. Rows 1 and 2 agreeing is the ruling:
    // the declaration moved, the faithful form did not.
    match entry_value(FIXTURE, ":user::faithful-spelled") {
        Ok(Value::i64(3)) => {}
        other => wrong.push(format!(
            "  :user::faithful-spelled -> {other:?}, want i64(3) — the FAITHFUL spelling \
             `wat.spawn.process/runner-count` must reach the same function"
        )),
    }

    // ⛔ ROW 3 — a legitimate `Type/member` join. `ProcessOpts` IS a type, so 255.4's `/`
    // member join stands. This stone must not move it.
    match entry_value(FIXTURE, ":user::type-parent-join") {
        Ok(Value::i64(77)) => {}
        other => wrong.push(format!(
            "  :user::type-parent-join -> {other:?}, want i64(77) — a TYPE-parent `/` join \
             (`:wat::spawn::ProcessOpts/max-message-bytes`) is UNTOUCHED"
        )),
    }

    // ⛔ ROW 4 — SAME IDENTITY, STATED ON THE REGISTRY. The declaration lives under exactly
    // one key. If both spellings were registered, rows 1–2 could agree for the wrong reason.
    match startup_from_file(FIXTURE) {
        Ok(world) => {
            for (key, want_present) in [
                (":wat::spawn::process::runner-count", true),
                (":wat::spawn::process/runner-count", false),
            ] {
                let present = world.symbols().get(key).is_some();
                if present != want_present {
                    wrong.push(format!(
                        "  registry: {key} present={present}, want {want_present} — the \
                         respelled declaration must hold ONE key, not two"
                    ));
                }
            }
        }
        Err(e) => wrong.push(format!("  {FIXTURE} -> did not freeze: {e:?}")),
    }

    // ⛔ ROWS 5-7 — THE WALLS. Each must REFUSE, and the refusal must NAME the head: a
    // startup that fails for some other reason is not this wall.
    for (path, must_name, why) in REFUSALS {
        match startup_from_file(path) {
            Ok(_) => wrong.push(format!("  {path} -> STARTED, want REFUSED — {why}")),
            Err(e) => {
                let msg = format!("{e:?}");
                if !msg.contains(must_name) {
                    wrong.push(format!(
                        "  {path} -> refused, but the reason does not name {must_name} \
                         — {why}\n      got: {msg}"
                    ));
                }
            }
        }
    }

    assert!(
        wrong.is_empty(),
        "the namespace join answered wrongly on {} row(s):\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}
