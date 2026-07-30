# EXPECTATIONS — vacate `SpawnOutcome` (written BEFORE the strike)

Independent scorecard. Fixed before the rider launches so the result cannot move the goalposts.
Every row is re-run by the orchestrator; the rider's report is a hypothesis until then.

**Pre-strike state, measured by my own runs at HEAD `f464223e` (tree clean):**

- floor — `cargo nextest run --release` → `Summary [152.805s] 4194 tests run: 4194 passed, 262 skipped`
- clippy — `cargo clippy --release --workspace --all-targets` → **1 warning** (`large_enum_variant`,
  `src/value/value.rs:1102`, `SpawnOutcome::Panic` 264 B vs `RuntimeErr` 56 B)
- clippy — `cargo clippy --release --workspace` (the form CI actually runs) → **1 warning**

## The scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | the build is clean | `cargo build --release --all-targets` | exit 0, **zero warnings** |
| 2 | the chain is gone | `grep -rn "SpawnOutcome\|ProgramHandleInner\|wat__kernel__ProgramHandle" --include="*.rs" src/` | empty, **or** only retirement-tombstone prose |
| 3 | **the clippy floor** | `cargo clippy --release --workspace --all-targets` | **0 warnings** (from 1) |
| 4 | **the floor is undisturbed** | `cargo nextest run --release` | **4194 passed / 262 skipped — byte-identical** |
| 5 | content integrity | `git diff --stat` | only the 7 files in the brief's blast radius + the one `.wat` comment |

**Row 4 is the load-bearing one.** This is a deletion, so the proof is not that something new goes
green — it is that **nothing moved**. Same shape as 24t's type annihilation (*"the delta is the proof:
`passed` held BYTE-IDENTICAL"*). A drop in `passed` means something's subject died with the chain, and
that is mine to disposition, not the rider's.

## What would have to break for this gate to go red (R59 — a pass that cannot fail proves nothing)

The gate genuinely depends on the mechanism, and here is how:

- **If anything actually constructs the chain**, the build fails at the construction site. Loud, located.
- **If a test's subject is one of these three types**, it fails to compile. Loud, located, and STOP-4.
- **If a `Value` match arm is missed**, non-exhaustive-match is a compile error. The compiler *is* the
  worklist here (R52 — the corrected law lights every violator).
- **If `passed` changes**, something working was disturbed.

None of these can pass silently. That is why the scorecard is a build + a count rather than a new
assertion: there is no new behaviour to assert, and inventing one would be the vacuous gate R59 names.

## Runtime prediction

**15–25 min** (sonnet). One cohesive mechanical deletion across 7 files; the compiler enumerates the
cascade. Wakeup cap at **2× the upper bound = 50 min**; overrun is itself data.

## Trap-doors — named before, not explained after

1. **The key-eligibility table** (`value_key_eligibility_table!`, `value.rs:1297`) carries a
   `ProgramHandle` row at `:1447`. It is macro-generated over `Value`'s variants, so removing the
   variant forces the row. Watch that the `ChildHandle` row above it — which holds the real
   interior-mutability reasoning and the pdeathsig/lifeline custody — survives untouched.
2. **`tests/diagnostics/probe_arc296_remediation_collapse.rs`** hand-builds a `TypeMismatch` carrying
   the *string* `":wat::kernel::ProgramHandle<:wat::core::String>"` to trigger an arc-114 shape
   remediation. A string literal will **not** break the build and the test will keep passing — but
   24t recorded `shape_remedies` dying with the arc-114 tombstones, so this may already be a green
   test whose subject is gone. **Explicitly out of scope for this strike; a post-strike check**, and
   the 24s "a fixture silently stopped being a fixture" shape.
3. **`runtime.rs:21953`** is an arc-114 tombstone, not stale prose. If it comes back deleted rather
   than extended, that is a content-integrity miss — history is kept, lies are rewritten.

## Then

Clippy at zero unblocks **stone E** — arm `-- -D warnings`, and widen the CI step from
`cargo clippy --release --workspace` to include `--all-targets`, so the wall covers the ground the
1874 → 0 campaign was actually measured on.
