# BRIEF — #88 v2: move the rete-defn check to REGISTRATION, and make its refusal a VALUE

> ⛔ **LANDED. Do not execute this brief.** `#88` (registration + value
> refusal) and `#87` (`ReteDefnRecursive` at load) are in the tree.
> `#87` is in `30725034` (local). Live breadcrumb:
> **`CURRENT-STATE-annihilate-interpretation.md`**. The "uncommitted"
> / "floor is red by construction" lines below are historical.

**Supersedes v1's strike path.** v1 built the form, the marker and the membrane — that work is IN
THE TREE, uncommitted, and three of its defects are already corrected. It did not work, for one
located reason, and this brief is that reason and its fix.

**Read first, do NOT re-derive:** `DESIGN-STONE-the-rete-defn.md` — especially the two new sections,
*WHAT THE FIRST STRIKE LEARNED* (four corrections, each proven by a run) and *THE DEPLOYMENT MODEL*
(the builder's ruling, which settles the failure shape).

## The state you inherit — read the tree before you touch it

Uncommitted in the working tree, all building clean (`cargo build --release` exit 0):

| what | where | status |
|---|---|---|
| `ReteContract` + `Function.rete` field | `src/value/environment.rs` | built; 20 construction sites carry `rete: None` |
| the form, parsed pre-expansion and re-headed to plain `defn` | `src/freeze/env.rs` | built — the form flows through the ORDINARY registration path |
| `apply_rete_defn_contracts` (the four-axis check) | `src/rete/purity.rs` | built; reuses the existing walks (STOP-1 held) |
| the membrane, scoped to law A | `src/rete/purity.rs`, `classify_fn`'s `Wat` arm | **corrected** — see stone §1 |
| `seen` seeded with the whole declared group | `src/rete/purity.rs` | **corrected** — see stone §2 |
| `ReteDefnAxisViolation { name, axis, head }` | `src/value/signal.rs` | built, located, span-carrying |
| the acceptance gate | `tests/rete/probe_arc278_rete_defn_gap.{rs,wat.bad}` + `_control.wat` | green, mutation-proven both ways |
| the recorded codemod + its `fix.wat` verb | `wat-scripts/fixes/rehead-rete-callees.wat`, `wat/fix.wat` | built, dry-run + diffed + idempotency-proven |
| 14 re-headed corpus files | `tests/rete/`, `wat-scripts/perf/grid/` | applied; **unproven** until the marker survives |

Floor is **red by construction** and will stay red until this brief lands. That is expected; do not
treat it as a regression to chase.

## The work, in one paragraph

The check stamps `Function.rete` inside `build_env`. Then `FrozenWorld::freeze` calls
`register_runtime_defs` (`freeze.rs:564`) which **re-registers every `defn` and drops the stamp**.
So the file loads and the runtime fence still refuses — the `Function` it reads is unstamped. Move
the check to **registration**, which is the one door both the boot path (`freeze.rs:564`) and the
live-session path (`runtime.rs:24475`) already call; delete the `build_env` call rather than keeping
both. Then make the refusal a **matchable value** rather than a raise, because rule compilation
happens at runtime on forms from a host we do not trust.

## Rooms — read in this order

| room | why |
|---|---|
| `src/freeze/env.rs:277` | step 6.975 — the `apply_rete_defn_contracts` call to **DELETE** |
| `src/freeze.rs:564` | `register_runtime_defs(&program, &env, &mut symbols)` — the boot caller |
| `src/runtime.rs:24475` | `register_runtime_defs(&world.program, env, &mut session_sym)` — the LIVE-SESSION caller. This is the one the wire model needs |
| `src/runtime.rs:2118` | `register_runtime_defs` itself — returns `Result<(), EvalBreak>`, so a value carrier already exists |
| `src/rete/purity.rs` | `apply_rete_defn_contracts` — moves; its body does not change |
| `src/freeze/env.rs` | `extract_rete_defn_names` — the declared-name set must reach the new call site |
| `wat/query.wat:154-157` | `SiftRulesResponse` — **the response convention**: one good result, N named bad ones |

## The strike path

1. **Move the check to registration.** `register_runtime_defs` walks the program's forms; the
   rete-defn names are derivable there the same way `extract_rete_defn_names` derives them now.
   Delete `freeze/env.rs`'s step 6.975 — one door, not two.
2. **Prove the marker survives.** The gate for this is the corpus itself: the 14 already-re-headed
   files must go green. That is the whole point of the move.
3. **Make the refusal a value.** `ReteDefnAxisViolation { name, axis, head }` + span already carries
   the payload. It must become something a caller can `match`, in the shape `SiftRulesResponse`
   already uses — one success, N named failures, each with located structured fields.
   **Scope: the VALUE shape only.** No service, no wire, no transport — those are the chaos engine
   (#7) and #17's contract. You are refusing to author a new raise at an IO boundary, not building
   the boundary.
4. **Re-run the floor and let the checker name the rest.** Law A is transitive; expect a waterfall.
   Add only names the checker names.

## STOP triggers — a rejection, not a permission slot

1. **STOP-1 — reuse the existing walks.** `apply_rete_defn_contracts` calls the four axis walks
   already in `purity.rs`. Do not write a second implementation of any axis.
2. **STOP-2 — the membrane stays scoped to `Axis::RetePrimitive`.** If you find yourself denying an
   undeclared fn on Pure/Deterministic/Total, STOP — that breaks `pure?` for ordinary functions and
   is the exact defect stone §1 records.
3. **STOP-3 — one door.** If the check ends up called from BOTH `build_env` and registration, STOP.
   Two call sites is two implementations of one law.
4. **STOP-4 — the codemod keys on (file, name).** Only files the checker named may move. If a name
   is declared in more than one file, do not assume both move — `:test::big?` is the proven
   counterexample.
5. **STOP-5 — if the session path (`runtime.rs:24475`) cannot carry the check**, STOP and report the
   signature that blocks it. That path is not optional garnish: it is the one the wire model needs,
   and a check that only runs at boot is the defect this brief exists to fix, relocated.

## What "done" means

The floor is green at ≥ 4376 with 0 failed, by the orchestrator's own `--release` re-run; the
acceptance gate still goes red-and-green by mutation; `:wat::rete::pure?` still answers true for an
ordinary pure fn; and a `(:wat::rete::core::defn …)` declared in a live session — not only at boot —
is checked and stamped.

## Known and out of scope

`:w::sum-of-squares` lives in an inline wat string inside
`tests/rete/probe_arc278_8custom_native_differential.rs` — a form-tree codemod cannot reach it.
Hand-edit that one; it is the class-4 surface (2026-07-24's lesson) and the floor is what surfaces it.
