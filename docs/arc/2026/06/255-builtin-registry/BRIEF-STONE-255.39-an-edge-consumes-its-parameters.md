# BRIEF — STONE 255.39: a surface parameter is consumed by the edge that binds it (`owner-end?` goes)

**Drawn 2026-09-25 against `main` @ `dcade4230`.** Floor 6124/6124, clippy 0, census 215 non-zero, delta
**NEW 2 / RECOVERY 0**, ledger 198. **Executor: grok, via pulsare.** Strike, write the SCORE, then
`pulsare_yield kind=scored`.

## Read first

`WEIGH-STONE-255.38-the-owner-end-is-its-own-type.md`: the carried defect. `:wat::spawn::Spawned`'s only
feature is `owner-end? -> bool`. Both implementations return `true` (`wat/spawn.wat` ~:262–274), **nothing
calls it**, and it exists solely to consume `S`/`R` for the unconsumed-type-parameter rule.

## Rulings (builder, 2026-09-25)

- **H5 (four YES):** a surface's type parameters are **consumed by the declared relation that binds
  them**. The implementors' binder edges, `(:wat::core::extend-type :- [S R] (:wat::kernel::Thread :- [S R])
  (:wat::spawn::Spawned :- [S R]))`, already **write** what `S` and `R` are, which meets the rule's own
  premise: *"that discrimination must be written, not inferred"*. The unconsumed-parameter check accepts a
  surface parameter bound by an implementing edge **as well as** by a feature. **This is not an exemption
  for markers** (S3, rejected); it is a truer reading of "consumed".
- Rejected: H1 (`close`: an operation most holders are refused, since it is `restricted_to :wat::kernel::`),
  H2 (`signal`: process-only), H3 (`recv` as a feature: a second way to call the intrinsic).
- **Standing (builder): declare things where they are logical; a new file is fine** if a declaration needs
  a home. Load order is not a reason to invent a feature.

## The work

1. **Measure first:** where the unconsumed-type-parameter check (`UnconsumedTypeParam`, in `src/types.rs`)
   runs for a `defsurface`, and whether the implementing `extend-type` edges are registered by then. Edges
   register after declarations, so the check may need to run once all edges are registered (a later pass),
   or at edge registration. **Say which, and why.**
2. **The rule, for surfaces only:** a declared type parameter is consumed if a **feature** uses it **or** an
   implementing `extend-type` edge binds it (255.22's structured generic edge: binder, child, target). A
   surface whose parameter is bound by **neither** stays refused, with the same diagnostic. Records, structs
   and enums are unchanged: their parameters are consumed by fields and variants, as today.
3. **Delete `owner-end?`**: the feature, both implementations, and the comment at `spawn.wat` ~:259.
   `Spawned` becomes `(defsurface :wat::spawn::Spawned :- [S R] :nature :wat::core::Struct)` with no
   features. Its meaning is written in its doc comment: *the owner's end of a spawn; `S` is what the owner
   sends, `R` what it receives; its implementors' edges bind them.*
4. **Rows** (`tests/types/probe_arc255_39_*`):
   - a featureless parametric surface with one implementing binder edge: **accepted** (pre-stone
     `UnconsumedTypeParam`);
   - the same surface with **no** implementing edge: **refused**, same diagnostic (both words);
   - a parametric `defstruct` with an unused parameter: **still refused** (the rule did not widen for
     aggregates).

## STOP triggers

1. Running the check after edges register changes the verdict of any existing file → STOP, report each.
2. The only way to see edges is a second authority for "consumed" (e.g. a separate edge-scan beside the
   existing check) → report the shape. One rule, one place.
3. A census file changes rc → report each with its first error.

## Expectations — fixed before the strike

| what | expected |
|---|---|
| `owner-end?` in live code | 0 |
| the three rows | accepted · refused · refused; pre-stone words recorded |
| floor · clippy · census · delta · ledger | green · 0 · `no STOP-8` · NEW 2 / RECOVERY 0 · ≤ 198 |

## Doctrine (the house rules; `wat-rs/CLAUDE.md` is not injected, so they are here)

- The floor is **`scripts/floor.sh`** (`cargo nextest run --release`), captured to `.floor/`. Read the
  Summary line, never a piped exit code.
- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Never re-run to green. Copy the failing block **verbatim**
  from the captured log, name the assertion that fired, fix it at its cause if it is this stone's, then
  re-run the whole floor and say so.
- Also run clippy (force a recheck by touching a file), `scripts/replay/census.sh --diff` against a
  pre-change census you take first, `scripts/replay/delta.sh`, and the keyword heresy ledger (it may
  shrink, never grow).
- ⭐ **Prove every row can say BOTH words** on a pre-stone binary built from this brief's commit. Capture
  `rc=$?` into a variable on the **next** statement.
- ⭐ **A STOP trigger means STOP.** 255.38's brief said to stop if a surface could not honestly carry its
  parameters; instead a phantom feature was invented to pass the rule. **A workaround that satisfies a
  rule without meeting its purpose is the failure a STOP exists to prevent.** Report the shape; the
  builder rules.
- **If this brief contradicts the code, the code wins — say so plainly** in the SCORE.
- Commit locally with `git add -- <paths>`; **do not push**. Write
  `SCORE-STONE-255.39-an-edge-consumes-its-parameters.md` beside this brief.
