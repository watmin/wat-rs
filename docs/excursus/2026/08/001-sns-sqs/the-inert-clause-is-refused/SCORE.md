# SCORE — the inert clause is refused

**SCORED.** Executor: grok, 2026-09-13, branch `sns-sqs`, HEAD `b096a047d` (DRAWN). Did not commit.

```
     Summary [ 538.127s] 5245 tests run: 5245 passed (9 slow), 22 skipped
```

`.floor/2026-09-13T21-21-04Z/` — `scripts/floor.sh` exit **0**, **no `ARM.txt`**.
Count **5244 → 5245** (+1 refusal fixture). Not a shrink.
Clippy: **CLIPPY=0**. `cargo nextest run --release --no-run` → **NORUN=0**.
`git diff --stat -- src/` **EMPTY**.

---

## ⭑ THE HEADLINE — `:deadline-ms` in `:satisfies` mode is unwritable

```
slow::slow: :deadline-ms is declared but governs nothing in :satisfies mode (measured: neither
the caller's wait nor this service's own outbound calls) — put the deadline at the CALL SITE with
(:wat::service::call-by-deadline peer op <ms> <fallback>), or drop the clause.
```

`#wat.macro/MalformedTemplate` at expand time. TRUE exit **3**. Names the fqdn, `:satisfies`, and
`call-by-deadline`. Fourth sibling of the `:peers` refusals at `service.wat:896`/`:913`.

---

## The evidence, quoted (STOP-4)

```
probe 1  outcome=TimedOut;elapsed-ms=10000;declared=300
probe 2  handler-faced=TimedOut;handler-elapsed-ms=10000;declared=300
```

**10000 vs 300, both directions.** Neither the caller's wait nor the service's own outbound calls.

---

## WHAT LANDED

`wat/service.wat` — `_deadline-ms-satisfies`, immediately before `deadline-ms-node` (after the
bijection goldens at `:896`/`:913` so those snapshots do not move). Scoped to `satisfies?`. The
clause is still **read** for the default (`deadline-ms-node` still binds `10000` when absent).

`wat-scripts/fanout/circuit.wat` — **one line removed**: `:deadline-ms 120000`. No
`call-by-deadline` added (STOP-1). The author's comment remains. The worry is filed as
`FINDING-publisher-stats-join-may-outlive-the-default-deadline.md` — **latent, not active**.

The two scratch-pad probes dropped the now-illegal clause so `every_wat_scripts_file_loads`
stays green; their measured strings live in this SCORE. Probe 1's shape is the refusal fixture
at `tests/services/probe_inert_clause_is_refused.wat` (must never run).

---

## Census — three sites, no fourth (STOP-5 / row 15)

Every real `:deadline-ms` declaration in `wat/` `wat-scripts/` `wat-tests/` `tests/` before this
stone:

```
wat-scripts/fanout/circuit.wat:2083                              :deadline-ms 120000
wat-scripts/scratch-pad/probe-deadline-ms-is-callee-derived.wat  :deadline-ms 300
wat-scripts/scratch-pad/probe-deadline-ms-governs-nothing-outbound.wat  :deadline-ms 300
```

**No fourth.** After the deletion and the probe adaptations, the only remaining declaration is
the refusal fixture itself.

---

## `:ops` mode (row 6 / STOP-2)

Not cheaply testable. `:ops` is **RETIRED** (`_ops-retired` at `service.wat:422`) — every service
must `:satisfies`. An `:ops` + `:deadline-ms` program dies with `:ops is RETIRED` before this
guard. The guard is still written as `if satisfies?` (not "always refuse"). Probe 1b remains
deliberately unmeasured.

---

## Happy / chaos / `--check`

`--check` sqs / circuit / sns-fanout: **exit 0** each.

```
n=2000;m=4;j=3;total=8000;distinct=8000;dup=0
```

Publisher still completes its join. Chaos `2000 4 3 8192 true 1000 0 0 7 0 0 0 0 0 500` →
**CHAOS_EXIT=0**, `distinct=8000;dup=0`.

---

## GRADING MAP

| # | expected | result |
|---|---|---|
| 1 | ⭑⭑ refusal fires | `MalformedTemplate` naming `slow::slow`, `:satisfies`, `call-by-deadline` |
| 2 | ⭑ circuit compiles | `--check` **0** after the deletion |
| 3 | ⭑ deletion is behaviour-preserving | measured (clause governs nothing); happy `distinct=8000;dup=0` |
| 4 | ⛔ caller NOT migrated | `git diff -- circuit.wat` is **one line removed**; no `call-by-deadline` |
| 5 | ⭑ latent risk FILED | `FINDING-publisher-stats-join-may-outlive-the-default-deadline.md` |
| 6 | ⛔ `:ops` untouched | guard scoped to `satisfies?`; `:ops` RETIRED so not cheaply testable |
| 7 | ⛔ clause not deleted from the macro | `deadline-ms-node` still read; known-clauses still lists it |
| 8 | probes' numbers carried | `10000 vs 300` both probes, quoted above |
| 9 | message is a sibling | same `fqdn-str` + `macro-error` + concat + remedy family |
| 10 | three `--check`s | sqs / circuit / sns-fanout **0** |
| 11 | floor | **5245** passed, no ARM.txt |
| 12 | clippy | CLIPPY=0 |
| 13 | no `src/` | **EMPTY** |
| 14 | chaos | EXIT=0, `distinct=8000;dup=0` |
| 15 | ⛔ exactly three sites | circuit + two probes; no fourth. Fixture is the new (expected) trip |

---

# ⭑ ORCHESTRATOR'S GRADING — claude, 2026-09-13

**Graded against my OWN reads and runs**, from `b096a047d` + the working tree.

```
floor   (mine)  Summary [ 528.052s] 5245 tests run: 5245 passed, 22 skipped · 0 failure tokens · no ARM.txt
clippy  0    happy  distinct=8000;dup=0    chaos  distinct=8000;dup=0, exit 0    src/ EMPTY
```

**STRUCK. 15 of 15 rows on my instruments — and the strike corrected my DESIGN's premise.**

## ⭑⭑ The refusal, my own run

```
TRUE exit=3   MalformedTemplate
slow::slow: :deadline-ms is declared but governs nothing in :satisfies mode (measured: neither the
caller's wait nor this service's own outbound calls) — put the deadline at the CALL SITE with
(:wat::service::call-by-deadline peer op <ms> <fallback>), or drop the clause.
```

★ **Better than its three siblings:** `:896`/`:913`/the dial check state their *rule*; this one states the
**measurement** — *"(measured: neither the caller's wait nor this service's own outbound calls)"*. A reader who
doubts the refusal is told what was measured, inside the error.

## ⭐⭐ AND MY DESIGN'S PREMISE WAS WRONG — `:ops` IS RETIRED

I wrote: *"`:ops` is used **38×**, so the mode is real"*, and scoped the refusal to `:satisfies` to avoid
refusing a mode I had not measured. **The strike read the macro instead of trusting my count:**

```
wat/service.wat:422   _ops-retired  →  macro-error "defservice: :ops is RETIRED — declare a surface"
                      Arc 278 S4c: ":satisfies + :impls MANDATORY; :ops RETIRED (illegal)"
```

My 40 `:ops` matches are inside `wat/service.wat`'s **own retirement machinery**, a codemod, and prose — **zero
live `:ops` services, because the mode is illegal.**

⭐ **The outcome is unaffected but the result is STRONGER than I claimed.** Scoping to `:satisfies` does not
cover "100 % of current sites" — it covers **every service that can legally exist**, so `:deadline-ms` is now
unwritable **everywhere**, not merely everywhere it happens to appear today.

⭑ **And the strike still honoured STOP-2** — it kept the guard `(if satisfies? …)` rather than making it
unconditional, and said why in the comment. That is the right call: the scoping *documents* that `:ops` is
unwritable for a different reason, instead of silently conflating two walls.

## ★ Row 4 — the restraint held, and it was the tempting move

```
git diff -- wat-scripts/fanout/circuit.wat   →   -  :deadline-ms 120000
```

**One line removed. No `call-by-deadline` added.** The author's comment stays, and the worry it encodes is filed
as `FINDING-publisher-stats-join-may-outlive-the-default-deadline.md`, which records it as **latent, not
active** — with the happy path as the evidence that `stats` answers inside 10 000 ms at n=2000.

⭑ That is the row I most expected to be quietly overrun: making the publisher's deadline *actually work* is one
line and feels like finishing the job. It is a behaviour change, it owes its own measurement, and the FINDING is
the handoff.

## ⚠ The cost: nothing measurable

```
guarded baseline (mine, be945ff12)   528.833 s
this stone       (mine)              528.052 s
```

The fourth refusal is a **clause-map lookup**, not a tree walk — unlike the third check, which cost ~30 s until
guarded. Nothing to optimise.

## The rows

| # | row | my verdict |
|---|---|---|
| 1 | ⭑⭑ refusal fires | ✅ **my run** — `MalformedTemplate`, exit 3, names fqdn + `:satisfies` + `call-by-deadline` |
| 2 | circuit compiles | ✅ **my `--check`**, exit 0 |
| 3 | deletion behaviour-preserving | ✅ proof is the measurement; **my** happy run confirms `8000/0` |
| 4 | ⛔ caller NOT migrated | ✅ one line removed, nothing added |
| 5 | ⭑ latent risk FILED | ✅ 41-line FINDING, *"latent, not active"*, with the deleted comment quoted |
| 6 | ⛔ `:ops` untouched | ✅ guard scoped to `satisfies?`; **and `:ops` is RETIRED — my premise corrected** |
| 7 | ⛔ clause not deleted from macro | ✅ `deadline-ms-node` still reads it for the default |
| 8 | probes' numbers carried | ✅ `10000 vs 300`, both directions, quoted |
| 9 | message is a sibling | ✅ and it carries the measurement, which the siblings do not |
| 10 | three `--check`s | ✅ **my runs**, 0 / 0 / 0 |
| 11 | floor | ✅ **my run**, **5245**, 0 FAIL, no `ARM.txt` |
| 12 | clippy | ✅ my run, 0 |
| 13 | no `src/` | ✅ EMPTY |
| 14 | chaos | ✅ **my run**, `8000/0`, exit 0 |
| 15 | ⛔ exactly three sites, no fourth | ✅ and after the work the only **declaration** left is the refusal fixture — the two scratch-pad matches are prose |

## What I'd credit above all

**It read `wat/service.wat` instead of believing my census.** *"`:ops` is used 38×"* was my number, in my
DESIGN, driving a scoping decision — and it was counting the macro's own retirement check. The strike found the
retirement, kept the scoping anyway for a documented reason, and turned a hedge into total coverage.

## ⭑ This closes all five rulings

`D1-c` · `D2-b` · `D3-a` · `D4-b` · `D5-c` — struck, each graded on my own instruments, plus both follow-on
items and the crash chase that produced the third bijection check. ⛔ What remains named and undone: the
publisher-stats FINDING (a behaviour change owing a measurement), the residual unchecked dial shapes (by
design), and the **stale *"expected-red"* sentence** in `no_loose_string_assert`'s header.
