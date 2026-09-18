# BRIEF — sort check errors into source order at their one exit

Two quarantined fixtures still emit the same four errors in a different order every run. The errors
are collected per function; the function map is a `HashMap`. Sort at the exit.

## Read in order

1. `src/check.rs:744-747` — the single site every check error returns through. **This is where the
   sort goes.**
2. `src/check/error.rs:24-27` — `CheckError { span: Span, kind: CheckErrorKind }`. The sort key is
   already on the struct.
3. `src/value/symbol_table.rs:34` and `:282` — `functions: HashMap<String, Arc<Function>>` and
   `functions_iter()`. **The root. Do not change it** — it is a hot lookup path.
4. `src/check.rs:649` and `:738` — the two `for (name, func) in sym.functions_iter()` walks that
   collect per-function errors in map order.
5. `tests/lint/diagnostic_output_is_deterministic.rs` — the `QUARANTINE` table and `QUARANTINE_LEN`.
   Its header explains why it pins the list rather than asserting the files are still broken; read
   that before touching either.

## Driven by the orchestrator at HEAD `645f219c4`

16 runs each: `c2_mixed_macro_swap` **5/11**, `w2a_kwargs_check_mint_swap` **8/8**. Same four errors,
identical spans and messages; the line-40 error moves between first and last position because it
belongs to a different function than the line-48/51 ones.

## The change

Sort `errors` by span before `Err(CheckErrors(errors))`.

⚠ **The sort must be TOTAL.** A key of `(line, col)` alone leaves same-span ties in input order —
which is hash order — so the randomness would survive for any program with two errors at one span.
**Break every tie deterministically** (file, then line, then col, then end, then a stable
discriminant of the kind), and **prove the tie-break is reachable** rather than assuming no ties
exist. If you cannot construct a same-span pair, say so — that is a corpus finding, not a pass.

## Blast radius

`src/check.rs` at the return site, plus `tests/lint/diagnostic_output_is_deterministic.rs` to move
`QUARANTINE_LEN` 2 → 0 **once the gate proves it**. **No change to `SymbolTable`.**

## STOP triggers

1. **If sorting changes which errors are reported** (not merely their order), stop — you have altered
   check semantics, which this strike must not do.
2. **If any golden `.edn` moves beyond reordering**, stop and report it.
3. **If the two fixtures still vary after the sort**, stop and report the remaining variance — there
   is a second root and it is a finding.
4. **If this needs `SymbolTable.functions` changed**, stop. That is a hot path and C10's ruling
   applies.

## Mutation proofs — run all four, report all four

1. ★ **Remove the sort** → both fixtures vary again over **24 runs**. ⚠ At p≈0.5, two runs miss a
   flip half the time; C19's sweep needed 24 runs/file and this arc has a memory about it.
2. ★ **Make the sort key partial** — key on `(line, col)` only, then construct or find a same-span
   pair → order varies again. **This is the row that proves the tie-break is load-bearing.** If no
   same-span pair can be constructed, report that instead of skipping the mutation.
3. **Reverse the sort** → the goldens move in a way that is visibly wrong (errors in reverse source
   order), proving the assertion reads the order rather than the set.
4. **Restore a quarantine row while the files are fixed** → show what the gate does. It pins a
   length, so it should RED on the count; say plainly whether it can distinguish "cured" from
   "still broken" — its own header says it cannot, and that answer is fine.

Restore by **hash** — `git checkout <sha> -- <path>` STAGES.

## What to report

- 24-run stability for both fixtures, before and after.
- All four mutation results; for mutation 2, whether a same-span pair exists.
- Every golden `.edn` that moved, and confirmation each moved only in order.
- Whether `QUARANTINE_LEN` can honestly go to 0, and on what evidence.
- Scoped nextest `Summary` lines including `binary_id(wat::lint)`.
- **Anywhere this brief was thin or wrong. Be blunt.** Nine consecutive strikes had their ★ be a
  false claim in a file the brief said to trust — **seven were the orchestrator's own artifacts**,
  the most recent naming a measurement instrument that structurally could not see the change it was
  asked to measure. Assume there is a tenth.

Do not commit.
