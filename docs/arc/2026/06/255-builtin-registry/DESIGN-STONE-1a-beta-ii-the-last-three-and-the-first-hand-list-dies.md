# DESIGN — STONE 1a-β-ii: the last three register, and the first hand-list dies

> The stone the campaign has been walking toward. `MISSING` is **3**; when it reaches 0 the equality
> holds and `freeze::is_liftable_declaration_head` — a nine-name hand-list — becomes a registry query
> and then stops existing.
>
> **Builder:** *"'hand lists'..... these do not survive for long......"*

## The three, measured

| form | declare-time processor | check | eval |
|---|---|---|---|
| `:wat::core::defmacro` | `macros/parse.rs` · `parse_defmacro_form` | shared silent-accept arm | — |
| `:wat::core::defalias` | `declare/parse.rs:228` · `parse_defalias_form` | shared silent-accept arm | — |
| `:wat::core::def` | `declare/register.rs` · `register_defines` | **`infer_def`** (`check.rs:2610`) | a **refusal** (`runtime.rs:2132`) |

★ `def` is the first row in this family with a check impl of its own, and the first to meet the
question below.

## ★★★ THE ONE CONTRACT DECISION — a REFUSAL is not an IMPLEMENTATION

`runtime.rs:2132` answers `:wat::core::def` in expression position with
`DeclarationInExpressionPosition`. It is reached at eval time. **It does not get `role = eval`.**

`role = eval` means *"here is the code that evaluates this form."* `def` has no such code — it has
code that says **this form cannot be evaluated here**. Annotating the refusal would make
`show-source :wat::core::def` present an error-raiser as the form's evaluator, which is a lie about
the substrate told by the authority we are making sole.

★★ And the consequence is load-bearing, not incidental: with no `Eval` role and no handler, `def`
keeps **`@Purity Unevaluated`** and the gate
`unevaluated_purity_carries_no_route_to_evaluation` stays satisfied by measurement rather than by
exception. **A refusal arm is the ENFORCEMENT of `Unevaluated`, not a counterexample to it.**

⛔ The alternative was measured and rejected: declaring `def` `@Purity Effectful` walks straight back
into `effectful_by_prefix`'s census — `:wat::core::` is not one of its eight prefixes — reopening the
exact red that `Purity::Unevaluated` was minted to close
(`[[NOTE-the-prefix-guess-has-run-out-of-road]]`).

## ★★ A measured finding this stone does NOT fix, recorded because it is the campaign's payoff

Nine declaration forms in expression position, at runtime:

```
(:wat::core::def …)        →  DeclarationInExpressionPosition   ← names the actual mistake
(:wat::core::defenum …)    →  UnknownFunction                   ← "no such verb"
(:wat::core::typealias …)  →  UnknownFunction
```

**`def` has a hand-written arm that gives a good diagnostic; its eight siblings fall through to the
generic fallback and tell the user their form does not exist — when it exists and is merely
misplaced.** A registry that knows `Category::Declaration` can give all nine the same named refusal.

⚠ That is a *consequence* the campaign unlocks, **not this stone's scope**, and not a deferral: it is
a new capability, nameable only because the registry now holds the category. Its own stone.

## The kill — and what replaces the hand-list

```
is_liftable_declaration_head(head)
    ≡  registry().lookup_entry(head) names a SpecialFormRole::Declare impl
```

Its **one** production caller is `closure_extract::split_body_prelude`
(`[[NOTE-there-are-TWO-is_declaration_form]]` — the other two call sites belong to `declare`'s
homonym, now renamed apart).

**The predicate does not survive as a thin wrapper in `freeze.rs`.** A predicate about *what a name
is* living in the freeze module is an authority in the wrong home — the shape this arc exists to
remove. `split_body_prelude` asks the registry directly, through one accessor that lives WITH the
registry.

⚠ **The meter dies with it.** `liftable_declaration_head_missing_and_foreign` reads
`freeze.rs`'s source for its domain; once there is no hand-list, there is no domain to read and
nothing to be MISSING from. **An absence ledger that has reached zero is deleted, not kept green** —
the RULING's own finish-line condition, executed for the first time in this campaign.

## THE FOUR QUESTIONS

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **register 3 · flip the caller · delete the predicate AND the meter** | YES | YES | YES | YES | ✅ **PICKED** |
| register 3, keep the predicate as a registry-query wrapper | YES | YES | **NO** | — | ⛔ DISQUALIFIED |
| register 3, flip, keep the meter green at 0 | YES | YES | **NO** | — | ⛔ DISQUALIFIED |
| give `def` `role = eval` on the refusal arm | YES | YES | **NO** | — | ⛔ DISQUALIFIED |
| split: register 3 now, flip later | YES | YES | **NO** | — | ⛔ DISQUALIFIED |

- **wrapper Honest? NO** — it leaves a name-classifying authority in `freeze.rs`. The hand-list is
  gone but the misplaced home is not, and the next reader still asks freeze what a name is.
- **keep-the-meter Honest? NO** — a gate whose domain no longer exists asserts over an empty set and
  can never fail. That is the "gate as destination" the RULING disqualifies.
- **`role = eval` Honest? NO** — see the contract. It also re-opens the purity census.
- **split Honest? NO** — MISSING reaching 0 and the kill are one act; separating them ships a stone
  whose only deliverable is a number, and leaves the campaign's first kill un-taken with no reason.

## Blast radius

```
src/intrinsic/special/    +3 doc-only structs (+3 mod lines)
declare/register.rs · declare/parse.rs · macros/parse.rs   3 role = declare annotations
check.rs                  1 role = check annotation (infer_def) — def's own, not the shared arm
src/freeze.rs             is_liftable_declaration_head DELETED
src/closure_extract.rs    split_body_prelude asks the registry
src/intrinsic/mod.rs      + the accessor · the meter DELETED · 3 debt-ledger names
tests/macros/probe_declaration_form_lift.rs   its membership half retires with the predicate
```

## Acceptance — rows chosen to be unfakeable

| what | command | expected |
|---|---|---|
| MISSING reached 0 | before deleting the meter, one run | empty, then the meter goes |
| ⛔ the LIFT still works | a fn body with a leading `def`/`defenum` prelude | lifted exactly as before |
| ⛔ the lift is not vacuous | a fn body whose first form is NOT a declaration | not lifted — the boundary still discriminates |
| the hand-list is gone | `grep -c "fn is_liftable_declaration_head"` | 0 |
| ⛔ nothing else lost its answer | `is_mutation_form`/`is_mutation_head` | untouched, still guarding |
| `def` is not evaluatable | `(:wat::core::def …)` in expression position | still `DeclarationInExpressionPosition` |
| ⛔ the `Unevaluated` gate still bites | give `def` a `role = eval` | RED, naming it |
| ledgers | `GAP_B` · `KNOWN_UNREVIEWED` | each drops `:wat::core::def` |
| floor | `scripts/floor.sh`, exit UNPIPED | green, and the count DROPS by one (the meter retires) |
| clippy | `-D warnings --all-targets` | 0 |

★ **The floor's total going DOWN is the acceptance row that matters**, and it is the first time in
this campaign. A ratchet that has done its job is deleted; one that is kept at zero is theatre.

## Out of scope = REJECTED

- **The eight siblings' generic `UnknownFunction`.** Named above; its own stone.
- **`declare`'s six-name population** (`DECLARATION_HEADS`, `is_declaration_form`) — a different
  question with more consumers, and not a Declare-impl query.
- **`is_mutation_form`/`is_mutation_head`.** They guard unexpanded AST; measured and kept twice.
- **`check.rs:4866`'s silent-accept arm.** Still unmeasured for deadness; its own stone.
