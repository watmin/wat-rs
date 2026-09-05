# DESIGN — STONE: the printer, and the round trip that proves the migration

> **Builder, 2026-09-04:** *"A2 and B2 have been reasoned."*

Amends `[[DESIGN-the-registry-prints-its-own-replacement]]`, which proposed a **wat script asking
the registry**. That is now A1 and it is **rejected**, on the four questions, below. The correction
is recorded rather than rewritten: the prior document is right about *what* prints (one authority,
printed rather than re-parsed) and wrong about *where*.

## DECISION A — where the printer lives

| | Obvious? | Simple? | Honest? | Good UX? |
|---|:---:|:---:|:---:|:---:|
| **A1 · a wat script asking the registry** | YES | **NO** | YES | YES |
| **A2 · Rust-side in `wat-doc`, printing a `DocComment`** | YES | YES | YES | YES |

**A1 fails Simple, structurally.** It requires first widening the wat-facing surface — `prose`,
`args` detail, `examples`, `see`, `syntax`, `yields`, `deprecated` are in `IntrinsicEntry` and are
NOT askable from wat (measured on `:wat::core::char`). Worse, it puts the printer in **wat** and the
reader in **Rust**, so no single test can hold both ends of the round trip.

★ **A2 deletes that step entirely.** `wat-doc` already owns `DocComment`, and BOTH readers
(`parse` at `:506`, `from_metadata` at `:1025`) produce it. A printer beside them is the inverse of
a function already there, and the gate is a plain unit test in one crate.

⚠ **And it sharpens WHICH round trip matters.** The consumer of migrated rows is the **proc-macro
reader**, not `edn::read`. So printer ↔ macro-reader is the gate; runtime tag resolution
(`:wat::doc::Row` as a registered type) is a separate nice-to-have and **not a prerequisite**.

## DECISION B — the docstring emitter's blast radius

| | Obvious? | Simple? | Honest? | Good UX? |
|---|:---:|:---:|:---:|:---:|
| **B1 · change `wat-edn::write-pretty` globally** | **NO** | YES | **NO** | **NO** |
| **B2 · a separate, named doc-row emitter** | YES | YES | YES | YES |
| **B3 · hand-roll the formatting in wat** | YES | **NO** | **NO** | — |

`write-pretty` feeds diagnostics, IPC and every golden `.edn` under `tests/`. Emitting literal
newlines inside strings there would churn goldens tree-wide and change every consumer to serve one.
**B3 is the third-decoder mistake in writer form** — a second authority on how EDN is written.

## THE SHAPE

```
print(doc: &DocComment) -> String          in wat-doc, beside parse and from_metadata
    emits  #wat.doc/Row { … }  with the docstring as a LITERAL multi-line string whose
    continuation lines carry the map's margin — the EXACT INVERSE of
    crates/wat-macros/src/edn_doc.rs:72 `dedent`.
```

**Its acceptance is the round trip, not how the output looks:**

```
from_metadata(edn_to_watast(wat_edn::parse(print(doc))))  ==  doc
```

Which is the same shape as `probe_can_doc_types_reconstruct_the_checker_scheme` (432/432, green)
and as the `char` stone's byte-identity check — generalised from one row to the corpus.

## ⛔ THE GATE MUST NOT INHERIT `char`'s BLIND SPOT

`:wat::core::char` exercised `@added @arg @ret @example` and the five axes. It exercised **none** of:

```
@see            258 uses   e.g. src/collection/transform.rs
@yields          11 uses   e.g. src/intrinsic/witness.rs
@example-norun  139 uses   e.g. src/intrinsic/kernel/resource.rs
@syntax          37 uses
@alias           38 uses   ⛔ NOT REACHABLE — see below
@deprecated       0 uses   ⛔ NO LIVE ROW EXISTS
```

★ `src/intrinsic/holon/hologram.rs` carries `yields` AND `example-norun` together — the richest
live witness found. `@deprecated` has **no live user at all**, so the gate reaches it only through a
constructed `DocComment`, or it must say plainly that the field is uncovered. **A round-trip gate
that only ever sees the fields `char` had would pass while losing seven of them.**

## OUT, AFFIRMATIVELY

- **The `@alias` path.** `@alias` rows go through `#[wat_special_form]` → `parse_special_form` →
  `DocSpecialForm`, a different struct with no `from_metadata`-equivalent. `#wat.doc/Alias` stays
  designed-and-unwritable; this stone does not open it.
- **`:wat::doc::Row` as a runtime type.** Not a prerequisite (see Decision A's note).
- **The `@`-form ratchet, and the sweep.** Both follow this stone; neither belongs in it. A 558-row
  rewrite lands under a gate or it does not land.
- **The unknown-key hole in `from_metadata`.** A printer never emits a stray key, so the migration
  cannot trip it — which is exactly why it is closed on its own merits, not here.
