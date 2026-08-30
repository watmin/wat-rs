# EXPECTATIONS — excursus 001 stone 2c: mem's `put` becomes a replace

**Written BEFORE the strike, 2026-08-30.** Blast radius below is **derived from the BRIEF's
(a)/(b)/(c)**, not written beside it.

## The scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | the re-put differential exists and runs | floor arm `wat::rete probe_ex001_reput_differential::…` | present, PASS |
| 2 | both backends now agree on re-put | the fixture's summary | `base=1:a;gsi=1:v9` on BOTH — was `MEM[base=2:a,a;gsi=2:v1,v9]` |
| 3 | mem drops the superseded row | mem's `put` reuses `key-hits-row?` | reused, not re-written |
| 4 | the GSI follows | mem's projection for the replaced row is the NEW one only | `gsi=1:v9`, not `v1,v9` |
| 5 | blast radius, per BRIEF (a)(b)(c) | `git status --porcelain` | `wat/query/mem.wat`, `wat/query.wat` (comment only), `DESIGN-store-contract.md`, two new test files, the SCORE. **`sqlite-store.wat` NOT touched.** |
| 6 | `put`'s signature is unchanged | `git diff -- wat/query.wat` | comment lines only — zero type/arity change |
| 7 | the surface states the rule | `put`'s doc-comment names replace-by-`(pk,sk)` | present |
| 8 | the design table is corrected | `DESIGN-store-contract.md:150` no longer says plain `INSERT` | corrected |
| 9 | floor | `./scripts/floor.sh; echo "FLOOR=$?"` | `FLOOR=0` |
| 10 | test count | vs 5096 | **5097** = +1; any other number needs explaining |
| 11 | stones 2 and 2b undisturbed | both arcs' arms | still PASS |

## Runtime prediction

**30–60 minutes.** The mem fix is one fold; the predicate exists. The doc edits are three
lines. The fixture is already written and committed — it needs its `:user::main` dropped and a
`.rs` harness. Verify tail ~1m20s build + ~5m floor.

## Trap-doors

1. **An existing test may depend on append semantics.** That is STOP-1, and it is the most
   likely red. Such a test was asserting a state DynamoDB cannot represent, so the fix is
   probably the test — but *which* test it is, is a finding worth naming, not a nuisance.
2. **Batch-internal duplicates.** Two rows with one `(pk,sk)` in a single `put` batch: the rule
   is last-wins. sqlite gets this free from `put-rows`'s recursion (each row clears then
   inserts). mem must not accidentally implement first-wins by dropping incoming rows instead
   of existing ones.
3. **mem derives GSI rows from `StoredRow`s** — it has no physical index table. So row 4 is
   satisfied *by construction* once row 3 is right. If row 4 fails while row 3 passes, the
   fixture is measuring something other than what it claims.
4. **`put` becomes O(rows × batch).** Finding 3 territory, already open, explicitly not this
   stone's to fix — but if the shape is worse than that, say so.

## Not in this stone

- **`sqlite-store.wat`.** It is the reference implementation and is correct.
- **Finding 3** (the O(rows × keys) shape). Still open, still stone 3's problem.
- **`wat/queue.wat`.** Stone 3, and it starts after this reports.
