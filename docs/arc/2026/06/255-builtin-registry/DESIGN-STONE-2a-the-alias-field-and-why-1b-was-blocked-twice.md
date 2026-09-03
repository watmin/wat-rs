# DESIGN — STONE 2a: the registry learns what an alias is — and why 1b was blocked twice

> I told the builder Phase 1b was *"no longer blocked"* because `and`/`or` had registered. **That
> was one of its two blockers, and I cited it without measuring the other.**
> `[[feedback_i_cited_a_rule_instead_of_measuring_whether_it_applied]]` — the campaign design has said
> *"1b needs the alias field — see 2a"* since the day it was written.

## The two blockers, measured

**① The registry cannot express an alias.** `IntrinsicEntry` has 22 fields; **none of them is
`core_name`.** A `ReteOp` row's entire content is *"this name means that name"*:

```rust
ReteOp { rete_name: ":wat::rete::i64::>", core_name: ":wat::i64::>", class, params, ret, meta }
```

and `resolve_core_name` — the hand-rolled resolver — has **8+ callers** across `rete/purity.rs` and
`rete/expr_ir.rs`, each re-deriving the mapping from `RETE_OPS`.

**② `role = eval` cannot stack, and 74 rows share ONE dispatch fn.**
`dispatch_rete_op`'s `Alias | Form | Redispatch` arm — **54 of the 74** — is a single line:

```rust
OpClass::Alias | OpClass::Form | OpClass::Redispatch =>
    dispatch_keyword_head_value(op.core_name, args, list_span, env, sym)
```

One fn, 74 FQDNs, and the shim is keyed on the fn identifier
(`[[NOTE-role-eval-cannot-stack-and-the-error-does-not-say-so]]`). Registering them with
`role = eval` is a duplicate-symbol compile error 74 times over, and the "fix" — 74 identical
delegates — would be 74 lies about how the substrate dispatches.

## ★★★ THE CONTRACT — the alias field is not metadata, it is the DISPATCH

Both blockers dissolve at once if the field is not a doc string but the thing the door reads:

```
registry().lookup_entry(head)  →  alias_of: Some(core)   ⇒  dispatch `core` instead
```

**An alias row needs no handler, no eval role, and no delegate.** The registry answers *"this name
means that name"*, and the door follows the answer. `dispatch_rete_op`'s 54-row arm stops being a
`match` on a class and becomes what it always was: a re-dispatch the registry can state.

★ That is Shape D — GENERATE — arriving early for one field: the class system's largest arm becomes
**derived output** rather than a hand-maintained table.

⚠ **It does NOT dissolve `Fallback`'s 20 rows.** Those carry real machinery — an arity split and a
terminal `:undefined` handler — and the campaign design already ruled that *"`Fallback` is genuine
machinery with four failure shapes and must survive as its own marker"* (Phase 2b). This stone
changes nothing for them.

## The shape

```
IntrinsicEntry              + alias_of: Option<&'static str>
IntrinsicSubmission         + the same, threaded
SpecialFormSubmission       + the same
the doc grammar             + @alias <fqdn>, declared once per row
crates/wat-doc              parse it; its legal-values message and gate, if it has one
crates/wat-macros           thread it through both proc-macros
the dispatch door           lookup_entry(head).alias_of ⇒ re-dispatch, before the literal match
```

## ⛔ THE GATE — an alias must point at something that exists

A field naming a target nobody checks is the drift this campaign exists to kill, and it would be
worse than a hand-list because it *looks* derived:

> **Every `alias_of` value must itself be a registered row**, and the chain must terminate — no
> alias may point at another alias.

★ Both halves are structural and cheap. The second matters: a cycle or a chain would make dispatch
non-terminating or order-dependent, and `RETE_OPS` today has no chains — a fact worth freezing while
it is still true rather than discovering when one appears.

## THE FOUR QUESTIONS

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **`alias_of` as the dispatch fact, + the target gate** | YES | YES | YES | YES | ✅ **PICKED** |
| fix the eval-shim naming, give all 74 `role = eval` | YES | **NO** | **NO** | — | ⛔ DISQUALIFIED |
| `alias_of` as documentation only, keep `dispatch_rete_op` | YES | YES | **NO** | — | ⛔ DISQUALIFIED |
| 74 individual eval delegates | **NO** | **NO** | **NO** | — | ⛔ DISQUALIFIED |

- **fix-the-shim Honest? NO** — it would let 74 rows each name a handler that is really one shared
  re-dispatcher. The registry would assert 74 implementations where there is one alias. **Simple? NO**
  — it is a proc-macro change in service of a shape we do not want.
- **documentation-only Honest? NO** — the RULING's whole complaint: a second copy of a fact, with the
  original still authoritative. `resolve_core_name`'s 8 callers would keep reading `RETE_OPS`.
- **74 delegates** fails everything; named only because it is the path of least resistance from where
  the campaign stands.

## The witness — and why one row, not seventy-four

A field with no user is dead, and 1b's 74 rows are a separate stone. **This stone registers exactly
one alias row** — the smallest, most mechanical: `:wat::rete::i64::+` → `:wat::i64::+`, `Alias`
class, `OpMeta { pure, deterministic, total }` all true (73 of the 74 share that triple).

⛔ **And its acceptance is behavioural, not structural:** `(:wat::rete::i64::+ 1 2)` must still
evaluate to `3`, through the registry's alias rather than through `RETE_OPS`' arm.

## Acceptance — rows chosen to be unfakeable

| what | expected |
|---|---|
| the alias DISPATCHES | `(:wat::rete::i64::+ 1 2)` → `3`, with `RETE_OPS`' arm unreachable for that row |
| ⛔ the other 73 are unchanged | each still routes through `dispatch_rete_op` exactly as today |
| ⛔ the gate can FAIL — dangling | point an `@alias` at `:wat::core::zorble` → RED naming it |
| ⛔ the gate can FAIL — chained | point an `@alias` at another aliased row → RED naming the chain |
| ⛔ NON-VACUITY | the gate inspects ≥ 1 row and names it |
| a non-alias row is untouched | every existing row has `alias_of: None` and dispatches as before |
| floor · clippy | green · 0 |

## Out of scope = REJECTED

- **The other 73 rows.** Phase 1b, unblocked by this stone.
- **`Fallback`'s machinery.** Phase 2b; ruled to survive as its own marker.
- **Retiring `resolve_core_name` or `RETE_OPS`.** Consumers ask, then the duplicate dies — and 1 of
  74 registered is not the moment.
