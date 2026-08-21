# SCORE — arc 109, binder strike α: the Rust declaration parsers accept `:- [T …]`

Rider: one flight, ~8.4 min, no STOP fired. Every row re-run by the orchestrator's own hand on the
built binary.

| # | what | result |
|---|---|---|
| 1–3 | all eight declarator heads take the binder | ✅ defenum · typealias · newtype · typeunion · recordtype · aggregatetype · structtype · defstruct |
| 4 | ★ every `<T>` spelling still works | ✅ additive control holds |
| 5 | ★ the corpus still loads | ✅ floor green |
| 6 | ★★ **`T` is a VARIABLE, not a concrete type** | ✅ **verified by RUNNING** — see below |
| 7 | both spellings at once → error | ✅ names the contradiction |
| 8–10 | `[:a :b]` · `[U [F :-> T]]` · `[T [f :- T]]` → error | ✅ one diagnostic covers all three |
| 11 | floor | ✅ **4855/4855, 0 FAIL, 71.7s** |
| 12 | clippy `-D warnings` | ✅ 0 |
| — | rustfmt parity | ✅ **0 added regions** across all three files (by content) |

Predicted 20–30 min; actual ~8.4. The predicate and the door both already existed.

## Row 6 — the row that could pass hollowly, and how it was proven

EXPECTATIONS named this as the trap: `parse_declared_name`'s whole job is making `T` a *variable*,
and a binder that fills `type_params` with the right strings but reaches the wrong place leaves `T`
a **concrete type** — symptom `"expects :T; got :wat::core::i64"`, which reads like an ordinary type
error while rows 1–3 stay green.

Proven by running the same enum three ways:

```
binder spelling,  Wrap 42        →  #user.Box/Wrap [42]
OLD <T> spelling, Wrap 42        →  #user.Box/Wrap [42]     ← identical
binder spelling,  Wrap "hello"   →  #user.Box/Wrap ["hello"]  ← T ACCEPTS A STRING TOO
```

The third line is the one that makes it non-hollow: a concrete type named `T` could not hold both.

## ⛔ TWO DEFECTS. THE FIRST IS A GAP IN MY BRIEF, AND THE RIDER FOUND IT.

**1 — my room list missed the wrapper that rewrites the arg list.** I listed the seven
`parse_declared_name` callers. I did not list **`parse_structtype`** (`src/types.rs:4142`), which
injects `:wat::core::Struct` as a synthetic parent at position `[1]` and *then* forwards to
`parse_aggregate`. A binder written after the name was displaced behind the injected parent and
swallowed by `parse_aggregate`'s trailing-arity error. Measured before the fix:

```
(:wat::core::structtype :user::S :- [T] [f :- T])  → "expected (:structtype :Name :Parent
(:wat::core::defstruct  :user::S :- [T] [f :- T])  →  [fields]); got 5 args"
```

while both `<T>`-spelled twins passed. **`defstruct`'s wat macro lowers through this head**, so the
declarator the design calls a required site was silently unreachable.

★ **The rider reported it as a finding rather than patching it**, correctly noting it sat outside
the brief's seven-site list. That is the right call and it is why it got fixed properly instead of
locally. Orchestrator fix: `parse_structtype` now CARRIES a `:- [T…]` pair across the injection, so
the binder stays adjacent to the name. Both heads verified clean afterward, both spellings.

⚠ The general shape: **a room list built from "who calls X" cannot see a caller that rewrites the
args before calling X.** `parse_structtype` never calls `parse_declared_name` — that is exactly why
my grep could not find it. `[[feedback_state_what_the_instrument_can_see_before_quoting_it]]`

**2 — my own edit landed inside the rider's doc block.** Anchoring on
`fn take_declared_binder<I: …>(` put my `is_binder_marker` helper and its doc BETWEEN the rider's
doc comment and its function — so the rider's careful doc silently re-attached to my helper and
`take_declared_binder` lost its documentation. Clippy's `doc_lazy_continuation` caught it as a lint;
the real defect was content, not formatting. Moved above the block; both docs now attach correctly.

## Trap-door 4 fired, exactly as predicted, in FIVE of the seven

EXPECTATIONS flagged: *"several parsers count `args.len()` for their arity diagnostics BEFORE
iterating."* Measured outcome — 2 clean, **5 needed the arity gate re-timed** to fire after
name+binder consumption (`parse_newtype`, `parse_typealias`, `parse_typeunion`, `parse_aggregate`,
`parse_defstruct`). None fought `Peekable`; the work was entirely re-timing. The rider judged this
anticipated-rather-than-STOP-2 and was right — the brief's guess ("six adapting, one fighting") had
the ratio inverted, and the trap-door had it exactly.

Old arity errors re-verified honest afterward: `(newtype :N)` → *"got 1 args"*;
`(typealias :A :i64 :String)` → *"got 3 args"*.

## What α did NOT do, affirmatively

- **`defrecord` and `defn`.** Both are wat macros gating on their own arity before Rust sees the
  form — `defrecord` still answers *"expects 2 arguments; got 4"*. β and γ.
- **The `<T>` retirement.** Additive; ③ hard-cuts.
- **`$bound/T` in storage.** `type_params` holds bare names, per `identifier.rs:145`'s own note
  that the namespace is derived today and stored at 251.8b.
