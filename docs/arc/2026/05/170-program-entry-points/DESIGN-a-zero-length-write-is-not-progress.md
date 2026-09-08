# DESIGN — a zero-length write is not progress

Small stone, one decision, named in `c2a8e45e3`'s commit as a follow-up.

## The defect

```
write_once:459-465     n > 0  → Wrote(n)     n < 0 → Errno(-n)     n == 0 → Wrote(0)
send:523-525           while written < framed.len() { … Wrote(n) => written += n … }
```

`written += 0` re-enters the loop and resubmits the **identical** Write. If a zero recurs, `send`
never returns and never reports — an unbounded loop on the transport's hot path.

The loop condition guarantees the buffer is non-empty, and a `write()` of a non-empty buffer returning
`0` is **not a documented outcome** for a pipe.

## ★ The codebase already contains the argument

`try_send`'s own short-write branch refuses exactly this:

> *"A short, non-blocking write mid-frame: rather than loop (which could spin against a still-full
> pipe), treat it as best-effort failure — never torn frames on the wire."*

**The two paths already disagree, and `try_send` is the honest one** — it names the spin hazard in
prose and declines to take it. `send` takes it.

## The decision, and a rung that was missing from it

The four questions ruled: `debug_assert!` + `SendError::Failed`; leaving it fails **Honest** (an
undocumented state becomes a silent infinite loop), and asserting dominates a bare error at no release
cost.

⚠ But that menu offered only the *check* rung. The **no-form** rung is available here and costs
nothing extra:

```rust
enum WriteWait { Wrote(NonZeroUsize), Errno(i32), Shutdown }
```

`Wrote(0)` then **cannot be constructed**. `write_once` is forced by the compiler to decide what
`n == 0` means, and `send`'s `written += n.get()` can never add zero. The failure is not caught — it
has no representation.

★★ That is the same climb the day has been making everywhere: convention → check → *a shape the
mistake cannot be written down in*. And it fixes the testability problem the drain classifier taught
us: there is no unreachable-from-a-test error path left, because the state does not exist.

## The one contract decision

**`n == 0` maps to `SendError::Failed(value, reason)`** — the existing *"any other write failure
carries its real reason"* arm. **No new variant, no change to any other arm.** The reason string names
what happened: a zero-length write of a non-empty buffer, which the kernel does not define.

## Out of scope — REJECTED

- **`try_send`.** Already correct, and its comment is the reasoning this stone adopts.
- **Any retry, budget, or backoff.** The point is that a zero is not progress; waiting longer does not
  make it progress.
- **Widening `SendError`.** `Failed` already carries a reason.

## Files

`src/comms/process.rs` only — the `WriteWait` enum, `write_once`, and `send`'s match arm.
