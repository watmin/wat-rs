# BRIEF — empty the DEFERRED fence, one verified citation at a time

34 dead path citations sit fenced in a `DEFERRED` const. Re-point the ones whose target you can
**prove** is the same artifact; delete the path from the rest. Shrink the const to zero, then remove it.

## Read in order

1. `tests/lint/no_stale_path_in_doc.rs` — the `DEFERRED` const (the population, 34 exact
   `(naming file, cited path)` pairs) and its own rule: **a row that stops matching is a hard
   failure**, so the list can only shrink. Removing a row is how you record a cure.
2. `docs/arc/2026/06/278-rules-engine/strike-a-cited-line-must-exist/SCORE.md` — why these were
   fenced, and the four goldens that pin `wat/core.wat` line numbers.
3. `tests/macros/probe_arc279_format__*.edn`, `probe_arc258_stone2b_*`, `probe_arc249_threading__*` —
   the four goldens. **Read one** so you know what a pinned line looks like before you edit a `.wat`.

## Driven by the orchestrator at HEAD `5aa25e0c4`

Each cited basename searched tree-wide: **12 rows have a same-named file somewhere (11 distinct
targets); 22 have none.** ⚠ That split is a **starting hint**, not the answer — see the trap below.

## The change

- **Re-point** where you can confirm the target is *the same artifact the sentence is about*.
- **Delete the path** where it is gone, keeping whatever the sentence still says truly.
- **Prefer a symbol to a path** where the sentence allows — a symbol survives a move.
- Remove each cured row from `DEFERRED`. **The const ends empty and is then deleted.**

## ⛔ STOP triggers

1. **⛔ A BASENAME MATCH IS NOT A VERIFICATION.** `wat/rete.wat` cites `kernel/tests.rs`, which
   basename-resolves to `src/macros/tests.rs` — almost certainly **wrong**; the rete kernel's tests
   were `src/rete/kernel/tests.rs`, deleted 2026-08-20. **Re-pointing on a name match replaces a dead
   pointer with a confident wrong one, which is worse.** If you cannot confirm the target is the same
   thing, treat the row as GONE and delete the path. **Report every row where you made that call.**
2. **If an edit changes a file's line count and any golden pins that file**, stop unless you can make
   it line-count-neutral. **Determine which files are pinned — do not assume `wat/core.wat` is the
   only one.**
3. **If the walk surfaces stale paths outside the const**, stop and report the list. That is a
   finding, not scope to absorb.
4. **If a citation's surrounding sentence becomes false once the path goes**, stop and report it —
   the prose may be asserting something that died with the file.

## Mutation proofs — run all three, report all three

1. ★ **Re-introduce one cured citation** → the gate REDs naming it. Proves each cure is actually
   gated and not just deleted text.
2. ★ **Leave a `DEFERRED` row in place after curing its citation** → the gate REDs with *"no longer
   match anything the walk found"*. Proves the const cannot rot into a stale allowlist.
3. **Empty the const while a stale path remains** → REDs. Proves removal is earned, not clerical.

Restore by **hash** — `git checkout <sha> -- <path>` STAGES.

## What to report

- Per row: re-pointed (with the evidence the target is the same artifact) or deleted (with why).
- **Every row where a basename matched but you judged it not the same thing** — that list is the
  strike's most valuable output.
- Which files are golden-pinned, and whether any edit needed to be line-count-neutral.
- All three mutation results.
- Scoped nextest `Summary` lines including `binary_id(wat::lint)`.
- **Anywhere this brief was thin or wrong. Be blunt.** Eleven consecutive strikes had their ★ be a
  false claim in a file the brief said to trust — **nine were the orchestrator's own artifacts**, the
  most recent a mandated cure that would have propagated a false attribution into three files.
  Assume there is a twelfth.

Do not commit.
