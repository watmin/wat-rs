# NOTE — a PARAMETRIC struct passes the containment gate, and the fix is proven

> **Written 2026-08-26 from branch `grok-rete` (arc 278), for whoever owns 293.W.**
> Found by `secare` cast against `wat/gen.wat`; reproduced, diagnosed, and the proposed fix
> **implemented, floor-tested, and then REVERTED** so this arc lands it deliberately. `grok-rete`
> is unmodified here — `src/check.rs` is byte-identical to what it was.

---

## The hole, in two probes

`defrecord` correctly refuses a **bare** function field:

```wat
(:wat::core::defrecord :user::Holder
  [card <- :wat::core::i64
   at   <- [:wat::core::i64 :-> :wat::core::i64]])
```
```
#wat.type/ImpureFieldInPureAggregate — pure aggregate ":user::Holder" may only hold pure
fields — field "at" has impure (struct) type "[:wat::core::i64 :-> :wat::core::i64]".
```

It **accepts the same function** when it is wrapped in a parametric struct:

```wat
(:wat::core::defrecord :user::Wrap
  [g <- (:wat::gen::Gen :- [:wat::core::i64])])       ;; loads CLEAN
```

`:wat::gen::Gen` is `(defstruct Gen :- [T] [card <- i64  at <- [i64 :-> T]])` — a struct precisely
*because* it carries a function. Wrapped, it crosses a comms boundary and arrives with `at` dead:

```
:at #wat.core/fn nil          (src/edn_shim.rs)
```

`card` arrives honest, `at` arrives nil, and **nothing in the value says so.**

## Why

`is_pure_type`'s `TypeExpr::Parametric` arm (`src/check.rs:13782`) resolves a head in three steps:

1. `is_registered_rust_opaque(head)` → impure;
2. a **hardcoded list** of impure heads (`Sender`, `Receiver`, `Peer`, `Thread`, `Process`, …);
3. otherwise **"pure iff all type args are pure."**

`:wat::gen::Gen` is a wat-defined stdlib struct. It is not a registered Rust opaque and it is not
in the hardcoded list, so it falls to (3) — and `(Gen :- [i64])` is judged pure because `i64` is.

**The arm never asks the `TypeEnv` what the head IS.** Struct-ness is a property of what the
aggregate *holds*, not of how it is instantiated, so no amount of arg-checking can recover it.

### This has happened before, and the previous fix was the stem

The comment at `src/check.rs:13804` records it verbatim:

> *"A peer of ANY locus holds a live resource … Its three siblings were absent, so each fell
> through to 'pure iff its type args are pure': a `(Peer :- [i64 String])` was judged PURE and
> `validate_aggregate_containment` admitted it into a pure Record — i.e. into a defservice
> `:durable`, and onto the wire."*

The fix then was to add `Peer`/`Thread`/`Process` to the hardcoded list. That closed three
instances and left the class open: **any parametric struct not on the list still falls through.**
`Gen` is simply the next one to be noticed, and it will not be the last — the list is a
convention, and this arc's own doctrine is that a convention is a failure class waiting for a
tired afternoon.

## The proposed fix — ask the registry, do not extend the list

The mechanism already exists: `validate_aggregate_containment` (`src/check.rs:13945`) reads
`TypeDef::Aggregate`'s `a.nature.is_pure()`. The Parametric arm should consult the same thing,
placed **before** the pure-if-args-pure fallthrough:

```rust
// A parametric head that names a STRUCT is impure whatever its args are:
// struct-ness is about what the aggregate HOLDS, not how it is instantiated.
// Note the colon: `TypeExpr::Parametric.head` carries none, TypeEnv keys do.
_ if types
    .get(&format!(":{head}"))
    .is_some_and(|d| matches!(d, TypeDef::Aggregate(a) if !a.nature.is_pure()))
    => false,
```

**⚠ The colon is load-bearing** — `TypeExpr::Parametric.head` is stored WITHOUT a leading colon
(`"wat::core::Option"`) while `TypeEnv` keys carry one. The same quirk bit arc 255's
`BARE_CONTAINER_HEADS` leaf loop, which has to `format!(":{fqdn}")` for the identical reason.

## Measured — this was RUN, not reasoned about

Applied to `src/check.rs`, built release, and exercised:

| probe | before | with the fix |
|---|---|---|
| `defrecord` holding `(Gen :- [i64])` | **loads clean** | **REFUSED**, `ImpureFieldInPureAggregate`, naming field `g` and the full type |
| positive control — `defrecord Box :- [T]` held by another record | loads | **still loads** (not a blanket parametric rejection) |
| **full floor** | 5087/5087 | **5085 passed / 2 failed** |

### The two failures are the SANCTIONED churn, not a regression

Both are arc 293's own probes, and neither is semantic:

- `probe_arc293_W_containment::a_record_cannot_declare_a_struct_field`
- `probe_arc293_W2b_enum_purity::pure_enum_with_struct_field_rejected`

Each pins an **internal `src/check.rs` `rust_caller_span!()` line** in its `.edn` golden. Inserting
the arm moved the raise from `:13955` to `:13967`, so the goldens' pinned line no longer matched.
The error message, aggregate, field and field-type were **byte-identical**.

`tests/types/probe_arc293_W_containment.rs:28-40` records the builder's 2026-08-15 overrule on
exactly this: the churn is trivial, a pinned line that gets updated stays in a constant state of
correctness, and **the span discriminates the emitter** — `ImpureFieldInPureAggregate` can be
raised from more than one call site, and dropping the pin would make the test go green the moment
a *different* path raised the same kind. It ends *"KEEP PINNING THE SPAN. Do not re-propose
dropping it."*

**So landing this fix means re-pinning those two `.edn` goldens, and that is the documented,
intended cost — not evidence against the change.**

## What this arc still has to decide, and why it was not decided here

1. **Does the registry lookup belong before or after the hardcoded list?** Before means a wat
   struct named in the list is caught twice (harmless); after means the list stays authoritative
   for Rust-backed heads. The patch above puts it after the list and before the fallthrough.
2. **Can the hardcoded list then SHRINK?** `Peer`/`Thread`/`Process` are Rust-backed — whether
   they carry an `Aggregate` `TypeDef` with a non-pure nature decides whether they are now
   redundant. Not investigated.
3. **What else does this newly refuse?** The floor says: nothing but the two pinned goldens.
   That is strong evidence, and it is not proof about code outside this tree.

Arc 278 had no mandate over the checker, and the containment rule is 293's. The finding, the
diagnosis, the patch and the floor result are all here so this can be landed as one deliberate
strike rather than rediscovered.

## The gen.wat side, which IS fixed on grok-rete

`wat/gen.wat`'s `Gen` comment claimed *"The checker names this itself if you try — it is a good
error."* That is true of a bare function field and **false of a `Gen`**, which is the shape the
comment is actually about. The comment is corrected there to say exactly what is and is not
caught, and points at this note.
