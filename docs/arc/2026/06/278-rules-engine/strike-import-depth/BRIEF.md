# BRIEF — give the import door a depth budget, shared across its whole recursive descent

`import_export` has no depth criterion. What it accepts is whatever the importing thread's stack
allows — the *same* 20,000-deep Export is ACCEPTED on a 256 MiB thread and aborts the process on a
2 MiB one, both driven. Give the descent one depth budget and refuse past a stated constant with
`malformed`, the way the other four walls refuse. Read `DESIGN.md` beside this file first: its ★
section pins the contract, its **THE CYCLE** section names why a counter on `unpack_expr` alone
does not hold, and its "out of scope" section cuts three shapes with reasons.

## Read in order

1. `src/rete/export.rs:723-1010` — `unpack_expr`. Thirteen arms; `:call`, `:call-fb`, `:user`,
   `:field`, `:ctor`, `:variant`, `:if`, `:and`, `:or`, `:let`, `:match` all descend.
2. `src/rete/export.rs:1011-1085` — `unpack_prog`. Its root at `:1074` re-enters `unpack_expr`;
   `unpack_expr`'s `:user` arm at `:779` enters *here*. **This is the cycle.**
3. `src/rete/export.rs:567-625` — `unpack_pat`, recursing at `:616`, reached from `unpack_expr`'s
   `:match` arm at `:973`.
4. `src/rete/export.rs:1357`, `:1401`, `:1452`, `:1512` — four more entries into `unpack_prog`
   from the cond/driver/rhs unpackers. They enter the same cycle and need the same budget.
5. `src/rete/export.rs:240-272` — `check_slot` and the `malformed(...)` refusal shape. Your
   refusal must read like these: same op, same kind, a reason naming the bound and the depth hit.
6. `src/rete/export.rs:2112-2128` — the A1 graph wall, the most recent wall added at this door.
   Copy its placement discipline and its comment register.
7. `tests/rete/probe_arc278_export.rs:271-300` — `import_one`, `poke_named`, `seq_values`, and
   `import_refuses_abi_mismatch` (`:137`) as the model for "tamper, import, expect refusal".

## Sketch

```rust
/// Measured: the deepest nesting the corpus produces is N (see the constant's comment);
/// this is N x <multiplier>. The smallest stack observed dies between 3,000 and 5,000.
const MAX_IMPORT_DEPTH: u32 = /* measured, not chosen */;

fn unpack_expr(v: &Value, span: &Span, depth: u32) -> Result<Expr, EvalBreak> {
    let depth = depth_check(depth, span)?;   // increments, refuses past the bound
    // … every recursive call passes `depth`, including the `:user` arm into unpack_prog
}
```

The budget threads through `unpack_expr`, `unpack_prog`, `unpack_pat`, and the four cond/driver/rhs
entry points. The top-level callers (`:2234`, `:2250`, …) start it at 0.

## Blast radius

`src/rete/export.rs` and `tests/rete/probe_arc278_export.rs`. No new fixture — `cool-export` and
`import-one` already exist beside that probe file. No wire-format change: this refuses inputs that
were previously accepted-or-fatal, and accepts every input the corpus produces.

## Traps named in advance — each with its step

1. **A `:user` tower bypasses an expr-only counter.** **Step:** write the second probe as a tower
   of `:user`/`:prog` alternation, not just `:and`. If it is refused only after you thread the
   budget through `unpack_prog` too, you have proven the cycle matters — say so in the report.
2. **Do not put a probe near the stack threshold.** The threshold is 3,000–5,000 *on this thread*
   and moves with stack size; a probe there is a flake generator. **Step:** probe just above the
   bound (e.g. bound + 8). Pre-fix that depth is ACCEPTED — which is the RED — and it never goes
   near the stack.
3. **The abort is not catchable.** `catch_unwind` will not save a probe that recurses deep enough,
   so no floor test may drive the actual overflow. **Step:** the recon that proved it is recorded
   in DESIGN; do not re-add it to the tree.
4. **Measure the bound.** **Step:** instrument the descent to record the max depth reached, run the
   export/import tests, report that number, then set the constant to it times a multiplier you
   name. Remove the instrument before you finish. A round number with no measurement behind it is
   the finding, not the fix.
5. **`check_expr_slots` / `check_pat_slots` inherit the bound only if every tree they see came
   through the unpack.** **Step:** check their callers. If one takes a tree from elsewhere, do not
   fix it here — report it as a separate door.
6. **The floor is the real control.** Every existing export/import test imports a legitimately
   nested program. **Step:** if the bound breaks one, the bound is wrong, not the test.

## STOP triggers

- **STOP-1** — if threading the depth parameter turns out to need a signature change on something
  outside `export.rs`, STOP and report the boundary. The blast radius says this file.
- **STOP-2** — if any currently-green test goes red, STOP and report which and why. A legitimate
  program hitting the bound means the measurement was wrong.
- **STOP-3** — if `check_expr_slots` or `check_pat_slots` turns out to be reachable on a tree the
  import unpack did not build, STOP and report it. That is a second door, and scoping it belongs
  to the orchestrator.

## Shape to copy

`docs/arc/2026/06/278-rules-engine/strike-lease-unwind/` — same arc, one strike back: DESIGN with a
pinned contract, probes RED before the change, one drive per arm, and a report that states per arm
**proven / reachable-but-not-driven / not-reachable-and-why**.

## The one thing worth more than the fix

**Tell me where this brief was thin.** Ten riders before you each returned a prescription of mine
that did not survive contact — the last one caught that I had prescribed the very step a gate's own
doc forbids, and that my scorecard's pinned test COUNT had silently capped its coverage. Those were
worth more than the code. If a step here is wrong, unnecessary, or impossible, say it plainly.
