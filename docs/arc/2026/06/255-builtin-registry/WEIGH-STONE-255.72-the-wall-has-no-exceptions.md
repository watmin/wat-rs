# WEIGH — STONE 255.72: the wall has no exceptions — ACCEPTED

**Executor: a Sonnet subagent, commit `4969e1907`.** Weighed by the orchestrator on 2026-10-01.

## Re-run by the orchestrator

| row | result |
|---|---|
| release floor at `4969e1907` | **6235 passed / 24 skipped**, exit 0 |
| site 2 typed by A1 | `wat/rete/oracle/accum-pass.wat` splices the fold's own declared element type (`elem-ty`) into `(wat.type/PersistentVector :- [elem-ty] …)` |
| the agent's oracle runs | the custom-fold differential tests 3/3 and the impurity fence 1/1 before and after, byte-identical; `--test rete` 524 passed |

## What landed

- **Site 2 (A1, measured):** the runtime **can** read a custom fold's declared parameter type, through
  `:wat::runtime::signature-of-defn` + `extract-arg-types` (already used in `wat/rete/compile.wat:833` and
  `wat/rete/oracle/stratify.wat:82`). On the oracle's own test fold (`[xs <- (PersistentVector :- [i64])]`) it yields
  `wat.type/i64`. One wrinkle: the reified `acc-hd` AST value had to be round-tripped through its name to reach the
  lookup. No `wat.type/Value` guess anywhere.
- **Site 1:** typed with the probe's own `~ret-ty`, per `wat/bracket.wat:491`'s precedent.
- **The wall has no exceptions:** the table's two `:stop1` rows are now `:typed`; 0 bracket-less constructor sites remain
  outside `.wat.bad` / `.wat.golden`.

## ⚠ A finding: a rotted probe

`wat-scripts/probes/arc-170/probe-s3b-astsplice.wat` **already crashes before this stone**, identically before and after,
at line 59 (`keyword-node` refuses an angle-bracket type name, arc 109's wall). Execution never reaches the typed site.
So site 1's typing is correct by reading, but **no run reaches it**. The loader gate only type-checks the file, so its
rot is invisible to the floor. It needs a ruling: repair it, or retire it as a dead probe.
