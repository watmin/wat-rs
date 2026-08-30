# EXPECTATIONS — arc 301 stone 2b: the delete differential

**Written BEFORE the strike, 2026-08-30.**

**Blast radius is DERIVED from the BRIEF, not written beside it.** The BRIEF instructs: two new
test files, no production file touched, no `mod.rs` edit. Row 3 below says exactly that and
nothing else. (Stone 2's row 4 contradicted its own BRIEF's promotion instruction — the
executor followed the BRIEF and reported the conflict. That was my drafting bug; this is the fix.)

## ⚠ The outcome is genuinely unknown

This stone measures code that has never run. **Two outcomes are both valid deliveries:**

- **AGREE** — the differential is green. Findings 1 and 2 close. Stone 3 proceeds.
- **DISAGREE** — the differential is red, or a GSI projection is orphaned. **That is a
  SUCCESSFUL stone**, and the SCORE records a substrate bug rather than a green number.

The failure mode is neither of those. It is a green result reached by **editing the backends
until they match** (STOP-1), or by **dropping the GSI half** (STOP-3) — either of which
produces a passing test that proves nothing.

## The scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | the differential runs at all | it appears in the floor as a `wat::rete probe_arc301_delete_differential::…` arm | present, not skipped |
| 2 | both backends actually ran | the fixture starts `mem-store` AND `sqlite-store` with `:path ":memory:"` | both, read from the fixture source |
| 3 | blast radius, per the BRIEF | `git status --porcelain` | exactly `tests/rete/probe_arc301_delete_differential.wat` + `.rs`, plus this arc's SCORE. **Zero files under `wat/`.** |
| 4 | ★ a GSI is actually declared | the fixture's `EnsureSchemaRequest` carries a non-empty `:indexes`, and rows carry `index-keys` | non-empty — an empty index set makes this stone vacuous |
| 5 | ★ `scan-index` is driven AFTER the delete | the fixture calls `scan-index` post-delete on both backends | present — this is the only thing that tests `clear-index-projections` |
| 6 | duplicate ack | deleting the same key twice returns `:Success` both times on both backends | agree (finding 2 closes) |
| 7 | the verdict is a SUMMARY, not a bool | the fixture returns a rendered string, not `true` | a string — two identically-broken backends must not pass |
| 8 | floor | `./scripts/floor.sh; echo "FLOOR=$?"` | `FLOOR=0` **if** the backends agree; a named red arm if they do not |
| 9 | test count | floor total vs 5095 | 5095 + 1 = **5096** if one test is added; any other number needs explaining |
| 10 | stone 2 not disturbed | `probe_arc301_store_delete::store_delete_removes_exactly_the_named_row` | still PASS |

## Runtime prediction

**30–60 minutes.** No production code. The template is a direct copy and both `start` calls are
one line each. The bulk is the roundtrip helper: ensure-schema-with-index, three rows carrying
`index-keys`, delete, two scans, a second delete, and rendering a summary. Verify tail is
~1m20s build + ~5m floor.

## Trap-doors — named now

1. **An empty `:index-names` on `sqlite-store::Record` silently skips
   `clear-index-projections`.** `wat/query/sqlite-store.wat:155` returns `Ok` immediately on an
   empty name vector. So a fixture that forgets the index passes *and proves nothing about
   STOP-2*. Rows 4 and 5 exist solely to catch this; if they cannot both be satisfied, the
   stone has not been struck.
2. **`index-names` is on the `Record`, `:indexes` is on `EnsureSchemaRequest`.** These are two
   different declarations of the same index set and they must agree, or sqlite will DDL a table
   the clear step never names (or vice versa). A mismatch here would look like an orphaned
   projection but be a fixture bug.
3. **mem-store may not implement `scan-index` projections the way sqlite does.** mem derives
   index rows from surviving `StoredRow`s (`SCORE-stone-2`, verified); sqlite keeps physical
   `index_<name>` tables. **They agree only if the delete clears the physical table** — which
   is precisely the claim under test. If they differ, that is finding-worthy, not a fixture bug
   to paper over.
4. **A green floor here means MAIN is green.** This branch tracks `origin/main`, which moved
   twice today. If something reds in `src/`, capture the arm FIRST, then check whether it reds
   on `origin/main` too. Do not re-run.

## Not in this stone

- **`wat/queue.wat`** — that is stone 3, and it starts after this one reports.
- **Any change to `delete` itself.** If this stone shows `delete` is wrong, that is stone 2c,
  drawn from this SCORE's findings.
