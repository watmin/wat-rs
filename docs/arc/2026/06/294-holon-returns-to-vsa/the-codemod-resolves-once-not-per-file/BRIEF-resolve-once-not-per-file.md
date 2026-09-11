# BRIEF — `positional-ctor-to-map` resolves ONCE, not per file

Anchor: `/home/john/work/holon/wat-rs`. Verify with `pwd`. **Never use worktrees.**
Do not touch `~/work/holon/` (the frozen root).

## Tree state

```
branch  merge/grok-rete      HEAD db60b092a   CLEAN, builds (--all-targets), NOTHING running
main    a3218644d            frozen, pushed, floor 5373/5373 green — do not touch
```

⛔ **The corpus is NOT migrated.** The merge is in, the migration chain is not. That is deliberate:
the chain cannot be run until this stone lands. **You are not migrating anything.**

## The defect — MEASURED, not estimated

`wat-scripts/fixes/positional-ctor-to-map.wat` takes **~15 seconds PER FILE**. Over the 2,176-file
corpus that is **~9 hours for this one codemod**. A run was killed at 4h08m, unfinished.

```
positional-ctor-to-map     1 file    14.34s
positional-ctor-to-map     5 files   75.91s     15.2   s/file   (LINEAR — constant, not blowup)
variant-separator-to-dot   5 files    0.27s      0.054 s/file   <- sibling codemod, 280x faster
wat --check  (Rust path)   1 file     0.22s
```

★ **IT IS NOT THE INTERPRETER.** Two interpreted wat codemods, same files, 280× apart — and the
fast one beats the Rust `--check` path. Do not "optimize wat". One codemod has the wrong shape.

### Where the time goes

`main` builds `base` ONCE (`stdlib-fmap`, :821) — that part is correct and stays. Then, **per file**:

```wat
fmap (:user::fmap-for-src src base)          ;; :798, once per file
  → read-string src
  → collect-keywords tree                     ;; conj-unique — O(n²) in keyword count
  → enum-paths-of vpaths
  → fill-paths base epaths decls              ;; foldl over epaths:
      → (:user::try-type-of ep decls)         ;; :165
          → (:wat::eval-with-defs! … (type-of-form ep) decls)     ⛔ A FULL EVAL, per candidate
          → on failure: retry without-defservice, then simple-type-decls   ⛔ 2–3 MORE evals
```

**The `#seen#` memo lives in the per-file accumulator and is discarded when the file is done**
(`fmap-for-src` restarts from `base` every time). So `:wat::core::Option` is re-evaluated in every
one of the 2,176 files that mentions it.

## The strike — ONE resolution pass, then migrate

**Build the field map ONCE for the whole run**, from the union of all input files' declarations, then
migrate every file against that frozen map.

Shape (yours to land; this is the content, not the spelling):

```
main
  ├── base  = stdlib-fmap                       (unchanged, already once)
  ├── PASS 1 — resolve: for each path, read + collect decls/epaths; resolve each UNIQUE
  │            ep ONCE across the whole run; fold into one frozen fmap
  └── PASS 2 — migrate: for each path, `migrate src frozen-fmap path`   (no resolution here)
```

★ **THIS WINS ON HONEST, NOT ON SPEED.** Today the same `ep` resolves against *whichever file you
are in*, so one name can get different answers in different files. One pass makes resolution a
property of the CORPUS, not of iteration order. Speed is the side effect.

## ⛔ STOP triggers — each is a REJECTION. Report and halt.

**STOP-1 — a name that resolves DIFFERENTLY in two files.** If the union pass finds one `ep` with
two answers, that is a FINDING for the builder, not something to resolve by picking one. Name the
`ep` and both files. (It is also the proof the per-file shape was hiding something.)

**STOP-2 — do NOT memoize per-file resolution across files as a shortcut.** That was option A and it
is DISQUALIFIED on Honest: a cached answer from file A can be wrong for file B, silently. If you
find yourself threading a cache through `rewrite-each` instead of building one map up front, stop.

**STOP-3 — do NOT run the chain, and do NOT run over `wat-scripts/fixes/`.** The codemods are wat
programs whose DATA is head names; `fmt-head-fqdn-to-clojure` rewrites string literals inside them
and silently disarmed this very codemod's unquote predicate earlier today
(`":wat::core::unquote"` → `"wat.core/unquote"`, after which it emitted `{:pred ~}src` and broke
`wat/query.wat`). **The tools are never their own input.**

**STOP-4 — if the output CHANGES for any file.** See acceptance row 1. This stone is a speed and
honesty fix; a different migration result is a defect unless it is a STOP-1 finding you have named.

## Acceptance

```
1. ⛔ DIFFERENTIAL, and it is the stone:
     pick 20 corpus files that the codemod actually CHANGES today.
     old binary+codemod -> output A     (keep the files)
     new codemod        -> output B
     A and B must be BYTE-IDENTICAL. Any delta is a finding, reported, not absorbed.
2. TIMED, on the same 20 files, both versions, reported as a table:
     today ~15 s/file. Target: the resolution pass is O(unique eps), not O(files x eps).
     State the measured s/file after. A 2x win is not this stone; state what you got.
3. the codemod still `--check`s clean
4. cargo build --release   BUILD_EXIT=0    (you changed a .wat, not Rust — this is a smoke check)
```

⚠ Row 1 before row 2. A faster codemod that migrates differently is worse than a slow one.

⛔ **Do NOT run the full corpus.** 20 files is the gate. The orchestrator runs the chain afterward.

## Tier

You edit `wat-scripts/fixes/positional-ctor-to-map.wat` and you REPORT. **Do NOT commit. Do NOT run
`scripts/floor.sh` or clippy.** The orchestrator measures centrally.

⛔ Do NOT call `pulsare_yield` until the differential in row 1 is done and tabled.
