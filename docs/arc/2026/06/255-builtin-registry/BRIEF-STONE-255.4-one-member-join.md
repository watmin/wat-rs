# BRIEF — STONE 255.4: ONE member join. `Type/member`, always.

**Drawn 2026-09-21 against `main` @ `db40a06db`** (floor 5937/5937, clippy 0, census `no STOP-8`).
Predecessor: **255.3 LANDED — delta 97 → 80, net −17**, with **2 regressions** this stone removes.

## THE BUILDER'S RULING

Weighed against the four questions: **Option 2 scored 4-YES, Option 1 scored 0-YES.**

> *"i agree with Option 2"*

**Option 2, as ruled:** *unify the registry on `/`. A member join is one thing.*

## ⛔ WHY — 255.3 proved the door cannot be made correct without this

`types::reconstruct_call_path` asks the registry *"is the namespace's last segment a type?"* and
joins with `/` when it is. That is right for **4,501** members and wrong for a legacy minority,
because **the member's join is a SECOND registry fact** the door has no access to
(`reconstruct_call_path` holds `TypeEnv`, not the function table).

`src/types.rs:150` already records the impasse **in the code**:

> *"Last-segment-is-a-type cannot tell `:wat::core::Option/expect` (slash, 82 names) from
> `:wat::core::Bytes::to-hex` (`::`, 5 names) — both are `Type/member` to the joiner and different
> keys in the registry."*

⭐ **This stone deletes the sentence rather than teaching the door to work around it.** No-form rung
over check rung.

⭐ **And the precedent is already in the tree:** `src/remedy/retirement.rs:121` retired
`:wat::core::option::expect` **in favour of** `:wat::core::Option/expect`. Unifying a member spelling
toward `/` is something this codebase has done before, with a ledger entry.

## ⛔ SCOPE — DERIVE IT. The orchestrator's counts DISAGREE WITH EACH OTHER BY POSITION.

**Honest disclosure of three different numbers, all the orchestrator's:**

| measurement | count |
|---|---|
| all `::`-joined `Type::member` occurrences, comments stripped | **83** |
| distinct names in that set | **11** |
| ⭐ **live CALL-HEAD sites** (`(` immediately before) | **63 sites · 9 names · 11 files** |

⛔ **The 11-vs-9 gap is real and instructive:** `:wat::Record::def` and `:wat::holon::Record::def`
are **already RETIRED** (arc 293.2 → `defrecord`). Their `.wat` occurrences are **retirement-ledger
rows, not call sites.** **Do not rename a retired name.**

⚠ **And a discrepancy the orchestrator could NOT resolve:** `Bytes::to-hex` shows **1** live call
head, yet **2** files in the 255.3 delta fail on `:wat::core::Bytes/to-hex`. So it is reached by
some route the head-position grep does not see (value position, reflection, a macro). ⛔ **Derive the
set from the registry and the resolver, not from the orchestrator's greps.** All three numbers above
are `grep`-shaped and this campaign has been bitten by that repeatedly
(`[[feedback_ask_the_tool_that_owns_the_fact]]`).

**The live families, as far as grep sees:** `wat.cache` — `Lru::{put,get,new,len}` and
`HolographicLru::{put,get,new,len}` (**62 of 63 sites**) · `wat.core::Bytes::to-hex` (1).

## The work

1. **Rename the registrations** `Type::member` → `Type/member`. Rust sites include
   `src/rust_deps/cache.rs` (the cache family), `src/intrinsic/bytes.rs`, and references in
   `types.rs` / `check.rs` / `runtime.rs` / `load/stdlib.rs`. ⛔ **Derive the list; do not paste
   this one.**
2. ⭐ **A RETIREMENT LEDGER ENTRY for each**, following `remedy/retirement.rs`'s existing shape —
   the `option::expect` row is the model. **An old spelling must teach its replacement, not merely
   fail.**
3. ⛔ **THE CORPUS MOVES BY A RECORDED CODEMOD.** This is the arc's **first `.wat` write**, and the
   builder's standing constraint governs it: *"we have many more large scale branch merges to
   contend with… we'll need to use these tools to fix them."* A hand edit helps this tree once and a
   future merge never. Dry-run on `/tmp` copies, `diff`, prove idempotence.
4. **Delete the impasse.** `types.rs:150-152`'s caveat and any code that exists to tolerate the
   second join.

## The gate

- ⭐ **THE DELTA (baseline 80, orchestrator's tree) + the classification table.** ⛔ **Measure
  before-and-after on ONE tree and name it** — 255.3 got this right; keep it.
  **Expect the 2 `Bytes/to-hex` regressions to CLOSE.** If they do not, that is the finding.
- **ZERO `::`-joined `Type::member` call heads remain** — and a **gate** that keeps it so. ⛔ A
  rename without a wall grows back (`[[feedback_a_lesson_learned_and_then_dropped]]`).
- `scripts/floor.sh` green; clippy `-D warnings --all-targets --workspace` **0**;
  `census.sh --diff` → `no STOP-8`. Run crate clippy **and the lint suite** yourself first.
- Predict the test delta; confirm with `cargo nextest list`.
- ⚠ **`.wat` files DO move in this stone** — by the codemod, under `git status`, all in the 11-file
  set (or whatever the derived set is; state the difference). ⛔ **No 8d conversion** — this is a
  member-join rename *within the current dialect*, not the `::`→clojure flip.

## Doctrine

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Capture whole the first time; never re-run first.
- ⛔ **R21 — the corpus moves BY THE TOOL.** An unreachable site is a STOP and a report.
- ⛔ **`wat/*.wat` is `include_str!`'d** — an on-disk edit is invisible until `cargo build --release`.
- ⛔ **REPORT WHAT A DOOR CANNOT EXPRESS.** Three stones running, that has been this arc's best
  output. Do it again.
- ⛔ **If this brief contradicts the code or the runner, THEY WIN.** **Nine corrections across seven
  stones**, and this brief openly carries three mutually inconsistent counts. **Assume a tenth.**
- Work only in `/home/john/work/holon/wat-rs`. **Do not push.**

## Out of scope

- **The position grammar** (`:wat::WatAST` 79 + declaration names, ~47 remaining UR). **That is
  255.5** — keeping it separate is what keeps the delta attributable.
- **8d-ii / 8d-iii**, the `wat.type` NAME SET, capacity `:panic` — all deferred by the builder.
