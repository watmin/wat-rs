# Arc 266 (STUB) — are records meant to be parametric? (the type_params asymmetry)

> **Status: STUB — a question to investigate, banked 2026-06-14.** Surfaced while naming the
> `listener'`-mint result (arc-209 host-parity / sub-stone `Bound`). NOT blocking anything.

## The observation

`RecordDef` (`src/types.rs:197`) has **no `type_params` field** — it carries only `name`, `parent`,
`field_names`, `field_types`. Every OTHER product/type def DOES carry `type_params`:
`StructDef` (126), `EnumDef` (142), `NewtypeDef` (159), `AliasDef` (168), `UnionDef` (184). Records
are the lone exception. So a parametric record `Foo<T>` is not expressible; a parametric **struct**
`Foo<T>` is.

## The question

Is this a deliberate design split or a latent gap?
- **Deliberate reading:** records are the EDN/VSA-encodable *concrete data* type; structs hold the
  non-EDN / parametric / opaque cases (builder: *"structs are meant to hold things that can't be EDN
  expressed"*). Under this reading, a parametric container is a struct by definition, and records
  stay concrete — no flaw.
- **Latent-gap reading:** there's no obvious *encoding* reason records must be concrete — holon/VSA
  encoding is value-based (field values at runtime), not type-based, so a parametric record could
  encode fine. We may simply never have needed one (concrete EDN data always sufficed), and
  `RecordDef` lacking `type_params` is an unaddressed asymmetry.

## Why it's banked, not acted on

It did **not** bite the `Bound` case that surfaced it: `Bound` holds `Listener'`/`Address'`
(RustOpaque, non-EDN) → it is a struct for the *right* reason (non-EDN contents), parametricity
aside. So this question is orthogonal. Pick it up only if a genuine need arises for a **parametric
record over EDN-expressible `T`** (a generic EDN container) — that's the moment to decide: add
`type_params` to `RecordDef`, or rule that parametric-EDN-containers are out of scope by design.

## To investigate when picked up
- Ground the record/VSA-encoding path: does anything assume records are concrete?
- Is there a real consumer wanting a parametric EDN record, or does the struct (parametric, non-EDN)
  + concrete records (EDN) split cover every honest need?
