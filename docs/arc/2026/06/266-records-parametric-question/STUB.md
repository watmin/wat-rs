# Arc 266 (RE-OPENED 2026-07-25 — its own trigger fired) — are records meant to be parametric?

> ## ⚠ RE-OPENED 2026-07-25 (arc 278, cache Stone 2). This is the STUB's OWN stated mechanism firing,
> ## not an override. The 2026-06-14 rejection is preserved verbatim below.
>
> The rejection said: *"**Re-open only if** a real caller needs a user-defined parametric EDN record that
> the collections + parametric structs cannot serve. None exists today."* **One exists now**, and two of
> the ruling's three load-bearing premises no longer hold. All three grounded by the orchestrator:
>
> **1. The mechanism it ruled on is GONE.** The rejection is stated as *"`RecordDef` will NOT gain
> `type_params`"* (citing `src/types.rs:197`). **`struct RecordDef` does not exist at HEAD.** Arc **293
> (struct-record-symmetry)** unified records and structs into `AggregateDef`, which **carries
> `type_params`** — guarded only by a comment: `pub type_params: Vec<String>, // structs use; records
> leave empty`. The structural barrier the cut relied on was dissolved by a later arc.
>
> **2. It is already violated IN CORE, and it works.** `wat/service.wat:56` —
> `(:wat::core::defrecord :wat::service::Alarm<O> [after <- :wat::time::Duration  op <- :O])`, a
> parametric record inside the defservice machinery itself. And `wat/cache.wat:49` —
> `(:wat::core::defrecord :wat::cache::Entry<K,V> [key <- :K  value <- :V])`, shipped in cache Stone 1
> (`a86f521c`) with a green gate that constructs it and reads `Entry/key`.
>
> **3. The caller has arrived: the cache service protocol.** Builder-ruled 2026-07-25 to option (a) —
> a PARAMETRIC protocol — after a four-questions pass (a: 4×YES; the concrete-messages alternative
> failed Obvious, Simple, AND Honest). `:wat::cache::lru-svc<K,V>` needs
> `Get [probes <- Vector<K>]` / `Put [entries <- Vector<Entry<K,V>>]` as **message records**, which
> must be **EDN** (they cross the wire) **and** parametric.
>
> **What the original ruling actually missed** — stated plainly, because the reasoning is otherwise
> sound: its dichotomy was *"records are concrete EDN shapes; structs are the flexible/parametric/
> non-EDN ones."* The cache needs **parametric AND EDN** — a quadrant that dichotomy does not admit.
> A struct cannot serve (it is the non-EDN half); the built-in collections cannot serve (this is a
> user-defined shape). That is precisely the gap the re-open clause was written to catch.
>
> **What is now owed** (the STUB's own "to investigate when picked up" list, still the right list):
> ground whether anything in the record/VSA-encoding path assumes records are concrete, and decide
> whether `AggregateDef`'s `type_params`-for-records is *ratified* or merely *unblocked-by-accident*
> — the comment says records leave it empty, and two core files already disagree with the comment.

# (historical) Arc 266 (CLOSED — REJECTED) — the 2026-06-14 ruling, preserved

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
