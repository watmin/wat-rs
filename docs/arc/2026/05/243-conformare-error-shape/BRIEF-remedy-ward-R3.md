# BRIEF — remedy/ WARD R3 — close the L1 (struct-restricted note) + L2 cluster

You are sonnet. The `src/remedy/` home's R2 re-cast came back 5/8 spells L1=0 L2=0; the
structural core is clean. R2's new `note` field *exposed* its own sibling gap — two spells
(cernere + conformare) independently caught it. This R3 closes the L1 + the L2 cluster so the
home earns its `vigilatum`.

ZERO git mutations — NO commit/add/stash/reset, NO scratch files outside named edits.
`git status`/`git diff`/`git grep` READ-ONLY only. The orchestrator commits after re-casting
the vigilia. If you think a commit is needed, STOP and say so. (#1 rejection trigger.)

The working tree already holds R2 (uncommitted, by design). ADD R3 on top; do NOT revert/stash.
Files in scope: `src/remedy/{mod,rank,retirement}.rs` + `src/check.rs` + a test in
`src/remedy/mod.rs` and/or `src/remedy/retirement.rs`.

---

## FIX 1 (L1, cernere + conformare — the substantive one) — struct-restricted silent whitelist drop

`src/remedy/retirement.rs` — the entry `(":wat::core::struct-restricted", ":wat::core::defstruct", None)`
carries `note: None`. But `struct-restricted` had ctor + per-field caller whitelists
(`StructRestrictions.ctor_whitelist` + `field_restrictions`, types.rs) that a plain swap to
`defstruct` SILENTLY DROPS — the exact class the `note` field was minted to fix (and that
`def-restricted`/`defn-restricted` already carry). It was missed because `note` didn't exist
when this entry was written. Add the note:
```rust
(":wat::core::struct-restricted", ":wat::core::defstruct",
    Some("re-express the ctor restriction as a `{:restricted-to [...]}` metadata-map, and per-field restrictions as `{:field-metadata {field {:restricted-to [...]}}}`, on the defstruct binding")),
```
(Verify the exact `defstruct` metadata keys against `src/types.rs` defstruct-parser — use the
real key names; the brief's `:restricted-to` / `:field-metadata` match what types.rs accepts,
but confirm by grep before writing.)

## FIX 1b (the coupled L3, fold in) — split the shared check.rs match arm so struct-restricted gets a real reason

`src/check.rs:~6507` — `":wat::core::struct" | ":wat::core::struct-restricted" =>` share ONE arm
producing the same bare `reason: "'{}' is retired (Stone 241.8)"`. That shared arm is *why* the
struct-restricted migration guidance had nowhere to live. Split it: give `struct-restricted` its
own arm whose `reason` names the metadata-map migration (mirror how `def-restricted`'s reason at
check.rs ~5092 gives concrete syntax). Plain `struct` keeps the bare reason (it IS a pure rename).
Both still call `remedies_for(k, std::iter::empty())` so the remedy note rides along too. Keep the
runtime-dispatch arm (~6580) untouched.

---

## FIX 2 (L2, cernere — directional inversion I introduced in R2) — fix the retirement annotation

`src/remedy/mod.rs:158` — `kind_annotation` renders Retirement as `"renamed from a retired form"`.
This is DIRECTIONALLY BACKWARDS: it reads "the suggested form was renamed FROM a retired form"
(i.e. the candidate used to be the retired thing). The truth is the opposite — the retired form
was superseded BY this candidate; `:wat::core::def` was never "renamed from" `def-restricted`.
Change to:
```rust
RemedyKind::Retirement => "replaces a retired form".to_string(),
```
Update any test asserting the old string (grep `renamed from a retired form` across src/ +
tests/ — the render test in mod.rs asserts it). Also update the format-rules comment (FIX 4).

---

## FIX 3 (L2, intueri) — rank.rs threshold-formula doc says len() but impl is chars().count()

`src/remedy/rank.rs:24` — the doc says `Formula: \`max(1, needle.len() / 3)\`` but the impl
(rank.rs:31) is `max(1, (needle.chars().count() / 3))` — char count, not byte len (the
char-vs-byte distinction is load-bearing + tested). Fix the doc:
```
/// Formula: `max(1, needle.chars().count() / 3)`. Longer identifiers tolerate more
```

## FIX 4 (L2, intueri) — stale format-rules comment in mod.rs

`src/remedy/mod.rs:~121` — the format-rules comment block still lists
`- Retirement: "[retirement replacement]"` (the pre-R2 string). After FIX 2 the rendered
annotation is `"replaces a retired form"`. Update the comment line to:
`- Retirement: "[replaces a retired form]"`. (And confirm the Typo line in that block is current.)

---

## FIX 5 (L2, struere) — extract the note-append helper (symmetry with kind_annotation)

`src/remedy/mod.rs` `render_remedies` — the `if let Some(note) = &r.note { …push_str(format!(" — {}", note)) }`
block is duplicated verbatim in BOTH the single-remedy branch (~136) and the multi-remedy loop
(~145). `kind_annotation(r)` already established the per-remedy-rendering-helper pattern; the
note-append should match it. Extract:
```rust
/// The ` — <note>` suffix for a remedy that carries a migration caveat; empty when None.
fn note_suffix(r: &Remedy) -> String {
    match &r.note {
        Some(note) => format!(" — {note}"),
        None => String::new(),
    }
}
```
Both branches become a single `.push_str(&note_suffix(r))` (or inline into the format). Removes
the duplicated `if let`.

---

## FIX 6 (L2 — the R2 feature has zero test coverage) — add note tests

R2's `note` field + its rendering have NO test. Add (in the appropriate `#[cfg(test)] mod tests`):
- In `retirement.rs` tests: assert `retirement_lookup(":wat::core::def-restricted").unwrap().note`
  is `Some(...)` (and the struct-restricted one from FIX 1 too).
- In `mod.rs` tests: a `render_remedies` test that builds a `Remedy { note: Some("X".into()), .. }`
  and asserts the rendered string contains `" — X"` (covers the note-append branch).
Keep tests minimal + structural.

---

## DO NOT TOUCH (rejected / L3 — leave)

- render.rs extraction — REJECTED-settled (module root holds its type + impls + API). LEAVE.
- `RETIREMENT_TABLE`/`levenshtein` `pub(crate)`→`pub(super)` tighten — L3, LEAVE.
- temperare cold-path allocs (chars-collect, format! in render, linear table scan) — L3, LEAVE.
- `note` field-name "too generic" — the doc disambiguates; L3, LEAVE.
- arc-history table "no note column" — doc-only, L3, LEAVE.

---

## VERIFY before returning — report EXACT numbers

- `cargo test --release --lib -p wat` (expect 890+ / 0 — your new tests add to the count)
- `cargo test --release --lib -p wat remedy`
- `cargo build --release --tests --workspace` (expect Finished)
- `cargo build --release -p wat`
- `cargo clippy --release -p wat 2>&1 | grep -c warning` (must not regress vs ~877)

Report: every file touched + one-line description; the struct-restricted table entry after FIX 1;
the split check.rs arms (struct vs struct-restricted reasons); the new annotation string; the
`note_suffix` helper + both call sites; the new test names; the five gate numbers; explicit
confirmation of ZERO git mutations + that you edited only the in-scope files. Raw report.
