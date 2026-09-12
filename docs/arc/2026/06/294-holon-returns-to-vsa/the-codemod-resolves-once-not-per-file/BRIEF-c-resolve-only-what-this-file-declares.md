# BRIEF — (c): resolve a LOCAL enum path only if THIS FILE could declare it

Anchor: `/home/john/work/holon/wat-rs`. Verify with `pwd`. **Never use worktrees.**
Do not touch `~/work/holon/` (the frozen root).

## Tree state

```
branch   merge/grok-rete @ 9479de65b   clean. A′ is COMMITTED (bbcb70e38) — it is your base.
binary   bootstrap/wat-main-a3218644d  ⛔ USE THIS. ./target/release/wat cannot boot on this branch.
gate     bootstrap/pctm-gate/{orig,A,B}  A = OLD's output on 20 changers; B currently = A′'s output.
```

## Why — measured on representative phase-1 files (spread sample of bootstrap/phase1-grok.txt)

```
A′  n=1    24.96s   files changed=0   corpus-invariant=29 per-file=6
A′  n=5   123.12s   files changed=0   corpus-invariant=93 per-file=92      ~25 s/file FLAT
```

**Five of five sampled files change NOTHING and still cost ~25s each.** `fmap-for-src` (`:403-416`)
runs EVERY local path from `enum-paths-of` (`:302` — every `::` keyword, plus its parent, plus any
alias; that includes function-call heads and type annotations) through `fill-paths` →
`try-type-of` (`:165`). Each is an `eval-with-defs!`; a failure retries up to twice more. Almost all of
them fail — they are learning, at ~0.66 s each, that `:u::helper` is not an enum.

## The rule — and why it is EXACT, not merely conservative

Keep `ctor-edits` (`:689`) as the ONE place a call becomes a candidate site — it decides by asking the
map (`fields-for-head`, `:712`); kwargs calls are already rejected by its arity check (`:725`).
**Do not add a syntactic positional/kwargs classifier.** (c) only prunes the resolution INPUT.

In `fmap-for-src`, between `local (:user::local-eps-of epaths)` and `fill-paths`, **keep a local path
only if THIS FILE could declare it:**

```
the file has a MACRO-SHAPED decl  (defservice · sift-rules-defsvc · defmacro)
    → keep ALL local paths for this file, exactly as today            (when unsure, resolve)
otherwise
    → keep a local path iff it equals, or is prefixed by (on a `::` or `.` boundary), the NAME
      (child 1) of one of the file's plain type decls:
      defenum · defrecord · defstruct · defsurface · newtype · typealias · typeunion
```

**Why this skips only lookups that were going to fail** — cite, do not re-derive:

1. `try-type-of` evaluates against THIS FILE's `decls` (`file-decls`, `:71`, top-level
   `decl-head?` forms only). A local path resolves only if those decls define it.
2. Its retries only SHRINK the set — `without-defservice`, then `simple-type-decls`, then EMPTY.
   A path the full set cannot define, no subset can.
3. The EMPTY retry lets the stdlib answer alone. **Measured on main's stdlib (`a3218644d:wat/`, the
   one frozen into the bootstrap binary): 325 declaration forms, 301 under `:wat::`/`:rust::`, and
   the other 24 are ALL `~placeholder` names inside macro templates.** The stdlib declares no real
   type outside the reserved prefixes, so the empty retry cannot resolve a LOCAL path.
4. A plain type decl defines types only under its own name. **A macro-shaped decl does not** — its
   generated types take names from its ARGUMENTS (`~req-kw`, `~resp-kw`, `~state-ty-decl` in
   `wat/query.wat` / `wat/service.wat`). Hence the file-level fallback above.

### ⛔ Order is load-bearing — FILTER, never reorder

`index-leaf` (`:250`) is FIRST-WINS on a leaf collision: `(if (names-eq? existing fields) m m)` —
both arms return `m`. Skipped paths resolve to None and add no entry, so removing them preserves the
insertion order of every path that succeeds. **Reordering would change which enum owns a shared
leaf, and therefore the output.** (That identical-arms `if` is a latent defect of its own. Out of
scope — do not touch it. Same for `pascal-leaf?` at `:717`, which is case-based logic.)

## ⛔ STOP triggers — each is a REJECTION. Report and halt.

**STOP-1** — never skip a local path in a file that has a macro-shaped decl.
**STOP-2** — never reorder the kept paths; filter in place.
**STOP-3** — the tools are never their own input: no `wat-scripts/fixes/` path in any worklist.
**STOP-4** — no new intrinsic; the codemod runs on the frozen bootstrap binary.
**STOP-5** — ANY output that differs from its oracle, on any file. Report with the diff.
**STOP-6** — if the AUDIT (row 2) finds even ONE skipped path that resolves to Some. That is the
exactness claim failing; it is a finding, not a tuning problem.

## Acceptance — in this order

```
0. ps -eo pid,etime,cmd | grep wat | grep -v grep  ->  NOTHING, before and after every run.
   Never two runs at once. TIME ONE FILE FIRST.

1. DIFFERENTIAL, byte-identical to the oracle:
     bootstrap/pctm-gate/orig -> (c)  ==  bootstrap/pctm-gate/A              (20 changers)
     the 5 sampled no-change files     ==  their HEAD content               (see paths below)
     accum.wat + retract-multiplicity.wat in ONE worklist == OLD, no halt

2. AUDIT — the exactness proof. An audit mode (a flag, or a one-off variant you delete after) that
   resolves EVERY local path as today AND computes the skip set, then asserts every skipped path
   resolved to None. Run it on the 20 gate files + the 5 samples. Report:
     local paths total · kept · skipped · skipped-that-resolved-to-Some (MUST be 0)

3. The tool prints its own counts per run:  kept-local=N skipped-local=M  (beside the A′ line)

4. TIMED on the SAME files, A′ vs (c), one run at a time: n=1, n=5 (the 5 samples), n=20 (gate).

5. bootstrap/wat-main-a3218644d --check wat-scripts/fixes/positional-ctor-to-map.wat  -> exit 0
```

The 5 sampled no-change files (row 1 and row 4):
```
docs/arc/2026/06/278-rules-engine/harness-experiri/experiri-acc-head.wat
tests/rete/probe_arc278_export.wat
tests/rete/probe_arc300_2_fix_defrule.wat
wat-scripts/perf/grid/where-not-fact.wat
wat-scripts/scratch-pad/probe-overlay-refire-cost.wat
```

⚠ Row 2 before row 4. A faster codemod whose skip set includes one real type is worse than a slow one.
⛔ **Do NOT run the full corpus** — the orchestrator runs phase 1 with `bootstrap/run-chain.sh`.

## Tier

You edit `wat-scripts/fixes/positional-ctor-to-map.wat` and you REPORT. **Do NOT commit. Do NOT run
`scripts/floor.sh` or clippy.** Do NOT call `pulsare_yield` until rows 0–5 are done and tabled.
