# DESIGN — STONE 2 of 2: the mirror wall — refuse an `ExpandOnly` head in program code

> **Builder, 2026-09-01:** *"stone 2.. we continue.."*
>
> Stone 1 minted `:ExpandOnly` and derived the doc gate's branch. **Zero behaviour changed.**
> This stone is the behaviour change: the wall that was only ever built on one side.

## The wall, and the half that is missing

```
macros/eval.rs:169   is_expand_time_legal(head)   refuses RuntimeOnly INSIDE a macro body   ✅ built
program code                                      refuses ExpandOnly OUTSIDE one           ⛔ absent
```

Measured against the stone-1 binary, the case this stone exists to close:

```
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::core::macro-error "boom"))
  --check  EXIT 0
  run      #wat.runtime/MacroAbort {:message "boom" …}   EXIT 1
```

**Misuse is caught at RUNTIME, by a raise** — the bottom rung. `ExpandOnly` declares the verb has no
runtime call site; nothing enforces it.

## ★★★ THE PROBE — and it makes the wall far simpler than expected

The obvious design is *"refuse it unless we are inside a macro body,"* which needs macro-body context
the checker does not carry. **That context is unnecessary.** Three probes, run against the stone-1
binary:

```
A  an UNRESOLVABLE call inside a defmacro body     --check EXIT 0   the walk never descends
C  macro-error inside a defmacro body              --check EXIT 0   the legitimate case
B  macro-error at top level, in a defn body        --check EXIT 0   the case to refuse
```

Probe A is the load-bearing one: `(:user::definitely-not-a-verb 1 2 3)` inside a `defmacro` body
passes `--check` clean. Corroborated by reading `check.rs:4871-4883` — `:wat::core::defmacro` and
`:wat::core::quasiquote` both `return` **without descending**: *"declaration forms, not
value-producing expressions"* and *"Body isn't fully type-checked (it's a template)."*

★ **So a legitimate `ExpandOnly` call is structurally unreachable by any post-declaration walk.**
The wall does not need to ask "am I in a macro body?" — it needs only to fire wherever it looks,
because the one place it must not fire is a place it cannot see.

⚠ **And the case that DOES survive is exactly the one worth catching:** a macro whose template
*quotes* a `macro-error` call emits it into expanded program code, where it would run at runtime and
raise. That is a real defect today, invisible until it fires.

## ⛔ THE SITE IS A DEPENDENCY DECISION, NOT A TASTE ONE

The natural-reading site is the type checker. **It is the wrong one**, and the reason is the
builder's own sequencing (*"we break the mega files up first… then crates"*):

```
check.rs   -> crate::intrinsic     0 references today
intrinsic/ -> crate::check         3 references
```

Putting the wall in `check.rs` **creates a fresh `check → intrinsic → check` cycle**, on top of the
`check ↔ runtime` cycle already measured. A crate graph is a DAG; every new cycle is paid for at
step 2. `[[NOTE-the-crate-boundary-is-the-real-cut-and-eight-homes-are-cyclic]]`

| candidate site | registry edge today | new cycle? | owns the tier question? |
|---|---|---|---|
| `src/check.rs` | none (0 refs) | ⛔ **YES** — `intrinsic → check` is 3 | no — it is a TYPE checker |
| `src/resolve/` | none (0 refs) | no (`intrinsic → resolve` is 0) | ⛔ its own comment scopes it out |
| **`src/macros/`** | ✅ **already has it** | **no** | ✅ **it owns the other half** |

- **`resolve/` is disqualified by its own text.** `walk.rs:262-267`: *"the name-resolution pass is
  scoped to catch 'no such namespace' mistakes, not 'wrong name inside a known namespace' mistakes…
  leaf-level validation is the type checker's concern."* Putting a leaf-level tier rule there
  contradicts the scoping statement arc 255 is otherwise trying to honour.
- **`macros/` already holds `is_expand_time_legal`** — the wall's existing half — and
  `expand_all(forms: Vec<WatAST>, …)` (`src/macros/expand.rs:23`) receives the **whole program's
  forms**. Both halves of one wall, one module, **zero new edges**.

## THE ONE CONTRACT DECISION — pinned

**The two halves of the expand-time wall live in one module and are read together.** A tier rule
split across `macros/` and `check.rs` is two authorities on one question — the exact shape this
campaign has spent the day retiring.

## ★ THE PREDICTION — falsifiable, and the controls matter more than the target

```
macro-error in a defn body        --check EXIT 0  ->  REFUSED at check time
macro-error inside a defmacro     --check EXIT 0  ->  UNCHANGED, still legal
a macro EXPANDING to macro-error  raises at run   ->  REFUSED (a defect made visible)
every other verb                  unchanged       ->  UNCHANGED
the floor                         5111 green      ->  5111 green + the new refusal's own test
```

⚠ **The control is the whole stone.** If probe C flips, the wall fired where the verb is legal and
`macro-error` is dead at its only call site. **That control must be written before the wall.**

## Out of scope = REJECTED (not deferred)

- **Any second `ExpandOnly` verb.** Population is **one** (measured). The wall guards one call site
  today; a second candidate is a finding, not a chore.
- **`macros/eval.rs:495`'s dead residue row.** `:wat::core::macro-error` still sits in the 58-name
  unregistered-verb residue although it IS registered, so the registry branch returns early and the
  row is unreachable text. Real, found by the stone-1 rider, **and not this stone's** — it is the
  residue's own documented cleanup, not the wall.
- **Moving `is_expand_time_legal`.** It stays where it is; this stone joins it.

## THE FOUR QUESTIONS — flat YES/NO

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **the wall in `macros/`, beside its other half** | YES | YES | YES | YES | ✅ **ADMITTED** |
| the wall in `check.rs` | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |
| the wall in `resolve/` | **NO** | YES | YES | — | ⛔ **DISQUALIFIED** |
| carry macro-body context so the wall can ask "am I inside one?" | YES | **NO** | YES | — | ⛔ **DISQUALIFIED** |
| leave it: the runtime raise is enough | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |

- **`check.rs` Honest? NO** — it creates a `check → intrinsic → check` cycle days before a ruled
  crate migration whose blocker is precisely cycles. Convenient now, paid for twice later.
- **`resolve/` Obvious? NO** — its own comment says leaf-level validation is not its job. A rule
  placed against a module's documented scope is a rule the next reader will move.
- **carry-the-context Simple? NO** — threading a new "am I in a macro body" flag through a walk, to
  discriminate a case probe A proves is unreachable. Machinery for a state that cannot occur.
- **leave-it Honest? NO** — the axis would then declare a property nothing enforces, which is what
  `ExpandTime` already did before stone 1. Minting a pole and not walling it is a doc comment with a
  variant's clothes on.

## Acceptance

| what | command | expected |
|---|---|---|
| ★ THE CONTROL, written FIRST | `macro-error` inside a `defmacro` body | `--check` 0 — **UNCHANGED** |
| the target | `macro-error` in a `defn` body | `--check` **REFUSED**, located, named |
| ★ the expanded-quote case | a macro whose template quotes `macro-error` | **REFUSED** (a defect made visible) |
| the error names the tier | the refusal text | says expand-time-only and where it IS legal |
| no new dependency edge | `grep -c "crate::intrinsic" src/check.rs` | still **0** |
| nothing else moves | every other verb's behaviour | unchanged |
| floor | `scripts/floor.sh`, exit read UNPIPED | 5111+ , 0 failed |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |
