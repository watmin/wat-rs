# DESIGN — a diagnostic must say the same thing twice

## Why

**C19.** The same binary, the same file, five runs — **five different outputs**:

```
expects (:wat::stream::Stream :- [:?125])
expects (:wat::stream::Stream :- [:?3399])
```

`InferCtx::fresh` (`check.rs:485`) is a monotonic `u64` from 0, so the printed id encodes *how many
type variables were allocated before this one*. That count varies per process — some traversal
upstream is `HashMap`-ordered under Rust's per-process random hasher.

**It is already costing us.** The D11 rider's corpus scan produced **9 false "regressions"** from it,
and had to normalise `:?[0-9]+` to get a usable answer. Any future output diff — a rider's, a
reviewer's, a script's — pays the same tax, and pays it as *noise that looks like signal*.

## Bounded, and the bound is what makes this cheap

| | |
|---|---|
| error kinds, their ORDER, spans, message text | **stable** — normalising `:?N` makes runs byte-identical |
| the rendered type-variable id | **varies per process** |
| render sites | **two**: `check.rs:16342` `format!(":?{}", id)`, `:16372` `format!("?{}", id)` |
| messages carrying a var, in 120 `.wat.bad` | **1** |
| those carrying MORE THAN ONE distinct var | **0** |
| goldens pinning a literal id | **0** |

**This is not nondeterministic checking.** Inference is sound — driven: correcting the fixture's
argument order yields the same error with the variable **resolved**. It is a nondeterministic
*rendering* of one field.

## ⛔ AND THE VALUE IS NOT MEANINGFUL EVEN WHEN STABLE

`:?3399` is an allocator counter. It tells a reader nothing, and **its sibling message in the same
output renders a declared type parameter as `T`** — so one output shows one unknown as `T` and
another as `:?3399`. Making the number stable without making it *meaningful* would fix the diff noise
and leave the confusion.

## ★ THE INVARIANT

> **A diagnostic renders the same bytes for the same program, every run — and an undetermined type
> reads as undetermined, not as a counter.**

Two shapes satisfy it; the measurement above says either works and neither is forced:

1. **Render `_`** (or `?`). Simplest and honest. Loses the ability to tell two distinct unknowns
   apart — **measured: no message in the sampled corpus carries two.**
2. **Per-diagnostic renumbering from 1** (`?1`, `?2`). Robust if a message ever carries several, at
   the cost of a render-time map.

Pick one and argue it. **Do not chase the traversal to determinism** — that is a much larger job, it
is not what the reader needs, and a stable-but-meaningless counter still fails the second half of the
invariant.

## The gate is the real prize

A two-line fix is worth little; **a gate that pins diagnostic determinism corpus-wide** would have
caught this and catches the next one. Run each `.wat.bad` **twice** and require byte-identical
output. That is cheap, it is a property rather than a fixture, and it converts "output diffs are
noisy" from folklore into a red build.

## Files

`src/check.rs` (two arms), plus a gate. No inference change.

## Out of scope = REJECTED

- **Making the upstream traversal deterministic.** Bigger, riskier, and unnecessary for the
  invariant — the render is what the reader sees.
- **C18's `.wat.bad` sweep.** Adjacent, separate row.
