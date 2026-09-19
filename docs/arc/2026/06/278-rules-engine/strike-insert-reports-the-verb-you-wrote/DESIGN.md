# DESIGN — `insert` reports an error naming `insert-all`

`conferre` L2-2 (`../vigilia-2026-09-05/reports/conferre.md:62-66`), re-grounded **and driven** at
HEAD 2026-09-07.

## ⚠ Severity — LIVE and user-visible, unlike the last several strikes

Not latent. A user who writes `:wat::rete::insert` is told about a verb they never typed.

## Driven, not read

```wat
(:wat::rete::insert s (:l22::F 1) (:wat::core::i64::+ 1 1))
```

→

```
#wat.runtime/TypeMismatch {:message ":wat::rete::insert-all: expected :wat::core::Record (a fact),
got wat::core::i64 `2`" … :op ":wat::rete::insert-all" …}
```

The program contains no `insert-all`. Both the message and the structured `:op` field name it.

## ★ And the parameter exists precisely to prevent this

`src/rete/kernel/insert.rs:142-145`, on the checker:

> *"Takes `op` rather than hardcoding one **so the same check serves `insert` and `insert-all` and
> reports the verb the user actually wrote**."*

The rule is written down, the mechanism is built — and the caller hardcodes past it.
`insert_facts_on_session` (`:191`) declares `const OP: &str = ":wat::rete::insert-all"` (`:197`) and
hands it to both `require_session_agg` and `require_record_fact`, while being reached from **two**
entry points:

| entry | the verb the user wrote | the verb reported |
|---|---|---|
| `eval_insert_all_native` (`:240`) | `insert-all` | `insert-all` ✓ |
| `eval_insert_public`, 3+-arity arm (`:80`) | **`insert`** | **`insert-all`** ✗ |

Same shape as eight cures this arc: **the correct rule, applied in exactly one place.** Here it is
worse than usual — the parameter that carries the rule is threaded to the checkers and then fed a
constant.

## THE ONE CONTRACT DECISION

**Thread `op` into `insert_facts_on_session` from its callers; delete the `const`.**

```rust
fn insert_facts_on_session(session: …, facts: …, list_span: &Span, sym: …, op: &'static str)
```

`eval_insert_public` passes `":wat::rete::insert"`; `eval_insert_all_native` passes
`":wat::rete::insert-all"`. Nothing else changes — the checkers already take `op` and already do the
right thing with it.

**Not** a runtime lookup of "what did the user write" — the entry point already knows, statically,
and the `&'static str` keeps it a constant at each site.

## The gate

A probe pair under `tests/rete/` asserting the **structured `:op` field**, not a substring of the
rendered message, for **both** entries:

- `insert` with a non-Record → `:op` is `:wat::rete::insert`
- `insert-all` with a non-Record → `:op` is `:wat::rete::insert-all`

Two arms, because one arm alone cannot tell "threaded correctly" from "hardcoded to the other
constant" — the mistake this strike is fixing would pass a single-arm test that only checked
`insert-all`.

Not a `.wat.bad`: this is a **runtime** `TypeMismatch`, not a check-time refusal, so the fixture
loads and type-checks and fails when driven.

## Mutation proof — available, and it is the driven evidence itself

Re-hardcode `op` at one entry and the corresponding arm must RED naming the wrong verb. Both arms
must be shown, because the defect is precisely that one of two callers was right.

## Out of scope = REJECTED

- **`conferre` L2-1** — the leading-accumulate seed asymmetry. Confirmed live at HEAD
  (`accumulate.rs:138` unguarded; `filter.rs:107,135` carries the `leading_emitted` guard), but its
  report is explicit: *"the downstream join consequence is inferred and **not driven** … Do not
  treat it as a defect until someone fires a leading accumulate feeding a HashJoin across ≥2 rounds
  and counts the rows."* That measurement is its own strike.
- **`conferre` L2-3** — native's stratify `+1` vs the oracle's *"NOT +1"*, with the oracle's own
  header claiming lockstep. Confirmed live; *"nothing compares strata"*, so it needs an instrument
  before it needs a cure. Its own strike.
- Changing what `insert`/`insert-all` accept, or the arity dispatch.

## STOP triggers

1. Either arm passes before the fix → STOP; the probe is not reading the `:op` field.
2. Threading `op` requires a signature change outside `src/rete/kernel/insert.rs` → STOP.
3. Any behaviour other than the reported verb changes → STOP.
