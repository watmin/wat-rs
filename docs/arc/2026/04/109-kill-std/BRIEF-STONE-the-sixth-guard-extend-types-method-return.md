# BRIEF — the sixth guard: `extend-type`'s method-member return slot

The last one. Same shape as the five already widened: **a slot that accepts a type KEYWORD and was
never taught the reference FORM `(Head :- [T …])`.** One tuple pattern, plus a golden recapture.

## ⚠ You inherit a PARKED branch, not a clean tree

Everything this stone completes already exists on `arc109-2iii-migrated-parked` (`438f3c35c`):
the whole ②-iii corpus migration (36 files, 865/865, 947 `:-` forms, idempotent) and five widened
guards. **Restore it to the WORKTREE without staging** — `git checkout <ref> -- <path>` STAGES what
it writes, and that cost me a bad commit to main today:

```bash
git restore --source=arc109-2iii-migrated-parked --worktree -- src/ wat/
git diff --cached --stat        # MUST be empty
```

Do not merge the branch. Do not revert the migration — it is the deliverable.

## The defect, exactly

`src/runtime.rs:8416-8419`, inside `parse_extend_type_form`:

```rust
let (body_forms, clause_return_type) = if body_items.len() >= 3 {
    if let (WatAST::Symbol(arrow, _), WatAST::Keyword(ret_kw, _)) =   // ← Keyword ONLY
        (&body_items[0], &body_items[1])
    {
        if arrow.as_str() == "->" {
            let ret = crate::types::parse_type_expr(ret_kw)…
            (body_items[2..].to_vec(), ret)
```

The arrow-strip is a tuple pattern requiring `(Symbol("->"), Keyword(ret))`. After migration the
return is a `List`, the pattern does not match, **the strip is skipped entirely, and `->` stays in
the body and gets evaluated:**

```
unbound symbol: ->        wat/seq.wat:88
(seq [self] -> (:wat::stream::Stream :- [T]) (:wat::core::seqable->stream self))
```

⚠ **It fails at DISPATCH, not at load.** `--check` is clean, the stdlib loads, and
`every_wat_scripts_file_loads` is 398/398 green with this defect present. Only running the code
finds it. That is why the previous round declared the bottom reached — see the terminal check below.

## The work

Teach that pattern to accept a `List` return as well as a `Keyword`, through
`crate::types::parse_type_node` — **the existing door**, the same one the other five now use
(`src/function/parse.rs:178` is γ-i's exemplar; `src/types/surface.rs:345` calls it *"the substrate's
one door that reads all four type node shapes"*). Additive only: the `Keyword` arm keeps
`parse_type_expr` and stays byte-identical.

⚠ Note the existing `unwrap_or_else(|_| TypeExpr::Path(":wat::core::nil"))` fallback on the keyword
path. Decide deliberately whether the List path shares it or propagates the error, and **say which
you chose and why** — a silent `nil` fallback on a malformed form would hide exactly the class this
arc has been digging out all day.

## Then the goldens — scoped, and read before you trust them

About five `wat::diagnostics probe_diagnostic_value_snapshot_in_errors::*` goldens pin a Rust source
line: they expect `src/runtime.rs:25463`, and the edits shifted it to `25559`. Mechanical drift.

```bash
UPDATE_EDN=1 cargo nextest run --release -E 'test(probe_diagnostic_value_snapshot_in_errors)'
```

⚠ **`UPDATE_EDN=1` rewrites every golden the selected tests touch, including already-passing ones.**
It has produced spurious re-pretty-printed diffs twice in this repo. Scope the filter, then
`git status` and revert any `.edn` you did not intend. **Read each captured golden** — a capture
records whatever happened, including a wrong thing.

## STOP triggers — ship nothing further and report

- **STOP-1 — if a SEVENTH guard appears**, fix it if it is the identical keyword-only-type-slot
  shape, and STOP and report if it is anything else. Six rounds have each found exactly one more;
  do not assume this is the last.
- **STOP-2 — if any failure is NOT a keyword-only slot and NOT golden line-drift**, STOP and report
  the full list before touching it.
- **STOP-3 — if widening changes what the KEYWORD path accepts or rejects**, STOP. Additive only.

## Acceptance

| # | what | expected |
|---|---|---|
| 1★ | the sixth guard takes a form | `probe_stone_118_b2c_surface_arm_never_dispatches` — all 8 pass |
| 2★ | the seq cluster | `binary_id(wat::kernel)` — the ~24 `seq_walkers` / `seqable` / `foldl_spec` deftests pass |
| 3★★ | the KEYWORD path is untouched | an `extend-type` impl written `-> :wat::core::i64` still dispatches |
| 4 | goldens recaptured | `probe_diagnostic_value_snapshot_in_errors` green; no unintended `.edn` in `git status` |
| 5 | the migration is intact | `git diff --stat wat/` still 36 files, 865/865 |
| 6 | clippy | 0 under `-D warnings` |

**Row 3 decides it**, as it has for every guard in this sequence: rows 1 and 2 go green for a pattern
that accepts anything in that slot. Only the keyword path still behaving proves the widening did not
become a hole.

## ⛔ THE TERMINAL CHECK IS BEHAVIOUR, NOT LOAD

The previous round reached a clean `--check`, a clean stdlib load, and 398/398 on
`every_wat_scripts_file_loads`, and reported the waterfall bottomed out. It had bottomed out the
LOAD waterfall. This guard was sitting underneath, invisible to every load-time instrument.

**So your terminal check is the scoped test runs in rows 1-2, not "it loads."** Run them. If they
pass and you believe you are done, say explicitly which instrument you used and what it cannot see.

## Boundaries

- `src/runtime.rs` (the one pattern), the `.edn` goldens, and the restored migration.
- Do NOT hand-edit any `.wat` under `wat/` — R21. The migration is already applied; a file needing a
  change the codemod does not make is a finding.
- Do NOT run `scripts/floor.sh` or a full `cargo nextest` — I measure centrally. Your checks are the
  scoped `binary_id(wat::types)` / `binary_id(wat::kernel)` / `binary_id(wat::diagnostics)` runs.
- Do NOT commit, push, stash, revert or amend. Keep the index EMPTY.

Prefix long commands with `systemd-run --user --scope -q -p MemoryMax=16G -p MemorySwapMax=0 timeout 1800`.
Read exit codes DIRECTLY — never through a pipe, never after a trailing `; echo`.

## Your report

Rows 1-3 together with verbatim output, row 3 especially. What you chose for the `nil`-fallback
question and why. Every golden you recaptured, with its content. Confirmation that `git diff --cached
--stat` is empty and the migration is still 36/865/865. Whether a seventh guard appeared. What
surprised you.
