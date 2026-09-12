# SCORE — the store says what it deleted

**SCORED.** Executor: grok, 2026-09-12, branch `sns-sqs`, HEAD `49fc14fca` (DRAWN). Did not commit.

```
     Summary [ 515.094s] 5237 tests run: 5237 passed (7 slow), 22 skipped
```

`.floor/2026-09-12T21-44-59Z/` — `scripts/floor.sh` exit **0**, **no `ARM.txt`**.
**Nothing was re-run to make anything go away.** `5237` unchanged — no deftest added.
Clippy: `cargo clippy --release --workspace --all-targets -- -D warnings` exit **0**.
No Rust change. `git diff --stat -- src/` is empty.

---

## ⭑ THE HEADLINE — the number was there; it is no longer discarded

`Store::DeleteResponse::Success` carries `deleted <- i64` — base-table rows actually removed. A probe that **reads** the field (the floor cannot) prints:

```
mem-partial=2
sqlite-partial=2
sqlite-gsi=2
mem-missing=0
sqlite-missing=0
PROBE=0
```

3 keys, 2 exist → **2** on mem (length delta) and sqlite (`stmt.execute()`). Same sqlite table **with** a GSI → still **2**, not `2+|indexes|`. A missing key → **0**, still `Success`.

`wat-scripts/scratch-pad/probe-store-says-what-it-deleted.wat`. Every corpus arm binds `_`. The floor is green whether the count is right. The probe is the proof.

---

## WHAT LANDED

| where | change |
|---|---|
| `wat/query.wat:534` | `:Success [deleted <- :wat::core::i64]` — the file's own ScanResponse precedent, applied to delete |
| `wat/query/sqlite-store.wat` | `delete-one-key` / `delete-rows` → `Result :- [i64 Error]`. Base `DELETE` count kept; GSI clear discarded. `delete-rows` **accumulates** (`n +` rest). `delete-response` `((Ok n) (Success n))`. Txn `Ok n` through commit. |
| `wat/query/mem.wat` | `deleted = count(rows before) − count(rows after)` on the `MemWrite` fold |
| match arms | recorded codemod `wat-scripts/fixes/bind-deleted-on-delete-response-success.wat` — 9 arm heads, `_` binder, constructions untouched |
| `probe-entry-three-of-ten.wat:137` | construction `Success 0` (empty kept → deleted nothing) |

`close-then-err` is now `:- [T]` so the delete txn's `Result :- [i64 Error]` joins with rollback-then-Err. Put still infers `T = nil`. Not a new verb.

No stash-dance (no Rust). One `cargo build --release` baked the three `wat/` files together.

---

## THE CODEMOD

Finder (`wat --grep`) reported **9** unapplied arm heads — the DESIGN's 9 match arms, not the 3 constructions. `probe-entry-three-of-ten.wat` has both; the ctor at :137 was not rewritten as `_`.

Path list **generated** (`rg -l 'DeleteResponse'` over `wat/ wat-scripts/ tests/ docs/`), not hand-typed. Four homes all present. The fix file itself matched the token and was left off the apply list.

Pilot on `/tmp` copy of the dual-form probe, then apply. Second run: grep empty (idempotent). Goldens not patched.

---

## STOP-1 / STOP-2 / STOP-3 / STOP-4

- **STOP-1 did not fire.** Base `execute` count is bound, then `clear-index-projections` is matched and dropped. `sqlite-gsi=2`.
- **STOP-2 did not fire.** Both backends report an exact i64. Missing key is 0, still Success.
- **STOP-3 did not fire.** No floor test asserted on Success arity. No golden patched.
- **STOP-4 did not fire.** `PutResponse::Success` and `EnsureSchemaResponse::Success` stay nullary.

---

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | variant carries `deleted` | ✅ `wat/query.wat:534` — 1 hit |
| 2 | no nullary Success left | ✅ `--grep` after apply: **0**. Value-position ctors carry `deleted` / `n` / `0` |
| 3 | count threaded, not invented | ✅ `delete-one-key` and `delete-rows` both `Result :- [i64 Error]` |
| 4 | `delete-rows` accumulates | ✅ `(Ok (i64::+ n m))` on the recursive rest, not the last key |
| 5 | ⭑⭑ number is REAL, both backends | ✅ `mem-partial=2` `sqlite-partial=2` |
| 6 | GSI clear not summed | ✅ `sqlite-gsi=2` (one index; summed would be 4) |
| 7 | missing key contributes 0 | ✅ `mem-missing=0` `sqlite-missing=0`, variant still Success |
| 8 | codemod recorded | ✅ `wat-scripts/fixes/bind-deleted-on-delete-response-success.wat`, idempotent |
| 9 | arms only | ✅ ctors: mem `deleted`, sqlite `n`, probe-entry `0` — never `_` |
| 10 | four `.wat` homes | ✅ `wat/` `wat-scripts/` `tests/` `docs/` all in `git diff --name-only` |
| 11 | integrity | ✅ intended files only; `src/` empty |
| 12 | floor | ✅ **5237 passed**, 22 skipped, 0 FAIL. `.floor/2026-09-12T21-44-59Z/` |
| 13 | tests compile | ✅ `nextest --release --no-run` exit 0 |
| 14 | clippy | ✅ `-D warnings` exit **0** |
| 15 | happy path | ✅ `distinct=8000;dup=0` HAPPY=0 |
| 16 | chaos | ✅ `distinct=100;dup=0` CHAOS=0 |
| 17 | no Rust | ✅ `git diff --stat -- src/` empty |
| 18 | ⚠ `rt-store` does NOT improve | ✅ happy `rt-store=6532` — same band as ~6628, **not** the −17.1 % PATCH. Protocol, not the optimisation. |

---

## WHAT WAS NOT DONE

- Did not commit.
- Did not add a `wat-tests` deftest (5237 stays).
- Did not land `PATCH-measure-variant.diff`.
- Did not widen `PutResponse::Success` or `EnsureSchemaResponse::Success`.
- Did not use `deleted` in `sqs.wat` (arms bind `_`; that is the next stone).
- Did not patch goldens.

---

# ⭑ ORCHESTRATOR'S GRADING — claude, 2026-09-12

**Graded against my OWN reads and runs**, from `49fc14fca` + the working tree.

```
floor  .floor/2026-09-12T21-57-45Z/  Summary [ 506.733s] 5237 tests run: 5237 passed (7 slow), 22 skipped
       0 lines matching ^ *(FAIL|TRY|TIMEOUT|ABORT|SIGSEGV) · no ARM.txt
clippy --release --workspace --all-targets -D warnings → exit 0
happy  distinct=8000;dup=0  rt-store=6452  rt-total=10891     chaos  distinct=100;dup=0  exit 0
blast  10 files, +34/−25 · git diff --stat -- src/ EMPTY
```

**STRUCK. 18 of 18 rows pass on my instruments**, including the three the floor could not give.

## ⭑⭑ Rows 5–7 are the stone, and I audited the INSTRUMENT before the output

My own run of `probe-store-says-what-it-deleted.wat` (true exit 0, unpiped):

```
mem-partial=2   sqlite-partial=2   sqlite-gsi=2   mem-missing=0   sqlite-missing=0
```

⭑ **I read the probe before trusting it**, because a probe that cannot fail is theatre. It is a valid
instrument: both rows carry an `IndexKey` under one index `by-v`, so a GSI-summed count would print
**4**, not 2. And the mem/sqlite pair reaches the same number by completely different routes — a length
delta over the `MemWrite` fold vs `stmt.execute()`. The `2 / 2 / 2 / 0 / 0` shape can only come from
base-table rows actually removed.

I verified the discard sites stopped discarding by reading them, not the report:
`delete-one-key` binds `((Ok n) …)` from `DELETE FROM main`, then matches `clear-index-projections`
and **returns `n`** — the index count is structurally unreachable. `delete-rows` accumulates
`(i64::+ n m)` with base case `Ok 0`, so it is a sum and not the last key's count.

## ⚠ Row 18 — the null I committed to in advance, and it held

`rt-store=6452` on my own run against my own **pre-change 6628** on `9a6ee5b76`: the same band, inside
the campaign's noted 2.3 % within-arm spread, and nowhere near `PATCH-measure-variant.diff`'s **5522**.
**The −17.1 % did not land, which is correct** — this stone is the protocol, not the optimisation.
EXPECTATIONS said a movement here would mean the scope had been misread; nothing moved.

## ⛔ My own instrument lied to me once, and the wrap-proof form is what settled row 2

A first pass with `grep -rn … | cut -c1-120` showed
`PROBE-store-has-no-delete.wat:71` as `((…::DeleteResponse::Success` with nothing after it — reading
like a **missed nullary site**. It was my own `cut` truncating the binder. The recovery doc's WRAP-PROOF
rule is the fix and I used it: normalize whitespace first, then match.

```
tr -s '[:space:]' ' ' < $f | grep -o 'DeleteResponse::Success *)'    → zero hits, every file
```

★ Third instance this session of the same family — a line-oriented instrument answering a question
about a form. Here it produced a **false positive** rather than an undercount, which is the friendlier
direction and still cost a read.

## ★ Row 8 — the second apply is a stronger census than the finder

I re-ran the whole codemod over all 12 generated paths: `10 files changed, 34 insertions(+), 25
deletions(-)` **before and after, identical**. That is a direct proof of zero unapplied sites, stronger
than the finder's silence. (My first `--grep` invocation starved on stdin — the finder needs the path
list too; with it piped, it returns empty.)

## ★ The codemod's rule is the structural one, verified in its source

Not a file allow-list, and not a token sweep that happened to work:

```
node-edits    fires match-edits ONLY when (head-kw-name node) = ":wat::core::match"
match-edits   folds over (drop ch 2) — the ARMS, never the head or the scrutinee
arm-pat-edits edits only when the arm's FIRST child is a nullary Success pattern
success-...?  requires (length ch) = 1 — so an already-bound head has 2 children and is untouched
```

`probe-entry-three-of-ten.wat` carries **both** forms and came out right: `:137` construction became
`Success 0` (honest — it fires on `empty? kept`), `:143`/`:243` arms became `_`.

⚠ **One residue, named not fixed:** the keyword test is `string::contains? … "DeleteResponse::Success"`,
not an exact match. I enumerated every name in the corpus containing that substring — all four are
qualified spellings of the same variant — so **no false positive exists today**. A future
`DeleteResponse::SuccessPartial` would be caught wrongly. Same looseness as the
`add-event-id-to-metric-log-ctors` precedent, so it is consistent rather than novel.

## ★★ `close-then-err` had to change, and my BRIEF never named it

`wat/query/sqlite-store.wat`'s delete txn now returns `Result :- [i64 Error]`, so its `Err` arm could no
longer call a `close-then-err` typed `Result :- [nil Error]`. The strike **parametrised** it —
`:- [T]`, `-> (Result :- [:T Error])` — rather than duplicating it.

**Sound, and I checked rather than assumed:** its body constructs **no `Ok` on any path** (`autocommit?`
→ `Err e`; rollback `Ok` → `Err e`; rollback `Err` → `assertion-failed!`), so `T` is free and
unconstructible. `put`'s four call sites still infer `nil`. Nine callers across `wat/`, `tests/services/`
and three scratch probes all type-check — the floor ran them.

⭑ **This is the second stone in a row where a file outside my rooms was forced to change** (the
capability stone had `wat/core.wat`, position 0). Both times the executor found it, not the brief. The
pattern worth naming: **when a return type widens, every function that returns into it widens too** —
and a brief that lists only the sites naming the changed *type* will miss the sites naming the changed
*function*.

## What I'd credit above all

The three rows a green floor could not give were run, on both backends, with the instrument audited
first. And the null in row 18 was reported **as a null** rather than dressed up as a win sitting one
patch away.

## What this hands the next stone

`PATCH-measure-variant.diff` is now landable in principle — `deleted` exists and is exact. ⛔ It still
needs the `:Unknown` state for the unknowable-**put** path (`the-queue-knows-its-own-depth/SCORE.md`
§7.3), which is a builder decision and was explicitly not taken here. Every corpus arm binds `_`, so
**nothing reads `deleted` yet**; the next stone is where the −17.1 % `rt-store` / −60.9 % `count`
actually banks.
