# BRIEF — a cited line number must be inside the file it names

Six prose citations point at `wat/rete.wat:1508`. That file is 533 lines. Nothing in the repo checks
the line half of a `path:line` citation, and nothing scans `wat/` prose at all.

## Read in order

1. `tests/lint/no_stale_path_in_doc.rs:88` — `vec![root.join("src/rete")]`, the scanned root. And its
   resolver at `:58-60`: it answers *does this path exist*, never *is this line inside it*.
2. `wat/seq.wat:163` and `:262`, `wat/core.wat:1585` — three of the six citations.
3. `wat-tests/core/core-nth-differential.wat:6`,
   `core-stream-materializers-differential.wat:7`, `core-foldl-spec.wat:7` — **the three the work-list
   row never mentions.**
4. `wat-scripts/fixes/rete-oracle-sigil.wat:6,44,55` — the codemod that retired the name, and its
   rune. **This file is correct; do not touch it.** It is the evidence the name is retired rather
   than phantom.
5. `wat/rete/oracle/insert.wat:45` — where `:wat::rete::insert-all$oracle` actually lives.

## Driven by the orchestrator at HEAD `c22cfe6e3`

`grep -rc 'rete.wat:1508'` → **5 hits across 4 files**; `wc -l wat/rete.wat` → **533**.
`insert-all-spec` appears in **5 files** outside the codemod that retired it.

## The change

1. **Depth**: a `path:line` in a comment must resolve **and** the line must be ≤ that file's length.
   Failure names the cited line and the real length.
2. **Scope**: add `wat/` and `wat-tests/` to the scanned roots.
3. **Cure the six** by naming the live symbol `:wat::rete::insert-all$oracle` and **no line number**.
   A symbol cannot rot by line drift; two of C14's citations had.

⛔ **Do NOT gate retired names in prose.** `rete_names_in_wat_scripts_resolve` rules deliberately that
*"prose may name a retired form"* — accurate history is not a defect. You are checking that a cited
**location exists**, nothing more.

## Blast radius

`tests/lint/no_stale_path_in_doc.rs`, plus the six comment lines. **No `src/` logic, no `.wat` code.**

## STOP triggers

1. **If widening the roots reddens citations you did not expect**, stop and report the list before
   fixing any — a corpus-wide count is a finding, and it is the orchestrator's call whether this
   strike absorbs it.
2. **If the depth check requires reading every cited file on every run** and that is too slow for the
   lint budget, stop and report the measured cost.
3. **If a citation names a line in a file that legitimately changes length** (a generated file), stop
   and report — the rule may need an exemption shape.
4. **If curing a citation requires touching `.wat` code rather than a comment**, stop.

## Mutation proofs — run all four, report all four

1. ★ **Restore one `:1508` citation** → the gate REDs, naming the cited line **and** the file's real
   length. Without both numbers the message cannot be acted on.
2. ★ **Cite a line one past the end** of a real file → REDs. Proves the boundary is `≤ len`, not a
   loose heuristic.
3. **Cite the last line exactly** → passes. Proves the check is not off-by-one in the strict
   direction — a gate that rejects valid citations gets disabled by the next hand.
4. **Point the roots at a directory with no comments** → the gate must FAIL as vacuous, not pass.

Restore by **hash** — `git checkout <sha> -- <path>` STAGES.

## What to report

- The full list of citations the widened scope newly examines, and how many were already wrong.
- All four mutation results.
- The gate's runtime, and whether it needed a nextest budget.
- Scoped nextest `Summary` lines including `binary_id(wat::lint)`.
- **Anywhere this brief was thin or wrong. Be blunt.** Ten consecutive strikes had their ★ be a false
  claim in a file the brief said to trust — **eight were the orchestrator's own artifacts**, the most
  recent a variant table that described one fixture and was presented as covering two. Assume there
  is an eleventh.

Do not commit.
