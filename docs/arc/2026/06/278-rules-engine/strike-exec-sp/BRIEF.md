# BRIEF — give the arena pointer an owner, then tell the truth about the mechanism

`with_exec_frame` restores `EXEC_SP` on the line *after* `f`, so any unwind skips it — and driven,
that strands `len` slots **per panic, cumulatively, forever**. The same file's doc claims nested
calls "stack", and they do not: the `RefMut` is held across `f`, so every nested call takes the heap
arm. Read `DESIGN.md` first — its ★ is one line, its ⚠ says the guard makes `EXEC_SP` provably dead
and you must *decide*, and its "out of scope" cuts the redesign.

## Read in order

1. `src/rete/expr_ir/eval.rs:104-128` — `with_exec_frame`. The `Ok` arm's three `EXEC_SP` lines are
   the site; the `Err` arm is correct and stays.
2. `:93-103` — the two doc paragraphs. The first says nested calls **stack** (false); the ⚠ three
   lines below describes the heap fallback (true). They contradict.
3. `src/rete/kernel/fire/rules.rs` and `wat/rete/syntax.wat` as landed by **B1** (`7319c1ea4`) — the
   same defect, already cured once in this arc, with an `ArmLease` whose `Drop` releases. **That is
   the shape to copy**, including its comment register.
4. `src/rete/kernel/tests/arm_lease.rs`, the two unwind probes — a wat error and a host panic driven
   separately, because they are different mechanisms. Yours needs the panic arm at minimum.

## Sketch

```rust
struct SpGuard(usize);
impl Drop for SpGuard {
    fn drop(&mut self) { EXEC_SP.with(|c| c.set(self.0)); }
}
// in the Ok arm, before f:
let _sp = SpGuard(start);
```

Then resolve the ⚠: with the pointer always restored, `start` is provably `0` at that arm. Delete
the mechanism and take `[0, len)`, or keep it and write down the re-entrancy it holds open.

## The probe

Drive the strand, not just the guard: panic through `f` **three times** and assert the pointer and
the arena length are unchanged across all three. A single panic would pass against a fix that resets
once.

## Traps named in advance — each with its step

1. **★ A guard on a `Cell` inside a `RefCell` borrow — check the drop order.** `_sp` and the `RefMut`
   both drop at scope end. **Step:** confirm the guard's `EXEC_SP.with` cannot run while the arena
   borrow is still held in a way that panics; drive it, do not reason about it.
2. **TLS teardown.** B1 hit this exactly: a `Drop` that touches a thread-local can run after that
   local is destroyed, and `.with()` **panics** there. **Step:** `try_with`, and say whether it was
   needed — B1's equivalent was measured to abort, not panic.
3. **⚠ Do not leave `EXEC_SP` "just in case".** DESIGN requires a decision. **Step:** whichever you
   pick, the ⚠ paragraph must stop claiming nested calls stack.
4. **The `Err` arm is correct.** It is the re-entrancy path and it stays. **Step:** if your change
   makes it unreachable, that is a finding — report it.
5. **This is a hot path** — `exec` runs once per row per fire. **Step:** the guard is a `Cell` write
   on drop; if you find yourself adding an allocation or a branch per call, stop and say so.
6. **New test code trips `wat::lint`, and clippy is not that binary.** **Step:** run
   `binary_id(wat::lint)`; keep the test idiomatic. Four riders have been green there and had clippy
   RED.

## STOP triggers

- **STOP-1** — if the strand turns out to be unreachable in production (no panic can cross `f`),
  STOP and report. DESIGN rests on `assertion-failed!` panicking, which B1 established.
- **STOP-2** — if any currently-green test goes red, STOP and report which.
- **STOP-3** — if deleting `EXEC_SP` turns out to change behaviour anywhere, STOP: that would mean it
  is not inert and the drive was wrong.

## Shape to copy

`docs/arc/2026/06/278-rules-engine/strike-lease-unwind/` — B1, this arc, the same defect cured with a
`Drop`.

## The one thing worth more than the fix

**Tell me where this brief was thin.** Twenty-six riders before you each returned a prescription of
mine that did not survive contact. The last found that my rule would have **deleted a true claim**,
and that my exclusion boundary was the wrong *kind* of boundary. If a step here is wrong,
unnecessary, or impossible, say it plainly.
