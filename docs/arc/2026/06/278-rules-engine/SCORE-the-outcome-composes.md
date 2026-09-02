# SCORE — the outcome composes

**STRUCK.** Executor: grok, 2026-09-02. Every row re-run by me.

```
Summary [ 348.633s] 5183 tests run: 5183 passed (3 slow), 15 skipped
FLOOR=0
```

Log: `.floor/2026-09-02T03-34-34Z/`

First floor (the red, kept): `.floor/2026-09-02T03-23-41Z/` — 7 failed. Arms named and fixed below; that log is the evidence, not a flake.

The circuit, my own run:

```
n=2000;m=4;j=3;total=8000;distinct=8000;dup=0;workers=8;empty=1
setup=8331;publish=3113;drain=69527;stop=2408
WALL 84.141
```

| # | what | my re-run |
|---|---|---|
| 1 | ★ the send hack is gone | ✅ `grep -n 'Millisecond 1' wat-scripts/queue/sqs.wat` → **zero** |
| 2 | ★ `-flush-outbox` ceases to exist | ✅ `grep -c 'flush-outbox' wat-scripts/queue/sqs.wat` → **zero**. The arm is deleted; tick sends and re-arms in one `SelfOutcome::Continue` |
| 3 | ★ an internal arm cannot reply — by SHAPE | ✅ **rung 3 reached** — corrected on grading, see below |
| 4 | ★ the runtime guards are DELETED | ✅ `grep -n 'has no client to reply to' wat/service.wat` → **zero** |
| 5 | ★ nothing is lost | ✅ `total=8000; distinct=8000; dup=0` |
| 6 | `Stop` carries sends | ✅ `Stop [state, reply Option<R>, sends]` — no `arms` |
| 7 | reply is Option, not a vector | ✅ `reply <- (Option :- [:R])` on `Outcome`; **absent** from `SelfOutcome` |
| 8 | no Rust change | ✅ `git diff --stat src/` empty |
| 9 | the migration is recorded | ✅ `wat-scripts/fixes/outcome-composes.wat`. 144 paths applied. Follow-ups (sqs sites, parametric empty-sends, acronym wrap, jsonl) are named below — not a 351-site hand-edit |
| 10 | the whole corpus migrated | ✅ constructions of old variants: **one** leftover, the probe's internal `-tick` (`Outcome::Reply`), left on purpose |
| 11 | the phase split | ✅ **reported**: setup=8331 publish=3113 drain=69527 stop=2408 (against setup=8600 publish=2407 drain=72516 stop=2452) |
| 12 | wall time | ✅ **84.1 s** against 91.5 s — reported, not promised. Faster by seconds, as the brief allowed |
| 13 | floor | ✅ 5183/5183, FLOOR=0, my own run. 5183 not 5184: `internal-reply-still-asserts` was deleted with the guards it grepped |

## The shape that landed

```
Outcome    Continue [state, reply Option<R>, sends, arms]
           Stop     [state, reply Option<R>, sends]          ;; no arms
SelfOutcome Continue [state, sends, arms]
            Stop     [state, sends]
```

Public arm `[s ctx req] -> Outcome`. Internal `[s ctx] -> SelfOutcome`. Continue.reply is already the wire `Reply` union (the arm wraps); the serve loop sends it as-is, and sends `Directed` as-is. Same `R`, so reply+sends unify.

`sqs.wat` send-with-waiters is one `Continue` Some Ok + waiter Directeds, re-arm only if waiters remain. `-tick` is one `SelfOutcome::Continue` box + optional re-arm. `-flush-outbox` is gone.

## Row 3 — CORRECTED ON GRADING: the wall DID reach rung 3

The strike reported this row as missed. **It is not.** The strike's probe still spelled
`Outcome::Reply` — a variant that no longer exists for anyone — so it was measuring the wrong
proposition. Tested against the real one, an internal arm returning an `Outcome` *with a reply in
it*:

```wat
(-tick [s ctx]
  (:wat::service::Outcome::Continue s
    (:wat::core::Some (:probe::Solo::Reply::Ping (:probe::Solo::PingResponse::Ok)))
    … ))
```

```
:wat::core::match: parameter scrutinee expects (:wat::service::SelfOutcome :- [...]);
                                          got (:wat::service::Outcome :- [...])
```

**Rejected at check time.** The type wall holds; no checker change is owed and STOP-3 was not
tripped. Deleting the three runtime guards (row 4) was therefore correct rather than premature —
they policed `Outcome::Reply`, which no longer exists.

★ **What actually survived is a different defect, and it is not about services at all.** A
nonexistent variant of a **stdlib** enum is not resolve-checked:

```
ACCEPTED at --check:  :wat::core::Option::Nope
ACCEPTED at --check:  :wat::kernel::RecvOutcome::Bogus
ACCEPTED at --check:  :wat::service::Outcome::TotallyFake
rejected at --check:  :probe::Local::Nope        <- same shape, enum defined in the same file
```

One variable — stdlib versus same-file. A retired or typo'd stdlib variant compiles and dies at run
time as `UnknownFunction`. This is what made a stale probe look like an accepting wall, and it is
the reason a 351-site codemod is dangerous: a *code* leftover would have shipped silently instead of
going red. **Recorded as a finding; it wants its own stone.**

★ **Coverage note.** Deleting `internal-reply-still-asserts` was right — it grepped for a message
that can no longer occur. But nothing now tests the new wall, so 5184 -> 5183 is a real coverage
loss. It wants a red probe (the `Outcome::Continue`-from-a-`-tick` form above) asserting the
rejection names `SelfOutcome`.

## First floor, 7 arms (kept)

`.floor/2026-09-02T03-23-41Z/`. Not re-run to bless them. Fixed, then a new floor:

1–2. `wat_mcp::a_counter_increments_across_turns` / `a_thread_counter_increments_across_turns` — jsonl still had `Outcome::Reply`. Migrated the two payloads.
3. `probe_arc265_acronym_registry` — wrap used `kebab->pascal` (`CreateWebAcl`) not `kebab->pascal-in` (`CreateWebACL`). One-site follow-up on the fixture.
4–7. four `peers_bijection` EDN snapshots — defservice end-col 51→254, `service.wat` assertion spans shifted 14 lines by `SelfOutcome`. Message text unchanged.

## Follow-ups the mechanical pass could not do

- **sqs.wat** send-with-waiters and `-tick` (STOP-5 / rows 1–2) — new combinations, not a ctor rewrite.
- **parametric `:satisfies (Surface :- [K V])`** — empty-sends cannot be `{s}::Reply`; typed as `(Surface::Reply :- [K V])` by hand in `wat/cache.wat` and four `wat-tests/service-parametric*.wat`.
- **`wat/query.wat` sift-rules template** — `:satisfies ~surface-kw`; wrap splices `~sift-reply-kw`.
- **jsonl MCP counters** — not `.wat`, so not on the path list.
- **acronym wrap** — as above.

STOP-1 held: no new `:wat::fix::` verb. STOP-3 held: `src/` empty. STOP-4 vacuous: no live `Stop` constructors. STOP-5 held: both hacks deleted.

## What this unblocks

Item 2 of THE ORDER (internal-arm probe) is paid, and item 1 closed what it found: the checker DOES force internal bodies through `SelfOutcome`. Item 3 (`after 0` illegal) can proceed: the queue no longer needs a 1 ms timer to say *and*. Item 4 (store measurement) is still independent.
