# BRIEF — remedy/ WARD R4 — close the Eq/Ord L1 + L2 cluster

You are sonnet. The `src/remedy/` home's R3 re-cast came back 5/8 spells L1=0 L2=0, but
surfaced a genuine correctness L1 (Eq/Ord contract) that R3's `note` field exposed, plus an
L2 cluster. This R4 closes them so the home earns its `vigilatum`.

ZERO git mutations — NO commit/add/stash/reset, NO scratch files outside named edits.
`git status`/`git diff`/`git grep` READ-ONLY only. The orchestrator commits after re-casting
the vigilia. If you think a commit is needed, STOP and say so. (#1 rejection trigger.)

The working tree already holds R1+R2+R3 (uncommitted, by design). ADD R4 on top; do NOT
revert/stash. Files in scope: `src/remedy/{mod,retirement}.rs` + `src/check.rs`.

---

## FIX 1 (L1, sequi — std contract violation) — make Ord consistent with Eq

`src/remedy/mod.rs` — `Remedy` derives `#[derive(PartialEq, Eq)]` (compares ALL fields:
form, score, kind, note) but `Ord::cmp` compares ONLY `score` then `form`. This violates the
Rust std contract that `a == b` iff `a.cmp(b) == Ordering::Equal`: two remedies with the same
score+form but different `note` (or `kind`) are `!=` under Eq yet `Equal` under Ord — a latent
hazard if `Remedy` is ever a `BTreeMap`/`BTreeSet` key or sorted with dedup.

Equality SHOULD stay structural (two remedies differing in note ARE different — different
guidance). So fix the ORD side: make it a total order consistent with Eq by adding the
remaining fields as FINAL tiebreakers, AFTER score+form (so ranking semantics are unchanged —
score+form still decide every real case; kind+note only break ties between otherwise-identical
remedies, which essentially never occurs):

```rust
impl Ord for Remedy {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.score
            .cmp(&other.score)
            .then_with(|| self.form.cmp(&other.form))
            // Final tiebreakers: kind, then note — carry ZERO ranking meaning
            // (score+form decide all real cases); present solely so the total
            // order is consistent with the derived Eq (std contract: a==b iff
            // cmp==Equal). Without them, two remedies equal on score+form but
            // differing in kind/note would compare Equal yet be Eq-unequal.
            .then_with(|| self.kind.cmp(&other.kind))
            .then_with(|| self.note.cmp(&other.note))
    }
}
```
This requires `RemedyKind` to be `Ord`. Add `PartialOrd, Ord` to its derive:
`#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]`. (`Option<String>` is already `Ord`.)

Update the `// ─── Ordering ───` comment block: it currently says "RemedyKind does not
participate in ordering — kind is metadata, not a ranking axis." Replace with an accurate
statement: ranking is by score then form; kind and note are appended only as Eq-consistency
tiebreakers and carry no ranking meaning.

---

## FIX 2 (L2, cernere — instruction phrasing) — align the struct-restricted reason + note

The migration instruction appears in BOTH `src/check.rs`'s `struct-restricted` arm `reason`
AND the retirement-table `note` (this is by design — reason serves the human reading the error;
note serves programmatic consumers). But the two are phrased inconsistently (different articles,
different placeholder `[...]` vs `[<prefix-kw>...]`, "with metadata-map:" dangling fragment in
the reason). READ both yourself (check.rs struct-restricted arm ~6519-6525; the note in
retirement.rs ~63-64) and align them so they read consistently:
- Use the SAME phrasing in both for the migration instruction.
- In the check.rs `reason`, fix the awkward "use ':wat::core::defstruct' with metadata-map:
  re-express…" clause junction — make it read as one clean imperative (mirror how the
  `def-restricted` arm reads — read that arm first to match register; do NOT change the
  def-restricted arm).
- Keep both layers (don't delete one) — they serve different audiences; just make them consistent.

---

## FIX 3 (L2, intueri + conformare — arc-history table conceals migration) — surface notes in the table

`src/remedy/retirement.rs` — the doc-comment "Arc history" table (~lines 35-48) has columns
`| Entry | Stone | Retired | Replacement |`. The Replacement column for `struct-restricted`
reads just `defstruct` — identical in form to the pure-rename `struct → defstruct` — concealing
that it (like `def-restricted`/`defn-restricted`) requires migration work. The `def-restricted`/
`defn-restricted` rows already say `def + metadata-map :restricted-to`. Make `struct-restricted`
consistent: change its Replacement cell to `defstruct + metadata-map {:restricted-to / :field-metadata}`
(or similar brief phrasing) so a reader scanning the table sees it's not a pure rename. (Pure
renames keep their bare replacement cell.)

---

## FIX 4 (L2, purgare + solvere — doc/visibility contradiction) — RETIREMENT_TABLE visibility

`src/remedy/retirement.rs` — `RETIREMENT_TABLE` is declared `pub(crate)` but the module doc
(~line 13) says "One private constant: RETIREMENT_TABLE", and grep confirms NO consumer outside
`retirement.rs` (only `retirement_lookup` + the in-file tests read it; the special_forms.rs hit
is a comment). The doc and the visibility contradict. Tighten the declaration to private
(`const RETIREMENT_TABLE` — drop `pub(crate)`). Confirm it still compiles (the tests are in the
same file's `#[cfg(test)] mod tests` and reach it via `super::*`).

---

## FIX 5 (L2 — test gaps the note feature left) — complete the note + render coverage

- `src/remedy/retirement.rs` tests: add `defn_restricted_retirement_note_is_some` mirroring the
  existing `def_restricted_…` + `struct_restricted_…` tests. (Three restriction-bearing forms;
  R3 tested two — complete the triad.)
- `src/remedy/mod.rs` tests: the existing note-render test covers only the SINGLE-remedy branch.
  Add a test that renders a MULTI-remedy list where one carries `Some(note)`, asserting the
  ` — {note}` appears on that entry in the multi-branch output (covers `note_suffix`'s second
  call site).

---

## DO NOT TOUCH (rejected / L3 — leave)

- render.rs file-extraction — REJECTED-settled. LEAVE.
- "ctor" jargon in the migration text — L3 (Rust readers recognize it; expanding is optional). LEAVE.
- `levenshtein` `pub(crate)`→`pub(super)` — L3, LEAVE (only RETIREMENT_TABLE had the doc-contradiction; levenshtein's doc doesn't claim private).
- temperare cold-path allocs — L3, LEAVE.
- arc-history table stone-number ordering (242.1 mid-241) — L3 doc-only, LEAVE.
- typo `note: None` property test — L3, LEAVE (rank.rs hard-codes it; structurally guaranteed).

---

## VERIFY before returning — report EXACT numbers

- `cargo test --release --lib -p wat` (expect 894+ / 0 — your 2 new tests add to the count)
- `cargo test --release --lib -p wat remedy`
- `cargo build --release --tests --workspace` (expect Finished)
- `cargo build --release -p wat`
- `cargo clippy --release -p wat 2>&1 | grep -c warning` (must not regress vs ~877; the Ord
  change must not trip clippy — verify no `derive_ord_xor_partial_ord` or similar fires)

Report: every file touched + one-line description; the new `Ord for Remedy` impl + the
`RemedyKind` derive line; the aligned struct-restricted reason vs note (paste both); the table
Replacement cell change; the RETIREMENT_TABLE visibility change; the 2 new test names; the five
gate numbers; explicit confirmation of ZERO git mutations + only-in-scope files. Raw report.
