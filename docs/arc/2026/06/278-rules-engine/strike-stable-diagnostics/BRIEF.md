# BRIEF — make a diagnostic say the same thing twice, and gate it

The same binary on the same file prints different bytes each run, because a type-variable allocator
counter is rendered into the message. Fix the rendering, and gate the property so the next one cannot
land.

## Read in order

1. **Reproduce it first**:
   `for i in 1 2 3 4 5; do ./target/release/wat tests/function/probe_arc247_hof_coll_first.wat.bad 2>&1 | md5sum; done`
   → five different hashes. Then re-run piping through `sed 's/:?[0-9]*/:?N/g'` → **identical**. That
   pair is your before/after target.
2. `src/check.rs:16342` — `TypeExpr::Var(id) => format!(":?{}", id)` in `format_type`.
3. `src/check.rs:16372` — `TypeExpr::Var(id) => format!("?{}", id)` in `format_type_inner`. **Both
   sites, or the leak survives in one of two renderers.**
4. `src/check.rs:485` — `InferCtx::fresh`, the monotonic counter. **Read it to confirm you should not
   change it**: the id is fine internally; only its *rendering* is the defect.
5. The full diagnostic from that fixture — note the sibling message renders a declared parameter as
   `T` while this one renders `:?801`. **One output, two spellings of "unknown".**

## ★ The invariant

> A diagnostic renders the same bytes for the same program every run, and an undetermined type reads
> as undetermined rather than as a counter.

Two shapes satisfy it — render `_`/`?`, or renumber per-diagnostic from 1. **Measured for you:** of
120 `.wat.bad` files, exactly **1** message carries a type var and **0** carry more than one distinct
var, so either works. Pick one, argue it, and say what it costs a reader when a message *does* carry
two.

## The gate is the real prize

Ship a gate that runs each `.wat.bad` **twice and requires byte-identical output**. A two-line render
fix is worth little on its own; this converts "diagnostic diffs are noisy" from folklore into a red
build, and it is a property rather than a fixture.

⚠ Scope it so it stays fast — the corpus is ~276 `.wat.bad`. If running all of them twice is too slow
for the floor, say so with the measured runtime and propose the cut.

## Blast radius

`src/check.rs` (two arms) and one gate. **No inference change**, no traversal change.

## STOP triggers

1. **If you find yourself making the upstream traversal deterministic**, stop. That is a much larger
   job and it is not what the invariant needs.
2. **If any existing golden changes**, stop and report which — 0 goldens pin a literal id today, so a
   changed golden means the fix reaches further than the render.
3. **If the twice-run gate is red on a file for a reason OTHER than a type var**, stop and report it
   — that is a second source of nondeterminism and it outranks this one.
4. **If a message turns out to carry two distinct vars**, stop and report it before choosing `_`.

## Mutation proofs — run all three, report all three

1. **Revert the render fix** → the twice-run gate goes RED, naming the file and the differing bytes.
2. **Break only ONE of the two sites** → the gate must still go RED. Proves both renderers are
   covered, not just the one the repro happens to exercise.
3. **The repro's message still names the right types** after the fix — a stable rendering that lost
   the type information would pass rows 1 and 2 and be useless.

Restore after each.

## What to report

- The five-run hashes before and after.
- Which shape you chose and the argument.
- The gate's runtime over the corpus.
- All three mutation results.
- Scoped nextest Summary lines including `binary_id(wat::lint)`.
- **Anywhere this brief was thin or wrong.** Thirteen riders have run on this arc and every one found
  a real defect in the brief; three times I named a proof set with a hole exactly where the design
  was pointing. Be blunt.

Do not commit.
