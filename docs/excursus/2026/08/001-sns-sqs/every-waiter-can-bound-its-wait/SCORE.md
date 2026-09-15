# SCORE — every waiter can bound its wait (the census)

**SCORED.** Executor: grok, 2026-09-15, branch `sns-sqs`, HEAD `7fe387b06` (DRAWN). Did not commit.

⛔ The headline is that **a census was taken. Nothing was bounded.**

```
     Summary [ 564.515s] 5251 tests run: 5251 passed (9 slow), 22 skipped
```

`.floor/2026-09-15T19-53-35Z/` — `scripts/floor.sh` exit **0**, **no `ARM.txt`**.
Count **5251**, unchanged. Clippy **0**. NORUN **0**.
Floor delta is a band, not a number (EXPECTATIONS).

Porcelain: `wat-scripts/census-waiter-bounds.wat` + this SCORE + `census-run.txt`.
`src/` `wat/` `wat-scripts/{queue,topic,fanout}` **EMPTY**. The leftover `wat-scripts/scratch-pad/probe-a-bare-recv-cannot-time-out.wat` was already untracked; not this stone, not touched.

---

## ⭑ THE HEADLINE — 679 waiter sites, classified; 0 waits bounded

```
waiter-sites=679  yes=15 no=646 UNKNOWN=18
```

| primitive | yes | no | UNKNOWN | STOP-1 instance kind |
|---|---|---|---|---|
| recv | **0** | 259 | 0 | no bounded sibling of the same primitive |
| recv-by-deadline | 2 | **0** | 0 | is the bound; no unbounded instance |
| select | 8 | 3 | 4 | **has both** (the only one) |
| poll | **0** | **0** | 14 | all UNKNOWN |
| accept | **0** | 8 | 0 | no bounded sibling |
| send | **0** | 215 | 0 | no bounded sibling (`try-send` is the other primitive) |
| try-send | 5 | **0** | 0 | non-blocking by construction; no unbounded instance |
| readln | **0** | 161 | 0 | no bounded sibling |

`by=`: `deadline-arg=2` `timer-in-select=8` `try-send=5` `none=646` `select-peer-not-in-let=1` `select-arg-not-a-vector=17`.

UNKNOWN is first-class (18). Never folded into `no`.

homes: `wat=48` `wat-scripts/service=19` `scratch-pad=119` `tests=267` `docs=2` `other=224`.
heads: `defservice=14` `defn=641` `deftest=0` `other=24` (`other` = defmacro templates; serve-body waiters live here with `enc=:wat::service::defservice`).

Live (`wat` + `wat-scripts/service`) = **67**. Live `bounded=yes` = **4**.

---

## Controls, pinned to enclosing defn NAMES (STOP-2)

Every primitive has a PRESENT and an ABSENT, both fired. No `file:NNN` pin.

| primitive | PRESENT (named form) | ABSENT (named form) |
|---|---|---|
| recv | child-main `:user::main` `bounded=no` **PRESENT** | same site `bounded=yes` **ABSENT** |
| select | `call-by-deadline` `bounded=yes` **PRESENT**; peer_select `[a b]` `bounded=no` **PRESENT** | `call-by-deadline` `bounded=no` **ABSENT** |
| recv-by-deadline | `owner-recv-loop` `bounded=yes` **PRESENT** | same site `bounded=no` **ABSENT** |
| try-send | defservice serve-loop `bounded=yes` **PRESENT** | same site `bounded=no` **ABSENT** |
| accept | `:user::accept-happy` `bounded=no` **PRESENT** | same site `bounded=yes` **ABSENT** |
| poll | defservice serve-body `bounded=UNKNOWN` **PRESENT** | same site `yes` **ABSENT** and `no` **ABSENT** |
| readln | census `:user::main` `bounded=no` **PRESENT** | same site `bounded=yes` **ABSENT** |
| send | peer_select `:user::compute` `bounded=no` **PRESENT** | same site `bounded=yes` **ABSENT** |

⭑⭑ **Row 3 — the `select` test discriminates.** `call-by-deadline` `bounded=yes by=timer-in-select`. peer_select `[a b]` `bounded=no by=none`. Not the same bucket. The timer is let-bound (`tmr` ← `first`/`conj`/`Vector`/`after`); the walker sees through the let init. That is the whole technical content of the stone, and it fired.

---

## STOP-1 — named, not folded

A PRESENT/ABSENT *control pair* exists for every primitive (table above). What does **not** exist is a bounded *instance* of recv/send/accept/readln, an unbounded instance of recv-by-deadline/try-send, or a yes/no instance of poll.

That is STOP-1 as the brief meant it: those counts are walker zeros plus the named controls, not a missing PRESENT of a site we failed to find. Do not invent a `recv-by-deadline` for a Peer on the back of this census.

---

## Exemplars (file:line, not totals)

### The headline unbounded (EXPECTATION 4)

```
wat/service.wat:3510:60  head=defn  home=wat  enc=:user::main  primitive=recv  bounded=no  by=none
```

Generated `child-main`. Bare `(:wat::kernel::recv ~cm-self-sym)`. The `TimedOut` arm next to it cannot fire.

### The bounded exemplar (call-by-deadline)

```
wat/service.wat:4283:28  head=defn  home=wat  enc=:wat::service::call-by-deadline  primitive=select  bounded=yes  by=timer-in-select
```

`(select [peer tmr])` with `tmr` let-bound to an `after`. Sole stdlib site that races a real Peer against a timer Peer.

### Per primitive, ≥1 real row

| primitive | row |
|---|---|
| recv | `wat/service.wat:3510` child-main (above) |
| recv-by-deadline | `wat/service.wat:4100` `owner-recv-loop` `bounded=yes by=deadline-arg` |
| select yes | `wat/service.wat:4283` (above); live copy `circuit.wat:3826` `:user::deadline-redial-is-fresh` |
| select no | `tests/kernel/peer_select_prime_process.wat:26` `:user::compute` `[a b]` |
| poll | `wat/service.wat:2452` `(poll self l ~peers-only-expr)` `UNKNOWN by=select-arg-not-a-vector` |
| accept | `tests/comms/probe_arc278_accept_outcome_wall.wat:23` `:user::accept-happy` `bounded=no`. **0 accept sites in `wat/` or `wat-scripts/service`.** |
| send | `tests/kernel/peer_select_prime_process.wat:25` `:user::compute`; live: `wat/service.wat:2219` defservice template |
| try-send | `wat/service.wat:2581` defservice serve-loop `bounded=yes by=try-send` (the positive example) |
| readln | `wat-scripts/census-waiter-bounds.wat:641` `:user::main`; live: `wat/grep.wat:434` `:wat::grep::run` |

### Live bounded=yes (all four)

```
wat/service.wat:2581     try-send            defservice template
wat/service.wat:4100     recv-by-deadline    owner-recv-loop
wat/service.wat:4283     select              call-by-deadline
circuit.wat:3826         select              deadline-redial-is-fresh
```

---

## UNKNOWN = 18, two shapes

`select-arg-not-a-vector=17` `select-peer-not-in-let=1`.

- **poll (14/14):** 4th arg is a symbol — a parameter (`clients`) or an unquote (`~peers-only-expr`). The walker does not guess that a Vector of peers contains no timer. `poll-no-peers-arg=0`.
- **select (4):**
  - `wat/bracket.wat:632` `collect-loop` `(select peers)` — `peers` is a parameter. `by=select-arg-not-a-vector`.
  - `wat-scripts/probes/arc-170/probe-s3a-select-peer.wat:22` same shape.
  - `wat-scripts/scratch-pad/probe-self-scheduling-loop.wat:28` same shape.
  - ⭑ `wat-scripts/scratch-pad/probe-client-deadline-via-select.wat:96` `(select [p tmr])` where `tmr` is let-bound to `(:cs::deadline-peer ms)` — the `after` lives *inside the helper*, which the walker does not inline. `p` is a parameter. `by=select-peer-not-in-let`. **Walker limit: a timer built by a helper is invisible. Do not count this site as unbounded.**

---

## STOP-4 — `readln`'s registered path

User-facing is **defmacro** `:wat::kernel::readln` in `wat/kernel/readln.wat`, expanding to the Rust intrinsic `:wat::kernel::readln'` (the prime; a wat string cannot hold the apostrophe, so the census LIMIT prints `readln-prime`). The walker matches any head containing `readln`, so both the macro call sites and the three quasiquote templates in the defmacro body count. A grep for `:wat::kernel::readln` as a kernel intrinsic is 0 because it is not one.

---

## LIMITS the census prints

- Unexpanded source. Quasiquote templates count; runtime expansion of `defservice` is the template, not a second site. Do not quote 679 as complete for generated code.
- `recv` of an `after` peer (e.g. `:fanout::await-timer-ms` `circuit.wat:1210`, `:demo::await-timer-ms` `sns-fanout.wat:676`, `:queue` retry-put/delete) is still `primitive=recv bounded=no`. The timer *is* the peer. The census classifies the primitive, not the peer's identity. A later stone may want a different question.
- Helper calls are not inlined (UNKNOWN row above).
- `wat-tests/` is out of the path list.

---

## Struck censuses, unchanged

### D5 `census-malformed-raising.wat`

```
CONTROL-A circuit.wat:544 raising-Malformed: ABSENT
CONTROL-A sns-fanout.wat:369 raising-Malformed: ABSENT
CONTROL-B wat/service.wat raising-Malformed: PRESENT
STOP-5 unclaimed let/do/if/macro body: PRESENT — do not quote raising-arms as complete
form-tree RecvOutcome::Malformed keywords=496; Malformed match-arms=493; raising-arms=466; unclaimed-arms=27
```

Own controls fire. sns `:369` is still the vacuous ABSENT (the twin lives at `:421`; D5 still hardcodes 369). Raising stayed **466**. keywords/arms/unclaimed drifted +1 from this tree (new census file + leftover scratch probe). Instrument not edited.

### Recoverable `census-recoverable-raising.wat`

```
recoverable keywords=3333; match-arms=3314; raising-arms=2279; unclaimed-arms=1035
STOPPED-BUCKET keywords=494; match-arms=493; raising-arms=325; unclaimed-arms=168
```

Struck numbers were 2276 / 1028. Drift +3 raising / +7 unclaimed (same tree growth). D5-inside-widened A/B still fire. FLAG `:2549` still unclaimed.

⚠ Two *line-number* PRESENT controls on the recoverable instrument are now ABSENT — the STOP-2 trap, now inside a struck census we must not edit:

| control | pin | now | arm still raises at |
|---|---|---|---|
| CallOutcome PRESENT | `circuit.wat:1015` Malformed | **ABSENT** | `circuit.wat:999` |
| ServiceEvent PRESENT | `bracket.wat:626` Closed | **ABSENT** | `bracket.wat:656` |

Orphan-naming (`holding` on collect-loop) and dial-failure (circuit rewrites) moved the lines after that census was struck. The arms exist and still `assertion-failed!`. This stone's controls are name-pinned so they cannot do this.

TrySendOutcome PRESENT remains NONE-IN-CORPUS (STOP-1 of *that* census).

---

## WHAT LANDED

- `wat-scripts/census-waiter-bounds.wat` — form walker, `head=`/`home=`/`enc=`, structural `select`-has-timer (argument tree **or** let-bound init contains `:wat::kernel::after`), `bounded=UNKNOWN` first-class, controls pinned to enclosing defn names.
- this SCORE, plus `census-run.txt` (the 679 rows).

Nothing else.
