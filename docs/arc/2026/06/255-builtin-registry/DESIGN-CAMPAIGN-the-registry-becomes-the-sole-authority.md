# DESIGN — CAMPAIGN: the registry becomes the sole authority

> **Builder, 2026-09-01:** *"the registry must become the sole authority.... we have done large
> refactors before.... we can do them again.... we had to mass update to use TrackedValue instead of
> Value or whatever.... if this is another one of those things.... so be it... **the registry will
> become the sole authority..... we move mountains when we see fit....**"*
>
> Governed by `[[RULING-the-registry-is-the-sole-authority]]`. Structured on the precedent the
> builder named — arc 233's `TrackedValue` migration, stones `233.2.a` → `233.2.l`.

## ★★★ The precedent's ruling, which licenses this campaign

`docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.2.g.md`, verbatim:

> *"Only Shape A passes **Honest**. It fails **Simple** on the transition cost — but **Simple measures
> the SHIPPED state, not the transition**. Post-transition, Shape A is simpler than Shape C (no
> discipline burden; structural enforcement). The transition cost is real but **bounded**. The
> trap-door class under Shape C is **UNBOUNDED** (every future producer + every future match site)."*

⚠ And the precedent's other half, which is the warning: **arc 233 shipped the wrong shape first.**
Shape C went in at `233.2.a`, its trap-door bit **four times in one session**, and `233.2.g`
re-evaluated and re-picked Shape A *mid-campaign*. **A campaign doc that cannot be re-picked is a
plan, not a design.**

## The measured ground

Three registries, not two. The `solvere` cast found the third; my own census had missed it.

| authority | rows | answers |
|---|---:|---|
| `IntrinsicEntry` (`src/intrinsic/mod.rs`) | 464 | the intended sole authority |
| `RETE_OPS` (`src/rete/vocabulary.rs`) | 74 | names · aliases · signatures · a class system |
| `SPECIAL_FORMS` (`src/special_forms.rs`) | 19 | names · syntax sketches · a `doc_string` placeholder |
| `register_builtins` (`src/check.rs`) | 350 | signatures |
| literal type-grammar arms (`src/check.rs`) | 118 | signatures, for forms with no scheme |
| the property residues (`intrinsic_meta` 37 · `is_expand_time_legal` 54 · `effectful_by_prefix` 8) | 99 | purity · determinism · totality · expand-time |

And **four absence ledgers that exist only because the split does**:
`GAP_A 89 · GAP_B 115 · DEBT 73 · TYPES_UNCHECKED 10`.

★★★ **Those four are the campaign's progress meter AND its finish line: when all four are empty and
their files deleted, the ruling is satisfied.** A falsifiable end condition, already instrumented.

## The shapes

**Shape A — FOLD.** Every row becomes an `IntrinsicEntry`. The registry gains an alias field
(`core_name`) and a fallback marker; the other tables are deleted.

**Shape B — FEDERATE.** A single query surface that consults all three tables. No mass migration.

**Shape C — GATE.** Keep the tables; keep the four absence ledgers cross-checking them. *This is
what exists today.*

**Shape D — GENERATE.** `RETE_OPS` and `SPECIAL_FORMS` become **derived output** of the registry.
⚠ Not an alternative to A — it is what A *enables*, and only after the registry carries the data.

## THE FOUR QUESTIONS — per shape, flat YES/NO

| shape | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **A — FOLD** | YES | **NO** (transition) | YES | YES | ✅ **PICKED** |
| B — FEDERATE | YES | YES | **NO** | — | ⛔ DISQUALIFIED |
| C — GATE (today) | YES | YES | **NO** | — | ⛔ DISQUALIFIED |
| D — GENERATE | YES | YES | YES | YES | ⬜ **composes with A, after it** |

- **A's Simple = NO is accepted on the precedent's own reasoning**: Simple measures the shipped
  state. Post-fold there is one table, one lookup, no ledgers, and no discipline burden. The
  transition is large and **bounded**; the ongoing cost of six authorities is **unbounded** — every
  new form must be added to each, and nothing but a gate notices when it isn't.
- **B Honest? NO** — a federation has N sources of truth behind one façade. The RULING says
  *eliminate*, not *unify the query*. A federated lookup would let the tables keep drifting and
  return whichever answered first.
- **C Honest? NO** — and this is the sharpest disqualification, because C is **what we have**. The
  RULING already names it: *"a gate that compares two tables is a measurement of the split, not a
  cure for it — this arc has built four of those. They were right as instruments; they are wrong as
  destinations."*
- **D** passes every question and is not an alternative: it cannot run until the registry holds the
  data, which is A.

## ⛔ THE SEQUENCING RULING — builder, 2026-09-02

> *"so... we continue to add names to the registry.... then we attack the hand lists... get this
> known on disk... then we continue..."*

**Register the population FIRST. Attack the hand-lists SECOND.** Not a preference — it is the
RULING's forced order (*registry answers → consumer asks → duplicate dies*) restated after the
campaign proved it, and after it twice tried to run ahead of itself:

- **1a-β-ii** could not flip `is_liftable_declaration_head` until `def`/`defmacro`/`defalias` had
  rows — and could not reach MISSING 0 until `defstruct`, a macro with no possible row, left the
  domain.
- **The mutation pair** (`is_mutation_head`/`is_mutation_form`, which **disagree on disk today**)
  cannot be flipped now: its natural equality is `@Purity Unevaluated`, and its population includes
  the loaders, the config setters, and `defstruct` — 4 unregistered, 2 unregistered, and 1
  unregisterable (`[[NOTE-there-is-a-FOURTH-registry-and-it-holds-defn]]`).

★ **A hand-list attacked before its population is registered produces a stone whose only honest
deliverable is a number.** Both times the campaign learned this, it learned it from a red or a
refusal — never from the plan.

## ⬜ THE REGISTRATION WORKLIST — measured 2026-09-02, not recalled

`src/special_forms.rs` holds **35** rows; **17** are registered; **18** remain, in four families
grouped by shared axis argument:

```
1a-γ  homoiconic 8   quote · quasiquote · unquote · unquote-splicing
                     macroexpand · macroexpand-1 · forms · struct->form
1a-δ  loaders 4      use! · load-file! · digest-load! · signed-load!      ⬅ unblocks the mutation pair
1a-ε  config 2       config::set-redef! · config::set-eval-redef!         ⬅ unblocks the mutation pair
1a-ζ  remainder 3    ann-form · do · stream::lazy
      ⛔ unregisterable 1   defstruct — a stdlib macro; no declare-time fn exists to name
```

⚠ **`defstruct` is the one name in the 18 that no stone can register**, and it sits in the mutation
pair's population where it legitimately belongs (measured twice: `eval-ast!` and `eval_in_frozen`
both see the literal, unexpanded head). **Either the registry learns to answer for macros, or the
mutation pair's equality is not `@Purity Unevaluated` but something that tolerates a macro row's
absence.** That fork is open and is named in the NOTE above.

★ After 1a-δ and 1a-ε land, the mutation pair's population is 12 of 13 registered — and the fork
above becomes the only thing between the campaign and its second hand-list kill.

## Execution decomposition

The RULING's ordering is forced and measured: **registry can answer → consumer asks → duplicate
dies.** A stone that does step 1 without step 2 is unfalsifiable — proven this session, and the
reason the ratchets exist.

```
PHASE 1 — EVERY NAME HAS A ROW                        (the registry can ANSWER)
  1a  the 19 SPECIAL_FORMS rows           ⬅ FIRST: the cast found 5 of 9 rete Form rows
      and/or/cond/quasiquote/...             cannot fold because their targets live here
  1b  the 74 RETE_OPS rows                   needs the alias field — see 2a
  1c  the remaining GAP_B names             the 115, by batches with named arms first
      → GAP_A and GAP_B reach 0

PHASE 2 — THE REGISTRY CARRIES WHAT THE DUPLICATES CARRY
  2a  core_name — the alias field           ⬅ the one genuinely homeless field (Q1)
  2b  the :undefined fallback machinery     ⬅ does NOT decompose; Totality::Partial is a bare
                                               label with no payload (measured by the cast)
  2c  TypeSchemes from the doc types        384/386 measured convertible (PROBE bb1aa686d)
      → DEBT and TYPES_UNCHECKED reach 0

PHASE 3 — CONSUMERS ASK                                (the duplicate can DIE)
  3a  resolve asks the registry             ⬅ kills is_reserved_prefix — THE ARC'S FOUNDING TARGET
  3b  check asks the registry               kills register_builtins + the 118 literal arms
  3c  the property gates ask                kills intrinsic_meta's residue, effectful_by_prefix,
                                               is_expand_time_legal's residue

PHASE 4 — THE DUPLICATES DIE
  4a  delete RETE_OPS, SPECIAL_FORMS, the residues
  4b  delete all four absence ledgers and their gates   ⬅ THE FINISH LINE
```

★ **1a is first on the cast's evidence, not on convenience**: 5 of 9 `OpClass::Form` rows cannot fold
because `and`/`or` live only in `special_forms.rs` and `cond` has **zero runtime registry entry at
all**. Phase 1b is blocked on 1a.

## ⛔ What the campaign must NOT do — each measured this session

- **Delete a table before its consumers can ask.** Flipping the blanket-accept today fails **578 of
  599** corpus files, because the registry cannot vouch for `fn`. Measured, not feared.
- **Assume `OpClass` folds whole.** It decomposes into two bits plus a lookup on the target's own
  entry — *except* `Fallback`, which is genuine machinery with four failure shapes and must survive
  as its own marker.
- **Treat `constructor_meta`/`accessor_meta` or `step_list` as duplicates.** The first two DERIVE
  from the frozen `TypeEnv`; the third declares a capability with `NoStepRule` as its honest refusal.
  **A campaign that cannot tell a duplicate from a derivation deletes correct code.**
- **Ship a stone that moves none of the four ledgers.** That is the progress meter; a stone claiming
  to eliminate duplication while moving none has eliminated none.

## FM 2-bis — a probe per execution stone

Arc 233 ran one before every stone in the chain, and this session has twice had a design refuted by
its own probe (`step_list` is not a door; the tail door fixes no live bug). Each stone below states
its probe before its brief.

★★ And the probe that must exist for the campaign as a whole: **after each phase, re-run the
corpus experiment** (`[[WORKLIST-the-121-the-registry-cannot-vouch-for]]` records the four steps) and
watch 121 fall. That number reaching 0 is what licenses Phase 3a.

## Risks, named in advance

1. **The wrong shape, shipped.** Arc 233's Shape C bit four times before `233.2.g` re-picked. This
   design is re-pickable: if the fold's transition cost turns out unbounded — say `core_name`'s alias
   semantics prove irreducible to a field — Shape D (generate) or a narrower fold is available, and
   the four ledgers will say so before the campaign is deep.
2. **A cascade nobody can attribute.** Phase 3 flips consumers; each flip is its own stone with its
   own floor.
3. **`Fallback`'s machinery.** The one piece with no precedent in `IntrinsicEntry`. Phase 2b may need
   its own shape dialogue rather than a field.
