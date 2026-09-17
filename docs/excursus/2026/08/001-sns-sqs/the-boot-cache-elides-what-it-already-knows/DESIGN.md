# DESIGN — the boot cache elides what it already knows (Tier A)

**Drawn 2026-09-17**, builder-ruled: *"let's do tier A, ten B"*. **NOT STRUCK.**

Tier A of the GO in `the-boot-cache-is-possible-or-it-is-not/FINDING.md`. **Tier B — eliding the check
sweeps — is explicitly NOT this stone** and is gated on a spike that has not run.

## The prize, measured

```
expansion       206.5 ms   51.0 %   ← stdlib-expand alone 199.5 ms
registration     30.4 ms    7.5 %
parsing          29.3 ms    7.2 %
                ─────────
Tier A elides   265.96 ms           accounted boot 404.9 → 154.7 ms   (61.8 %)
read+decode      15.67 ms cold / 7.43 warm   (2,177 bodies · 95,824 nodes · 2.04 MB)
```

⚠ **15.67 ms is a LOWER BOUND, not a target.** The probe serialised function bodies only; it did not
serialise `TypeExpr` fields, could not reach `MacroRegistry` (no public iterator), and **did not build
the real `HashMap` + `Arc<Function>` spine at load.** Building that spine is this stone's job and its
cost is unmeasured. **Report what it actually costs; do not inherit 15.67 ms as a claim.**

## What is already established, so it is not re-litigated

- **Natives are NOT serialised.** `FunctionBody::Native` is a payload-free unit variant, **0 of 2,177**
  functions carry it, and natives dispatch **by string** through a process-global `OnceLock` +
  `inventory` registry that boot never touches. Re-register by name — the mechanism already exists and
  is the live dispatch path.
- **`closed_env` is empty** (0 of 2,177) and `runtime_def_values` is 56 scalars.
- **`MacroRegistry` holds `WatAST` templates** — no `MacroBody::Native`, no native-expander table.
- **`ScopeId` must be REMAPPED**, not reused: `Function::params`' own doc — *"an exec'd child restarts
  `fresh_scope()` at 1, so imported scopes must be REMAPPED."*

## ⛔ THE TRAP THAT WOULD MAKE A BROKEN CACHE LOOK CORRECT

`crates/wat-reader/src/span.rs:137` — **`impl PartialEq for Span { fn eq(&self, _: &Self) -> bool { true } }`**,
and `Hash` is a no-op.

**Therefore `assert_eq!(original, decoded)` PASSES ON A DECODER THAT DROPS EVERY SPAN**, and a cache that
silently discarded every span would ship with green tests and destroy every diagnostic in the language.
Use the probe's shape: a **byte-exact fixpoint**, `encode(decode(encode(x))) == encode(x)`, plus explicit
coverage of the `WatAST` variants the stdlib payload does not exercise.

## Design decisions this stone must take and report

1. **When is the cache built?** At `cargo build` (a bake step / build script / separate bin), or on first
   boot? ⭑ Build-time avoids a first-run penalty and is deterministic; first-run is simpler and needs no
   build plumbing. **Measure the cost of whichever you pick**, and say what the other would have cost.
2. **How is staleness detected?** The cache MUST be invalidated when stdlib source changes.
   `src/hash.rs` exists and the probe judged it "looks right" without trying it. ⛔ **A stale cache must
   be DETECTED, never used** — and detection must be cheaper than what it saves, or the win evaporates.
3. **What happens when the cache is absent, stale, or corrupt?** The only acceptable answer is *fall back
   to deriving it*, silently and correctly. A boot that fails because a cache file is bad is a worse
   product than a slow boot.
4. **Format.** The probe's crude hand-rolled encoder needed **no dependency**. If you add one, justify it
   with a measurement against the hand-rolled baseline — not with taste.

## Trap-doors

1. ⛔ **Behaviour must not change.** The circuit must come back byte-identical
   (`distinct=8000;dup=0` and `timeout=yes;discarded=yes;redial=Connected;retry-on=fresh`), and the whole
   floor must pass. A cache that changes one diagnostic has failed.
2. ⛔ **Do NOT elide the check sweeps.** That is Tier B, it is worth 126 ms, and it is gated on an
   unproven soundness question. Touching it here makes one floor prove two things.
3. ⛔ **Do not touch `.config/nextest.toml`.** Raising timeouts is the habit this whole thread exists to
   end — and if Tier A works, the loader gate's 687 startups get *cheaper*, which is the point.
4. **Cold vs warm both, always.** `stdlib-parse` alone moves 29 → 48 ms cold, and every hand-timing in
   this campaign before the census was ambiguous by ~15 ms.
5. **Use the census** (`WAT_BOOT_CENSUS=phases|files`, `src/freeze/census.rs`) rather than re-timing by
   hand; it reconciles to 1.8 % and its own overhead is 0.08 %.
6. **The queue promotion is the acceptance test that matters later** — parked on
   `queue-promotion-blocked-on-startup-cost` (`6136d144f`). Do not un-park it here; just do not make it
   harder.

## Out of scope

- **Tier B** and its spike.
- The two sharpening follow-ons (attributing `check:body-infer` to files; per-form expansion in
  `telemetry/journal.wat`).
- Any other optimisation. If you see one, write it down and leave it.
