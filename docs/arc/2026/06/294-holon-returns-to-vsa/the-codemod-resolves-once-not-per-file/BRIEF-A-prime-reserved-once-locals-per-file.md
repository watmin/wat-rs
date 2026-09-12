# BRIEF — A′: reserved type names resolve ONCE; everything else resolves PER FILE

Anchor: `/home/john/work/holon/wat-rs`. Verify with `pwd`. **Never use worktrees.**
Do not touch `~/work/holon/` (the frozen root).

## Tree state — read all four lines

```
branch   merge/grok-rete     your B WIP (positional-ctor-to-map.wat, uncommitted) is your BASE
main     a3218644d           frozen, pushed, green — do not touch
binary   bootstrap/wat-main-a3218644d    ⛔ USE THIS ONE. See .gitignore's bootstrap/ comment.
target   ./target/release/wat            ⛔ CANNOT BOOT on this branch (merged stdlib fails its
                                          own purity gate). Do not use it; do not rebuild hoping.
```

## The ruling — A′ (builder, 2026-09-11)

Your option-B strike was correct on its own terms — A == B, 20/20, re-diffed independently by the
orchestrator. **Its premise was the orchestrator's, and it was wrong.** B called "one name, two
answers in two files" a hidden defect and made STOP-1 a runtime assertion. The corpus refutes it:

```
21 independent programs declare :grid::Result  (each its own main, no load-file!)
18 × axis size derived native-ns oracle-derived oracle-ns
 3 × the same six + insert-ns fire-ns query-ns protocol-ns    (accum, deep-cascade, fanout)
```

Both shapes are LEGAL. Per-file resolution was the right model for local names all along. So B, as
written, **halts on any full-corpus run** the moment `accum.wat` meets `retract-multiplicity.wat`.

**A′ keeps what was right in both:**

```
names under :wat:: / :rust::   resolve ONCE, cache across ALL files   (corpus-invariant)
every other name               resolve PER FILE, against its own decls (as OLD always did)
```

### Why the cache is HONEST by construction — cite this, do not re-derive it

`src/resolve/registration.rs:165` (`gate`) checks `Existing::Equivalent → NoOp` BEFORE `Reserved`
(pinned by the tests at `:317-319`). So a user re-declaration of a `:wat::`/`:rust::` name is either
**equivalent** (a no-op — same answer) or **refused** (`ReservedPrefix`). It can never be a second
answer. That is what makes cross-file caching of those names safe, and nothing else is.

### ⛔ The cache key is NOT `is_reserved_prefix`

`RESERVED_PREFIXES` (`src/resolve/reserved.rs:14`) has THREE entries: `:wat::`, `:rust::`, and
**`:$bound::`**. The third is reserved for a DIFFERENT reason — binder unforgeability; every local
binder carries it — and its members are **per-scope**, not corpus-invariant. Caching a `$bound` name
across files is exactly the dishonesty option A was disqualified for.

So the codemod's predicate is **not** "is reserved". Name it for what it asserts —
`corpus-invariant-name?` or similar — cover `:wat::` and `:rust::` ONLY, and carry a comment citing
`RESERVED_PREFIXES` and stating why `:$bound::` is excluded. A deliberate, documented divergence
from the Rust list is honest; a silent copy of it is a second implementation that happens to disagree.

## ⛔ STOP triggers — each is a REJECTION. Report and halt.

**STOP-1 — never cache a non-`:wat::`/`:rust::` name across files.** Not as a speed shortcut, not
behind a flag. Locals resolve per file.

**STOP-2 — the tool must NOT halt on a local shape collision.** Remove B's STOP-1 assertion from the
codemod. Two programs defining one local name differently is legal input, not an error.

**STOP-3 — the tools are never their own input.** No path under `wat-scripts/fixes/` in any
worklist. (`fmt-head-fqdn-to-clojure` rewrote a string literal inside THIS codemod today when that
rule was broken.)

**STOP-4 — if you need a NEW intrinsic, stop.** The codemod runs on the frozen bootstrap binary. A
verb you add to `src/` is not in it. Everything A′ needs must already exist in main's binary.

**STOP-5 — any output that differs from OLD on any gate file.** Report it with the diff. Do not absorb it.

## Acceptance — in this order

```
0. ⛔ BEFORE ANY RUN, and AFTER EVERY RUN:  ps -eo pid,etime,cmd | grep wat | grep -v grep
   must show NOTHING. Never start a second run while one is live (this corrupted a corpus today).
   TIME ONE FILE FIRST, then five. Two points give per-file cost and linearity.

1. DIFFERENTIAL on the preserved gate — bootstrap/pctm-gate/{orig,A,B}:
     copy orig -> run NEW -> must be BYTE-IDENTICAL to A, all 20 files.
   PLUS the collision pair: accum.wat and retract-multiplicity.wat in ONE worklist.
     OLD (per file) is the oracle for local names -> expected output.
     NEW must NOT halt, and must be BYTE-IDENTICAL to OLD on both.

2. TIMED on the SAME input set, both versions. Your last table compared 15 changers + 5 no-ops
   (OLD) against 20 changers (NEW); that ratio is not a measurement. Same files, both columns.

3. The codemod PRINTS the split of unique enum paths: corpus-invariant vs per-file.
   This is the unmeasured axis A′'s speed rests on. The orchestrator's grep estimate failed its
   own calibration (67 against your printed 617), so only the tool's own count is admissible.

4. Drive the gate once on bootstrap/wat-main-a3218644d: a tiny user file declaring a NEW
   :wat::-prefixed type must be REFUSED with ReservedPrefix. That is the premise, proven on
   the binary that will actually run.

5. bootstrap/wat-main-a3218644d --check wat-scripts/fixes/positional-ctor-to-map.wat  -> exit 0
```

⚠ Row 1 before row 2. A faster codemod that migrates differently is worse than a slow one.
⛔ **Do NOT run the full corpus.** The gate plus the pair is the whole test.

## Tier

You edit `wat-scripts/fixes/positional-ctor-to-map.wat` and you REPORT. **Do NOT commit. Do NOT run
`scripts/floor.sh` or clippy.** Do NOT call `pulsare_yield` until rows 0–5 are done and tabled.
