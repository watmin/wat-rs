# AMEND — STONE 255.4: the rename is HALF-APPLIED. 28 reds.

**Orchestrator's verification row, `46cc976a5` (not pushed).**

## ⭐ THE WAT-SIDE WORK IS RIGHT — do not undo it

| row | result |
|---|---|
| clippy `-D warnings --all-targets --workspace` | ✅ **0** |
| the 2 `Bytes/to-hex` regressions | ✅ **CLOSED**, as the brief predicted |
| delta (your tree) | ✅ **84 → 81**, 3 closed, 0 newly broken |
| ⭐ **zero `Type::member` call heads remain in `.wat`** | ✅ verified independently — the codemod did its job |
| ⭐ **the derived set beat my greps** | ✅ `from-hex` and the whole `HandlePool` family were invisible to all three of my counts — **the tenth correction** |
| ⭐ **the ONE DOOR find** | ✅ `wat-macros/src/codegen.rs` `method_wat_path` was *generating* the `::` join. Changing one `format!` unified `rust.cache`, `rust.sqlite` and the shims at once. **That is a better fix than renaming nine names.** |

## ⛔ FLOOR RED — 28 of 5939. The rename stops halfway down the call path.

```
     Summary [ 286.664s] 5939 tests run: 5911 passed (8 slow), 28 failed, 22 skipped
```

**25 of the 28 are `wat_dispatch_*`.** The arm:

```
test expects : callee == ":rust::test::MathUtils/add"
actual       : UnknownCallee — "unknown callee: :rust::test::MathUtils::add"
```

**Three sites, and only two moved:**

| site | spelling | state |
|---|---|---|
| the macro's registration (`codegen.rs method_wat_path`) | `/` | ✅ you changed it |
| the `.wat` call site (`wat_dispatch_193a.wat:7`) | `(:rust::test::MathUtils/add 40 2)` | ✅ already `/` |
| ⛔ **the lookup path that builds the callee key** | **`::`** | **unchanged** |

⇒ **A third site still joins with `::` between the call site and the registry lookup.** The
orchestrator could not find it by grep — the obvious `format!("{}::{}")` shapes are gone — so it is
built some other way (a `push_str`, a `join`, an identity/canonicalisation step, or the surface-member
resolution at `src/check.rs:5560` *"Surface found but `method_name` is not any member"*).
⛔ **You changed the emitter; find the consumer. You know the change — do not take my guess as the
answer.**

## The other 3

| test | cause |
|---|---|
| `no_bootstrap_path_in_committed_rust` | ⛔ **`tests/lint/one_member_join.rs:25` names the gitignored `bootstrap/` dir.** That dir is **empty on a clone**, so the new gate is **vacuous for anyone but you** — `[[feedback_a_dev_box_floor_reads_ignored_instruments]]`. **A wall that only fires on your machine is not a wall.** |
| `wat_scripts_fixes_load` | `wat-scripts/probes/arc-293/s4c-carrier-probe.wat` → `unknown callee: :my::Counter/surface-forms`. ⚠ **A USER type's member** — so the rename reaches beyond the builtin families. Same root as the dispatch reds, or a second one; **say which**. |
| `probe_arc255_ivb2a_examples_seam` | *"seam must return … including `Bytes::to-hex`"* — a reflection seam still expects the **old** spelling. A ledger/seam row that moved with the name. |

## The fold

⛔ **Fold into `46cc976a5`.** Not a repair commit — the stone must be green at its own landing.

1. **Find the third join** and route it through the same door. ⛔ **One door** — if the fix is a
   second `if is_known_type` somewhere, that is the shape this arc exists to delete.
2. **Fix the gate's own defect.** `one_member_join.rs` must not read `bootstrap/`. Derive its corpus
   from tracked files (`git ls-files`), or it passes vacuously on CI and on every clone.
3. **The seam row** — update the reflection expectation to the new spelling, and check whether any
   *other* seam/ledger row names an old member spelling.

⚠ **Re-run the delta after the fold.** 81 should hold or improve; if the third-join fix closes more,
say so — those would be files that were failing for this reason all along.

## What I could not verify, stated

Your SCORE says the derived set came from `wat_intrinsic` (5) + `wat_dispatch` (the one door) +
wat-side `defn` (8). ⛔ **I could not independently re-derive that** — my instruments are the greps
the brief already told you not to trust. **The `:my::Counter/surface-forms` red suggests the set is
still short**, which is why the brief said derive it from the registry and the resolver. **State the
final derived set in the amend SCORE.**
