# EXPECTATIONS — STONE: `:wat::eval::walk` faces `:wat::WatAST`. Written BEFORE the strike.

| # | what | command | expected | derived from |
|---|---|---|---|---|
| 1 | the composition probe round-trips | the new scratch-pad probe | the returned form == the walked form | STOP-1 |
| 2 | the holon tag is gone from output | same probe, before/after | `#wat/holon` absent after | the whole point |
| 3 | the real caller is unaffected | `-E 'binary_id(wat::types)'` | green; it reads only `second` | measured: no element-0 read |
| 4 | today's probe still runs | run it | terminal renders as a plain form | it also reads only `second`-shaped output |
| 5 | the loader gate | `-E 'test(every_wat_scripts_file_loads)'` | pass | a new `wat-scripts/` file |
| 6 | `wat` + reflection suites | the two scoped runs | green | `holon_to_watast`'s other users |
| 7 | full floor | `scripts/floor.sh`, orchestrator, unpiped | 5129 passed, 0 FAIL | current floor |
| 8 | clippy | `-D warnings --all-targets` | 0 | standing |

## Ledger movement

**None, and derived.** No registration, no retirement, no property grade. `:wat::eval::walk` already
has a `CheckEnv` scheme — this changes one field inside it, so DEBT does not move either.

```
registry 556 · GAP_A · GAP_B · DEBT 121 · TYPES_UNCHECKED 10 · corpus 37   ← ALL UNCHANGED
```

⚠ **The round trip must stay 435/435.** `probe_can_doc_types_reconstruct_the_checker_scheme` compares
this scheme against its doc types, and the freeze list is EMPTY as of `STONE-the-round-trip-closes`
— zero tolerance. If `walk`'s `@ret` doc names `HolonAST` and the scheme now says `WatAST`, that gate
goes red and it is RIGHT to. **Fix the doc, do not re-freeze the row.**

## Runtime

**20-30 min.** Two edits are trivial; the composition probe and its before/after capture are the work.

## Trap doors, named in advance

1. **A lossy conversion behind an honest signature.** The stone's only real risk, and it would be
   strictly worse than the asymmetry it replaces — the caller would then be told `WatAST` and handed
   something degraded. The current probe's terminal is the scalar `5`, which cannot detect this.
   STOP-1, and it is why the brief demands a COMPOSED terminal form.
2. **The `@ret` doc going stale against the scheme.** See the ledger note — the zero-tolerance gate
   will catch it, and the correct response is to fix the doc.
3. **Assuming the callers.** Both are named in the DESIGN as not reading element 0. That was
   measured once, by the orchestrator, from two files. Confirm it.
4. **Widening into the residue.** 27 other `Value::holon__HolonAST` producers sit in the same file
   and are correctly hidden behind `WatAST`-facing surfaces. STOP-2.
