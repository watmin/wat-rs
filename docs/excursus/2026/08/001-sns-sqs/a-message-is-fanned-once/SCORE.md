# SCORE — a message is fanned once

**SCORED.** Executor: grok, 2026-09-06. Tree dirty, uncommitted.
One extra send. No loop. `dup` fell. `publish` did not.

```
Summary [ 404.714s] 5220 tests run: 5220 passed (6 slow), 22 skipped
```

`.floor/2026-09-06T10-58-36Z/`

## THE IMPL

After the first `Queue/send` returns `Accepted k`:

```
rem  = k mod nsubs
need = nsubs - rem
```

- `rem == 0` → `Accepted (k / nsubs)` — unchanged
- `rem > 0`  → one top-up of the tail message's remaining subscriber
  indices `rem .. nsubs-1`, same `"{i}|{msg}"` bodies
  - top-up `Accepted need` → `Accepted (floor + 1)`
  - anything less, Lost, Closed, TimedOut → `Accepted floor` (today)

No loop. STOP-2 did not fire as an exit: a split top-up degrades, it does
not retry.

## THE PROBE

`wat-scripts/scratch-pad/probe-a-message-is-fanned-once.wat`, idle inbox
(no workers, so the top-up is refused — cap 6, 6 pairs already in):

```
split=Accepted(1);msg0=4;n=6;bodies=0|p0,1|p0,2|p0,3|p0,0|p1,1|p1;nsubs7-pub10=Accepted(9)
```

`Accepted 1`. Exactly **4** pairs of message 0, in order, format `"{i}|p0"`.
Two orphan pairs of message 1 (`0|p1,1|p1`) — the prefix that split it.
Never 5, 6 or 7 of message 0. `nsubs 7` publish 10 → `Accepted 9`, alive.

The top-up, when it lands, sends `2|p1,3|p1` — the same format, the missing
indices. On this idle probe it is refused (`Accepted 0` of `need 2`) and we
report floor. That is row 4.

## CIRCUIT ×5

`ps` before: grok 11.5 %, claude 4.5 %, else < 1 %.

Every run: `distinct=8000`. `publish-calls=200`.

```
dup            2741  2759  2859  3015  2677     median  2759
publish       70420 68769 70244 69404 71721     median 70244
seen-skipped  13607 12824 13269 13257 13462     median 13269
setup         10249 10174 10271 10232 10205     median 10232
stop          11132  9313  9742 11235  9220     median  9742
```

outbox (items = `total`):

```
items         10741 10759 10859 11015 10677     median 10759
50-250         3092  2992  2974  3121  2842     median  2992
250-1000       7618  7736  7854  7863  7804     median  7804
max-ms          699   662   657   661   698     median   662
```

Prior (`docs/excursus/2026/08/001-sns-sqs/the-server-manages-its-own-capacity/SCORE.md`): dup **4226**,
outbox 12161 items (`50-250=4982 250-1000=7148 max 562ms`), publish **61944**.

## THE FINDING — `dup` fell, `publish` did not

`dup` 4226 → **2759** (−35 %). outbox items 12161 → **10759**. The
partial-fan duplicates this stone named are real, and completing the tail
removes some of them.

`publish` 61944 → **70244**. It moved the wrong way.

The DESIGN predicted they move as one (drain-bound: fewer items, shorter
waits, lower publish). They did not. Extra top-up send is a `Queue/send`
on the publish path even when it is refused (idle cap); when it lands it
still adds a round-trip. Remaining outbox items wait **longer** (250-1000
bucket 7148 → 7804, max 562 → 662).

★ That refutes the drain-bound model as the whole story of this 61 s.
Reported, not papered over. Inbox `:cap` not raised. Driver not touched.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★★ counted message is fully fanned | ✅ `Accepted 1`, exactly 4 pairs of message 0 |
| 2 | ★★ top-up bodies identical | ✅ same `"{i}\|{msg}"`; stored prefix is `0\|p0..3\|p0,0\|p1,1\|p1`; top-up would be `2\|p1,3\|p1` |
| 3 | ⛔ at most one extra send | ✅ one `if rem > 0` send. No loop |
| 4 | ⛔ refused top-up degrades | ✅ idle cap 6: top-up `Accepted 0`, reply `Accepted 1`, alive |
| 5 | ⛔ delivery exact | ✅ ×5 `distinct=8000` |
| 6 | ⛔ nsubs cliff stays gone | ✅ `Accepted 9`, no assertion |
| 7 | ⛔ floor | ✅ `5220 passed (6 slow), 22 skipped` |
| 8 | ⛔ blast | ✅ `sns-fanout.wat`, scratch probe, this SCORE. No `circuit.wat`, no `sqs.wat`, no `wat/`, no `src/` |

## REPORTS

| ▪ | what | prior → now |
|---|---|---|
| a | dup median | 4226 → **2759** |
| b | outbox | 12161 items, 50-250=4982 250-1000=7148 max 562 → **10759** items, 50-250=2992 250-1000=7804 max 662 |
| c | publish median | 61944 → **70244** |
| d | seen-skipped | 13594 → **13269** |
| e | setup / stop | 10232 / 9742 |

## STOP TRIGGERS

- **STOP-1** did not fire. `k mod nsubs` names the missing indices because the fanout is msg-major and admission is a prefix (probe bodies in order).
- **STOP-2** did not fire as an exit. Top-up that itself splits (`ntop < need`) reports floor. No loop.
- **STOP-3** did not fire: `dup` **fell**. The companion prediction (publish follows) failed; that is the finding above, not STOP-3.
- **STOP-4** did not fire. No `:cap` change. No `circuit.wat` / `sqs.wat`.

## NOT TOUCHED

`circuit.wat`. `sqs.wat`. `wat/`. `src/`. Inbox `:cap`. The per-message-pairs alternative (named in the DESIGN, not taken).

Tree uncommitted. Do not commit unless asked.
