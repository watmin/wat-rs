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

## ⛔ AMENDED — the builder asked whether the TYPE hole is tracked. It was not. Now it is.

> **Builder:** *"but... we tracking to ensure this only accepts a linkedlist later?...."*

The section above named the type hole and pointed at a probe commit. **That is prose, not tracking** —
the same "out of scope, see elsewhere" shape the ledger discipline rejects when no mechanism carries
it. A stone whose whole ruling is *the misconfiguration cannot occur* cannot leave its own residue
tracked by a paragraph.

### The mechanism — a SECOND frozen list, in the shape that already works here

`FROZEN_CHECKER_DEBT_LEDGER` is gated by `checker_skip_debt_is_named_and_frozen`, a **bidirectional
name-freeze** over a measured population: a new name fails, and a *resolved* name fails as STALE. It
works, and its criterion is `check_env.get().is_none()` — "no TypeScheme". That criterion **cannot
see** the distinction this stone found: of its 71 rows, which are type-checked by an `infer_*` arm
and which by nothing at all.

So this stone adds the sibling the ledger was missing:

```
FROZEN_TYPES_UNCHECKED — registered rows whose TYPES nothing checks.
  measured by DRIVING THE CHECKER, not by grepping for an infer_ arm
  (a text search for `infer_*` is exactly the wrong instrument — my own
   text predicate said all twelve were unchecked and the behavioural probe
   corrected it to eleven).
```

**Each row carries its own wrong-typed call.** A generic "wrong argument" cannot be synthesized —
for a parameter declared `:T` no argument is wrong — so the probe is explicit per row, e.g.
`(:wat::linkedlist::length "a string")`. Eleven rows, eleven one-line calls. The harness already
exists and is `OnceLock`-cached: `check(src) -> Result<(), CheckErrors>` in `src/check.rs`'s test
module, driving the real pipeline (`expand_all` → `register_types` → `register_defines` →
`check_program`).

### ★★ What that buys, and it is exactly what was asked

The gate is bidirectional, so **the list can only shrink**:

- a row **not** on the list that starts accepting its wrong-typed call → **NEW**, named, fails.
- a row **on** the list that starts rejecting → **STALE**, named, fails until it is deleted.

So the day anyone gives `:wat::linkedlist::length` a real type — a `TypeScheme`, an `infer_` arm, or
the registry-as-type-authority work — **the gate goes red and forces the name off the list.** The
answer to *"are we tracking that this only accepts a linkedlist later?"* is: the tree fails until
someone removes the row, and the row cannot be removed while the hole is open.

⚠ **This is a ratchet, not a fix.** It does not type `linkedlist::length`; it makes the untyped
population visible, bounded, and unable to grow silently — which is the difference between a known
flaw and an unknown one. The fix is the registry-as-type-authority ceiling, already measured at
384/386.

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

- **Typing the eleven.** The ceiling, measured at 384/386. ⚠ Their *untypedness* is NOT out of
  scope — it is tracked by `FROZEN_TYPES_UNCHECKED` in this same stone (see the amendment above),
  because a residue this stone creates is this stone's to bound.
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
| the type residue is FROZEN BY NAME | `FROZEN_TYPES_UNCHECKED` + its bidirectional gate | 11 rows, each with its own wrong-typed call |
| ⛔ the type gate can FAIL BOTH WAYS | sabotage: drop a name; add a name that now rejects | both arms fire, both name the offender |
| the control still discriminates | same abuse on `first`/`join`/`assoc`/`nth` | rejected (as today) |
| ⛔ the probe is not vacuous | the scaffold with NO call | must NOT produce the same error |
| the checker can see the registry | `grep -c "crate::intrinsic::registry()" src/check.rs` | 0 → **≥1** |
| schemes keep their own authority | a scheme-carrying row's TYPE error still names the type | unchanged message |
| floor | `scripts/floor.sh`, exit read UNPIPED | 5115/5115, 0 failed |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |


---

# ⛔ SHIPPED — and the measured coverage is 48/71, not 71/71

`STONE` landed. What the stone actually did, measured against the built binary rather than claimed:

```
scheme-less registry rows ............................. 71
  reached by the new check-time arity consult ......... 48   ✅ arity now enforced AT CHECK TIME
  SHADOWED, never reach it ............................ 23   ⚠ the whole :wat::kernel:: verb surface
of the eleven silent rows: now rejected ............... 9
  :wat::core::variant ................................. Arity::Variadic — correctly unaffected
  :wat::kernel::peer-pid .............................. SHADOWED (see below)
```

## ★★★ The stone surfaced a SECOND prefix authority, which is why 23 rows are untouched

`src/check.rs` carries `_ if k.starts_with(":wat::kernel::") || k.starts_with(":wat::std::")` — an
arm that returns `CheckResult::ok(fresh.fresh())` for any such head with no scheme. Every
`:wat::kernel::` verb takes it and never reaches the registry consult. **That is a namespace guess
standing in for the registry — the same class as `effectful_by_prefix`**, and it is the next forcing.

⚠ **My DESIGN claimed the check covers "EVERY row the registry knows and the checker does not."**
That was false, and it had already been written into a comment on disk before it was measured. The
comment now states the real coverage and names the shadowing predicate **by grep-token, not by line
number** — the doc that sent me hunting for this cited a `check.rs:5561` that had long since drifted.

## What the 23 are NOT

They are **not unchecked**: `runtime.rs`'s `dispatch_substrate_impl` raises `ArityMismatch` for them
at eval time — verified, `(:wat::kernel::peer-pid 1 … 9)` fails when run. The stone moves enforcement
EARLIER for 48 rows; for the 23 it stays where it already was.

⚠ Nor is "a nonexistent `:wat::` verb type-checks clean" a finding of this stone — that is the
documented blanket-accept `tests/cli/retirement_table_reachable.rs` exists to police, and the runtime
raises `UnknownFunction`. Measured before reporting, because it looked like a much larger hole.

## STOP-5, which the rider correctly reported it could not run

Both directions sabotage-tested by the orchestrator, both name the offender:

- **NEW** — dropped `:wat::linkedlist::length` from `FROZEN_TYPES_UNCHECKED` → *"carries a wrong-typed
  probe in `TYPE_RESIDUE_PROBES` but is absent from `FROZEN_TYPES_UNCHECKED`. Add it."*
- **STALE** — repointed its probe at a rejecting call → *"its wrong-typed call is now REJECTED …
  delete it from `FROZEN_TYPES_UNCHECKED` and `TYPE_RESIDUE_PROBES`"*, quoting the rejection.

★ A free control fell out: sabotaging the wrong list first proved
`checker_skip_debt_is_named_and_frozen` fires too.

## STOP-1 held, and found something the DESIGN's probe could not see

`:wat::core::variant` is typed `@arg xs… :wat::core::Value`, and `types.rs`'s `is_subtype` makes
`Value` the **universal top** — no argument is ill-typed against it, now or ever. Its residue entry
cannot exist, so the probed population is **ten**, not eleven. ★ The arity-abuse probe that found the
eleven was structurally blind to this: it never asks what the individual argument *types* are.

## ⬜ NEXT — and it is named, not deferred

**Close `check.rs`'s `:wat::kernel::`/`:wat::std::` prefix arm** so the 23 shadowed rows reach the
registry. The 23 are listed in this stone's commit message; the predicate is greppable; the cascade
is the same SUBSTRATE-AS-TEACHER shape this stone predicted and did not get.
