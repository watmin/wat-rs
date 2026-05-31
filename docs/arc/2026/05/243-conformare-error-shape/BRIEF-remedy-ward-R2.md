# BRIEF — remedy/ WARD R2 — annihilate the live-cast L1+L2 (earn the stamp)

You are sonnet. Ward the `src/remedy/` home (ranked-remedy infrastructure, Stone 241.10).
A live 8-spell vigilia found 2 real L1s + a doc/naming L2 cluster. This R2 annihilates them
so the home earns its `vigilatum`. (5 of 8 spells already returned L1=0 L2=0 — the
structural core is clean; these are honesty/naming fixes + one contained struct change.)

ZERO git mutations — NO commit/add/stash/reset, NO scratch files outside named edits.
`git status`/`git diff`/`git grep` READ-ONLY only. The orchestrator commits after re-casting
the vigilia. If you think a commit is needed, STOP and say so. (#1 rejection trigger.)

Work ONLY in `/home/watmin/work/holon/wat-rs/`. Files in scope: `src/remedy/{mod,distance,rank,retirement}.rs`
and `tests/` ONLY if a test constructs `Remedy` directly (FIX 1 adds a field).

---

## FIX 1 (L1, cernere — the substantive one) — make the retirement remedy's structured output honest

`src/remedy/retirement.rs` table maps `def-restricted`→`:wat::core::def` and
`defn-restricted`→`:wat::core::defn`. These are NOT pure renames: the retired forms carried a
caller whitelist that must be re-expressed as a `{:restricted-to [...]}` metadata-map on the
binding. The structured `Remedy.form` field (the home's RAISON D'ÊTRE per 241.10 — structured
data for programmatic consumers: LLM agents, IDEs) names only `:wat::core::def`, silently
dropping the restriction. The current NOTE at retirement.rs:~69-73 DOCUMENTS this gap rather
than closing it — a documented-non-fix (`feedback_dont_document_non_fixes`). Close it: carry
the caveat IN the structured data via an optional note.

**1a. `src/remedy/retirement.rs`** — widen the table to carry an optional note:
```rust
pub(crate) const RETIREMENT_TABLE: &[(&str, &str, Option<&str>)] = &[
    (":wat::core::struct",            ":wat::core::defstruct", None),
    (":wat::core::struct-restricted", ":wat::core::defstruct", None),
    (":wat::core::enum",              ":wat::core::defenum",   None),
    (":wat::core::define",            ":wat::core::defn",      None),
    (":wat::core::Char",              ":wat::core::char",      None),
    (":wat::core::def-restricted",    ":wat::core::def",
        Some("re-express the caller restriction as a `{:restricted-to [...]}` metadata-map on the binding")),
    (":wat::core::defn-restricted",   ":wat::core::defn",
        Some("re-express the caller restriction as a `{:restricted-to [...]}` metadata-map on the binding")),
    (":wat::core::def-dispatch",      ":wat::core::defclause", None),
    (":wat::core::try",               ":wat::core::Result/try",       None),
    (":wat::core::option::expect",    ":wat::core::Option/expect",    None),
    (":wat::core::result::expect",    ":wat::core::Result/expect",    None),
];
```
Update `retirement_lookup` to return `Option<(&'static str, Option<&'static str>)>` (the
replacement form + its optional note) instead of `Option<&str>`. Update the module doc: the
prior NOTE at ~69-73 stops being a non-fix-defense — reword it to state that the caveat is now
CARRIED in the remedy's `note` field (not just the error reason). Keep the arc-history table.

**1b. `src/remedy/mod.rs`** — add the field to `Remedy`:
```rust
pub struct Remedy {
    pub form: String,
    pub score: u32,
    pub kind: RemedyKind,
    /// Optional migration caveat for replacements that need more than a form-swap
    /// (e.g. a retired restricted-def whose whitelist must be re-expressed as a
    /// `{:restricted-to [...]}` metadata-map). `None` for pure renames + all typos.
    pub note: Option<String>,
}
```
- `remedies_for` (mod.rs ~167): populate `note` from the `retirement_lookup` tuple's note
  (`.map(str::to_string)`).
- `render_remedies`: when a remedy has `Some(note)`, append it to that remedy's rendered line
  (e.g. `… [retirement replacement] — <note>`). Keep empty-list → `""` behavior.

**1c. `src/remedy/rank.rs`** — `nearest_match`'s `Remedy { ... }` construction (~59) gets
`note: None` (typos never carry a migration caveat).

**1d.** grep `Remedy {` and `Remedy{` across `src/` and `tests/` — any OTHER direct
construction site adds `note: None`. (Expected: only the two above; verify.)

---

## FIX 2 (L1, intueri) — distance.rs Wagner-Fischer baseline label is wrong

`src/remedy/distance.rs:~45` — the comment on `prev[j] = j` says `(baseline: j deletions)`.
The transformation direction is a→b; `prev[j]` = cost to turn empty-`a` into `b[0..j]` =
**j insertions** (line ~50 correctly says "i deletions" for the symmetric a→empty case). Fix:
`(baseline: j insertions)`.

---

## FIX 3 (L2 cluster, intueri + solvere — doc/naming honesty)

- **distance.rs:~27** — "treated as sequences of bytes" is false (impl is `.chars().collect::<Vec<char>>()`,
  i.e. Unicode scalar values; char-vs-byte is explicitly tested in rank.rs). Reword to:
  "treated as sequences of Unicode scalar values (`char`s), via `collect::<Vec<char>>()`." Drop "bytes".
- **mod.rs `Remedy.score` doc (~59-60)** — "Edit distance from the needle" is false for Retirement
  (score 0 is an ordering sentinel, not a distance). Reword: "Ranking score: Levenshtein distance
  for `Typo`; `0` (ordering sentinel, not a distance) for `Retirement`."
- **rank.rs:1 (module title)** — drop "candidate combination" (the body at ~17 says the combinator
  `remedies_for` lives in mod.rs). New title: "Ranking logic — threshold tuning, top-N capping, `nearest_match`."
- **rank.rs:~16 (module doc)** — `TOP_N` is called a "private helper"; it's a `const`. Reword:
  "One private function (`typo_threshold`) + one private constant (`TOP_N`)."
- **rank.rs intra-remedy import (~20, `use crate::remedy::distance::levenshtein`)** — siblings address
  each other via `super`, not the crate root (matches `retirement.rs`'s `use super::{...}`). Change to
  `use super::distance::levenshtein;`. grep rank.rs + mod.rs for any other `use crate::remedy::…`
  intra-home import and convert to `super::`.
- **mod.rs `kind_annotation` "retirement replacement" wording (~143)** — opaque (doesn't convey
  "authoritative rename" vs fuzzy guess). Reword the Retirement annotation to `[renamed from a retired form]`
  (Typo annotation `[typo, distance N]` stays).

---

## DO NOT TOUCH (rejected w/ reasoning + L3 — leave)

- **solvere L2 "extract render_remedies+kind_annotation → render.rs"** — REJECTED → L3. Four-questions:
  a module ROOT (`mod.rs`) legitimately holds its own public type (`Remedy`/`RemedyKind`), that type's
  inherent `Ord`/`PartialOrd` impls, and the home's primary API (`render_remedies`, `remedies_for`).
  `distance`/`rank`/`retirement` are the algorithm INTERNALS split out; `mod.rs` is the type+surface.
  That is a standard, cohesive Rust module shape — not a 4-strand smear. This is an architecture-taste
  call, not an honesty defect; do NOT extract render.rs.
- All temperare findings — cold path (remedy runs only on error/teaching), bounded N (≤TOP_N=5,
  ≤~200 candidates). L3. LEAVE (double `chars().count()`, `to_string()` on table hit, `format!` in
  `push_str`, full-sort-before-truncate — all sub-µs per error event, `feedback_let_need_reveal`).
- purgare L3 (`levenshtein` `pub(crate)`→`pub(super)` tighten; `PartialEq/Eq` only used in tests) —
  LEAVE.
- L3 doc nits (arc-history insertion-order vs chronological; `nearest_match` singular name;
  render spacing 1-vs-2 space) — LEAVE.

---

## VERIFY before returning — report EXACT numbers

- `cargo test --release --lib -p wat` (expect 890+ / 0 — note FIX 1 adds a field; if any test
  constructs `Remedy` it must be updated to compile)
- `cargo test --release --lib -p wat remedy`
- `cargo build --release --tests --workspace` (expect Finished)
- `cargo build --release -p wat`
- `cargo clippy --release -p wat 2>&1 | grep -c warning` (must not regress vs ~877)

Report: every file touched + one-line description; the new `RETIREMENT_TABLE` shape + the new
`Remedy` struct + the `render_remedies` note-append; the grep proving all `Remedy {` construction
sites carry `note`; the five gate numbers; explicit confirmation of ZERO git mutations. Raw report.
