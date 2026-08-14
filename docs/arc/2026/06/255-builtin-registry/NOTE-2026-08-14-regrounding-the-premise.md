# NOTE — re-grounding 255's premise against the disk (2026-08-14)

**MEASURED, not proposed.** Read this before `DESIGN.md`; the design's premise has drifted from the
code and both of its "decide before code moves" questions are already answered by the substrate.

**Scope of my reading, stated so nobody over-trusts this note:** I read `DESIGN.md`'s first ~60
lines (the defect table, the registry sketch, the entry shape, the two open questions) and measured
against `src/`. I have **not** read the whole design, nor the sibling stone docs. This note corrects
the premise; it does not re-plan the arc.

---

## 1. "Builtins are registered NOWHERE" — FALSE. They are registered in THREE PARTIAL TABLES.

The design's defect table says builtins are *"registered **nowhere** — a 454-arm compile-time
`match`"*, carry no metadata, and are not reflectable. Measured:

| what | count | form | site |
|---|---|---|---|
| check-time **type signature** | **332** | **DATA** — `TypeScheme` | `register_builtins`, `src/check.rs:15216–20033` (4,817 lines) |
| check-time **hand inference** | **141** | code — match arms | inside `infer_list`'s Keyword block, `src/check.rs:2518–5568` (3,050 lines) |
| **runtime dispatch** | **678** | code — match arms | `src/runtime.rs` (scattered; `dispatch_keyword_head` itself is only **73** lines) |
| **resolve** | **0** | — | hence the blanket-accept, `src/resolve/walk.rs:257` |

So the asymmetry is not *absence*. It is **three tables of different sizes that do not agree**, and
one consumer (`resolve`) that can ask none of them.

**Bounds:** the 332 is brace-bounded and clean. The 678 counts keyword literals at line-start in
`runtime.rs` and may include non-dispatch occurrences. The 141 counts keyword arms inside
`infer_list`'s span. **None is a census** — when the registry lands, the CHECKER enumerates the real
worklist (R65 `SCVTVM IDEM INDEX`). This arc has been burned by grep counts repeatedly; on
2026-08-13 two greps of the same corpus returned 1025 and 998.

## 2. Open question 2 — "is `type_sig` day-one?" — RULED DAY-ONE, and the premise was wrong.

The design recommends: *"defer `type_sig`; ship `arity`/`category`/`doc` first, grow in."*

**`type_sig` is not a capability to add. 332 builtins already carry it as data.** Deferring it means
preserving **two ways a builtin's type is known** — a `TypeScheme` for some, a hand-written
`infer_list` arm for 141 others. That is the 2×2 this project collapses on sight (#30 one door for
defclause registration; #75 one door for a type head's FQDN).

Four questions, flat:

| | Obvious | Simple | Honest | |
|---|---|---|---|---|
| day-one — finish the uniformity | YES | YES | YES | **4/4 — RULED** |
| defer (the design's recommendation) | **NO** | **NO** | **NO** | disqualified |
| a subset | **NO** | **NO** | — | disqualified |

Deferring fails **Obvious** (the phrase reads as "not built yet" — false for 332), **Simple** (two
mechanisms, one question), **Honest** (a deferral of *uniformity* described as a deferral of
*capability*). The builder: *"deferral to me usually screams 'wrong fucking idea'."*

## 3. Open question 1 — "align `BuiltinMeta` with user-form metadata?" — the mechanism ALREADY EXISTS.

The design asks whether to reuse the user-form metadata shape or invent a superset. Measured:
`SymbolTable.binding_metadata: BindingMetadata` (`src/value/symbol_table.rs:142`) is a
name → metadata-map already populated for user forms (e.g. `":restricted-to"`, via
`restriction_entry.rs`) and **already mirrored into `CheckEnv.binding_metadata`**
(`restriction_entry.rs:43`).

So there is no shape to invent. Builtins take entries in the map user forms already use.

---

## ★ WHAT THIS DOES TO THE ARC

Both "decide before code moves" questions resolve the same way: **the mechanism exists and is
unevenly applied.** 255 is therefore *not* "build a registry from scratch" — it is:

1. **finish** the check-side table (the 141 hand-inferred builtins get schemes like the 332 do),
2. **point `resolve` at it** (delete the blanket-accept at `walk.rs:257`),
3. **make the runtime dispatch read the same source** rather than a parallel match.

⚠ **THEREFORE `BuiltinRegistry` AS A NEW STRUCT MAY BE THE WRONG SHAPE.** The "registry" may already
be `CheckEnv.schemes` + `binding_metadata`, and minting a third table beside them would *add* an
asymmetry while claiming to remove one. `examinare`'s standing warning applies: *"the thing you
would build almost always already exists."* **Decide this before any code moves** — it is now the
arc's real first question, and it replaces the two the design listed.

## WHAT 255 BUYS DOWNSTREAM (all measured 2026-08-13)

- **`wat.type` gets somewhere to live.** Today it is a `strip_prefix("wat::type::")` at exactly two
  sites (`types.rs:4503`, `:4702`) — an alias, not a namespace. `:wat::type::Vector` annotates but is
  an **unknown function**, which is why `(wat.type/Vector [wat.type/i64])` cannot construct. The
  builder ruled `wat.core` loses the type constructors and `wat.type` gains them; that ruling needs a
  registry to land in.
- **#95 closes** — a dotted call head is unchecked because `infer_list` gates all call inference on
  `if let WatAST::Keyword`. With the signature as data keyed by NAME, the head's spelling stops
  mattering. **This holds BECAUSE `type_sig` is day-one; it would not have followed from the design
  as written.**
- **The parametric arity table becomes explicit** — the builder's objection to the flat
  `(HashMap :K :V …)` form is that the type/member split lives in an implicit per-head table. That
  table is exactly what the entry carries.

## ON THE MEGAFILES — related, and deliberately NOT braided in

`register_builtins` (4,817 lines) + `infer_list` (3,050) are **38% of `check.rs`'s 20,863**. 255
necessarily touches both, and it shrinks them by turning code into DATA rather than by relocating
functions — the only kind of shrinking that is not deck chairs.

But **"build the registry AND break up the mammoths" is two arcs braided** and fails Simple. Keep
them separate: 255 makes the breakup easier by removing 38% of `check.rs` first;
`docs/MODULARIZATION-NOTES.md` (queued 2026-05-08, still not a numbered arc) stays its own thing.

⚠ That note's own table is stale in the direction that matters: it recorded `runtime.rs` at 23,801
and `check.rs` at 15,108. Today they are **35,066** and **20,863** — +47% and +38% while the note
waited on "after arc 109 wraps." Together they are **64%** of `src/`. `docs/CONVENTIONS.md:1110`
already rules the target shape: *"a module is a DIRECTORY (2026-07-26)"*, forward-looking.
