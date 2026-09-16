# BRIEF — refuse a call whose argument count does not match its parameter count

`exec_program_on` never compares `args.len()` to `program.params.len()`, and its `else if` branch
writes a surplus argument into the slot whose number equals the argument's position. Driven: that
silently changes the answer (1 hit → 0), silently drops arguments (→ 2 hits), or surfaces as
`unbound symbol: slot 1`. Add the comparison at the one place the two quantities meet and delete
the branch. Read `DESIGN.md` beside this file first — its ★ section pins where the check goes AND
why it goes nowhere else, and its ⚠ section names the one exemption you must verify rather than
assume.

## Read in order

1. `src/rete/expr_ir/eval.rs:383-418` — `exec_program_on`. The `for (i, v) in args.iter()` loop is
   the site; the `else if i < inner.len()` branch is the one you delete.
2. `src/rete/expr_ir/eval.rs:363-372` — `exec`'s `Expr::CallUser` arm, including the
   `args.is_empty()` short-circuit. This is the exemption DESIGN's ⚠ section tells you to verify.
3. `src/rete/expr_ir/eval.rs:424-440` — `exec_foldl`'s existing `ArityMismatch` refusal. Copy its
   shape: same error kind, `op` naming the callee, `expected` and `got` both filled.
4. `src/rete/expr_ir/eval.rs` (`exec_foldl` body) — the `exec_program_on(&program, &[acc, x], …)`
   call. Two args, two params: confirm for yourself that the new check passes here.
5. `src/rete/expr_ir/mod.rs:620-630` — where `Expr::CallUser` is built with no arity comparison.
   Read it to see why the wire is not the only door; **do not add a check here** (out of scope).
6. `tests/rete/probe_arc278_export.rs:639-700` — `kw`, `vec_of`, `tamper_first_prog_root`,
   `rete_ops_names`, and the four A6 depth probes as the model for "build a wire expr, tamper the
   first prog root, import, assert".

## Sketch

```rust
// exec_program_on, before filling the frame
if args.len() != program.params.len() {
    return Err(RuntimeError::new(span.clone(), RuntimeErrorKind::ArityMismatch {
        op: /* the callee, as named as this Program allows */,
        expected: program.params.len(),
        got: args.len(),
    }).into());
}
…
for (i, v) in args.iter().enumerate() {
    let idx = program.params[i] as usize;   // total: the lengths are equal
    inner[idx] = Some(v.clone());
}
```

## Observing it end-to-end

The fixture mouth is already in the tree: `:user::import-and-hits` in
`tests/rete/probe_arc278_export.wat` takes an Export and returns the hit count through
seed → fire → query. Untampered it answers **1**. The three tampers and their driven pre-fix
answers are in DESIGN's table — your probes assert the refusal instead, and each must be RED
before the change for its own stated reason.

## Blast radius

`src/rete/expr_ir/eval.rs` and `tests/rete/probe_arc278_export.rs`. The `.wat` fixture already
carries what you need; no other file.

## Traps named in advance — each with its step

1. **`op` wants a name a `Program` may not carry.** Check what `Program` holds (`names`, and what
   `lower_rete_defn` puts there) before inventing a field. **Step:** if there is no callee name
   available, say so and use the best identifier the struct actually has — do not add a field to
   `Program` to make the message prettier; that is outside the radius.
2. **The `args.is_empty()` short-circuit.** DESIGN's ⚠ section reasons it is a zero-parameter call
   and passes the check. **Step:** run the suite. If something goes red there, **report it** — do
   not carve an exemption. An exemption re-opens the branch this strike deletes.
3. **`exec_program_on` is on the fire path** (per token, and per element under `foldl`). **Step:**
   the check is one integer comparison; keep it that way. Do not build a formatted message on the
   success path.
4. **Three arms, three probes.** Arm 1 (surplus collides with a param slot), arm 2 (surplus past
   the frame), arm 3 (missing argument) are three different pre-fix behaviours — a wrong answer, a
   dropped argument, and an `UnboundSymbol`. **Step:** one probe each, and state per arm what its
   pre-fix failure actually was. One drive cannot prove three arms.
5. **Arm 3's pre-fix state is already an error**, so a probe asserting "it errors" would pass
   before and after. **Step:** assert the refusal is an **ArityMismatch naming the counts**, not
   merely that an error occurred — otherwise it is a counter-proof that cannot fail.
6. **`parent` is copied into the frame BEFORE args.** Not a defect on the `CallUser` path (parent
   is `None` there) but it is why the loop order matters. **Step:** leave the parent copy where it
   is; only the args loop changes.

## STOP triggers

- **STOP-1** — if `args.len() != params.len()` turns out to be legitimate on some path, STOP and
  report which path and why. That is a contract question and it is the orchestrator's.
- **STOP-2** — if any currently-green test goes red, STOP and report which. Do not adjust the
  check to accommodate it.
- **STOP-3** — if deleting the `else if` branch does not compile because some caller depends on
  the surplus write, STOP and report the caller by name.

## Shape to copy

`docs/arc/2026/06/278-rules-engine/strike-import-depth/` — the strike immediately before this one:
DESIGN with a pinned contract, probes RED before the change, one drive per arm, and a report
stating per arm **proven / reachable-but-not-driven / not-reachable-and-why**.

## The one thing worth more than the fix

**Tell me where this brief was thin.** Eleven riders before you each returned a prescription of
mine that did not survive contact. The last one found that I had named ONE unbounded tower when
there were THREE — because a doc comment stated the defect as a feature and I read it as settled.
That was worth more than the code it came with. If a step here is wrong, unnecessary, or
impossible, say it plainly.
