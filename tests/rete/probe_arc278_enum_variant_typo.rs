//! DISCONFIRMING PROBE — vigilia Class D1: a misspelled enum variant in a rete constraint
//! compiles, fires, and matches nothing, with no diagnostic.
//!
//! `validate/typing.rs`'s `keyword_constant_segment` types a bare keyword constant by PREFIX only
//! and never checks the variant exists, so `:evt::G::Hii` types as "enum" and the rete checker
//! passes it. The runtime resolves through `sym.unit_variant` — an EXACT lookup — gets `None`, and
//! falls back to a plain keyword. `enum::=` then compares Enum vs keyword: always false.
//!
//! ⛔ CORE REFUSES THE IDENTICAL EXPRESSION at check time. `matcher::enum_variant_ctor` already
//! exists as the one resolution, documented "ONE COPY … hand-written at THREE independent sites".

use std::path::Path;
use std::process::{Command, Stdio};

fn run(rel: &str) -> (bool, String, String) {
    let bin = env!("CARGO_BIN_EXE_wat");
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let out = Command::new(bin)
        .arg(&path)
        .stdin(Stdio::null())
        .output()
        .unwrap_or_else(|e| panic!("spawn {bin} {}: {e}", path.display()));
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// The control. Without it, a green probe arm below is indistinguishable from a rule that never
/// fired for some unrelated reason — "matched nothing" is also what a broken fixture looks like.
#[test]
fn a_real_enum_variant_in_a_rete_constraint_matches() {
    let (ok, out, err) = run("tests/rete/probe_arc278_enum_variant_typo.wat");
    assert!(ok, "the control fixture must run\n{out}{err}");
    assert_eq!(
        out.trim(),
        "1",
        "`:evt::G::Hi` exists and exactly one seeded Req carries it — if this is not 1 the \
         fixture drifted and the probe below proves nothing\n{out}"
    );
}

/// ⚠ EXPECTED RED until Class D1 lands.
#[test]
fn a_misspelled_enum_variant_in_a_rete_constraint_is_refused() {
    let (ok, out, err) = run("tests/rete/probe_arc278_enum_variant_typo_bad.wat");
    assert!(
        !ok,
        "SILENT WRONG ANSWER: a rule constraining on `:evt::G::Hii` — a variant the enum does not \
         declare — compiled, fired, and printed {:?} with exit 0 and no diagnostic. Core REFUSES \
         the identical expression at check time (`parameter #2 expects :wat::core::keyword; got \
         :evt::G`), so the two engines disagree about the same input and rete ships the wrong \
         answer. A typo became a constraint that compiles, fires, and matches nothing.\n{out}{err}",
        out.trim()
    );
}

/// ⚠ ARM 2 — the arm the obvious fix does NOT close. `enum_variant_ctor` resolves Unit **and**
/// Tagged, so routing through it alone still types the bare `:tg::P::Hi` (arity 1) as an `enum`
/// while the runtime's `sym.unit_variant` is UNIT-ONLY and yields a plain keyword. The typing must
/// additionally require **arity == 0**. EXPECTED RED until that lands.
#[test]
fn a_bare_tagged_enum_variant_in_a_rete_constraint_is_refused() {
    let (ok, out, err) = run("tests/rete/probe_arc278_enum_variant_typo_tagged.wat");
    assert!(
        !ok,
        "SILENT WRONG ANSWER: a rule constraining on the BARE tagged variant `:tg::P::Hi` — which \
         has no bare value form at all, `(:tg::P::Hi 7)` is the only way to write one — compiled, \
         fired, and printed {:?} with exit 0 and no diagnostic. Core REFUSES the identical \
         expression at check time (`parameter #2 expects [:wat::core::i64 :-> :tg::P]`), so the \
         two engines disagree about the same input and rete ships the wrong answer.\n{out}{err}",
        out.trim()
    );
}
