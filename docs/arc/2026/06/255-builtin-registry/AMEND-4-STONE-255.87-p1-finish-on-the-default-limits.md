# AMEND 4 — STONE 255.87: P1 — finish on the default limits

**Drawn 2026-10-03.** **Executor: grok via pulsare, working solo** (it runs the floor). Continues at `ea873e90e`
(STOP-2). Commit locally on `main`; **do not push**.

## The ruling on STOP-2 (builder, 2026-10-03): P1

The cures in `369162eb2` are accepted (`is_subtype_parent` a set membership; symbol spellings stored once beside the
keyword key at registration; reconstruct only an ambiguous spelling). Shard 2, one run each on the same Rust:
unconverted 8.493 s, converted **9.379 s**; this morning's unconverted baseline was 9.707 s (six-run mean). **The residual
0.886 s is accepted and recorded**, not chased inside hygiene: under the builder's symbol rule a slashed symbol in a body
can be a local (`(let [completely/insane 42] completely/insane)`), so every reference keeps its scopes, and the orchestrator's
P2 ("a namespaced reference carries no scope") rested on a false premise. The larger win (one frozen stdlib shared across
worlds) is a separate design, the builder's to draw.

## The work

1. **The floor on the cured tree, no limit changed.** The four time-limit rows must pass on their default limits; if one
   does not, capture it verbatim and STOP (do not raise it).
2. **The six-run table** for shard 2 (and `keyed_gather`) on the cured tree, both columns, as amendment 1 measured, so the
   residual is recorded with a mean, not one run.
3. **The brief's item 5:** `census.sh --diff` (no rc flips) and `delta.sh` on the 179-file sample against the converted
   stdlib (NEW 0, or each explained).
4. Clippy; `git status` clean before the floor.

A STOP means STOP. Append to the SCORE, commit, **do not push**.
