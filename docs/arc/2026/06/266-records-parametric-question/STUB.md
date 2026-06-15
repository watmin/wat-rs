# Arc 266 (CLOSED — REJECTED) — are records meant to be parametric? (the type_params asymmetry)

> **Status: CLOSED / REJECTED 2026-06-14.** Records stay **concrete by purpose**; `RecordDef` will
> NOT gain `type_params`. Decided by four-questions (below). NOT a deferral — an affirmative cut.

## Verdict (four-questions)

A record's purpose is a **concrete EDN data shape** (`Stats`, `Handle`, the defservice
Request/Response records). Making `RecordDef` parametric was rejected:

- **Obvious? NO** — parametric blurs the line the design draws: *records are concrete EDN shapes;
  structs are the flexible/parametric/non-EDN ones.*
- **Simple? NO** — adds a concept to records for no demonstrated use.
- **Honest? NO** — no caller has surfaced (this very question bit nothing — `Bound<S,R>` is a
  struct, correctly). Adding `type_params` with no consumer is a forcing function / optional-is-a-smell.
- **Good UX?** — the generic-container role is already filled: built-in collections
  `Vector<T>`/`HashMap<K,V>`/`Option`/`Result`/`Tuple` (declared via the `wat.type/…` parametric-form
  plan) **plus** parametric **structs**. A user wanting a generic container reaches for those.

A concrete record may still **carry** a parametric field at concrete args (e.g.
`field <- :HashMap<String,i64>`); it just is not generic *over* those args — and that generic role
is the collections + structs, not records.

**Re-open only if** a real caller needs a user-defined parametric EDN record that the collections +
parametric structs cannot serve. None exists today.

> The `type_params` asymmetry (`RecordDef` is the lone product type without it) is therefore
> **intentional**, not a latent gap. The observation below is preserved as the record of why.

---

## (historical) The observation that raised the question

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
