# DESIGN — the checker must read the registry, because eleven verbs are unchecked

> **Builder, 2026-09-01:** *"the registry has been forcing our hands.... this is more of the
> forcing.... we found code in active violation.... the registry must assert this
> 'misconfiguration' .. can no longer occur....."*

## The violation, measured behaviourally

Eleven **registered** verbs accept a call with **nine arguments of arbitrary type** and the checker
says nothing:

```
:wat::core::fresh-symbol · struct-field · type-equal? · type-params-used-in · variant
:wat::kernel::peer-pid · :wat::runtime::metadata-of
:wat::linkedlist::{get, length, empty?, contains?}
```

`(:wat::linkedlist::length 1 2 3 4 5 6 7 8 9)` type-checks clean. So does every other row above.

**The controls discriminate**, which is what makes this a measurement rather than an anecdote:
`first`, `join`, `assoc` all reject the same abuse — and so does **`nth`, which is ON the
`FROZEN_CHECKER_DEBT_LEDGER` with no TypeScheme**, because it has a hand-written `infer_nth`. So
"no scheme" does not mean "unchecked"; these eleven are the rows with **neither**.
(`macro-error` was the twelfth candidate and is rejected — by arc 255's own `ExpandOnly` wall, not
by any type check. It is not in this stone's population.)

⛔ **My first probe of this was VACUOUS and said all twelve were rejected.** Its scaffold used a bare
`()`, retired by arc 179, so every "rejection" was `BareLegacyUnitValue` about my own program. The
negative control — the same scaffold with no call at all — errored identically, which is what caught
it. `[[feedback_a_green_test_can_prove_nothing]]` · `[[feedback_a_gate_must_fire_the_mechanism_the_way_production_fires_it]]`

## ★★★ THE ROOT — `src/check.rs` references `crate::intrinsic::registry()` ZERO times

```
registry rows ................................. 457, every one carrying a declared `arity`
`entry.arity` consulted at RUNTIME ............ src/runtime.rs:5631, :6419
`entry.arity` consulted at CHECK time ......... nowhere
`crate::intrinsic::registry()` in src/check.rs   0 occurrences
```

The registry holds the fact that would have caught all eleven, and the checker cannot see it. A row
registered without a hand-written scheme or `infer_*` arm is unchecked **by construction** — not by
anyone's oversight. That is why the convention rung failed eleven times.

★★ **And the arity is not a hand-maintained claim: it is SNIFFED from the `#[wat_intrinsic]`
handler's own Rust signature** by the proc macro. It cannot drift from the implementation, because
the compiler establishes it. Connecting the checker to it adds no new source of truth — it reads one
that already exists and is already trusted at runtime.

## The ladder — and the top rung is reachable here

- **convention** — *"remember to write an `infer_*` arm when you register a verb."* This is what
  stands today. It failed for eleven rows, silently, and nothing in the tree could tell.
- **a check** — a test asserting every registered row rejects an absurd call. Catches the eleven;
  does not stop the twelfth from being written.
- **⭐ no form** — **the checker consults the registry's declared arity for every registered row.**
  `arity` is a required field on every submission, so *registering a verb IS declaring its arity
  check.* "Registered but unchecked" stops having a representation.

The top rung is available only because `arity` is mandatory and machine-derived. That is the whole
reason this stone can be a wall rather than a warning.

## THE ONE CONTRACT DECISION — pinned

**The checker enforces the registry's declared arity for every registered row that has no
TypeScheme, mirroring `runtime.rs:5631` exactly — same predicate, same `ArityMismatch` error.**

Not "for the eleven". Not "for rows we choose". For **every** row the registry knows and the checker
does not, so the next registration inherits the check without anyone remembering to ask for it.

⚠ **A row WITH a TypeScheme keeps being checked by its scheme.** The scheme is strictly stronger —
it checks types, not just count — and a second arity check in front of it would be a second authority
for one question, which is the shape this arc exists to delete.

## ⚠ WHAT THIS DOES NOT FIX — stated so nobody reads the stone as more than it is

**Arity is not typing.** After this stone, `(:wat::linkedlist::length "a string")` still checks
clean — one argument, and nothing says what type it must be. The eleven go from *"accepts anything,
any count"* to *"accepts anything, right count"*.

★ The ceiling is already measured and it is the same mechanism: `PROBE(255)` (`bb1aa686d`) showed
**384 of 386** registered rows' `@arg`/`@ret` docs reconstruct their checker `TypeScheme` exactly,
with **71/71** generic quantifiers recoverable. So the checker reading the registry for *types* is
the same door this stone opens for *arity* — this is the floor of that work, not a detour from it.

## THE FOUR QUESTIONS — flat YES/NO

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **checker reads registry arity for every scheme-less row** | YES | YES | YES | YES | ✅ **ADMITTED** |
| write the 11 missing `infer_*` arms | YES | YES | **NO** | — | ⛔ DISQUALIFIED |
| a test asserting all 457 reject an absurd call | YES | YES | YES | **NO** | ⛔ DISQUALIFIED |
| enforce arity for ALL rows, schemes included | YES | **NO** | YES | — | ⛔ DISQUALIFIED |
| add the 11 to `FROZEN_CHECKER_DEBT_LEDGER` | YES | YES | **NO** | — | ⛔ DISQUALIFIED |

- **write-the-11 Honest? NO** — it fixes eleven cases and leaves the class. The twelfth row registers
  unchecked next week and nothing notices. The builder's ruling is that the misconfiguration *cannot
  occur*, not that it currently does not.
- **a-test Good UX? NO** — Obvious/Simple/Honest all hold, so this is a real cut: a test names the
  offender after it is written; the wall means it cannot be written. And a behavioural test over 457
  rows is 457 process spawns.
- **all-rows Simple? NO** — braids two authorities over one question for the 386 rows whose scheme
  already answers it, strictly better.
- **ledger-the-11 Honest? NO** — the ledger's stated criterion is `check_env.get().is_none()`,
  *"no TypeScheme, not unchecked"* (its own W4 correction). These eleven are a **different and worse**
  condition, and filing them under the existing name would hide exactly the distinction that found
  them. ⚠ If anything, the ledger should learn to tell the two apart.

## Out of scope = REJECTED (not deferred)

- **Types for the eleven.** Named above as the ceiling, with its measurement already committed.
- **`macro-error`** — rejected by the `ExpandOnly` wall; not in this population.
- **The 2 `Kind::SpecialForm` rows** (`if`, `let`) — a rank-1 arity check is the wrong shape for a
  special form, and the runtime precedent this mirrors does not cover them either.

## ⚠ The expected cascade — named in advance, not discovered

Turning on arity checking for ~71 previously-unchecked rows across a corpus that has never been
held to it **will go red**, and the count is not predictable from here. That is
`docs/SUBSTRATE-AS-TEACHER.md`'s pattern: the failures are the work, each naming a call site that was
always wrong. ⛔ **A red here is a real defect surfaced, never a reason to weaken the gate** — and if
a row's sniffed arity turns out to disagree with its real call sites, THAT is the finding, because
the arity came off the handler's own signature.

## Acceptance — rows chosen to be unfakeable

| what | command | expected |
|---|---|---|
| the eleven stop being silent | `wat --check` on `(<verb> 1 2 3 4 5 6 7 8 9)` for each | **11/11 rejected** |
| the control still discriminates | same abuse on `first`/`join`/`assoc`/`nth` | rejected (as today) |
| ⛔ the probe is not vacuous | the scaffold with NO call | must NOT produce the same error |
| the checker can see the registry | `grep -c "crate::intrinsic::registry()" src/check.rs` | 0 → **≥1** |
| schemes keep their own authority | a scheme-carrying row's TYPE error still names the type | unchanged message |
| floor | `scripts/floor.sh`, exit read UNPIPED | 5115/5115, 0 failed |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |
