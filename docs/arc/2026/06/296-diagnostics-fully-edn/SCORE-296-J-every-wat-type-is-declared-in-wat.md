# SCORE — 296 J: every type wat uses is declared in wat

No commit. Floor and clippy left to the orchestrator.

The 26 hand-written `EnumDef` literals in `register_builtin_types` are gone. Each is a
`defenum` in its family `.wat` file, registered via `wat_enum_register_from!`. A lint
refuses the literal form at the door; it was shown to fire, then the sabotage was reverted.

---

## Row 1 — zero hand-written enum *registration* literals

`grep 'register_builtin(TypeDef::Enum(EnumDef {' src/types.rs` = **0**.

The EXPECTATIONS grep `TypeDef::Enum(EnumDef {` is **4** and those four are not type
homes: `parse_defenum` constructs an EnumDef from a wat form; the defservice synthesizer
emits op/reply enums the surface declared; a match pattern reads `variants`. Refusing
them would be an exemption on day one (STOP-5). The wall keys on the registration FORM
the brief named.

## Row 2 — registered FROM WAT

`grep -c 'wat_enum_register_from!' src/types.rs` = **30**:

- 26 this stone
- 2 H-3 (Option/Result)
- 1 H-2c (LociDiedError)
- 1 unit-test proof (`ServiceEvent` arity 3)

Design counted **25**. The scan found a 26th: `:wat::io::IOReader::ReadFrameOutcome`
(nested under IOReader, sibling of `kernel::ReadFrameOutcome`). Declared in existing
`wat/io.wat`. Not skipped.

## Row 3 — the declarations exist

```
wat/core.wat                 ReadOutcome, ReadWithCommentsOutcome     (+ Option/Result from H-3)
wat/holon.wat                VectorDecode, Combine, DegenerateSide, Cosine, Dot
wat/kernel/outcomes.wat NEW  ReadFrame, Readln, Recv, Send, TrySend, Close,
                             Signal, SignalOutcome, Accept, Connect, RunResult
wat/edn.wat             NEW  ReadJson, ReadForeign, Validation
wat/eval.wat            NEW  StepResult, WalkStep, FormOutcome
wat/stream.wat          NEW  NextOutcome
wat/io.wat                   IOReader::ReadFrameOutcome   (the 26th)
```

## Row 4 — params survive, per type

`nine_parametric_outcomes_keep_params_in_declaration_order` **PASSES**.

```
WalkStep [A]  ReadJson [T]  ReadForeign [T]  Readln [T]  FormOutcome [T]
Recv [O]  Next [T]  Accept [R, S]  Connect [S, R]
```

Accept vs Connect opposite orders held.

## Row 5 — prose survived

Each Rust `//` block immediately above a literal was transcribed to `;;` above the
`defenum`, including per-variant comments and purity inline notes (`PURITY Impure: …`).
The Option/Result essay (`★★ 2026-08-05`) stayed with H-3 in `types.rs` / `core.wat`,
not attributed to ReadOutcome.

No line deliberately dropped. `types.rs` still carries the original comments at the
registration site (a pointer + the `wat_enum_register_from!` call).

## Row 6 — new files joined the load set

Inserted in `STDLIB_FILES` immediately after `diagnostics.wat` (outcomes need
`Failure`/`LociDiedError`). `wat_record_from_sources_are_loaded` **PASSES**. The new
lint also checks every `wat_enum_register_from!` path is in the load set (**PASSES**).

## Row 7 — re-declarations are NoOps

H-2 probe (wat fixtures) and the param probe both built a world. No `Duplicate`.

## Row 8 — floor (ORCHESTRATOR)

Not run.

## Row 9 — clippy (ORCHESTRATOR)

Not run.

## Row 10 — THE WALL EXISTS AND BITES

Planted `register_builtin(TypeDef::Enum(EnumDef { name: ":wat::core::Sabotage" …}))`
next to Option. Lint went **RED at line 1203**, naming the site. Plant reverted.
Post-revert lint **green**.

## Row 11 — no exemptions

`grep allow\|rune:` on the lint file = **0**.

---

## STOP rows

| STOP | result |
|---|---|
| STOP-1 variant/field/purity | **held.** Stdlib loaded. |
| STOP-2 prose dropped | **held.** Transcribed; none named as dropped. |
| STOP-3 load-set cycle | **held.** New files sit after diagnostics.wat. |
| STOP-4 unreachable | **not a finding.** CloseOutcome is used (`signal_kill_produces_close_outcome_signaled`); DegenerateSide is CosineOutcome's payload. Both declared. |
| STOP-5 exemption | **not taken.** Wall keys on `register_builtin(…)` so parse_defenum/defservice synthesis stay legal. |
| STOP-6 record/struct | **not taken.** |

## Targeted checks (executor)

```
cargo test --release --test lint -- no_hand_written_enumdef     3 passed (after sabotage revert)
cargo test --release --test types -- probe_arc296_j             passed (9 param orders)
cargo test --release --test types -- probe_arc296_h2            3 passed
cargo test --release --test types -- option_and_result          passed
cargo nextest run --release -E 'test(option_result_tagged)'     8 passed
cargo test --release --test lint -- wat_record_from_sources     passed
wall sabotage: RED at types.rs:1203, then reverted
```

## Files (this stone)

```
wat/kernel/outcomes.wat          NEW — 11 kernel outcomes
wat/edn.wat                      NEW — 3 edn outcomes
wat/eval.wat                     NEW — 3 eval outcomes
wat/stream.wat                   NEW — NextOutcome
wat/core.wat                     + ReadOutcome, ReadWithCommentsOutcome
wat/holon.wat                    + 5 holon outcomes
wat/io.wat                       + IOReader::ReadFrameOutcome
src/types.rs                     26 literals → wat_enum_register_from!
src/load/stdlib.rs               four new files in STDLIB_FILES
tests/lint/no_hand_written_enumdef.rs
tests/types/probe_arc296_j_params_survive.rs
```
