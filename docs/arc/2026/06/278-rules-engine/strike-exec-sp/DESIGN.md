# DESIGN-STONE — the arena pointer is restored by a line the unwind skips

> **Origin (2026-09-01).** Class **D4**, found by `struere`. Driven at HEAD `35e0938cb`. **The row
> understates it: the strand is not `len` slots, it is `len` slots PER PANIC, forever.**

## Why — both halves driven

`with_exec_frame` (`expr_ir/eval.rs:104`) holds the `RefMut` across `f`, sets `EXEC_SP` to `end`
before the call, and restores it **after**:

```rust
EXEC_SP.set(end);
let out = f(&mut g[start..end]);
EXEC_SP.set(start);          // ← an unwind never reaches this
```

### 1. `EXEC_SP` is inert, and the doc claims the opposite

The `RefMut` is live for the whole `Ok` arm, so a nested `with_exec_frame` **always** fails
`try_borrow_mut` and takes the heap arm. Driven — outer set `sp=4`, and the inner call **observed
`sp=4` and never changed it**:

```
D4 nested: outer set sp=4, inner observed sp=4
D4 after clean nesting: sp/arena = (0, 4)
```

So `start` is always `0` at the `Ok` arm and the `set(end)`/`set(start)` pair is dead bookkeeping.
The doc at `:96` says *"Nested calls therefore stack rather than collide."* **They do not stack —
they fall back to the heap**, which the ⚠ paragraph three lines below describes correctly. **Two
adjacent doc claims contradict each other**, and the true one is the second.

### 2. The strand is cumulative and permanent

```
after panic 1: sp/arena = (8, 8)
after panic 2: sp/arena = (16, 16)
after panic 3: sp/arena = (24, 24)
```

`EXEC_SP` never resets. Every subsequent frame starts further into a **monotonically growing**
arena, for the life of the thread. The row says *"strands `len` slots permanently"*; it is `len`
slots **every time**, unbounded.

The panic path is reachable: `:wat::kernel::assertion-failed!` panics the host (established by B1,
`runtime.rs:15922`) and a `where` fence or fold runs user code.

## ⛔ THIS IS B1's SHAPE, IN A DIFFERENT SUBSYSTEM

B1 (`7319c1ea4`): a release call after the body, skipped by any unwind, with **no `Drop` guard**.
Same here — and B1's finding applies verbatim: *the shape was never the defect; the absence of an
owner is.* `with_open_file` was safe with the identical `let`+`do` shape because a Rust value owned
the release.

## ★ THE ONE CONTRACT DECISION

**The arena pointer is restored by a `Drop`, not by a line after the call.** A guard holding `start`
restores it on every exit — normal return, `?`, and unwind alike.

## ⚠ AND THE GUARD MAKES `EXEC_SP` PROVABLY DEAD — decide, do not leave it

With the pointer always restored, `start` at the `Ok` arm is **provably** `0`: the only route to that
arm is an unborrowed arena, which now implies a restored pointer. The mechanism then computes
nothing.

**Do not simply keep it because it looks like bookkeeping.** Either delete it and take the window as
`[0, len)`, or keep it and state the future re-entrancy it is holding open — with the ⚠ paragraph
rewritten so the file stops claiming a stacking that does not happen. This arc has removed enough
machinery that "looked deliberate" to insist the choice be written down.

## Blast radius

`src/rete/expr_ir/eval.rs` only, plus a probe. No wire change, no outcome change.

## Out of scope — AFFIRMATIVELY CUT

- **Making nested calls actually stack** (dropping the `RefMut` before `f` so the inner call can
  take a real window). That is a *performance* change to the hot path — `exec` runs once per row per
  fire — and it needs its own measurement. This strike fixes the leak and tells the truth about the
  mechanism; it does not redesign the arena.
- **The heap fallback itself.** Correct, and the ⚠ paragraph's account of it is the accurate half.
- **C3, C5, C6, D5, D6, D7.** Their own rows.
