# SCORE — a read declares its page

**SCORED.** Executor: grok, 2026-09-07. Tree dirty, uncommitted.
`:max-page` truncates. `scan-index-all` is a lazy Stream. `receive` is bounded and has no `-all`.

```
Summary [ 486.713s] 5221 tests run: 5221 passed (7 slow), 22 skipped
```

`.floor/2026-09-07T02-57-53Z/`

## THE OPTION

`:max-page [field N]` on a read op, parsed like `:max-entries`. Emits
`:<S>::<OP>-MAX-PAGE` and `-MAX-PAGE-FIELD`. Absent → neither def.

Enforcement is **truncation, not rejection.** `limit` is a request. The
service serves `min(asked, N)`. Opposite of `:max-entries`, which rejects.

Clamp lives at the boundary, not in each impl:

- **Path B** (`runtime.rs`): after eval of the request Value, if field
  `limit` is an i64 `> N`, write `N` in place.
- **serve-op-arms**: if `limit > N`, reconstruct the request with `limit
  = N` before the handler. Impl already honours `limit` (mem
  `take-index-page`, sqlite `full?` cursor, queue `take`).

STOP-3 did not fire: no handler was edited. The named response field is
the declaration; the request's `limit` is the clamp point (declaration-time
wall: request must have `limit <- i64`, response variant must carry the
named sequence field).

## THE TOOL

Emit `<op>-all` iff `:max-page` is declared **and** the request has
`cursor`. That is receive's exclusion: it has the bound and no cursor,
so no tool. Reads are leased; an `-all` would drain.

The helper returns `(:wat::stream::Stream :- [Item])`. First `next`
fires one RPC. The rest of the page is `cons` onto a `lazy` of the next
RPC. Page 2 is not fetched until the stream is consumed past page 1.

Same two spellings as the write side (`/scan-index-all` and
`::scan-index-all`). User services that `:satisfies` a `:wat::` surface
emit the helper under the **service** fqdn (`:fs::failing-store::scan-index-all`),
not the surface — otherwise `ReservedPrefix`. Stdlib `mem-store` keeps
the surface name.

`Store::scan` does **not** declare `:max-page`. BRIEF adopters are
`scan-index` and `receive`. Putting `[rows 64]` on `scan` clamped
Journal's internal pages; sift-rules-arena (limit 100, 12-page loop)
went 720 → 689 deductions. `scan-all` is therefore not emitted.

## FIRST FLOOR — RED, captured, not re-run

`.floor/2026-09-07T02-43-28Z/`

```
Summary [ 492.028s] 5221 tests run: 5218 passed (6 slow), 3 failed, 22 skipped
```

| arm | what fired | fix |
|---|---|---|
| `every_wat_scripts_file_loads` | `ReservedPrefix` `:wat::query::Store/scan-all` (and three siblings) from user `:satisfies Store` | helper names use service fqdn when proto is `:wat::` and fqdn is not |
| `sift_rules_arena_*` (thread+process) | expected 720 deductions, got 689 — `Store::scan` page 64 × 12 loops < 800 logs | removed `:max-page` from `scan` (not a BRIEF adopter) |

## THE PROBE

`wat-scripts/scratch-pad/probe-a-read-declares-its-page.wat`:

```
page-n=64;some-cur=yes;all-n=250;ordered=yes;first-n=1;pages=4;send=Accepted(70);recv-n=64;recv-max=64;field=envelopes
```

250 index rows, `scan-index :limit 1000` → 64 rows and a `Some` cursor.
`scan-index-all` yields 250 in isk order across 4 pages. `(take stream 1)`
returns 1. `receive :limit 1000` of 70 staggered bodies → 64.
`:queue::Queue::RECEIVE-MAX-PAGE` evaluates to 64.

## CIRCUIT ×3 at 25 ms (both sites pinned, then restored)

`ps` before: grok 13.4 %, claude 4.4 %. `circuit.wat` restored; not in
the landed diff. Every run: `total=8000;distinct=8000;dup=0`.

```
                 r1      r2      r3    med    prior @25ms
publish       20700   20706   20520  20700         18363
retries         592     596     586    592           522
receives       4645    4721    4717   4717          4692
```

Publish **+2337 ms** of 18363. Circuit `:limit` is 10 (< 64), so the
clamp is a no-op compare on every receive/scan-index. Delivery exact.
Not a win; the 2.4× is still the next stone.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★★ bound truncates | ✅ 250 rows, limit 1000 → 64 and `Some` cursor |
| 2 | ★★ `-all` yields everything, in order | ✅ 250, ordered, 4 pages |
| 3 | ★★ it is lazy | ✅ `stream/lazy` around each RPC; `(take 1)` → 1 |
| 4 | ⛔ receive bounded, no tool | ✅ recv-n=64; `grep receive-all` empty |
| 5 | ⛔ emission both ways | ✅ `scan-index-all` exists. **No `scan-all`** (scan has no `:max-page`). No `receive-all`, no `stats-all` |
| 6 | ⛔ defs readable | ✅ `RECEIVE-MAX-PAGE` → 64, field `envelopes` |
| 7 | ⛔ no caller's `:limit` changed | ✅ `sqs.wat:170` still `:limit lim :cursor None`. circuit/sns-fanout clean |
| 8 | ⛔ delivery exact | ✅ ×3 `distinct=8000` |
| 9 | ⛔ the floor | ✅ `5221 passed (7 slow), 22 skipped` |
| 10 | ⛔ blast | ✅ `surface.rs`, `types.rs`, `runtime.rs`, `service.wat`, `query.wat`, `sqs.wat`, probe, SCORE. No `circuit.wat`, no `sns-fanout.wat` |

STOP-1 did not fire: collection field is `:max-page`'s name; cursor is the request field `cursor`.
STOP-2 did not fire: no `receive-all`.
STOP-3 did not fire: clamp is `min(limit, N)` at Path B and serve-op-arms.
STOP-4 did not fire: `stream/lazy` + `cons` over the page.
STOP-5 did not fire: blast held; no `:limit` at a call site changed.

## REPORTS

| ▪ | what |
|---|---|
| a | publish median **20700** (prior 18363). Finding, not a win. Guard is live on every receive; clamp does not fire at limit 10 |
| b | `scan-index-all` of 250 at bound 64 walks **4** pages |
| c | truncation is a request `limit` clamp at Path B + serve-op-arms, not a response rewrite and not per-impl |
