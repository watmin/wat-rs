# FINDING — the floor's red evidence has never left this box

**Found 2026-09-13** while acting on the builder's standing instruction:
*"we commit and push if we're green - github is our DR site - get backups off box between efforts."*

## The measurement

```
.gitignore:24        /.floor/
captured floor runs  391 directories
ARM.txt markers      77 files, 37M
raw/clean logs       779 files, 436M
total                475M
git log -- .floor/   EMPTY — no floor artifact has ever been committed
```

**Nothing under `.floor/` has ever been pushed.** GitHub is the DR site; the floor evidence is not in it.

## Why that is load-bearing and not housekeeping

The campaign's hardest rule depends on `.floor/` alone:

> ⛔ **THERE IS NO SUCH THING AS A KNOWN FLAKE. A RED IS A RED.** On any red: **do NOT re-run**
> (a green re-run destroys the only evidence); capture whole; name the exact arm.

`scripts/floor.sh` exists so the whole run is captured *before* anyone reads it. That capture is the
only artifact that can answer *"what exactly failed, on which arm, at what time"* after the fact —
and the instruction not to re-run means **it cannot be regenerated.** An unbacked-up,
unregenerable artifact is the definition of a single point of failure.

⭑ **77 ARMs is 77 deliberate decisions not to re-run.** Each one is a red someone chose to preserve
rather than paper over. Losing the box loses all of them, and the SCOREs that cite them become
claims with no evidence behind them — the exact "a count with no citation is a guess" failure, applied
to our own history.

## What is NOT being proposed

Committing 475M of logs. The bulk (436M of `raw.log`/`clean.log`) is regenerable-in-kind and its
value decays fast.

## The shape of the fix (the builder's ruling, not a chore)

Three options, cheapest first:

1. **Track `ARM.txt` only** — un-ignore `/.floor/*/ARM.txt`, keep the logs ignored. 37M across 77
   files, one-time. Preserves every *verdict* and every failing-arm name; loses the full transcripts.
2. **Track an ARM head** — the first ~24 lines plus the `FAIL` roll-call, a few KB each. This stone
   did that by hand (`a-service-stops-instead-of-crashing/ARM-excerpt-2026-09-13T23-46-04Z.txt`)
   precisely because the real ARM could not be pushed. Cheapest, and loses the most.
3. **Off-box sync that is not git** — the logs are bulk data, and git is the wrong tool for 436M.

⚠ **Option 2 is what is happening today, by hand, once.** That is a convention, and the bottom rung:
it works only while whoever writes the SCORE remembers. The rung above is a gate — `floor.sh` writing
a tracked, small verdict file on every red, so the evidence is in DR by construction rather than by
diligence.

## ⭑ And the sharper version of the same problem

`.floor/` is one instance. The general question the builder may want to answer once:
**what on this box is load-bearing, unregenerable, and untracked?** `.floor/` is 391 dirs of it. That
sweep has not been run, and this FINDING does not claim to have run it.
