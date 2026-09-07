# BRIEF — 296 H-3: Option and Result live in wat

> H-2 flipped the wire; H-2c moved `LociDiedError`'s identity into `wat/kernel/diagnostics.wat`.
> H-3 does the same for the two types that are 75% of the corpus surface. Floor is 5211/5211 green.

## THE HOME IS `wat/core.wat`, AND THE RECORD ALREADY SAYS SO

`src/load/stdlib.rs:54`, verbatim, written long before this stone:

> *"No eval-deps beyond **wat/core.wat's builtins** (defrecord/defenum/defclause/typealias/
> **Result/Option**/Vector/keyword)"*

The map already names `core.wat` as their home; only the territory disagrees — they are a Rust
literal at `src/types.rs:1243-1268`. Grounded, against the alternative of a new `wat/core/` dir:

```
wat/core.wat   MUST be first (stdlib.rs:35) · already declares 25 types · already proc-macro-sourced
               9× · the doc above already calls Option/Result its builtins
wat/core/…     does not exist. Two files holding one enum each would fragment what the load-order
               doc already treats as one thing, for no measured benefit.
```

**Load order does NOT gate this.** `wat_enum_register_from!` reads the file at COMPILE time into
`TypeEnv::with_builtins`, so the type is registered before any wat loads; the `defenum` at load time
is an EQUIVALENT re-declaration that `resolve::registration`'s gate absorbs as `NoOp`. That is
exactly how `LociDiedError` works today and the floor is green on it.

## ⛔ THE PREREQUISITE — H-3 IS THE FIRST PARAMETRIC CONSUMER

Option and Result are parametric (`["T"]`, `["T","E"]`). Measured 2026-09-06:

```
crates/wat-source-derive/src/lib.rs:727   type_params: ::std::vec::Vec::new()   wat_enum_register_from!
crates/wat-source-derive/src/lib.rs:497   type_params: ::std::vec::Vec::new()   wat_record_from!
crates/wat-source-derive/src/lib.rs:73-87 the parser DETECTS the `:- [T …]` binder — and uses it
                                          ONLY to shift the payload offset. The vector is discarded.
```

**This is not a defect today.** 14 types are registered through those macros and **not one is
parametric** (measured). It is an unbuilt capability, and this stone is its first consumer — so
building it is the stone's first act, not a surprise.

★ The file's own header names the hazard: *"getting the second one wrong is SILENT: the payload
shifts by two when a binder is present, so a scan with a fixed offset reads the binder's `[T …]`
vector as the first field list."* A wrong capture here does not fail loudly; it silently registers
the params as a variant's fields.

## READ IN ORDER

```
src/types.rs:1243-1268                    the two literals being replaced — READ THE PROSE ABOVE
                                          THEM. The purity choice (Pure) is argued there and is a
                                          DECISION, not a transcription; carry it, do not re-derive.
src/load/stdlib.rs:35-56                  the load-order contract + the sentence naming the home
crates/wat-source-derive/src/lib.rs:73-87 the binder parser (detects, discards)
crates/wat-source-derive/src/lib.rs:715   the register expansion with the hardcoded empty vec
wat/kernel/diagnostics.wat:122            H-2c's defenum — the shape to copy
src/types.rs:1651-1655                    H-2c's wat_enum_register_from! call — the shape to copy
wat/spawn.wat:195                         a LIVE parametric defenum: `ServiceEvent :- [I O A]`
wat/service.wat:58 · wat/cache.wat:172    two more, for the binder's spelling in today's surface
```

## THE WORK

1. **Teach `wat_enum_register_from!` (and its `wat_record_from!` sibling, same defect) to CAPTURE
   the `:- [T …]` binder** into `type_params`, in declaration order.
2. **Declare `:wat::core::Option` and `:wat::core::Result` as parametric `defenum`s in
   `wat/core.wat`**, matching the Rust literals exactly — variants, field names (`value` / `value`
   + `error`), purity `Pure`, and param order.
3. **Replace the two `register_builtin` literals** with `wat_enum_register_from!` calls.

**Do NOT add a `wat_enum_from!` generated Rust enum** unless a consumer needs one. Option/Result are
`Value::Option` / `Value::Result` in Rust, not a matched generated enum — that is the difference
from H-2c, where the consumers matched on variants. If you find a consumer that wants it, STOP and
report rather than adding it speculatively.

## STOP TRIGGERS

- **STOP-1 — the params come back empty or reordered.** `tests/types/probe_arc296_h3_option_result_
  keep_their_params.rs` is GREEN AT HEAD and guards exactly this. If it goes red, the capture is
  wrong — fix the capture, never the probe.
- **STOP-2 — the binder capture reads the payload.** The offset hazard above. Prove it on the
  3-param case (`ServiceEvent :- [I O A]`), not just the 1-param one; a fixed-offset bug can pass
  at arity 1 and fail at 3.
- **STOP-3 — the load-time re-declaration is not a `NoOp`.** If the gate reports `Duplicate`, the
  wat declaration and the macro-produced registration DISAGREE. That is the wall doing its job:
  the declaration is wrong. Do not widen the gate.
- **STOP-4 — purity changes.** `Pure` is argued in the prose at `types.rs:1233-1242` and gates two
  real things (wire-crossing and `:durable`). Carry it verbatim; do not re-decide it.
- **STOP-5 — a THIRD type needs moving to make this compile.** Report it; do not widen.

## OUT OF SCOPE, AFFIRMATIVELY

The match arm (map pattern) · the variant constructor form · the record-steals-a-variant's-
constructor NOTE · every other builtin still living as a Rust literal.
