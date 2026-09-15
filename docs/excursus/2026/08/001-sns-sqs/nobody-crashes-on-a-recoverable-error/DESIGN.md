# DESIGN — nobody crashes on a recoverable error (the census)

Builder: *"clients and services both need to tolerate failures - if the pipe between a client and
service goes - the client needs to tolerate it... as does the opposite for a server ... let's get all
of these covered. **no one is allowed to crash on a recoverable error**."*

**Drawn 2026-09-15. NOT STRUCK.** ⛔ **REPORT-ONLY. This stone migrates NOTHING.**

## WHY A CENSUS FIRST, and why that is the builder's own precedent

The doctrine is a wall over a population **nobody has measured**. D5 was ruled exactly this way —
*"census then sequence"* — and produced 544 raising arms classified on two axes, of which only ~59 were
in scope. **A migration drawn before that count would have been drawn against a number 9× too big.**

⚠ And this session has five of the orchestrator's claims die on contact, four of them counts or
structures quoted before an instrument disagreed. **The last one died an hour ago**, when a probe
refuted *"a dying client kills the service"* — see
`FINDING-the-client-vanished-arms-are-unreached.md`. ⭑ **The remedy that has worked all session is
making something else do the counting.** A form-walking census with built-in controls *is* that.

## The instrument already exists and the change is ONE LINE

`wat-scripts/census-malformed-raising.wat` (D5, struck): walks **form trees, not lines**; reports
`file:line:col head=<defservice|defsurface|defn|deftest|other> home=<…> raises=<form>`; recognises
`assertion-failed!` / `raise!` / `panic!`; and **carries its own controls** — CONTROL A must be ABSENT,
CONTROL B must be PRESENT.

Its variant filter is a single predicate at **`:83`**:

```wat
(:wat::string::contains? (:census::kw-name node) "RecvOutcome::Malformed")
```

## ⛔ THE ONE CONTRACT DECISION — what counts as RECOVERABLE

**A transport fact about a peer is recoverable. A substrate defect is not.** The census reports the
first set and must not report the second.

| direction | variants in scope |
|---|---|
| **client → service** (my call failed) | `RecvOutcome::{Lost, Closed, TimedOut, Malformed}` · `SendOutcome::{Lost, Closed}` · `TrySendOutcome::{Lost, Closed, WouldBlock}` · `CallOutcome::{Lost, Closed, DeadlineFired, Malformed}` · `ConnectOutcome::{Refused, Rejected, Failed}` |
| **service → client** (my caller vanished) | `ServiceEvent::{Closed, Lost, Malformed}` |

⭑ **Direction is derived from the variant family, not guessed:** only a serve loop matches
`ServiceEvent::*`, so that family *is* the service-facing-a-client half. Everything else is a caller
facing its peer. That makes the third axis mechanical.

⚠ **`RecvOutcome::Stopped` is DELIBERATELY EXCLUDED and must be reported separately if at all.** It
means *the world is stopping*, not *a peer failed* — it may not be an error. Classifying it as
recoverable would sweep shutdown paths into a migration nobody ruled. **Report it in its own bucket;
the builder rules whether it belongs.**

## Out of scope = REJECTED

- **Migrating anything.** This is D5's shape. A migration stone comes after the builder rules on the
  ranked output, and each arm still owes its own reading — *"a template over all of them is the
  fallback-that-collapses-failures defect"* (breadcrumb, D5).
- **`wat/service.wat:2549`'s `ServiceEvent::Lost` arm.** ⛔ It will show up in this census, and it must
  be reported with a flag: it is **live code with a fatal body that NOTHING KNOWN REACHES** — two
  documented producers, neither observed. **Which kind of unreachable is unanswered**, so it is not
  rankable as work yet (`FINDING-the-client-vanished-arms-are-unreached.md`).
- **Probes, tests and scratch-pad.** They must be *reported* (the `head=`/`home=` axes already do this)
  and are **not** in any migration scope: dying loudly is correct in a probe. D5 measured 544 arms of
  which ~490 were exactly this.
- **Building a client-side injector.** Named as the real gap by the same FINDING, and a separate stone.
  ⭑ Rank it as **coverage, not a bug chase** — the mirror's most obvious defect was measured absent.

## Blast radius

```
wat-scripts/census-*.wat   one census (extend a copy; do NOT break the struck D5 instrument)
everything else            0. REPORT-ONLY.
```

## Trap-doors named up front

1. ⛔⛔ **A census with no controls is a number, not a finding.** The D5 instrument carries two; the
   widened one needs **a control per variant family it newly covers** — one arm that must be PRESENT
   and one that must be ABSENT. ⚠ Five of this session's wrong claims came from an instrument nobody
   forced to disagree first.
2. ⛔ **Do not break `census-malformed-raising.wat`.** It is struck and its numbers are cited in D5's
   SCORE. Extend a copy, or parameterize it so the D5 controls still pass unchanged.
3. ⚠ **A count is not a finding until you have looked at the matches.** Report rows, not just totals;
   the SCORE must quote exemplars per bucket.
4. **`grep -c` counts LINES.** This corpus emits long single lines with several arms on them — the D5
   census exists because line tools undercounted here.
5. **Scratch `.wat` lives in `wat-scripts/scratch-pad/`** and every file under `wat-scripts/` is
   parsed and type-checked by the floor gate.
