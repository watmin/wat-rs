# BRIEF — STONE `sort'` → `sort$native`

Rename one verb, `:wat::core::sort'` → `:wat::core::sort$native`, across the `.wat` corpus (via the
codemod) and five Rust sites; retire the five `rune:lint(retired-name)` exemptions the old spelling
earned; add one `RETIREMENT_TABLE` row so the old spelling teaches its remedy. The public `sort` /
`sort-by` surface does not move. DESIGN:
`docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-sort-prime-becomes-sort-native.md`.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, and no notification is coming.
Make text edits and report. The orchestrator builds, floors and clippies centrally; you do not run
cargo. **You may not spawn sub-agents.** Work only in `/home/john/work/holon/wat-rs`; verify with
`pwd` first and use `git -C /home/john/work/holon/wat-rs` for any git read. Do not commit, push,
stash, revert, or `git checkout --` anything.

## Read in order

1. `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-sort-prime-becomes-sort-native.md`
   — the scope, the pinned contract decision, and what is affirmatively cut.
2. `wat-scripts/fixes/reclaim-ipc-prime-names.wat` — **copy this as the shape** for step 1. It is the
   recorded prime-renaming codemod: a `:user::renames` table of `(Tuple "<old>" "<new>")`, a
   `:user::migrate` fold applying `:wat::fix::rename-keyword-prefix` per row, `:user::apply-each`
   writing each file, and a `:user::main` reading the path list from stdin.
3. `wat/core.wat:1510-1548` — the ordering surface. Four `sort'` call sites (`:1522 :1530 :1537
   :1546`) plus two `;;` prose lines (`:1513-1514`). This is the **only** corpus caller.
4. `src/remedy/retirement.rs:97-120` — `RETIREMENT_TABLE` rows, for the format.

## The work

### 1 — the codemod (the `.wat` side)

Write `wat-scripts/fixes/rename-sort-prime-to-native.wat`, mirroring reference (2), with a
one-row rename table:

```
(:wat::core::Tuple ":wat::core::sort'"  ":wat::core::sort$native")
```

`rename-keyword-prefix` matches from the START of a keyword and is boundary-aware, so this cannot
touch `:wat::core::sort` or `:wat::core::sort-by`. It stays idempotent: after the run no keyword
begins with `:wat::core::sort'`, so a re-run matches nothing.

**Dry-run first:** copy the two target files to a `/tmp` scratch dir, run the codemod against the
copies, and `diff` them against the originals. Confirm the diff is exactly the intended keyword
rename and nothing else moved. Then apply for real:

```
printf '["wat/core.wat" "wat-scripts/scratch-pad/255-probe-can-a-user-make-sort-effectful.wat"]\n' \
  | cargo wat ./wat-scripts/fixes/rename-sort-prime-to-native.wat
```

Then re-run it once and confirm it reports zero changes (idempotence).

`fix-source` walks the FORM tree, so `;;` comment prose is invisible to it. After the codemod,
`wat/core.wat:1513-1514` will still read `sort'` in its prose — update those two comment lines to
`sort$native` as prose. That is comment text, not a structural rewrite.

### 2 — the five Rust sites

Each currently carries a `// rune:lint(retired-name) — live prime …` exemption. Rename the string
and **delete the rune comment** at each: the lint only fires on a `word'` shape inside a Rust string
literal, so `sort$native` produces no hit and the exemption is no longer earned.

```
src/collection/transform.rs:282   const OP: &str = ":wat::core::sort'";
src/runtime.rs:6023               ":wat::core::sort'" => {
src/check.rs:20272                ":wat::core::sort'".into(),
src/macros/eval.rs:505            | ":wat::core::sort'"
src/rete/purity.rs:2046           ":wat::core::sort'",
```

Also update the surrounding doc/comment prose that names the old spelling in those same files
(`transform.rs:4,253,255,294`; `check.rs:20267-20269`; `macros/eval.rs:451`). Where a comment records
the arc-251 rename as **history**, keep the historical sentence accurate — say that arc 251 named it
`sort'` and this stone renamed it to `sort$native`; do not rewrite history into a claim it was always
`sort$native`.

### 3 — the retirement row

Add one row to `RETIREMENT_TABLE` in `src/remedy/retirement.rs`, in the format the neighbours use:

```rust
// Arc 255 STONE sort$native — the `'` native-impl marker becomes the `$native` convention.
RetirementEntry { retired: ":wat::core::sort'", replacement: ":wat::core::sort$native", note: None },
```

## Blast radius

`wat/core.wat` · `wat-scripts/scratch-pad/255-probe-can-a-user-make-sort-effectful.wat` ·
`wat-scripts/fixes/rename-sort-prime-to-native.wat` (new) · `src/collection/transform.rs` ·
`src/runtime.rs` · `src/check.rs` · `src/macros/eval.rs` · `src/rete/purity.rs` ·
`src/remedy/retirement.rs`. No new types. No signature changes. No registry registration.

## STOP triggers — each rejects; ship nothing and report

**STOP-1 — the grammar examples are NOT this verb.** `tests/lint/one_name_grammar.rs:129-130` and
`crates/wat-reader/src/identifier.rs:228,229,267,275,377,378,421,427` use a bare `:sort'` as the
worked EXAMPLE for the `prime` / `deprimed` / `receiver` helpers — they illustrate what a primed
name *is*, and are not references to `:wat::core::sort'`. Leave every one of them exactly as it is.
If your edit would land in either file, STOP and report.

**STOP-2 — history stays.** `wat-scripts/fixes/reclaim-service-fixture-names.wat:19` names `sort'`
in the prose of an already-recorded migration. That is a historical record. Do not touch it.

**STOP-3 — the codemod is the only way in for `.wat` forms.** If the codemod cannot perform the
rename (a missing `fix.wat` primitive, a dry-run diff that changes anything beyond the intended
keyword, or non-idempotence on the second run), STOP and report what you observed. Do not hand-edit
`wat/core.wat`'s forms, and do not reach for sed/python on it.

**STOP-4 — a rune you cannot retire.** If any of the five sites still produces a retired-name hit
after the rename, STOP and report which and why, rather than restoring the rune.

## Report

Give the diff summary per file, the dry-run diff you saw before applying, the second-run
idempotence result, and — the part the orchestrator cannot reconstruct — anything that surprised
you: a site the DESIGN did not predict, prose that turned out to be load-bearing, or a place where
the rename read wrong.
