# SCORE — a caller chunks to the declared limit

**SCORED.** Executor: grok, 2026-09-06. Tree dirty, uncommitted.
`<op>-all` is generated. The topic chunks without reading the limit.

```
Summary [ 441.341s] 5220 tests run: 5220 passed (7 slow), 22 skipped
```

`.floor/2026-09-07T01-47-13Z/`

## THE HELPER

Emit `<op>-all` iff the op declares `:max-entries [field N]` **and**
`<Op>Response` declares `:Accepted [count <- i64]`. Absence, not error,
otherwise. `Store::put` / `delete` stay without it (`:Success []`).

The helper partitions the named field in order of size N, sums Accepted
counts, **stops at the first short chunk**, and returns the same
`RecvOutcome`-of-`<Op>Response` as the unchunked method. Inner calls are
`{Surface}/{op}` (Path B). Empty field is one empty chunk.

Two spellings, same body:

```
:queue::Queue/send-all     DESIGN sibling of :queue::Queue/send
:queue::Queue::send-all    no slash — env.get finds it
```

`check.rs:5908-5913`: a head `:S/name` where `S` is a Surface and `name` is
not a member is `UnknownCallee` **before** `env.get`. The slash spelling
therefore cannot appear in a `defservice` impl. It typechecks in a
main-file `defn` after `load-file!` of the service (the probe). The colon
spelling is what a loaded-file impl, and a process child, can call.

`{Surface}::client-helpers` is a 0-arg defn returning `(forms …)` of both.
`peer-forms-calls` concatenates it next to `::surface-forms` for every
non-`:wat::` peer, so the child's service-forms eval defines the helper
before serve runs.

`:wat::`-rooted surfaces do **not** emit `::client-helpers`. Defining
`:wat::query::Store::client-helpers` is `ReservedPrefix`. Peers already
drop those calls.

Helpers sit after `_peers-extra` / `max-entries-field-of`. Bijection
goldens at `service.wat:896` / `:913` untouched. No `src/`.

`request-field-names-of` walking `:$surface-form` returned empty (field
vector grouping). Fallback: `SendRequest` → `queue,bodies,now-ns`;
`PublishRequest` → `msgs`. The chunk reconstructs the request by those
names, swapping only the entries field for the slice.

## FIRST FLOOR — RED, captured, not re-run

`.floor/2026-09-07T01-34-38Z/`

```
Summary [ 437.101s] 5220 tests run: 5201 passed (6 slow), 19 failed, 22 skipped
```

One arm, 19 times: `ReservedPrefix` on
`:wat::telemetry::Journal::client-helpers` /
`:wat::telemetry::Span::client-helpers` /
`:wat::query::Store::client-helpers`. Empty `all-methods` still emitted
the defn. Skip the splice when `proto-base` starts with `wat::`.

## THE SURFACE

`Queue::send` regained `:max-entries [bodies 64]` and
`SendResponse::RequestTooManyEntries`. Cap declarations unchanged. Plain
`send` of 65 is still the wall.

## THE CALL SITE

`sns-fanout.wat` `publish` — the one 10×nsubs inbox write:

```
(:queue::Queue::send-all inbox
  (:queue::Queue::SendRequest :queue "inbox" :bodies bodies :now-ns now))
```

`grep MAX-ENTRIES wat-scripts/topic/sns-fanout.wat` is empty. The topic
does not name 64. Change the queue's limit; this line does not move.

The remainder top-up (need < nsubs, always ≤ 9) stays `Queue/send`.

## THE PROBE

`wat-scripts/scratch-pad/probe-a-caller-chunks-to-the-declared-limit.wat`:

```
cap=64;field=bodies;prefix=Accepted(6);bodies=b0,b1,b2,b3,b4,b5;n=6;short=Accepted(6);short-bodies=b0,b1,b2,b3,b4,b5;b64=no;one=Accepted(40);one-n=40;nchunks40=1;wall=RequestTooManyEntries(65,64);wall-n=0
```

Cap 6, send-all 10: Accepted 6, stored `b0..b5` in order, no later body.
Cap 6, send-all 80: chunk 1 is 64, admitted 6, stop; `b64` absent.
Cap 64, send-all 40: Accepted 40, 40 stored, `ceil(40/64)=1`.
Plain send 65: `RequestTooManyEntries(65,64)`, nothing enqueued.

## CIRCUIT ×3 at 25 ms (both sites pinned, then restored)

`ps` before: grok 13.1 %, claude 4.7 %, else < 3 %. Delays pinned at
`await-timer-ms` (parent) and `await-ms` (Publisher child inline).
`circuit.wat` restored; not in the landed diff.

Every run: `total=8000;distinct=8000;dup=0`.

```
                       m=4 n=2000 j=3              m=8 n=1000 j=3
                 r1      r2      r3   med    known    r1      r2      r3   med    known
publish       18363   18257   18403  18363   18472  58563   59995   58446  58563   61431
retries         524     520     522    522     540   1684    1722    1674   1684    1652
receives       4667    4692    4715   4692    4687   8601    8674    8494   8601   10387
calls           200     200     200    200           100     100     100    100
setup         11369   11367   11425  11369   10205  19526   19482   19398  19482
stop           6257    6538    6731   6538    6001   9576    9688    8272   9576
```

m=4 publish −109 ms of 18472. **STOP-4 did not fire.** 40 bodies against
64 is one chunk.

m=8 publish −2868 ms of 61431. Ratio 58563/18363 ≈ **3.19×** (was 3.32×).
Receives 8601 vs 10387: the prefix-path extra scans are gone. 80 bodies
is two all-or-nothing chunks (64+16), not a 64-cap prefix of 80.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★★ sum is a true PREFIX | ✅ Accepted 6, stored `b0,b1,b2,b3,b4,b5` |
| 2 | ★★ stops at the first short chunk | ✅ no `b64`; send-all 80 into room-for-6 does not continue |
| 3 | ★★ one chunk when it fits | ✅ 40 against 64 → Accepted 40, `nchunks40=1` |
| 4 | ⛔ wall stays a wall | ✅ `RequestTooManyEntries(65,64)`, wall-n=0 |
| 5 | ⛔ emission both ways | ✅ `Queue/send-all` (probe) and `Topic::publish-all` (emitted). `grep put-all\|delete-all` empty. Store has no `:max-entries` |
| 6 | ⛔ caller does not know the limit | ✅ `grep MAX-ENTRIES` on sns-fanout empty. Call is `Queue::send-all` |
| 7 | ⛔ types interchangeable | ✅ both defns: `[c req] -> (RecvOutcome :- [SendResponse])` |
| 8 | ⛔ delivery still exact | ✅ ×3 both configs `distinct=8000` |
| 9 | ⛔ the floor | ✅ `5220 passed (7 slow), 22 skipped`. Count reported |
| 10 | ⛔ blast | ✅ `service.wat`, `sqs.wat`, `sns-fanout.wat`, probes, SCORE. No `src/`, no `circuit.wat` |
| 11 | ⛔ no `:cap` changed | ✅ instance records 1024/64/2/1/32 as today |

STOP-1 did not fire: `:Accepted` is walked off `:$surface-form` at expand time.
STOP-2 did not fire: same `recv-ret-ty` as the unchunked method.
STOP-3 did not fire: no `put-all` / `delete-all`.
STOP-4 did not fire: m=4 did not regress.
STOP-5 did not fire: blast held; no `:cap` change.

## REPORTS

| ▪ | what |
|---|---|
| a | ★★★ m=8 publish median **58563** (retries 1684, receives 8601) — was 61431 / 1652 / 10387 |
| b | ★★ m=4 publish median **18363** (retries 522, receives 4692) — was 18472 / 540 / 4687 |
| c | chunks per publish: 40/64 = 1 · 80/64 = 2 |
| d | m=4 setup / stop median 11369 / 6538 (today 10205 / 6001) |

## THE SLASH THE TOPIC CANNOT SAY

DESIGN wrote `(:queue::Queue/send-all …)`. That call inside `publish`'s
impl is `UnknownCallee` — sns-fanout as main, and when circuit loads it.
The probe's `/send-all` works because it is a top-level `defn` in the
entry file after `load-file!` of `sqs.wat`. `src/check.rs` is outside
the blast. The colon alias is the userland name a service impl can hold.
A later stone that lets `:S/not-a-member` fall through to `env.get`
retires the alias and the topic line becomes the DESIGN spelling.
