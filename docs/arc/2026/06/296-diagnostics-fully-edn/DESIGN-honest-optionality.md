# Honest Optionality — records are total, `None` is spoken, `Option` is a normal enum

> **Status: DESIGN — co-designed with the builder 2026-07-01, mid derive-sweep.** Emerged from the RuntimeError
> span-policy fork (A/B) which the builder rejected as a false choice: *both* eliding and sentineling make "we don't
> know" implicit. The real question was optionality itself. This doctrine PINS the rulings; three strikes decompose from
> it; the 296 derive sweep (RuntimeError/MacroError) is **blocked** on it (deriving over Option-erasing data bakes in the
> lie — the D1 lesson). Arc placement (own arc vs 296 expansion) — see § Placement.

## What opened it
Deriving `LoadError` (Strike 3b) landed; the last derive-sweep room is RuntimeError, whose span field **always-emits a
sentinel** for an unknown location: `#wat.kernel/DuplicateDefine {:name "foo" :span {:file "<runtime>" :line 0 :col 0}}`
— a **fake coordinate** on the wire (~107 `Span::unknown()` construction sites). Presented as a fork:
- **A — elide** the span key when unknown (like every other error family): `{:name "foo"}`.
- **B — preserve** the sentinel (byte-identical).

The builder rejected the frame: *"you are forcing users to know that the absence of something is semantically
meaningful."* Eliding hides meaning in an absent key; the sentinel lies with a fake value. Both make "unknown"
**implicit**. That cracked open the real question — how does wat model "not present" at all — and the answer became a
doctrine.

## The builder's rulings (verbatim, 2026-07-01)
- *"i think the answer to 'is it optional?' is 'use an enum.'"*
- *"aggregates must allow Option<T> … suppose we impl some s3 service and some value in the request blob is nil/null …
  the thing fits the spec and null has a meaning of 'not supplied' … for the rpc-as-edn to work we need some+none to work."*
- *"we need some+none to be tagged correctly — i thought we built this months ago … we are pressured to use it."*
- *"None means nil means null … there is no good usable value, so this is the best default value."*

## The doctrine (5 rulings)
1. **A record/aggregate is TOTAL — every declared field is ALWAYS emitted (present key, uniform shape). NEVER elide.**
   A reader must never infer meaning from an absent key. The record's shape does not vary with its values.
2. **`None` is a SPOKEN, TAGGED value: `#wat.core.Option/None nil`. Never an absent key, never a fake sentinel.**
   "Not present" is represented, on the wire, as a value you can read and match — not as a hole and not as a lie.
3. **`Option` is a NORMAL enum.** It currently has a **transparent special-case** in the codec (`Some(v) → v`,
   `None → nil`, documented `edn_shim.rs:34`, coded `edn_shim.rs:1571` read + `:1965/:2091/:2824` write). DELETE that
   special-case → `Option` falls into the general enum path already present (`edn_shim.rs:1386-1387`:
   `#ns/Variant nil` unit / `#ns/Variant [items]` tagged). This is a **decomplection** (remove the exception), not new
   machinery — and it is exactly the tagged form the builder had in hand.
4. **`Option<T>` is LEGAL on aggregate fields.** RPC/protobuf-as-EDN requires it: an S3-style request record where a
   field is genuinely "not supplied" is a coherent, spec-conformant message. The type is welcome; only its **dishonest
   representations** (elide, sentinel, transparent-erasure) die. `None = nil = null` — the honest default when there is
   no usable value.
5. **The `Span::unknown()` sentinel DIES.** Location is honest one of two ways, decided by a triage of the ~107 sites:
   - **mandatory** — every error carries a real `:wat::kernel::Location`; `Span::unknown()` is annihilated (the DESIGN's
     original lean: *"a locationless error becomes a bug to fix"*), OR
   - **explicit Option** — `location <- :wat::core::Option<Location>`; genuinely-unknowable → `#wat.core.Option/None nil`
     (present, spoken). NOT elided, NOT sentineled.

## Grounded facts (this session, on disk)
- **The `:wat::core::Error` surface already mandates location** (`wat/core.wat:1089`: `location <- :wat::kernel::Location`,
  NOT Option) — the impl (sentinel + `Failure.location: Option`) never obeyed it.
- **Option is transparent by a deliberate carve-out** — `edn_shim.rs:1571` (`"wat::core::Option" =>`), write path at
  `:1965/:2091/:2824`. Every OTHER enum is already tagged.
- **Aggregate-Option violations today** (fields the doctrine makes honest, not illegal): `wat/doctest.wat:16`
  (`expected <- Option<WatAST>`), `wat/lint.wat:57` (`fix <- Option<FixEdit>`), Rust `Failure` (`location`/`failure`
  Option fields). These stay Option — they just serialize tagged + always-present.
- **Blast radius (measured, not adjective):** ~4 codec sites change; ~134 `core::Some/None` sites are CONSTRUCTION
  (unaffected — they build Option, don't assert wire form); the real cascade is the subset of tests asserting the
  transparent wire form (~2 flagged directly). A **medium** fight; the fail-count is the progress meter.
- **The floor already emits `:location nil`** (present, explicit) in the `WatError` floor form — the honest-present-none
  pattern already exists; the sentinel lives only in the raw `:span` variant form.

## Decomposition (three strikes, in order)
- **Strike O — tag Option** (the keystone). Delete the transparent special-case; `Option` serializes like every enum:
  `None → #wat.core.Option/None nil`, `Some v → #wat.core.Option/Some …`. **Pin the single-field body form** (bare
  `#…/Some "x"` per the builder vs the general `#…/Some ["x"]` vector body — decide at strike start). Ride the test
  cascade to zero. Round-trip (edn→value) must lift the tagged form back. RPC/protobuf (297) now has a real Option wire.
- **Strike S — kill the span sentinel.** Triage the ~107 `Span::unknown()` sites (knowable → fix to carry a real span;
  unknowable → the residue). Decide mandatory-Location (annihilate the symbol) vs `Option<Location>` (explicit None).
  The `{:file "<runtime>" :line 0 :col 0}` sentinel ceases to exist.
- **Strike D — resume the derive sweep** over now-honest data: RuntimeError + MacroError (the last smuggle-capable
  families), then 296 closes → **R1 *NE SIBI OBSOLESCAT* → PROBATUM EST**.

## Four-questions (the tagged-Option ruling)
- **Obvious?** YES — `#wat.core.Option/None nil` says "none" out loud; `nil` says nothing (schema-dependent).
- **Simple?** YES — it's the REMOVAL of a special-case; Option obeys the one enum rule, not two.
- **Honest?** YES — self-describing, unambiguous (`None` ≠ `Some(nil)`; nested Option survives); no schema needed to read.
- **Good UX?** YES — RPC clients decode without out-of-band knowledge; a record field is always present, its value spoken.

## Placement
This outgrew 296's diagnostic-EDN scope — it's a **codec + type-system doctrine** that 296 (errors) AND 297 (protobuf
bridge) both depend on. Two honest framings; builder decides:
- **Own arc (298 "honest optionality")** — foundational, precedes 296's close + 297; cleanest separation.
- **296 expansion** — "this floor has more loot" (builder); the rooms belong to the floor being cleared.
Pairs: 297 protobuf-IPC (needs the Option wire), 293 aggregate model (totality is a structural property), 296 R4 *ITERVM
SVRGIMVS* (the quick fix was the true size — again) + R5 *VTRAQVE FACIE SERVATVR* (honest structure, no fake data).
