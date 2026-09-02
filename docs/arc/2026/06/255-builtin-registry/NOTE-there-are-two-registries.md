# NOTE — there are TWO registries, and 55 of the membership gap is just the second one's contents

> **Builder, 2026-09-01:** *"these match.... starts_with..... these feel.... shitty.... the registry
> is not doing any of this right?... it either does or doesn't exist?... the registry is a lookup for
> a call... its either there... or it isn't... right?....."* then, on being shown what that led to:
> **"the registry must be the sole authority for these things................"**
>
> The instinct went past the prefix check it started from. This NOTE records what it found.

## The prefix check was the smallest part

The thing that prompted this — `head.starts_with(RETE_PREFIX)` inside `eval_tail` and
`dispatch_keyword_head_value` — is a fast path in front of a genuine lookup:

```rust
rete_op_for(head)  =  rete_op_index(head).map(|i| &RETE_OPS[i])       // a lookup; misses return None
resolve_core_name(head)  →  "a PURE LOOKUP, zero behavior change for anything not in RETE_OPS"
```

`resolve_core_name` already exists and already has **6+ callers** across `rete/purity.rs` and
`rete/expr_ir.rs`, whose own docs call it *"THE ONE discriminator."* **`runtime.rs`'s two dispatch
sites are the last places hand-rolling it.**

⚠ They are not quite equivalent: the hand-rolled block filters `class == OpClass::Form`;
`resolve_core_name` remaps any rete row. So the `starts_with` is redundant and the class filter is
real. A `resolve_form_core_name` sibling would collapse both sites to one call — small, and **not
the finding.**

## ★★★ THE FINDING — `RETE_OPS` is a second registry

```rust
pub(crate) struct ReteOp {
    rete_name:   &'static str,   // the row's name
    core_name:   &'static str,   // the name it ALIASES
    class:       OpClass,        // Alias 35 · Fallback 20 · Form 9 · Redispatch 10
    params:      &'static [ParamType],
    ret:         ParamType,
    type_params: &'static [&'static str],
}
```

**A name, a signature, type parameters, and a classification. That is `IntrinsicEntry` with different
field names.** 74 rows.

And the overlap is not incidental:

```
RETE_OPS rows ...................................................... 74
of those, sitting in REGISTRY_MEMBERSHIP_GAP_A ...................... 55
   (:wat::rete::i64::> · i64::+ · i64::< · f64::> · … )
```

★★★ **55 of Gap A's 89 are not "names nobody registered" — they are names registered in the OTHER
table.** The intrinsic registry cannot answer for them because a second registry owns them. The
membership gap has been, in large part, a measurement of the split.

## What a fold would have to answer — measured, not assumed

`IntrinsicEntry` already has a home for most of a `ReteOp`:

| `ReteOp` field | `IntrinsicEntry` today |
|---|---|
| `rete_name` | ✅ `name` |
| `params` / `ret` / `type_params` | ✅ `args` / `ret_type` as doc strings — and `PROBE(255)` (`bb1aa686d`) measured **384/386** of those reconstructing a real `TypeScheme`, with **71/71** generic quantifiers recoverable |
| `core_name` | ⛔ **no equivalent — an ALIAS concept the registry has no field for** |
| `class` | ⛔ **no equivalent** |

So exactly two things are missing, and they are the interesting two:

- **`core_name` is an alias**: *"this name means that name."* That is a registry's business by
  definition — and it is what every `starts_with(RETE_PREFIX)` re-mapping in the tree is hand-rolling.
- **`OpClass` is dispatch semantics** the intrinsic registry has no concept of:
  `Alias` (same routine as core) · `Form` (lazy/short-circuiting, no `TypeScheme`) ·
  `Fallback` (alias plus a terminal `:undefined` handler) · `Redispatch` (a plain fn whose type
  **cannot be stated as a rank-1 `TypeScheme` at all** — polymorphic over the container constructor).

⚠ **`Redispatch` is the one that resists.** Its own doc says its type cannot be a rank-1 scheme.
That is not a field the registry lacks; it is a shape rank-1 cannot hold. A fold has to say what
happens to those 10 rows, and "give them a scheme" is not available.

## ⛔ What this NOTE does NOT claim

It does not claim the two tables should merge, and it does not draw a stone. **55 rows and a class
system with a rank-1-resistant member is a design, not a cleanup.** The measured claim is narrower
and is enough to act on:

> **A second registry exists, it owns 74 rows, 55 of them are why Gap A is 89, and nothing measures
> the two against each other.**

★ The parallel worth naming: this is the same shape the campaign has already found three times —
`is_reserved_prefix` (membership by prefix), `effectful_by_prefix` (a property by prefix),
`intrinsic_meta`'s residue (a hand-list beside the registry). Each time the answer was that the
registry should own it. `RETE_OPS` is the largest instance and the only one that is itself a
well-built table rather than a guess.

## ⬜ The open questions, in the order they must be answered

1. **Can a `ReteOp` row be an `IntrinsicEntry`?** Needs an alias field and a class field. The
   signature half is already measured as convertible.
2. **What happens to `Redispatch`'s 10 rows**, whose type is not rank-1 expressible?
3. **Does `OpClass` survive the fold, or does it decompose** into properties the registry already
   has (an alias target, a `handler` vs `tail_handler`, a "no scheme" marker)?

⚠ Question 3 is the one that decides whether this is a fold or a merge. Answering it by reading is
how this campaign has gone wrong; it wants a cast or a probe.
