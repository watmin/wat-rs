# NOTE — a record's `:restricted-to` is parsed, stored, and never enforced

**Found 2026-09-06**, by probe, while measuring whether the rete fact base could be sealed
(arc 278, the `FactBag` exemplar). **NOT worked in that session — rete was the scope.** Written
here so the next hand does not re-derive it, and does not trust a seal that is not one.

## The claim under test

`{:restricted-to [<prefix-kw>…]}` and `:field-metadata {<field> {:restricted-to […]}}` are
declared access control, enforced at CHECK time by `walk_for_restricted_call` (`src/check.rs:652`).
Three artifacts say it applies to every aggregate nature:

- `src/types.rs:4592` — *"Metadata (restrictions) is optional for ANY nature (GAP-5 capability built here…)"*
- `src/types.rs:4665` — *"GAP-5 capability available to any nature"*
- `docs/arc/2026/06/293-struct-record-symmetry/DESIGN-293-declaration-unification.md:43` —
  *"optional metadata on `defrecord`/`holon::defrecord`) — closing GAP-5 + GAP-6. **The capability
  is built once, here.**"*

Two others say the opposite — `src/types.rs:303` and `:311`, *"always `None` for records"*.

**All five are wrong.** The truth is a third thing none of them states: for record nature the
restriction is parsed, stored on the `AggregateDef`, and then silently dropped before enforcement.

## Driven — identical metadata, identical denied caller, one difference

```
defstruct  + :field-metadata only   ->  EXIT=1   #wat.check/DefRestrictedCallerNotAllowed
recordtype + form-level AND field   ->  EXIT=0   silently allowed
```

Repro A — **must FAIL, and does** (this is the shipped fixture
`tests/types/struct_restricted_field_denied.wat.bad`; a field-metadata-only variant fails too, so
the form-level whitelist is not what makes it fire):

```
(:wat::core::defstruct :my::Vault
  {:field-metadata {:secret {:restricted-to [:my::admin::]}}}
  [secret <- :wat::core::i64
   name   <- :wat::core::i64])
(:wat::core::defn :user::outsider::read-secret [v <- :my::Vault] -> :wat::core::i64
  (:my::Vault/secret v))
```

Repro B — **must FAIL, and does NOT.** Same metadata, record nature:

```
(:wat::core::recordtype :my::Vault :wat::core::Record
  {:restricted-to  [:my::admin::]
   :field-metadata {:secret {:restricted-to [:my::admin::]}}}
  [secret <- :wat::core::i64
   name   <- :wat::core::i64])
(:wat::core::defn :user::outsider::read-secret [v <- :my::Vault] -> :wat::core::i64
  (:my::Vault/secret v))
```

Drive both with `./target/release/wat --check <file>`. **Do not read the exit code through a pipe**
— `… | head` returns `head`'s status, which reports 0 for a failing check.

## The mechanism, pinned

1. `parse_aggregate` (`src/types.rs:4675`) builds `StructRestrictions` and stores it on the
   `AggregateDef` for **any** nature. The parse half is genuinely universal — this is why the
   declaration is accepted rather than rejected, and why the hole is silent.
2. The loop in `src/runtime.rs` (~`:1500`) that writes those restrictions into
   `sym.binding_metadata` — the map the check-time walker reads — is guarded by
   `TypeDef::Aggregate(a) if a.nature == crate::types::Nature::Struct`. Records fall through.
3. `walk_for_restricted_call` (`src/check.rs:652`) then finds no `:restricted-to` entry for a
   record's accessor, and allows every caller.

Parse is universal; the *publication* to `binding_metadata` is struct-only.

## And `defrecord` cannot express it at all

`(:wat::core::defrecord :Name {meta} [fields])` is a `MalformedDecl` — *"parent must be a type
keyword; got map"*. The macro picks slots **from the ENDS** (`wat/Record.wat:128-134`: `fqdn` is
first, `fields` is last, anything between is the `:- [T…]` binder), so a metadata map lands in the
parent slot. Only the raw `recordtype` form can carry the metadata — and raw `recordtype` mints
accessors but **no kwargs constructor** (`:Name` resolves as *"call head — not a builtin, not a
registered function"*). So today the two capabilities are mutually exclusive for records:

| route | ctor | restriction expressible | restriction enforced |
|---|---|---|---|
| `defrecord` | ✅ | ❌ `MalformedDecl` | — |
| `recordtype` | ❌ | ✅ parsed + stored | ❌ dropped at publication |

## Why it bites — the composition that has no escape

A record cannot delegate the seal to a struct. `arc 293.W`'s containment rule:

> *"pure aggregate `:X` may only hold pure fields — field `f` has impure (struct) type `:S`. A
> struct cannot be reconstructed from EDN bytes across a comms boundary; a record or holon holding
> a struct field could never cross — it must not exist."*

That rule is correct and should stand. Its consequence is that any datum living inside a record
must itself be a record, so *"wrap the sealed part in a struct"* is not available. **Records have
no working encapsulation by any route.**

## Severity

A restriction that type-checks clean and enforces nothing is worse than none: it manufactures the
confidence it fails to earn, and the `struct_restricted_*` fixtures make the capability look
covered because they are all struct nature. Any record field carrying `:restricted-to` today is
unsealed and reads as sealed.

## What closes it

Publish record-nature restrictions into `binding_metadata` (lift the `Nature::Struct` guard in
`src/runtime.rs`), and teach `:wat::core::defrecord` to carry a metadata map — the slot rule needs
the map recognized rather than swept into the binder. Then:

- a `.wat.bad` twin of every `struct_restricted_*` fixture at **record** nature (Repro B above is
  the first one — it must go RED), because the current gate's population is half the natures; and
- fix the five stale comments listed at the top, which currently disagree with the code in **both**
  directions.

## Who is waiting on it

`docs/arc/2026/06/278-rules-engine/` — the rete `FactBag` exemplar. Its doors and its corpus gate
ship without this; the gate exists **only** because the type cannot hold the seal. When this note
closes, `FactBag` gains one `:field-metadata` line and **that gate is deleted** — its deletion is
the proof the capability arrived.
