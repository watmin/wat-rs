# DESIGN — STONE 1a-β-i: the type-declaration family joins the registry

> Phase 1a-β of `[[DESIGN-CAMPAIGN-the-registry-becomes-the-sole-authority]]`. Five of the nine names
> in `freeze::is_declaration_form` — the hand-list picked as the campaign's first kill because it is
> the only one of the five with no `starts_with`, and therefore a pure set.
>
> **Builder, 2026-09-01:** *"'hand lists'..... these do not survive for long......"*

## ★★★ The correction that makes this stone possible — the vehicle is NOT `@Category`

`[[NOTE-the-sloppy-registries-a-measured-census]]` claimed `@Category` was the annihilation vehicle
for the five hand-lists. **Measured, and it is wrong.** `:wat::string::declare-acronyms` already
declares `@Category Declaration` and is **not** in `is_declaration_form` — it is an ordinary
evaluated intrinsic that happens to register something. Flipping the consumer to a category query
would silently capture it.

```
Category::Declaration     = what the verb DOES         (it registers a program-level entity)
is_declaration_form       = a STRUCTURAL fact          (freeze must lift this head from a body)
```

★ The right equality is the role Stone 1a-β-0 just minted:

```
is_declaration_form(h)   ≡   the registry row for h names a Declare impl
```

Derived from a fact the row **proves by construction** — it names the freeze-time fn that processes
it. `declare-acronyms` carries a handler and no Declare impl, so it is excluded by measurement rather
than by exception. **`SpecialFormRole::Declare` earns its keep beyond reflection.**

## The five, and why they are ONE stone

```
:wat::core::defstruct    → parse_defstruct     (src/types/defstruct.rs:520)
:wat::core::structtype   → parse_structtype
:wat::core::defenum      → parse_defenum
:wat::core::newtype      → parse_newtype
:wat::core::typealias    → parse_typealias
```

`parse_type_decl` is a **pure router** — measured, every arm delegates to a dedicated parser — so each
row has an unambiguous, form-specific declare-time fn. No shared-router judgment calls, and no
stacking needed (though stacking is proven precedent at `check.rs:15553`).

★★ **And all five share every axis verdict, which is what makes five rows tractable rather than five
stones.** The shared ground, measured:

| axis | verdict | ground |
|---|---|---|
| `@Category` | `Declaration` | registers a program-level entity, visible to everything after it |
| `@Purity` | **`Unevaluated`** | ⬇ see below — the decisive measurement |
| `@Determinism` | `Deterministic` | same form + same preceding declarations → the same `TypeDef` |
| `@Totality` | `Partial` | each parser raises `TypeError` on a malformed declaration; a raise is not an outcome |
| `@ExpandTime` | `RuntimeOnly` | needs the `TypeEnv` `register_types` builds in source order — state that does not exist while a `defmacro` body expands |

⚠ **Sharing a verdict is not sharing a ground.** Each row argues its own; where the argument is
genuinely the same one, it cites `defsurface`'s row as the precedent rather than being retyped.

## ★★★ THE DECISIVE MEASUREMENT — none of the five is ever evaluated

Each appears in `src/runtime.rs` exactly **once**, and that one occurrence is inside
**`is_mutation_head`** — one of the five hand-lists — **not a dispatch arm**. There is no eval arm,
no tail arm, and no handler for any of them.

**So all five take `@Purity Unevaluated`, for exactly the reason `defsurface` did**
(`[[DESIGN-STONE-1a-beta-0b-a-form-that-never-evaluates]]`): every consumer of `@Purity` asks a
runtime question, and these forms have no runtime. `Pure` would trip the runnable-example mandate on
a form that cannot be run; `Effectful` would claim an effect there is no call to have; `Preserving`
would claim sub-forms that are never evaluated.

★ And the pole pays for itself immediately: minted for one row a stone ago, it is the honest answer
for five more without a single consumer change.

## THE ONE CONTRACT DECISION — pinned

**This stone is a step-1 stone — "the registry can ANSWER" — and it says so.** It registers five
names and flips no consumer. `is_declaration_form` still holds nine names and still decides;
`is_mutation_head`/`is_mutation_form` are untouched.

⛔ **The RULING's standing verification applies and this stone must not evade it:** *"a stone that
claims to eliminate duplication and moves none of the ledgers has eliminated none."* None of the five
is in `GAP_A`/`GAP_B`/`DEBT`/`TYPES_UNCHECKED`/`KNOWN_UNREVIEWED` — measured — so **this stone brings
its own meter**, below, rather than borrowing credit from a ledger it cannot move.

## ★★ THE METER — the equality's first half, buildable now

A **bidirectional** gate over the fixed 9-name domain of `is_declaration_form`:

```
name in is_declaration_form  ∧  no Declare impl   →  MISSING   (the worklist, shrinking)
Declare impl                 ∧  name not in the 9 →  FOREIGN   (a role claimed off-domain)
```

```
MISSING   8  →  3      after this stone (def · defmacro · defalias remain)
FOREIGN   0  →  0
```

**MISSING reaching 0 is what licenses 1a-β-ii to flip the consumer and delete the hand-list.** The
number can only fall by registering, never by editing the gate — the domain is the hand-list itself,
read from `freeze.rs`, not a second copy.

⚠ The gate's domain must be the ACTUAL `is_declaration_form`, not a transcription of it. A frozen
copy would be a sixth hand-list, which is the joke this campaign cannot afford to make.

## THE FOUR QUESTIONS

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **five rows + the MISSING/FOREIGN meter** | YES | YES | YES | YES | ✅ **PICKED** |
| all nine + flip + delete, one stone | YES | **NO** | YES | — | ⛔ DISQUALIFIED |
| five rows, no meter | YES | YES | **NO** | — | ⛔ DISQUALIFIED |
| flip the consumer to `@Category Declaration` | YES | YES | **NO** | — | ⛔ DISQUALIFIED |
| one stone per form | YES | YES | YES | **NO** | ⛔ DISQUALIFIED |

- **all-nine Simple? NO** — three more forms with three unrelated processors (`register_runtime_defs_form`,
  `parse_defmacro_form`, `parse_defalias_form`), plus `def`'s own ledger movement in `GAP_B` and
  `KNOWN_UNREVIEWED`, plus the consumer flip and the deletion. Four concerns wearing one hat.
- **no-meter Honest? NO** — it would be a stone that moves nothing measurable and asks to be believed.
- **category-flip Honest? NO** — measured above: it captures `declare-acronyms`.
- **one-per-form Good UX? NO** — five stones each re-arguing one shared axis table.

## Blast radius

```
src/intrinsic/special/     + five doc-only structs (+ their mod lines)
src/types/defstruct.rs · src/types.rs · src/types/*   five #[wat_special_form_impl(role = declare)]
src/intrinsic/mod.rs       + the MISSING/FOREIGN meter
```

No `.wat` corpus change. No consumer flip. No hand-list edited. No dispatch, checker or freeze
behaviour moves — **every one of the five is unevaluated, so there is no runtime path to change.**

## Acceptance — rows chosen to be unfakeable

| what | command | expected |
|---|---|---|
| the meter moved | the MISSING list | `8 → 3`, naming `def`·`defmacro`·`defalias` |
| ⛔ the meter can FAIL — MISSING | drop one row's `role = declare` | RED, naming that row |
| ⛔ the meter can FAIL — FOREIGN | annotate `declare-acronyms` `role = declare` | RED, naming it |
| ⛔ the meter reads the REAL list | change `is_declaration_form`'s arms | the domain changes with it |
| the `Unevaluated` gate covers them | `unevaluated_purity_carries_no_route_to_evaluation` | inspects 6, all clean |
| ⛔ nothing gained a runtime path | `grep` each name in `runtime.rs` | still exactly 1 hit, in `is_mutation_head` |
| `@syntax` is FQDN-headed and REAL | `--check` each declared grammar | parses; verified against its parser |
| floor | `scripts/floor.sh`, exit UNPIPED | 5124/5124, 0 failed |
| clippy | `-D warnings --all-targets` | 0 |

## Out of scope = REJECTED (affirmatively)

- **`def`, `defmacro`, `defalias`** — 1a-β-ii, with the flip and the deletion. `def` alone carries
  `GAP_B` and `KNOWN_UNREVIEWED` movement and three distinct processors.
- **Flipping or deleting any hand-list.** MISSING is 3, not 0. Flipping now would be a measured lie.
- **`is_mutation_head`/`is_mutation_form`.** They ask a different question (may this head be
  EVALUATED) than `is_declaration_form` (must this head be LIFTED). One kill at a time, and the
  second needs its own equality.
