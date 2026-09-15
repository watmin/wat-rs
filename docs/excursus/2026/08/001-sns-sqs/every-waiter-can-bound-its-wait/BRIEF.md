# BRIEF — every waiter can bound its wait (the census)

**Read `DESIGN.md` beside this first.** It carries the invariant, the seven waiting primitives with
what each blocks on, why the asymmetry is a *usage* gap rather than a substrate one, and five
trap-doors — including the one that made a struck control pass vacuously for weeks.

⛔ **REPORT-ONLY. This stone bounds nothing.** If you find yourself adding a deadline, stop.

## The work, in one paragraph

Enumerate every call site of the seven blocking primitives and report, per site, whether the wait is
**bounded**. `recv-by-deadline` and `call-by-deadline` are bounded by construction; a bare `recv` is
not; and a `select` is bounded **iff some element of its peer vector is an `after` call** — a
structural question about a sibling expression that no line tool can answer, which is why this is a
form walk. Classify, count occurrences, quote matches, and give every primitive a PRESENT/ABSENT
control pair.

## The rooms

| where | why you are going there |
|---|---|
| `wat-scripts/census-recoverable-raising.wat` | ⭐ THE INSTRUMENT TO COPY — form walk, `head=`/`home=` axes, per-family controls, and it **prints its own limits** (`"do not quote raising-arms as complete"`). Copy that habit especially. |
| `wat-scripts/census-malformed-raising.wat:423` | ⛔ THE CAUTIONARY CONTROL — `CONTROL-A sns-fanout.wat:369` is pinned to a **line number** that drifted to `:421` and has been **passing vacuously**. Pin yours to forms or names. |
| `wat/service.wat:4276`–`:4283` `call-by-deadline` | ⭐ THE BOUNDED EXEMPLAR — `(:wat::kernel::select [peer tmr])`, with the timer's tier chosen by `peer-wire?` so the tiers match. Your ABSENT control for `select`, and the shape the missing primitive would generalise. |
| `wat/service.wat` `child-main` (find it; `grep -n 'child-main-form'`) | ⛔ THE UNBOUNDED HEADLINE — a bare `(:wat::kernel::recv ~cm-self-sym)` awaiting the owner's startup ship, with a `TimedOut` arm that cannot fire. Your PRESENT control for `recv`. |
| `src/intrinsic/kernel/message.rs:44` · `:236` | the substrate's own words: `select` *"blocks on `sel.select()`"*; `poll` *"blocks on the owner/admin link"*. Use these to justify each primitive's inclusion rather than assuming. |
| `src/intrinsic/kernel/message.rs:119` `try-send` | `TrySendOutcome … WouldBlock` — ⭑ the only non-blocking sibling in the set. Report it as the positive example of what bounded looks like. |
| `docs/arc/2026/04/109-kill-std/NOTE-an-outcome-variant-no-primitive-can-construct.md:43` | *"a bare `(:wat::kernel::recv peer)` blocks forever and can never return `TimedOut`"* — and its CLOSED sweep, which asked about **variants** and not about **waiters**. That gap is this census. |
| `the-crash-surface-is-enumerated/FINDING-the-crash-surface.md` §1 AcceptOutcome | records *"keeping `Bound` alive and not dialing **hangs**; `timeout` without `-k` never returns"* — `accept` is a waiter and that is the evidence. |

## Implementation sketch

```wat
;; per call site, report:
file:line:col  head=<defservice|defn|deftest|other>  home=<…>
               primitive=<recv|recv-by-deadline|select|poll|accept|send|readln>
               bounded=<yes|no|UNKNOWN>   by=<deadline-arg|timer-in-select|none>

;; the ONE structural test — a select is bounded iff an `after` is among its peers
(:census::select-has-timer? node)   ;; walk the vector argument's elements for
                                   ;; a call whose head is :wat::kernel::after
```

⭑ **`bounded=UNKNOWN` is a first-class answer**, not a failure — a timer built in a `let` above the
`select` is genuinely bounded and the walker may not see it. **Report the UNKNOWNs and their count**;
do not fold them into `no`.

## Blast radius

One new census under `wat-scripts/`. **Everything else: 0.**

## STOP triggers

1. ⛔ **STOP-1 — if any primitive cannot be given BOTH a PRESENT and an ABSENT control**, STOP and say
   which. `child-main`'s bare `recv` (PRESENT-unbounded) and `call-by-deadline`'s `select [peer tmr]`
   (ABSENT-from-unbounded) come free; the others you must find.
2. ⛔ **STOP-2 — pin controls to FORMS or NAMES, never to line numbers.** A struck control in this very
   corpus has been passing vacuously because its line drifted.
3. ⛔ **STOP-3 — do NOT bound anything.** Report-only; the builder rules the sequence.
4. **STOP-4 — if `readln`'s registered path is not `:wat::kernel::readln`**, find the real one before
   reporting it. My grep returned 0 while the codemods plainly call it.
5. **STOP-5 — if the `select`-has-a-timer test cannot be made structural**, STOP and report. A textual
   approximation here would produce exactly the false confidence this census exists to remove.

## Verify

- `cargo build --release` **and** `cargo nextest run --release --no-run` (the floor gate type-checks
  every `.wat` under `wat-scripts/`).
- `./scripts/floor.sh`, read the **Summary line** — never a piped exit code, and never `$?` after a
  pipe (that is `tail`'s status).
- ⭑ **Both struck censuses still pass their own controls**, run unchanged.
- ⭑ **Every primitive's control pair fires** — name each in the SCORE.
- ⭑ **Exemplars, not totals:** ≥1 real `file:line` row per primitive, plus the `home=`/`head=` split so
  live-service waiters are separable from probes and tests.
- ⭑ **The `select` test discriminates:** `call-by-deadline`'s select must classify `bounded=yes`, and a
  `select` with no timer must classify `no`. If both land the same way the test is inert.
- ⚠ Report **`bounded=UNKNOWN`** as its own count, and say what shape defeated the walker.

## Shape to copy

`wat-scripts/census-recoverable-raising.wat` for the walk, the axes, the controls, and above all the
habit of printing its own limits. And `the-malformed-census/FINDING-the-malformed-census.md` for how a
census gets written up so a builder can rule a scope constraint off it.
