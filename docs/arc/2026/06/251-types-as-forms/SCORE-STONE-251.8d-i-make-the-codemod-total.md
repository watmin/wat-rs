# SCORE — STONE 251.8d-i: make the codemod TOTAL (zero corpus writes)

Branch: `main`. **Committed, not pushed.** Stone `22118e145`.
Drawn against `817c85213`. Parent: `BRIEF-STONE-251.8d-i-make-the-codemod-total.md`.
Floor / workspace clippy / `census.sh --diff`: orchestrator's row (not run).
8d-ii and 8d-iii not started.

## Gate: census on copies is 0 / 0 / 0

One process per file, copies under `/tmp/8d-i/census/tree`. Timed one file first: **0.28 s**.
Full run 2145 files (the 2140 `(:wat::` heads plus this stone's 4 new fixtures and one extra
on the tree). Second pass on the already-converted copies.

| class | files |
|---|---|
| **OK** | **2145** |
| A — `ast-name` partiality | **0** |
| B — refusing to splice | **0** |
| C — namespace-prefix marker | **0** |
| other | **0** |

Idempotence: **second-pass-changed=0**. Every copy **would change** on the first pass (2145 / 2145).
That is the flip's real size.

Timed one item first (`tests/kernel/wat_arc198_def_restricted.wat` copy): 0.28 s, rc=0,
`:my::kernel::` → `my.kernel`.

## ZERO tracked conversions

`git diff --stat -- '*.wat'` is **only** `wat/fix.wat` (the engine). Four **new** fixtures under
`tests/kernel/wat_arc251_8d_restricted_to_*.wat`. No corpus file was rewritten.

## The three cures — artifacts and the command

One command for all three, on an arbitrary path list (no hard-coded tree):

```
printf '["pathA" "pathB"]\n' | ./target/release/wat ./wat-scripts/fixes/to-faithful-clojure.wat
```

Must `cargo build --release` after editing `wat/fix.wat` (`include_str!`).

| class | artifact | what |
|---|---|---|
| **A** | `wat/fix.wat` `annotated-if?` | kind-guard `head` as `"keyword"` before `ast-name`, matching `c2`'s existing guard |
| **B** | `wat/fix.wat` `source-matches-name?` | edit a leaf only when span text **equals** `ast-name`; else skip |
| **C** | `wat/fix.wat` `marker-keyword?` + `marker-to-namespace-text`; `src/check.rs` + `src/types/defstruct.rs` | trailing-`::` keyword → namespace symbol; parser accepts Symbol and hard-errors on neither |

### Class B door — not the forbidden one

`source-matches-name?` calls `fix-text-span-text` to **compare** span text against `ast-name`, then
**declines**. It does not fill `old-text` from the span. `fix-text-apply`'s splice check is
untouched. A class-B copy (`tests/macros/probe_let_splice_enum_via_macro.wat`) converts rc=0.

### Class C — context-free, re-derived

Comment+string stripped, tracked `.wat`: **33** trailing-`::` tokens, **13** distinct spellings,
**0 outside** a `:restricted-to` vector. (Brief: 32 / 12 — the extra is this tree, not a
counter-example.) The rule is context-free: a keyword whose name ends in `::` converts
(`:my::kernel::` → `my.kernel`). Exact FQDNs (`:my::kernel::specific-caller`) never ended in
`::`; they already go through `keyword/to-symbol` → `my.kernel/specific-caller`.

Parser: `extract_prefix_list_from_metadata` and `parse_defstruct_metadata` accept Keyword **or**
Symbol; neither → `MalformedForm` / `MalformedDecl` (no silent drop). Matching discriminates on
`/` after canonicalize.

⚠ **Brief improvement:** "contains no `/`" is the prefix-vs-exact discriminator, but a prefix
must still admit **both** `ns/fn` and `ns.Type/method`. Canonical `wat.spawn` matches
`wat.spawn/foo` **and** `wat.spawn.ThreadOpts/spawn-runner` (the old keyword prefix
`:wat::spawn::` was a character prefix of both). Matching only `entry + "/"` reds the stdlib
(`ThreadOpts/spawn-runner`, `stdout-svc::init`). Landed as `entry/` **or** `entry.`.

## Test count

Predicted delta: **+6** (`+4` kernel fixtures, `+2` lib match tests).

- `tests/kernel/wat_arc198_def_restricted.rs` `#[test]`: HEAD 6 → 10
- `cargo nextest list --release -p wat`: **3997**
- `one_variant_separator::only_identifier_rs_spells_the_variant_separator` — 1 passed

## Walls I ran (not the floor)

- `cargo clippy --release --all-targets -p wat --offline -- -D warnings` — first pass
  `result_large_err` on `Result<_, CheckError>`; helpers now return `Result<_, Span>`. Second
  pass: **exit 0**. Did not `#[allow]`.
- `cargo test --release --offline --test kernel wat_arc198_def_restricted` — 10 passed
  (6 old + 4 new).
- `cargo test --release -p wat --lib restriction_entry_match_tests` — 2 passed.
- Class census on copies — table above.

Floor + workspace clippy + `scripts/replay/census.sh --diff`: **not run** (orchestrator,
uncontended). Do not push. Do not start 8d-ii.
