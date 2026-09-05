# SCORE — STONE: a rule owns ONE node's children

No commit. Floor and clippy left to the orchestrator. R11 is still all-or-nothing (STOP-1). No `BlankBefore` (STOP-2).

## Both horns, after

**HORN A** (`claim-demo.wat`) — binders stay name-with-value. `IDEMPOTENT=true`.

```
(:wat::core::let
  [n (:wat::core::length xs)
   m (:wat::core::first xs)]
  (:wat::core::do (:wat::kernel::println "a") (:wat::kernel::println "b") n))
```

The binding vector is claimed by its own rule. R11 does not re-break it.

**HORN B** (`unruled-inside-defn.wat`) — the half-broken `do` breaks inside the `defn`. `IDEMPOTENT=true`.

```
(:wat::core::defn :fix::u
  [x <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::do
    (:wat::kernel::println "a")
    (:wat::kernel::println "b")
    (:wat::core::+ x 1)))
```

`ClaimedUnder` is gone, so the default reaches unruled forms at any depth.

The one-line `do` in claim-demo stays packed: R11 is still all-or-nothing, and that form's children share a line. Not a miss — STOP-1.

## The wall, shown firing

Throwaway rule: `Break` on child 1 of any `+` (a grandchild of `defn`, parent unclaimed). On `defn-multi.wat`:

```
fmt: rule positioned a grandchild — node 15's parent is unclaimed
```

Then the rule was deleted.

## Finding — R11 cannot assert `Claim`

The DESIGN's wall is "parent of a broken node must be claimed." R11's parents are unclaimed by construction. Asserting `Claim` from R11 is `not Claim -> Claim` and it races the per-child Breaks (first child claims, the rest never fire).

So R11 asserts `Fallback {node}` on the unruled parent, then Breaks its children. The wall accepts **Claim or Fallback** as ownership. Same site (`apply-break`), same raise, not a subtree closure. `ClaimedUnder` is not back.

## The split

`let-binder-per-line` → `let-bindings.wat` (claims the vector).
`defn-arg-per-line` → `defn-args.wat` (claims the vector).
Each parent file `load-file!`s its child so drivers did not need edits.

## Walls

```
grep -c ClaimedUnder wat/fmt.wat wat-scripts/fmt/rules/*.wat  →  0
grep -c 'col'        wat-scripts/fmt/rules/*.wat              →  0 0 0 0 0 0
```

## Commands

| command | result |
|---|---|
| `cargo build --release` | clean |
| `run-all.wat` on `claim-demo.wat` | binders packed, **IDEMPOTENT=true** |
| `run-all.wat` on `unruled-inside-defn.wat` | `do` breaks, **IDEMPOTENT=true** |
| `run.wat` on `defn-multi.wat` / `defn-empty.wat` | ruled, idempotent |
| `run-let.wat` on `let-two.wat` | ruled, idempotent |
| `run-r4.wat` on `half-broken.wat` | ruled, idempotent |
| `run-all.wat` on `all-four.wat` / `unruled-top.wat` | ruled, idempotent |
| `run.wat` on `wat/io.wat` | **COMMENTS=28**, IDEMPOTENT=true |
| wall sabotage on `defn-multi.wat` | **raises**, names the node |
| `every_wat_scripts_file_loads` | **1 passed** |

---

## ORCHESTRATOR VERDICT — 2026-09-05, weighed against my own re-run

**ACCEPTED. No edit.** Second strike in a row that needed none.

| what | result |
|---|---|
| ★ **HORN A** (`claim-demo.wat`), my run | binders keep name-with-value; **`IDEMPOTENT=true`** |
| ★★ **HORN B** (`unruled-inside-defn.wat`), my run | the `do` **BREAKS** inside the `defn`; **`IDEMPOTENT=true`** |
| the wall, shown firing | `fmt: rule positioned a grandchild — node 15's parent is unclaimed` |
| `grep -c ClaimedUnder` (fmt.wat + all rules) | **0** |
| `grep -c 'col'` (all rules) | **0 0 0 0 0 0** — last stone's wall holds through the split |
| floor | **5179 run, 5179 passed, 0 FAILED, 18 skipped** |
| clippy `--all-targets -D warnings` | **0** |

**Both rows 2 and 3 pass.** That was the load-bearing pair — either alone is achievable by picking a
gate, which is the entire dilemma; only the ownership ruling gets both.

### The `Fallback` finding is correct, and it does NOT smuggle the subtree closure back

The DESIGN said *"the parent of a broken node must be CLAIMED"*, assuming R11 could claim. It cannot:
`not Claim -> Claim` is a cycle, and it would also race its own per-child Breaks (the first child
claims, the rest never fire). So R11 asserts `Fallback {node}` and the wall accepts **Claim OR
Fallback** — "this parent is owned by some rule."

★ **That is the DESIGN's intent, expressed correctly, and it is strictly better than what I wrote.**
It is one flat fact, not a transitive closure — verified: `ClaimedUnder` is 0 everywhere.

### ⚠ RESIDUAL, NAMED NOT FIXED — the ownership sets absorb duplicates silently

```
breaks-map   (:wat::hashmap::assoc m (Break/id b) (Break/kind b))    ← last writer wins
claims-set   (:wat::hashmap::assoc m (Claim/form c) true)            ← a set; a double claim vanishes
```

Two rules asserting **different** `Break.kind` for one node, or two rules **claiming** one node, are
resolved by the map rather than detected.

**Not reachable today**, and the reason matters: R11 fires only under `not Claim`, and stratified
evaluation completes every `Claim` before the negation is evaluated — so a parent is either Claimed
or Fallback'd, never both. (Reasoned from the engine's stratification, and corroborated by horn A
staying idempotent, which is exactly R11 declining to fire on a claimed vector. Not directly
instrumented.)

⛔ **But the DESIGN's exclusivity argument is an unenforced convention.** It says *"two rules for
`defn` cannot both exist"* — nothing makes that true. A second `defn` rule file would double-claim
in silence. **That is the next wall and it is cheap:** a node claimed twice should raise, at the same
site the grandchild wall already lives.

### Not disputed

STOP-1 and STOP-2 held — R11 is still all-or-nothing, no `BlankBefore`. The split is clean
(`let-bindings.wat`, `defn-args.wat`, each claiming its vector, each `load-file!`d by its parent so
no driver changed). Every prior fixture keeps its ruled shape and is idempotent. `wat/io.wat`:
**COMMENTS=28**. The one-line `do` in `claim-demo` staying packed is STOP-1, not a miss — R11 is
all-or-nothing until the next stone makes it always-break.
