# BRIEF — close F2's bullets, and do not leave a corrected number behind


> ⛔ **THIS FILE WROTE THE CORRECTED NUMBER IT FORBADE, SIX TIMES.** Found by the rider,
> 2026-09-05. EXPECTATIONS row 1 says writing `39 of 487` *anywhere* fails the strike; DESIGN,
> BRIEF and EXPECTATIONS each wrote it twice, one of them in a table presenting it as measured
> truth. A dated draw-record is the charitable reading, and by F0's own argument those six
> sites rot within the month. **The reading is kept ONLY where it is stamped as a dated
> observation; the live figure is the command:**
>
>     `grep -rlc 'src/rete/kernel\.rs' docs/arc --include='DESIGN-STONE-*.md' | wc -l` of `find docs/arc -name 'DESIGN-STONE-*.md' | wc -l`

Five live prose claims plus 39 stones citing a file deleted 2026-08-20. Every one of the row's own
citations has drifted. F0 governs the counts; the deferred-34 lesson governs the paths.

## Read in order

1. `docs/arc/2026/06/278-rules-engine/VIGILIA-2026-08-30-WORK-LIST.md`, the **F2** block — the seven
   bullets. Two are already struck; **do not re-open them.**
2. `docs/arc/2026/06/278-rules-engine/strike-the-deferred-thirty-four/SCORE.md` — the path doctrine
   you are inheriting: verify or delete, **never re-point on a basename**, and the two shapes that
   trapped it (a plausible wrong target; a provenance clause whose basename resolves to the citing
   file itself).
3. `docs/arc/2026/06/278-rules-engine/strike-a-counter-with-no-unit/SCORE.md` — why three dated
   stones quoting 80,200 were **left alone**, and therefore why a dated *measurement* differs from a
   dead *pointer*.
4. `tests/lint/no_stale_path_in_doc.rs` — the gate. **It does not scan `docs/`**, which is why 39
   stones can name a deleted file. Establish whether extending it there is cheap; if it is not,
   **say so** rather than half-doing it.

## Driven by the orchestrator at HEAD `6db874fc9`

- **39 of 487 _(a dated reading, 2026-09-04 — derive it, do not quote it)_** stones name `src/rete/kernel.rs`; **26 carry a dated `Origin (` header**.
- Bullet 2 confirmed: the cited heading exists only in the source line and the work-list row.
- Bullet 3 live; the gate it says is absent is at `purity.rs:2115` (row said `:2093`).
- Bullets 4 and 7 live; both row citations drifted (`:391`→`:400`, `:419,446`→`:436,539`).

## The change

- **Counts become the command.** Bullet 6 must not become *"**39 of 487 _(a dated reading, 2026-09-04 — derive it, do not quote it)_**"*. Write the `grep`.
- **Paths: verify or delete.** `src/rete/kernel.rs` → `src/rete/kernel/` where the citation meant the
  module; verified-to-item or deleted otherwise.
- **The row's own citations become symbols**, not corrected line numbers.
- **Each cured bullet is struck in place**, in the work list, with its evidence — never appended below.

## STOP triggers

1. **If a stone's citation cannot be resolved to a specific successor**, delete the path and keep the
   prose. **Report every such row** — that list is worth more than the count cured.
2. **If curing a claim would change what a DATED stone recorded**, stop. A dated measurement is a
   record; you are removing dead pointers, not editing history.
3. **If extending the gate to `docs/` surfaces a large pile**, stop and report the count. **Do not
   start fixing it** — the deferred-34 fence exists because that call is the orchestrator's.
4. **If a bullet turns out already cured**, say so and strike it. An inherited row is a claim.

## Mutation proofs — run all three, report all three

1. ★ **Re-introduce one cured `src/rete/kernel.rs` citation** → if you extended the gate to `docs/`,
   it REDs. **If you did not extend it, say plainly that these cures are ungated** — that is the
   honest answer and it decides whether the class can regrow.
2. ★ **Replace one command-form count with a hard number** → nothing reds, and that is the point:
   **prove by demonstration that a number here is ungated and will rot silently.** Report it as the
   argument for F0 rather than as a passing test.
3. **Revert one symbol citation to a line number** → nothing reds either. Same demonstration: the
   symbol form is a discipline, not a wall, and the strike should say which of its cures are gated
   and which are conventions.

Restore by **hash** — `git checkout <sha> -- <path>` STAGES.

## What to report

- Per bullet: cured / already-cured / struck, with evidence.
- Per stone: re-pointed (to what, on what evidence) or path-deleted.
- **Which cures are GATED and which are conventions.** Be explicit; that is mutation 1–3's whole job.
- Whether extending the gate to `docs/` is cheap, and the pile if it is not.
- Scoped nextest `Summary` lines including `binary_id(wat::lint)`.
- **Anywhere this brief was thin or wrong. Be blunt.** Twelve consecutive strikes had their ★ be a
  false claim in a file the brief said to trust — **ten were the orchestrator's own artifacts**, the
  most recent an under-reported golden set that omitted a file the strike itself was editing. Assume
  there is a thirteenth.

Do not commit.
