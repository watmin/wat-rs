# #181 — Warding the TEST SURFACE (245's true heart)

> The tests ARE the demos (builder doctrine; the arc-245-v1 correction). The
> corpus is now GREEN for the first time ever (the full clear) — the right
> moment to ward, because warding green tests is quality judgment; warding red
> ones is archaeology. Surface: wat/test.wat (the framework, 933 lines) + all
> 45 wat-tests/**/*.wat (the corpus). Cast mechanic: spells fetched from the
> signed channel, embedded verbatim. The test-kind guard's FIRST full muster.

## The six casts (2026-06-06; complectens re-cast after a python-heredoc stall — vanilla-commands-only)

| spell | verdict | the load-bearing finding |
|---|---|---|
| cernere | **15 L1** + 9 L3 | LATENT BOMBS: 10 value-position `:wat::core::nil` + 5 `:wat::core::struct-restricted` (hard-cut 241.8) — live phantom forms HIDDEN inside #[ignore]'d deftests; they detonate at startup the day arc-170 lifts the ignores. + 9 stale framework doc-comments teaching retired forms (define/struct/enum expansion examples) |
| intueri | 5 L1 + 4 L2 + 2 L3 | the framework's OWN docs lie: deftest expansion shows `define` (emits `defn`); "deftest currently expands to run-hermetic" (flipped long ago); deftest-hermetic named via a retired mechanism; README teaches a `test-` prefix rule the runner DROPPED 2026-04-25 |
| probare | 9 L1 + 8 L2 + 4 L3 | the 4 tmp-* files are REAL regression guards wearing scratch names (arc-139 inference anchors, arc-140 diagnostic guard) → RENAME-to-purpose into core/, not delete. 3 ACTIVE hollow tests (assert-eq true true proving nothing) + 1 on-mismatch-only helper (passes silently by construction). 9 L1 = the fully-dark proof files (declared "proof", zero running assertions — legit arc-170 suspension, but the gap is total) |
| vocare | 1 L1 + 1 L3 | a test that CANNOT fail (struct-to-form: inner run-thread outcome discarded, unconditional sentinel). File-audience taxonomy clean; service-template + the proof files correctly at implementer vantage (headers claim it) |
| exigere | 4 L1 | bare deferrals to rewrite as present-truth: test.wat:766 "deferred to a future arc"; service-template:39 "a future arc may wire it"; test.wat:652 "if a kernel-layer caller surfaces"; counter-actor-proof-process:162 "will eventually hide" |
| complectens | 0 L1 + 8 L2 | ALL 8 in arc-170-ignored deftests (the proof files' prelude helpers lack sibling deftests + 2 error-discard sites). **The RUNNING corpus is complect-clean.** 4 existing runes all audit VERIFIED |

**Aggregate: 34 L1 + 20 L2 + 15 L3.** All 4 pre-existing runes (complectens) audit clean.

## The strategy — the surface has two halves, ward each honestly

**Half A — the RUNNING surface (green, active): ward to zero NOW.** This is
what teaches every reader today. Everything fixable without touching the
arc-170-gated concurrency machinery:
- **cernere stale-doc sweep** (9 sites in wat/test.wat): retired-form expansion
  examples → modern forms (defn/defstruct/defenum); the "8 declaration heads"
  list corrected.
- **intueri framework-doc truth sweep** (5 L1 + 4 L2): the deftest/deftest-hermetic
  expansion docs, the run-thread/run-hermetic flip comments, the README `test-`
  prefix lie → corrected to what the runner ACTUALLY does (signature-only
  discovery; verify against src/test_runner.rs).
- **probare hollow-test realification** (3 active sentinels + 1 on-mismatch
  helper): give each a real assertion on an observable (or, where genuinely
  only no-panic is provable, name that honestly with the WHY — not a
  true/true lie). vocare's struct-to-form CANNOT-FAIL test is the same item.
- **probare tmp-* renames** (orchestrator's git mv): tmp-3tuple-inferred →
  core/generic-tuple-infer; tmp-3tuple-probe → core/generic-tuple-turbofish;
  tmp-baseline-nongeneric → core/generic-tuple-nongeneric-baseline (or merge);
  tmp-totally-bogus → core/unknown-call-head-panics + STATUS-comment fix.
  Verify each still discovered + green after the move.
- **exigere deferral→present-truth** (4 sites).

**Half B — the DORMANT surface (arc-170-ignored proof files): DEFUSE the bombs
now, WARD the composition when arc-170 reanimates them.**
- **DEFUSE NOW (cernere 15 L1)**: the value-position-nil + struct-restricted
  phantoms inside the ignored bodies get modernized THIS stone — a bomb is a
  bomb whether or not it's currently armed; the lru lesson is exactly "dark
  code rots." These are mechanical (nil→bare nil; struct-restricted→defstruct
  +{:restricted-to}). The files stay ignored (arc-170 gate intact) but carry
  no retired forms.
- **DEFER the composition (complectens 8 L2 + probare 9 L1 dark-proof)**:
  the sibling-deftest gaps + the "declared-proof / zero-running-assertions"
  gap are NOT fixable without un-gating arc-170 (the helpers can't get
  isolation deftests while the whole concurrency layer leaks). Affirmatively
  scope to the arc-170 reanimation: when those tests run again, they get the
  composition ward. Named, not a silent defer. (#151's run-tier / the 67
  excluded leaky binaries are the same arc-170 frontier.)

## After the fight

circumspicere LAST over the whole surface (the perimeter the six inward lenses
left: does the harness CLAIM coverage it doesn't run? does README's taxonomy
match the live runner? is there a test-discovery default that silently skips a
file? the error-chain-discard CLASS as a finding-generator for #181's own
doctrine) → termination by judgment → the `vigilatum` stamp on wat/test.wat
(the framework is the warded unit; the corpus is its demonstrated surface) →
**245.N INSCRIPTION** (FM-11 wrap-proof grep) → arc 245 CLOSED.
