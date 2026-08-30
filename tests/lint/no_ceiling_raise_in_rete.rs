//! Arc 278 the outcome wall, S2d — **a session ceiling may not become a raise.**
//!
//! ── THE CLASS ────────────────────────────────────────────────────────────────────────────────
//!
//! A rete session carries two ceilings — `max-fire-rounds` and `max-session-bytes` — and neither
//! can be proven at load (this arc measured it: a guarded counter's bound is its SEED, which is
//! input data). So the breach is irreducibly dynamic, and the builder's ruling is that a dynamic
//! failure here is a VALUE: *"let's impose session's strict limits via totality."* Every wat-facing
//! rete verb answers a matchable outcome — `(FireOutcome :- [T])` or `InsertOutcome`.
//!
//! **A raise that escapes to wat would put the hole straight back**, and it would do it quietly:
//! the type would still say `FireOutcome`, every call site would still compile, and the failure
//! would simply unwind past the `match` the caller wrote. Nothing else in the build would notice.
//! That is the shape this gate exists to make impossible.
//!
//! ── WHY A GATE AND NOT A TYPE (the honest rung) ──────────────────────────────────────────────
//!
//! The top rung would be a `RuntimeErrorKind` whose ceiling variants are unconstructible outside
//! the two doors. Rust has no per-variant visibility — an enum variant is as public as its enum —
//! so that shape is not available without splitting the error type, which every one of ~400
//! construction sites across the substrate would pay for. **This is the highest rung the material
//! allows: a check that fires at build time**, and it is named here rather than left as a silent
//! compromise (`extirpare`: hold the rung you reached, and say which one it is).
//!
//! ── THE RULE ─────────────────────────────────────────────────────────────────────────────────
//!
//! Inside `src/rete/`, the three ceiling variants may be CONSTRUCTED only at the doors that own
//! them, and CONVERTED only at the single site that turns them into arms:
//!
//! | variant | may be constructed in |
//! |---|---|
//! | `SessionMemoryCeilingExceededOnInsert` | `kernel/session.rs` (the shared insert check) |
//! | `SessionMemoryCeilingExceeded` | `kernel/fire/delta.rs` (the fixpoint's round boundary) |
//! | `FixpointRoundCapExceeded` | `kernel/fire/delta.rs` |
//!
//! `kernel/outcome.rs` is the one place allowed to MATCH them, because it is the one place that
//! turns a breach into a value. A second converter is the drift this arc pulls out most often.

use std::path::{Path, PathBuf};

/// The variants a ceiling breach is reported with.
const CEILING_VARIANTS: &[&str] = &[
    "SessionMemoryCeilingExceededOnInsert",
    "SessionMemoryCeilingExceeded",
    "FixpointRoundCapExceeded",
];

/// Files under `src/rete/` permitted to name a ceiling variant, and why.
const ALLOWED: &[(&str, &str)] = &[
    ("kernel/session.rs", "owns the shared insert-door check"),
    ("kernel/fire/delta.rs", "owns the fixpoint's round boundary"),
    ("kernel/outcome.rs", "the ONE site that converts a breach into a matchable arm"),
];

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_rs(&p, out);
        } else if p.extension().and_then(|x| x.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

/// A line that NAMES a ceiling variant in code (not in prose).
///
/// Doc comments and line comments are skipped: this whole wall is explained in prose across the
/// rete tree, and a gate that flagged its own rationale would be unusable.
fn names_a_ceiling_variant(line: &str) -> Option<&'static str> {
    let t = line.trim_start();
    if t.starts_with("//") {
        return None;
    }
    CEILING_VARIANTS.iter().copied().find(|v| line.contains(v))
}

#[test]
fn a_ceiling_breach_is_never_raised_outside_the_two_doors() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let rete = Path::new(manifest).join("src/rete");
    let mut files = Vec::new();
    collect_rs(&rete, &mut files);

    // NON-VACUITY: the walk must actually find the tree. A typo'd path finding zero files would
    // make this gate pass forever while checking nothing.
    assert!(
        files.len() > 20,
        "the ceiling-raise walk found only {} .rs files under src/rete — it is not looking at the \
         tree it claims to guard",
        files.len()
    );

    let mut violations = Vec::new();
    let mut allowed_hits = 0usize;
    for f in &files {
        let rel = f
            .strip_prefix(&rete)
            .unwrap_or(f)
            .to_string_lossy()
            .replace('\\', "/");
        let permitted = ALLOWED.iter().any(|(p, _)| rel == *p);
        let Ok(src) = std::fs::read_to_string(f) else { continue };
        for (i, line) in src.lines().enumerate() {
            if let Some(v) = names_a_ceiling_variant(line) {
                if permitted {
                    allowed_hits += 1;
                } else {
                    violations.push(format!("  {rel}:{} names `{v}`", i + 1));
                }
            }
        }
    }

    // NON-VACUITY, the second half: the permitted files must STILL name these variants. If the
    // ceilings were ever deleted, or moved to files not in ALLOWED, this gate would go quietly
    // green while guarding nothing — the exact failure mode of a control that outlives its subject.
    assert!(
        allowed_hits >= 3,
        "the permitted doors name a ceiling variant only {allowed_hits} time(s) — the ceilings \
         have moved or gone, and this gate is now guarding an empty room. Re-point ALLOWED."
    );

    assert!(
        violations.is_empty(),
        "⛔ A SESSION CEILING IS BEING RAISED OUTSIDE ITS DOOR — arc 278, the outcome wall.\n\n\
         {}\n\n\
         A ceiling breach is a matchable VALUE, never a raise: `(FireOutcome :- [T])` for the fire\n\
         verbs, `InsertOutcome` for staging. A raise here would unwind straight past the `match`\n\
         every caller writes, while the type kept insisting the failure was handled.\n\n\
         If you are adding a new door that can breach: construct the variant in the door that owns\n\
         it, and convert it through `rete::kernel::outcome` — the ONE site that maps a breach to an\n\
         arm. Do not add a second converter; two places deciding that is the drift this arc pulls\n\
         out most often. If a genuinely new owner file is right, add it to ALLOWED with its reason.",
        violations.join("\n")
    );
}
