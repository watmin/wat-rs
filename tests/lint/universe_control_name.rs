//! A NAME THAT EXISTS ONLY TO BE A CONTROL, SO NO BENCHMARK CAN EVER BURN IT.
//!
//! `tests/lint/rete_citation_resolves.rs` proves its resolver's universe reaches OUTSIDE `src/` by
//! naming functions attested only under `tests/`. If the universe ever narrowed back to `src/`,
//! every citation of a test-only name would become a false finding, and that control is what would
//! say so.
//!
//! ## The ratchet this file stops
//!
//! Those controls used to be two REAL test functions, picked because a rete comment cites them. The
//! `engine: gated by <test fn>` form — see `tests/lint/rete_engine_label_names_its_evidence.rs` —
//! writes a test function's NAME into a string literal under `src/`. That is enough to attest the
//! name in `src/`, which is exactly what disqualifies it as a control. One of the two was consumed
//! that way on 2026-09-01.
//!
//! The coupling is GENERAL, not a one-off: every future `gated by` claim retires its gate's name as
//! a test-only control anywhere in the tree. A search of the whole tree found NO replacement — only
//! one other test fn is both cited in a rete comment and absent from `src/`. So the population of
//! usable controls only ever falls, and the last one dies silently: the citation gate would keep
//! reporting green while proving nothing.
//!
//! Low probability, high consequence. The cure is a control that is structurally unburnable — a
//! name no benchmark could plausibly reference, because it names no behaviour. That is the function
//! below. It is the FLOOR, not the replacement: the real cited control stays alongside it, because
//! two independent controls beat one owned one.
//!
//! ## Why this is not self-vouching
//!
//! The property under test is *"the resolver's universe reaches `tests/`"*. A function the lint
//! suite owns is legitimate evidence for REACH — the same shape as the `POSITIVE_CONTROL` consts
//! already used in this directory. What would be self-vouching is resolving a name against the
//! gate's own text, which `rete_citation_resolves.rs` separately excludes by skipping its own file.
//!
//! And this file cannot host the control's consumer: that gate excludes itself from its own
//! universe, so a name defined there would not resolve at all.

use std::path::{Path, PathBuf};

/// The control's own name, as data. `rete_citation_resolves.rs` lists it in `IN_TESTS_ONLY`; the
/// test below keeps the promise that listing depends on.
const NAME: &str = "zz_universe_control_never_cite_this";

/// Files under `src/`, measured 2026-09-01: 213.
const SRC_FLOOR: usize = 150;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_rs(&p, out);
        } else if p.extension().and_then(|x| x.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

/// The universe control. Its NAME is the payload; this body keeps the name's one promise.
///
/// Named to be uncitable on purpose: it describes no behaviour, so no benchmark row, design stone
/// or `engine: gated by` claim has any reason to reference it, and the `zz_` prefix keeps it last
/// in every sorted listing where a reader might otherwise mistake it for a real gate.
#[test]
fn zz_universe_control_never_cite_this() {
    let mut files = Vec::new();
    collect_rs(&root().join("src"), &mut files);
    files.sort();

    // NON-VACUITY: a walk that comes back empty would confirm this name's absence from a tree it
    // never read, which is the vacuous PASS this whole directory exists to refuse. The floor sits
    // under the 213 files the walk finds today (driven 2026-09-01).
    assert!(
        files.len() >= SRC_FLOOR,
        "the control walk found only {} .rs file(s) under src/ — it is not reaching the tree whose \
         silence it certifies, so its green means nothing",
        files.len()
    );

    let cited: Vec<String> = files
        .iter()
        .filter(|p| {
            std::fs::read_to_string(p)
                .map(|s| s.contains(NAME))
                .unwrap_or(false)
        })
        .map(|p| {
            p.strip_prefix(root())
                .unwrap_or(p)
                .display()
                .to_string()
        })
        .collect();

    assert!(
        cited.is_empty(),
        "\n\n`{NAME}` now appears under `src/`, in {} file(s). That is the one thing it must never \
         do: `rete_citation_resolves.rs` lists it as a name attested ONLY under `tests/`, and an \
         identifier present in `src/` proves nothing about whether the resolver's universe reaches \
         the test corpus.\n\
         \n\
         THE FIX: take the name back out of `src/` — do not adjust this test. If an \
         `engine: gated by` claim pulled it in, that claim named the wrong test: this function \
         gates nothing and is not evidence for any benchmark row.\n\
         \n\
         Cited in:\n\n{}\n",
        cited.len(),
        cited.join("\n")
    );
}
