# BRIEF — conditions compile once; the matcher stops re-deriving a static program

## The work

`alpha_match_inner` re-derives its program from a `WatAST` on **every fact**: it re-classifies clause
shapes that never change, linear-scans field names for an index fixed at compile time, and performs
**two heap allocations** rebuilding the constant binding key `"?l"` — then linear-scans the bindings
comparing those freshly-built Strings. Compile each condition **once**, into a pre-resolved
instruction sequence, at the same setup site the alpha tree is built.

**Read `## ⚠ AMENDED 2026-08-01` in the DESIGN before anything else.** This is **not** a perf stone.
Measured, the entire target is **1.211 ms of a ~115 ms fire (1.1%)**. It is built because
re-deriving a static program per fact is *wrong, not slow*; because ~1M allocations per fire threaten
the jitter-free-tail property this engine claims over Clara; and because the shape is what the
streaming regime will need. **Do not chase a speedup. Prove the mechanism.**

All edits land in `/home/watmin/work/holon/wat-rs/`. Verify with `pwd` first; if the reported path
contains `.claude/worktrees/`, re-anchor there and use `git -C /home/watmin/work/holon/wat-rs`.

## Read in order

1. **`docs/arc/2026/06/278-rules-engine/DESIGN-STONE-compiled-conditions.md`** — the mechanism, the
   contract, the amendment. The contract decision is the whole stone.
2. **`src/rete/matcher.rs:209-237`** — `alpha_match_inner`, the function you are specializing. Note
   the head string-compare at `:223-228`: it is **redundant**, the caller already proved the type.
3. **`src/rete/matcher.rs:285` and `:365`** — `ReteClauseShape` and `classify_rete_clause`. Your
   compiler **consumes** this; it is the single source of "what shape is this form" (arc 294).
4. **`src/rete/matcher.rs:406-446`** — the `Bind` and `Constraint` arms. `:411` is allocation one.
5. **`src/rete/matcher.rs:498-520`** — `resolve_operand`. `:509` is allocation two, same constant.
6. **`src/rete/matcher.rs:710-717`** — `read_fact_field`, the linear name scan per bind.
7. **`src/rete/alpha_tree.rs`** — the shape precedent, landed yesterday: built once at setup from the
   immutable network, consumes `classify_rete_clause`, `Arc` not `Rc`. Mirror its posture.
8. **`src/rete/kernel.rs`, `build_alpha_index` + the P8/P8c setup block** — where `alpha_cond` and the
   tree are built. Your compiled forms are built **there**, once per fire, beside them.
9. **`src/rete/kernel.rs`, `census_count` / `with_count_census`** — the counter instrument your
   load-bearing gate uses.

## ★ THE ONE CONTRACT DECISION

**Slots internally; the public `Arc<[(Value, Value)]>` is materialized ONCE, on SUCCESS ONLY.**

The accumulator becomes a `Vec<Value>` indexed by slot. Only when every op has held does the executor
zip `slot_keys` with the slot values into the canonical bindings array.

The reason is the failure path: **most calls fail — that is what a matcher is for.** Today a failing
call allocates both key Strings before discovering the mismatch. Under this contract a failed match
allocates **nothing**. The public binding representation does not change — `Element` and `Token` see
exactly the array they see today, same keys, same order. The stone is invisible above the matcher.

`alpha_match_inner` is **NOT deleted**. It stays as the reference implementation and the other half
of the differential.

## Blast radius

`src/rete/matcher.rs` (compiler + executor) and `src/rete/kernel.rs` (build at setup, call from step
1). **Nothing under `wat/`** — the oracle does not move.

## STOP triggers (each is a rejection: ship nothing, report the gap)

1. **STOP-1** — if the compiled executor cannot produce a bindings array **identical** to
   `alpha_match_inner`'s (same pairs, same order, same values), STOP and report the divergence. Do
   not relax the comparison to "both matched."
2. **STOP-2** — if `classify_rete_clause` cannot be reused and a second condition parser would be
   needed, STOP. That re-opens the drift hole arc 294 extracted it to close.
3. **STOP-3** — if the failure path cannot be made allocation-free, STOP and report **where** the
   allocation is forced. That is the stone's whole mechanism; a version that still allocates on
   failure is not it.
4. **STOP-4** — if the work appears to need an edit under `wat/`, or a change to the public binding
   representation, STOP and report what forced it.

## Definition of done

- **A differential on the BINDINGS ARRAY** over (condition, fact) pairs drawn from the live grid
  axes: `compiled(cond, fact)` equals `alpha_match_inner(cond, fact)` including the exact array.
- **A counter test**: allocations on the failure path are **zero**, asserted via `census_count`, and
  non-zero when measured against the interpreter — so the row cannot pass vacuously.
- `a0_depth_cost_split_at_equal_work` and `fanout_per_call_alpha_census` re-run: report both tables.
  These are **no-harm** rows. Do not tune toward them.
- `cargo nextest run --release` — read the Summary line, never a piped exit code.
- **`cargo clippy --release --all-targets --workspace`** — the deny wall. Capture its **own** exit
  code; do not infer it from a grep count.
- Report both tables, both new tests' output, and `git diff --stat`.

Leave the tree dirty and uncommitted. Do not commit, push, or stash.

## A prior result to copy for shape

`src/rete/alpha_tree.rs` (commit `e3e97fb7`) is the same posture one layer over: a new module
consumed by `kernel.rs`, built once at setup, oracle untouched, gated by an invariant test with a
bypassed comparison so it cannot pass vacuously.
