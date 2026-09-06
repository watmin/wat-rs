# RELAND — STONE: three head spellings, one seam

**The seam itself is right and I would accept it as designed.** One function, three arms, folding to
the clojure target; rules compare `wat.core/defn`; row 6's deletion demonstrated with the rules hash
unchanged. That is exactly the builder's requirement, built the way that makes the end of the
migration a deletion rather than a rewrite.

## ⛔ BUT `Named.name` IS CANONICAL FOR EVERY NODE, AND ONLY THE FMT RULES WERE MIGRATED

The SCORE's own last delta names it and then under-scopes it:

> *"Grep rules in `wat-scripts/grep/` that still compare FQDN head strings would miss FQDN source
> until they take the same codemod — out of this blast radius, and not a test."*

It **is** in the blast radius — the stone changed the fact those rules read — and **"not a test" is
why it is dangerous, not why it is acceptable.**

### It is 11 files and 35 comparisons, not 5 and 13

```
wat-scripts/grep/       5 files, 13 compares   bare-variant-constructors · can-raise ·
                                               defined-twice · head-position · unwrap-of-lookup
wat-scripts/fixes/      5 files, 20 compares   ★ RECORDED MIGRATIONS — rename-core-vectors,
                                               rename-core-set-and-list, rename-math-stat-seq,
                                               rename-string-verbs, rename-four-families
wat-scripts/scratch-pad/ 1 file,  2 compares   probe-four-homes-census
```

### And it is PROVEN dead, not inferred — one rule, one file, two spellings

```
can-raise.wat  with  ":wat::core::first"  on wat/deporder.wat   →   0 matches
same rule      with  "wat.core/first"     on the same file      →   9 matches
```

## ★★ THE RECORDED MIGRATIONS ARE THE SHARP EDGE

`wat-scripts/fixes/` is the R21 tooling — *"we use wat-fix to unfuck the farm"* — and this stone
silently disarmed five of them.

⚠ **And the harm is worse than "they stopped working": it is that they became UNFALSIFIABLE.** A
recorded migration that has already been applied correctly reports `CHANGED=0`. A recorded migration
whose rules can no longer match **also** reports `CHANGED=0`. The idempotence check — the thing every
codemod SCORE in this arc has used as proof — **can no longer tell the two apart.**

## THE CORRECTION

1. **Run the same codemod over all 11 files.** `fmt-head-fqdn-to-clojure.wat` already exists and is
   proven idempotent-and-effective; this is the same rewrite over a wider path list.
2. **Prove each grep rule matches again**, the way `can-raise` was proven dead: run it over a target
   that must match and report a non-zero count. **A `--check` is not proof; only a match is.**
3. **A gate**, because nothing catches this. Five rule files were silently dead and the floor was
   green — `wat_grep::*` passes because its fixtures compare `"<-"`, which is bare data and
   therefore identity under the fold. **A smoke test that drives each `wat-scripts/grep/*.wat` rule
   over a known target and asserts a non-zero match count would have caught it on the first run.**

## STOP TRIGGERS

- **STOP-1 — a `--check` is NOT the measurement.** All 11 files `--check` clean right now and every
  one of their name comparisons is dead. Only a **match count** discriminates.
- **STOP-2 — the recorded migrations in `wat-scripts/fixes/` are IN SCOPE.** They are live code under
  the loader gate and they are the tooling R21 mandates. Migrating them is not revisionism; git holds
  what they were.
- **STOP-3 — if a `fixes/` rule compares a name that is bare data** (`"<-"`, `":-"`, `":else"`, a
  field name), **leave it alone** — the fold is identity for those, and rewriting them would be churn
  that changes nothing. **Report which you left and why.**
- **STOP-4 — the gate is not optional.** Without it the next fact-shape change disarms the same rules
  the same way, silently. If a smoke test cannot be written, STOP and report what blocks it.
- **STOP-5 — the four corpus files must stay byte-identical.** Re-verify after the wider sweep;
  `deporder c69f0460e9f91672` · `spawn 1c4bbb19bee10093` · `io ba89b65cbd61abc6` · `fmt cae5250235f652ec`.
- **STOP-6 — if any existing test goes red, STOP.** Capture the whole block verbatim; do not re-run.

## What is already accepted and must not regress

Rows 2-8 and 10-15 all held under my own re-run, and row 6 is the one that matters most: deleting the
FQDN arm left the clojure fixture byte-identical, collapsed the FQDN fixture to the generic
fallthrough, and **touched no rule file**. That is the builder's requirement, demonstrated rather
than asserted. Keep it.
