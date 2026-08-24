# DESIGN — STONE: `string::{=,not=}` close the equality family

> **Builder's ruling, 2026-08-24, verbatim:** *"we have equality for all the primitives but string..
> that's a clear miss we've been carrying for a long time - the wat.core/{=,not=} should just be a
> dispatch to the same tooling wat.string/{=,not=} perform -- the core dispatch and per-type explicit
> needs to be supported"* and *"`:wat::string::{=,not=}` join the fray and we replicate the names to
> `:wat::rete::string::*`"*.
>
> ⛔ **And the ruling that produced it:** *"if no one is calling it now doesn't mean no one will call
> it later - if its functional we keep."* **Call-site count is not a test of whether a verb belongs.**
> The orchestrator ruled these out on 1 reference each; that was the wrong instrument.

## The gap, measured

| spelling | registered in `check.rs` | rete `OpExec` arm | `RETE_OPS` row |
|---|---|---|---|
| `:wat::core::=` / `not=` | ✅ generic dispatch | ✅ `Eq`/`NotEq` | ✅ |
| `:wat::core::i64::=` / `not=` | ✅ | ✅ | ✅ |
| `:wat::core::f64::=` / `not=` | ✅ | ✅ | ✅ |
| **`:wat::core::string::=` / `not=`** | ❌ **absent** | ✅ `StrEq`/`StrNotEq` | ❌ **none** |

Probed: `(:wat::core::string::= "abc" "abc")` → `#wat.runtime/UnknownFunction`. The generic covers
strings — `(:wat::core::= "abc" "abc")` → `true` — so nothing is broken today; the family is simply
incomplete, and the rete IR has carried arms for the missing member all along.

★ **The rete row proves the intent AND the workaround.** `vocabulary.rs:1068` declares
`rete_name: ":wat::rete::core::string::="` with `core_name: ":wat::core::="` — the rete surface
already offers string equality, backed by the GENERIC as a stand-in because the typed verb does not
exist. `expr_ir.rs:1260`'s `StrEq` arm is therefore unreachable: no row names it.

## The shape — one tooling, two doors

`values_equal` (`src/runtime.rs:11423`) is the one tooling. It already holds the String arm:

```rust
(Value::String(x), Value::String(y)) => Some(x == y),
```

The generic reaches it via `eval_eq` (`runtime.rs:11347` → `:6196`). **`string::=` must reach the
same arm** — not a second comparison. This is the builder's "dispatch to the same tooling", and it is
the [[feedback_a_slot_with_two_implementations_is_two_slots]] guard made structural.

⚠ **Note the precedent is NOT uniform today:** `i64::=`/`f64::=` route through `eval_compare`
(`runtime.rs:11807`) with an `Ordering` predicate — a different path from the generic's
`values_equal`. So there are already two shapes in the family. **`string::=` follows the GENERIC's
tooling (`values_equal`), because that is what the builder ruled and because equality over Strings is
not an ordering question.** Whether `i64`/`f64` should later converge on the same door is a separate
question and is NOT this stone.

## The rooms

1. **`src/check.rs` ~:17489** — where the string family's `TypeScheme`s are registered. Add
   `String × String -> bool` for both. (`declare-acronyms` is handled separately nearby — do not
   copy its shape; it is a declaration form, not a function.)
2. **`src/runtime.rs:6196`** — the dispatch table beside `":wat::core::=" => eval_eq(…)`. The new
   arms evaluate both operands and land on the SAME String comparison.
3. **`src/rete/purity.rs`** — classify both `pure ∧ deterministic ∧ total`, beside the existing
   `i64::=`/`f64::=` rows at `:532-533`. A verb with no classification trips the purity gate.
4. **`src/rete/vocabulary.rs:1068` and `:1077`** — repoint the two existing rows' `core_name` from
   `":wat::core::="` / `":wat::core::not="` to the new typed verbs. The `rete_name` stays as-is here;
   E renames it later.
5. **`src/rete/expr_ir.rs:1260-1261`** — no edit needed. Repointing (4) makes these two arms LIVE for
   the first time, which is the tell that they were written in anticipation of this stone.

## Out of scope — affirmatively cut

- **Three MORE dead arms.** `expr_ir.rs` also carries `string::starts-with?`, `string::ends-with?`,
  `string::contains?` — all three are REAL registered core verbs (39 / 17 / 101 sites) with IR arms
  but **no `RETE_OPS` row**, so they cannot be used in a rete `where` clause. That is a rete
  CAPABILITY gap, and **rete is grok-rete's** (builder, 2026-08-24). Surfaced for their side; not
  taken here.
- **Whether `i64`/`f64` equality should converge onto `values_equal`.** Named above; its own question.
- **The rename.** `:wat::core::string::*` → `:wat::string::*` is stone E.

## Sequencing — recommendation, builder's call

**This stone FIRST, then E.** It is small and self-contained, and E's codemod then migrates two fresh
call sites for free along with the other seventeen verbs — so the new names land once, which is E's
whole purpose. Minting directly at `:wat::string::*` before E would leave the family split across two
namespaces for the duration of the rename.

## Acceptance

- `(:wat::core::string::= "abc" "abc")` → `true`; `(:wat::core::string::not= "abc" "xyz")` → `true`.
- A non-String operand is REFUSED at check time (this is the typed door's whole point — the generic
  accepts anything comparable; the typed one must not).
- `(:wat::core::= "abc" "abc")` still `true` — the generic is unchanged and still dispatches.
- A rete `where` using `:wat::rete::core::string::=` still compiles, and now routes through `StrEq`
  rather than `Eq`. **A differential probe against the previous behaviour is the load-bearing row** —
  same answers, different IR op.
- Floor green with every move accounted BY NAME; clippy 0.
