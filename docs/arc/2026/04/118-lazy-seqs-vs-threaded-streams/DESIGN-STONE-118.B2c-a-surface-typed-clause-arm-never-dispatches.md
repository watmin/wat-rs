# DESIGN STONE — 118.B2c · clause dispatch asks the checker's question, and overlapping arms are refused

**Door 1.** Found by B2b (`d4c6f3a5`); pre-existing since B1 (`488eacd0`). Evidence and the sibling
door: `NOTE-118.B2b-two-doors-the-checker-opened-and-the-runtime-did-not.md`. Witness on disk and
green: `tests/types/probe_stone_118_b2c_surface_arm_never_dispatches.{rs,wat}`.

**RULED by the builder, 2026-08-18: refuse overlap at registration.** The four questions below are
what produced that fork; the ruling closes it.

## The defect

```
no clause of :wat::core::reductions matched (3 args);
called with (fn, i64 `0`, Vector `[1, 2, 3, 4]`);
clause 0 skipped (arg 2: expected :wat::core::Seqable<T>, got :wat::core::Vector)
```

The checker ACCEPTS (B1a). `value_matches_type_by_name` (`src/runtime.rs:8760`) REFUSES: its
`TypeExpr::Parametric` arm resolves the value to a `StreamContainer` and demands the declared head
equal that container's canonical name, so `wat::core::Seqable` matches nothing. The program
type-checks and dies when called.

Precedent is twenty lines up in the same function — the arc-278 record-top fix, whose comment
carries the principle *and* the safety argument. **A surface is the container-top.** That the same
function needed this twice, for two different tops, is the finding: the arm enumerates concrete
heads, so every new top is a fresh instance.

## ★ THE SECOND DEFECT — found while ruling this one, and it is LIVE TODAY

Two arms with the **identical** declared type and **different** bodies:

```wat
(:wat::core::defclause :my::pick
  ([x <- :wat::core::i64] -> :wat::core::String "FIRST")
  ([x <- :wat::core::i64] -> :wat::core::String "SECOND"))
```

`--check` is **clean**. Running it prints **`"FIRST"`**. The second body is dead code, silently, with
no error and no warning. Dispatch is `for (clause_idx, clause) in cs.clauses.iter().enumerate() { …
return Ok((clause_idx, scope)) }` — **first match wins, declaration order**, no most-specific
selection (`src/runtime.rs`, verified this session).

**This is a hole in the redef rule.** Arc 054 (`docs/arc/2026/04/054-idempotent-redeclare/`) made
`typealias` / `define` / `defmacro` *"if byte-equivalent, no-op"*, else `DuplicateDefine`; arc 157
added the opt-in `redef_allowed` with a mandatory type-stability check. **`defclause` arms were never
covered**, because an arm is not a definition *by name* — so the one registry that dispatches on
types is the one registry with no define-once rule.

Builder, 2026-08-18: *"this is the same concept as how re-defs work? you may only express something's
def once and all other attempts must be identical."* **Yes — and at the point where the two rules
touch, the substrate currently enforces neither.**

## The two rules, and how they relate

They share a principle — *one question, one answer* — and differ in predicate:

| | unit | discriminator | identical → | conflicting → |
|---|---|---|---|---|
| **redef** (arc 054/157) | two declarations of ONE NAME | **identity** of the definitions | no-op | `DuplicateDefine` |
| **overlap** (this stone) | two ARMS of one name | **disjointness** of their domains | *see below* | refused |

They meet at exactly one case: two arms with the **identical** declared type. Under disjointness that
is maximal overlap → refuse. Under redef that is a duplicate → no-op if the bodies are byte-equal.
**The overlap rule adopts redef's escape hatch at the touching point**, so the two agree:

```
identical declared types + byte-identical bodies  ->  NO-OP        (arc 054's rule, extended to arms)
identical declared types + different bodies       ->  REFUSED      (the live defect above)
overlapping-but-not-identical declared types      ->  REFUSED      (this stone's rule)
disjoint declared types                           ->  ACCEPTED     (every arm in the corpus today)
```

## The four questions — every option, flat YES/NO

**A — a third special case: a surface branch inside the `Parametric` arm.**
*Obvious? YES* — the record-top branch above it has the same shape.
*Simple? **NO*** — it is the THIRD hand-maintained answer to one question ("is this value acceptable
for this declared type?"), beside the container-head case and the record-top case, with no shared
door. The container case did not cover records so a second was added; the second did not cover
surfaces so a third. The fourth top needs a fourth. **A is the mechanism that produced this bug,
applied again.** Disqualified.

**B — one door: ask the checker's own satisfaction question.**
*Obvious? YES* — "the runtime accepts a value for a declared type exactly when the checker would" is
one sentence, and it is the invariant this bug violates.
*Simple? YES* — one routine. `satisfies_bare_surface` (`src/types.rs:752`, `pub(crate)`) is already
the checker's answer; the `TypeEnv` is reachable from the runtime via `sym.types_deref()`. The
container-head enumeration and the record-top case become instances rather than siblings.
*Honest? **NO — as stated.*** Today arms cannot overlap: disjoint concrete heads mean at most one
matches. Under a satisfaction question a `Vector` satisfies BOTH a `Vector<T>` arm and a `Seqable<T>`
arm, and dispatch is first-match-wins — so **arm order silently decides which body runs.** B without
an overlap rule ships order-dependent dispatch and does not say so. Disqualified.

**B′ — one door, PLUS refusing overlapping arms at registration.**
*Obvious? YES.* *Simple? YES* — one satisfaction routine, one wall.
*Honest? YES* — it names the consequence B hides and makes it unrepresentable rather than resolved
by luck. It also closes the live redef hole above.
*Good UX? YES* — a multi-arity verb over `Seqable<T>` dispatches in agreement with the checker, and
an ambiguous clause set cannot be written down. ★ **RULED.**

**C — precompute a matcher at `ClauseSet` registration.**
*Obvious? **NO*** — an indirection whose purpose is invisible at both sites, and it answers nothing:
it relocates the same wrong answer. A performance idea wearing a correctness fix's clothes.
Disqualified.

**D — close the door: make the checker REJECT a surface-typed clause arm.**
*Obvious? YES.* *Simple? YES.*
*Honest? **NO*** — it does not resolve the checker/runtime disagreement, it amputates the capability
to avoid it. The runtime CAN answer the question. It would freeze `reduce`/`reductions` at ten arms
permanently and make route B's end state unreachable for every multi-arity verb — half the sequence
surface. Disqualified.

> ⚠ **What the four questions revealed.** The stone originally posed the fork as *third special case
> vs general question*, and on that framing B wins on Simple. Both options rested on a premise
> neither examined: **that clause arms cannot overlap.** True today only because dispatch asks
> name-equality. The moment it asks satisfaction, overlap is possible and first-match-wins becomes
> load-bearing semantics nobody ruled. `[[feedback_four_questions_cannot_see_a_shared_premise]]`

## ⛔⛔ STOP-1 HAS FIRED — 2026-08-18, BEFORE ANY FIX WAS WRITTEN

The strike-1 census ran. **The corpus is NOT clean**, and the offenders refute this stone's own
premise. Full evidence: `MEASURED-118.B2c-strike1-the-corpus-is-NOT-clean.md`.

`wat/bracket.wat` has TWO `defclause`s — `thread-enter` and `process-work-forms` — that declare a
**concrete `:wat::core::keyword` arm FIRST and a type-var `:W` wildcard arm SECOND**. `:W` matches
any value (`is_type_var`), so the wildcard arm is a deliberate FALLBACK, and the pair is correct
today only because dispatch is first-match-wins in declaration order.

**That is clause specificity, in production, load-bearing for the thread-pool bracket machinery.**
The "Out of scope" section below affirmatively rejected specificity as a hypothetical future want.
It is not hypothetical. **The wall as ruled would refuse `wat/bracket.wat` and cannot be armed at
zero offenders.**

⛔ **STRIKE 1 IS BLOCKED PENDING A BUILDER RULING.** Per STOP-1 the offenders are reported, not
fixed, and no wall is armed over them. The three available shapes — rule specificity IN, rule it OUT
and migrate the two sites, or rule that a type-var param is not a dispatch wildcard — are laid out in
the MEASURED doc, each needing its own four questions. **Strike 2 is unaffected and independent.**

## ⛔ THE ORDER IS LOAD-BEARING — the wall lands FIRST

**Strike 1 — the overlap wall, armed at zero offenders.** Under today's name-equality dispatch,
overlap is only possible via *identical* declared types, so the corpus should already be clean. Arm
the wall while that is true. This is the house pattern (task #41: *"turned on at zero offenders"*),
and it means the wall's first RED is always a real one.

**Strike 2 — the satisfaction door.** Route `value_matches_type_by_name` through
`satisfies_bare_surface`/`is_subtype` so a surface-typed arm dispatches.

**Reversing the order is the trap.** Opening the door first makes overlap expressible for one commit,
with first-match-wins silently deciding — and the wall would then be armed against a corpus that may
already contain offenders it caused.

## ACCEPTANCE

**Strike 1**
| # | assertion |
|---|---|
| 1 | a census of every `defclause` in the corpus reports **ZERO overlapping arm pairs** BEFORE the wall is armed — form-tree, never grep |
| 2 | the live defect goes RED: identical types + different bodies is refused, by name, at registration |
| 3 | identical types + byte-identical bodies is a NO-OP (arc 054's rule, extended) |
| 4 | a NON-VACUITY control: a normal multi-arm `defclause` (disjoint types) still registers and dispatches |

**Strike 2**
| # | assertion |
|---|---|
| 5 | ★★ `probe_stone_118_b2c_surface_arm_never_dispatches`'s four `clause_*` rows go **RED** — that RED is the acceptance, and the fix replaces them with the mirror of `control_*` |
| 6 | the four `control_*` rows stay green (a plain `defn` over `Seqable<T>` never regressed) |
| 7 | the CONCRETE-binding rows measured in `MEASURED-118.B2d-the-blast-radius-is-exactly-seqable.md` still dispatch — `probe::ISBox`, `probe::Multi`, and the `defservice` handle trio |
| 8 | container arms still DISCRIMINATE: a `Vector` must not match a `Stream<T>` arm (the arc-118.2a property the concrete-head check exists for) |

**Both:** floor ≥ 4737 passed / 0 failed / 19 skipped · clippy 0.

## ⚠ STOP triggers

- **STOP-1 — the strike-1 census finds overlapping arms in the corpus.** Do NOT arm the wall over
  them and do NOT "fix" them silently. Report the list; each one is a live ambiguity whose disposition
  is the builder's.
- **STOP-2 — a concrete-binding satisfier stops dispatching.** Rows 7/8. The change must be ADDITIVE.
- **STOP-3 — the floor goes red for any reason other than a line-number shift in a golden.**
- **STOP-4 — `#[ignore]`/skipped moves off 19.**

## Out of scope — affirmative cuts

- **Door 2** (`DESIGN-STONE-118.B2d-…`) — the checker's generic-satisfier binding. Different file,
  different mechanism. Independent.
- **Collapsing `reduce`/`reductions`' arms** — this stone's PAYOFF, not this stone. A
  `wat/seq.wat`-only follow-up once the door is open.
- **Clause SPECIFICITY** (most-specific-wins) — affirmatively rejected by the ruling. If a verb later
  needs a fast concrete path plus a generic fallback, that is a NEW ruling on whether the language
  has specificity at all, not a quiet relaxation of this wall.
- **B3.** Its precondition is met and depends on neither door.
