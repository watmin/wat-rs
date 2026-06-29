# The Aggregate Audit — 293's closure gate (the holder is a passing policy, nothing else)

> **▶ The live close ORDER + STATUS for 293+294 is `../294-holon-returns-to-vsa/CLOSE-SEQUENCE-293-294.md` (the single
> maintained tracker). This doc is the closure-gate DETAIL (the model + the ~99-branch checklist), not the sequence.**

> **Status: OPEN — 293 CANNOT BE RESOLVED until this audit is complete (builder, 2026-06-28).**
> *"the audit we need to do — mark 293 cannot be closed without doing this — is finding every example of a record
> not being used like a struct. nearly all things need to be 'aggregate'-whatever, neither struct nor record. the
> only place these aggregates differ is when they are being passed around."*

## The principle (settled this session)

There is **one aggregate citizen** — `class` + positional `fields`. `defstruct` / `defrecord` / `holon::defrecord`
are not three kinds; they are **one thing wearing a holder**. The holder is **purely a passing policy** — it speaks
*only at boundaries*, and is silent everywhere else (construction, field access, identity, codegen, the data itself).

**A record handled differently from a struct for any reason that is NOT a boundary concern is a spurious split — a
bug, not a feature.** The thesis of the whole arc (R2 *FRANGE UT UNUM FIAT*, R3 *SUB SUPERFICIE QUOD ES*), stated
operationally as a closure condition.

## The holder is the capability trit — it governs THREE boundaries (builder's correction, verbatim)

> *"structs cannot cross any comms, core-records and holon-records must be edn-repr, core-records can never be passed
> to a func needing a holon-record."*

An earlier draft of mine collapsed this to one binary question ("can it cross comms?"). **Wrong** — flattened three
distinct facts and dropped the directionality. The holder = `Struct (−1)` / `Record (0)` / `HolonRecord (+1)`, and it
answers exactly THREE boundary questions:

1. **Comms eligibility.** A `Struct` crosses **no** comms, ever (in-locus, holds non-portable resources). `Record` /
   `HolonRecord` may cross.
2. **EDN-repr requirement.** To cross, a value **must** be EDN-representable. `Record` / `HolonRecord` are; `Struct`
   is not. (This is *why* a struct can't cross — the same wall stated as a capability: `is_portable = holder != Struct`.)
3. **Assignability — DIRECTIONAL.** `holon <: core`: a `HolonRecord` satisfies a slot wanting a core record (it has
   everything a core record has, plus the hologram). The reverse is **forbidden** — a `Record` can **never** be passed
   where a `HolonRecord` is required (it lacks the VSA capability). One way down the ladder, never up.

Inside those three boundaries the holder is silent and the value is pure **aggregate**.

## The classification criterion (every holder-branch is one or the other)

A holder-branch is **LEGITIMATE (keep)** iff it serves one of the three boundary families:
- **COMMS** — `send'` / `recv'` / the locus wall (a struct may not leave the locus).
- **WIRE / EDN-repr** — `is_portable_type`, `edn_shim` encode/decode (a struct has no wire form).
- **ASSIGNABILITY** — `register_subtype` / `is_subtype` / `assignable` — the `holon <: core` lattice edge (directional).

Every **other** holder-branch is **SPURIOUS → must unify to aggregate** — construction, field accessors, identity
(already done — 294.c.1), declaration (`structtype`/`recordtype`, `parse_defstruct`/`parse_recordtype`), the def
macros, codegen, value-handling in rete/collection/closure_extract, etc.

**293 closes only when the SPURIOUS column is empty.**

## Scope (surveyed against the disk, 2026-06-28 — pre-audit estimate)

- **~99 holder-branches** in `src/` (`grep "Holder::Struct | holder == | holder != | == Holder | != Holder"`),
  clustered across **14 files**: `runtime.rs`, `check.rs`, `value/value.rs`, `types.rs`, `types/defstruct.rs`,
  `edn_shim.rs`, `closure_extract.rs`, `collection/eval.rs`, `collection/map_container.rs`, `rete/kernel.rs`,
  `rete/matcher.rs`, `value/observe.rs`, `test_runner.rs`, `types/surface.rs`.
- **~160** struct-vs-record split heads (`eval_struct_new`/`eval_record_of`, `structtype`/`recordtype`,
  `parse_defstruct`/`parse_recordtype`, `struct-field`/`Record/field-at`, `struct-new`/`Record::of`, …).
- **~15 `is_portable_type` sites** — the WIRE family (a keep-bucket, not the only one).

These are a pre-audit estimate; the audit replaces them with a per-branch classified table.

## The audit (to be produced against a stable tree, post-294.c.2)

Fan read-only auditors across the 14 file-clusters; each returns every holder-branch with `file:line`, what it does,
and a verdict (COMMS | WIRE | ASSIGNABILITY | SPURIOUS). The orchestrator weighs each against the disk and synthesizes
the master table here. Format:

| file:line | what the branch does | verdict | if SPURIOUS: the unification |
|---|---|---|---|
| _value/value.rs:626/828_ | Eq/Hash keyed on hologram | ~~SPURIOUS~~ **DONE (294.c.1)** | identity = (holder, class, fields) |
| … | … | … | … |

## How it sequences the campaign (the audit is the spine)

The audit is the **master checklist** for 293 closure; strikes knock items off it:
- ✅ **identity split** (Eq/Hash on hologram) — closed by **294.c.1** (`ed7ecd50`).
- ✅ **construction split** (`struct-new`/`Record::of`/`holon::Record::of`) — closed by **294.c.2a** (`f301a6fc`):
  `aggregate-new` is the one holder-dispatched ctor; all three macros + struct codegen emit it; the hologram is
  derived in Rust. The of-funcs stay registered until **294.c.2b** (their annihilation — still OPEN on the audit).
- ▶ **declaration split** (`structtype`/`recordtype` → one `aggregatetype`; `parse_defstruct`/`parse_recordtype` →
  one `parse_aggregate`; the three def macros → thin holder-keyed delegations over one emission) — **NEXT.** The
  builder's catch (confirmed on the disk by c.2a's diff): the two record macros are now byte-identical except the
  `recordtype` holder keyword. *"why are struct and record being tolerated as yet another split not a unification."*
- ▷ **the remainder** — whatever the audit surfaces in `edn_shim` / `closure_extract` / `rete` / `collection` /
  accessors, each classified COMMS/WIRE/ASSIGNABILITY (keep) or SPURIOUS (unify).

293's **293.5 close is GATED on this audit reaching zero SPURIOUS** (see `DESIGN.md` Decomposition).

## Path of voices (this session's corrections — kept, not flattened)
The principle and its sharpenings are the **builder's**, quoted above: the audit-as-closure-gate, the
nearly-all-is-aggregate framing, the passing-only-difference, the three-boundary correction of my flattened
"one question", the recordtype-is-the-only-variance catch, and (the related law) *"'replicate' sounds like
'duplicate'"* → one extracted guard, never a copy. The apparatus's part: grounding each against the disk, the
three-boundary classification criterion, and the audit's table structure.

## Pairs
`DESIGN.md` (HOLDER × SURFACE — the model; the closure gate added to Decomposition) ·
`294/DESIGN.md` (the value-layer gut) · `294/REMAINING-PATH.md` (the 9 steps; the audit subsumes/sequences them) ·
`294/DESIGN-294.c.2-aggregate-new.md` (the ctor strike) · `feedback_replicate_is_a_duplication_smell` ·
`feedback_option_carrying_semantics_screams_enum` (the holder is the enum) · R2/R3 realizations.
