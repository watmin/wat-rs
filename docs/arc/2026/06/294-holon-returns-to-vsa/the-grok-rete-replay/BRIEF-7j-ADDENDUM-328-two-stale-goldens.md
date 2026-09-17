# BRIEF 7j ADDENDUM — #328's two stale goldens: fold the regeneration into the step

**Read `BRIEF-7j-replay-batch-4j.md` first.** This addendum resolves a composition defect found at the
orchestrator's verification floor, after batch 4j reported complete. It supersedes only what it names.

## The defect

The floor at `24ca5ef5c` is **RED, 2 of 5735**:

```
FAIL wat::cli        pprintln_doc_row::doc_row_pprintln_matches_byte_golden
                     tests/cli/pprintln_doc_row.rs:32:5
FAIL wat::reflection probe_stone_metadata_of_whole_row::from_metadata_of_the_lookup_equals_the_entry_doc
                     tests/reflection/probe_stone_metadata_of_whole_row.rs:256:5
```

**One cause.** #328 (D6) rewrote the `#[wat_intrinsic(":wat::rete::step-payload")]` doc comment in
`src/rete/step_payload.rs`. Two goldens pin that doc's rendered text and were not regenerated:

| golden | pinned by | regenerate how |
|---|---|---|
| `tests/cli/pprintln_doc_row__step_payload.edn` | `include_str!` + plain `assert_eq!` against the **binary's stdout** | ⛔ **NO bless path.** `UPDATE_EDN=1` does nothing. Capture it: `target/release/wat tests/cli/pprintln_doc_row.wat > <golden>` |
| `tests/reflection/probe_stone_metadata_of_whole_row__step_payload_row.edn` | `wat::assert_edn_matches_file!` | `UPDATE_EDN=1 cargo nextest run --release -E 'test(from_metadata_of_the_lookup_equals_the_entry_doc)'` |

⚠ **The two mechanisms are different and the difference is load-bearing.** A blanket `UPDATE_EDN=1` run
silently leaves the first golden untouched while its test still fails — measured, 2026-09-16.

## This is a FOLD, not a repair commit

Both goldens are **main-only**: absent from grok's tip, absent from its entire branch history, never
touched by grok's own #328 (whose 12-file list is byte-identical to ours — nothing was dropped in the
landing). They were authored by main's own arc-277/296 work. This is the **#270 ledger-reseed class**: a
main-only artifact pinned to text a replayed step legitimately rewrote.

Per the fold rule (ruled 4-YES 2026-09-14): the defect belongs to **#328**, so it folds **into #328**, and
**#329 → #340 are rebuilt on top**. ⛔ **Never a repair commit after the batch.**

## Measured in advance: both regenerations are PROSE-ONLY

The orchestrator measured both before writing this, without mutating the tree:

- **Golden A** (stdout capture): 30 changed lines, **all inside the `:doc` block** — the new D6 paragraph
  and the new `Arguments:` list. `:added`, `:args`, `:ret`, `:examples`, `:purity`, `:determinism`,
  `:totality`, `:category` all byte-identical.
- **Golden B** (`UPDATE_EDN=1`): **exactly one changed line**, the `:doc` string. Every other key untouched.

⛔ **VERIFY THIS YOURSELF — DO NOT TAKE IT ON TRUST.** #328 changed real code as well as prose (it added
`CONSTRAINT_NOT_RENDERED` and `render_constraint_operand`, and dropped `classify_constraint_head` from the
imports), and these goldens pin **rendered values** too — `:constraints`, `:bindings`, `:pattern`,
`:examples`. If your regeneration moves any key other than `:doc`, that is a **behaviour change, not a
format fix**: **STOP and report it** with the verbatim diff. The orchestrator's own first attempt at this
check was rigged — it filtered out the very words D6 changes and came back blank — so the check that
matters is "which keys moved", asked without a filter.

## How to land it

1. Fold the two regenerated goldens **into #328's own commit** (detach → amend → rebuild descendants, or
   `git rebase --onto`; **never `git filter-branch`**, finding 35).
2. Rebuild **#329 → #340** unchanged on top.
3. **Prove the rebuild inert**: `git diff <old-tip> <new-tip>` must name only the two golden paths. Paste
   it. The batch's own earlier rebuild (for #327's wrapped verdict line) did exactly this and verified the
   resulting tree byte-identical — reproduce that standard.
4. ⛔ **Published history must not move.** `git merge-base --is-ancestor origin/replay/grok-rete HEAD`
   must still succeed and `git for-each-ref refs/original/` must be empty. #321–#323 are already pushed
   and must remain byte-identical.
5. #328's commit body states: which two goldens were regenerated, by which mechanism each, that both are
   prose-only with the key list you measured, and that they are main-only artifacts grok never carried.

## Rows this affects

`EXPECTATIONS-7j` is **not amended** (finding 34 — a contract is not edited after its results are seen):

- **E11** (the checkpoint) is the row that caught this; it stays the orchestrator's.
- **E19** gains a required disclosure: the SCORE must state that #328 regenerated two main-only goldens
  and why, in the row reporting #328 green.
- **E14** (no knowingly-red commit) still holds and is the reason this is a fold rather than a follow-up.

Nothing else in BRIEF-7j changes. After the fold, the orchestrator re-runs the floor.
