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

## Half-B VERIFICATION — the contained un-ignore proof (orchestrator, first-hand)

The struct-restricted → defstruct migrations live inside #[ignore]'d tests, so
the gates CAN'T prove they check. The strategy doc owed each defused file a
contained un-ignore. Doing it surfaced a finding the gates never could —
and a BETTER truth than the brief assumed:

- **The retired `struct-restricted` form was ITSELF broken at resolve** — it no
  longer registers its `Type/new` constructor + accessors (7 unresolved refs).
  The proof files weren't just carrying a phantom; they were carrying a phantom
  that ALREADY didn't resolve. (Proven by un-ignoring the ORIGINAL form,
  contained: `:counter::User/new` etc. all UnresolvedReference.)
- **The `defstruct` migration FIXES the resolve** — the auto-synthesized
  constructor + accessors register correctly (matches the green reference
  tests/wat_arc203_struct_restricted.rs). Verified on the two struct files
  (counter-client-capability-proof + counter-service-capability-N3): un-ignored
  + contained (setsid+timeout+reap), the panic moved from test_runner.rs:459
  (startup/check) to :487 (the post-check RUN phase = the arc-170 concurrency
  layer, the documented ignore reason). **Check phase CLEAN.**
- Content-grep confirms zero surviving retired forms across the whole corpus
  (no `struct-restricted` in code; no value-position `:wat::core::nil`).
- One line-shift bug caught + fixed (a sed nil-target drifted after the
  multi-line defstruct edit; redone by content, re-verified).

**Conclusion: the defuse is VERIFIED.** The bombs are out; the migrated forms
check clean; the files' ONLY remaining failure is the arc-170 concurrency they
are correctly ignored for. The deeper composition wards (complectens
sibling-deftest gaps) remain affirmatively scoped to the arc-170 reanimation —
when those tests run again, they'll check-clean AND get their layering ward.

## circumspicere — the perimeter (cast LAST, on `9b91368e`)

The perimeter earned its place here too — six inward lenses passed; the
surround surfaced what none of them could see.

**Verdict: 0 L1 + 5 L2 + 2 L3 + 9 claims verified clean.**

| # | surround | finding | weighing (four-questions, driven) |
|---|---|---|---|
| F1 | discovery | claims verified clean — no silent-skip surface today; recursive walk + signature-only filter both honest | NO ACTION (sealed by reference in the stamp's gates) |
| F2/F3 | claim-vs-code | **9 fully-dark "proof" files** (counter-actor/service/client + ambient-stdio) — declared "proof", zero running assertions; "tests are demos" doctrine bent at 18% dark; stream.wat 12/14 dark | **DEFER affirmatively to arc-170 reanimation** — same scope as the strategy's Half-B composition deferral; named, not silent |
| F4 | unenforced invariant | **NO GATE on arc-170 ignore removal** — 53 ignores carry "remove before arc 170 closes" but nothing fires when arc-170 closes; gate is designed green-under-dark-state | **NEW NAMED STONE (`#181-followon: arc-170 ignore-removal gate`)** — four-questions on the gate's *shape* failed Obvious + Simple (multiple enforcement mechanisms, each with different failure modes; cleanly answering needs grounding the arc-170 closure mechanics, which is outside #181's scope). The honest closure is a sibling stone with slow-head, not an inline tail. The stamp's invariants list calls out F4 by its tracker so the closure is not a lie. |
| F5 | negative space | three framework verbs publicly documented but UNEXERCISED — `:wat::test::run`, `:wat::test::run-in-scope`, `:wat::test::run-ast` (README presents them as "reach for it when you need to drive the sandbox by hand") | **FIGHT NOW** — one live witness per verb in wat-tests/test.wat; the corpus-as-teacher doctrine requires every public verb has at least one live demo |
| F6 | claim-vs-code | 2 corpus files violate the README's own hyphen-naming convention: `holon/list_round_trip.wat`, `holon/char_round_trip.wat` | **FIGHT NOW** — git mv to hyphenated names |
| F7 | clarity | 7 framework helpers in wat/test.wat are internal (called only by macros) but indistinguishable from public verbs in the file's surface | **FIGHT NOW (L3)** — mark each with a single `;; Internal — not for direct corpus use` comment |

**Claims verified clean (9)** — signature-only discovery (test_runner.rs:610-620);
recursive walk; parse-fail surfaces with full diagnostics (no silent-skip
unless the file has zero `:wat::test::` forms, which the live corpus does
not); random per-file shuffle; the `(:wat::test::ignore)` form is a real
parsed/checked deftest body that returns nil; the stream/holon README pairing
honest; the wat/holon/Project alongside Reject; the integration-run footer's
ignored-count gives visibility; arc-170 reasons machine-readably uniform.

## The strike (drawn 2026-06-06)

Four-questions-driven decisions (each option atomized, judged YES/NO per
question — see the conversation record; the verdict below is the result):

1. **FIGHT NOW**: F5 + F6 + F7. Each is one cleanly-scoped surface, each
   closure is one verifiable edit. Obvious + Simple + Honest + Good UX all
   hold per item. Bundled into one commit; corpus + lib gates green.
2. **DEFER (named-stone)**: F4. Obvious + Simple failed (the gate's *shape*
   is not yet decided; multiple enforcement mechanisms with different failure
   modes; deciding without grounding the arc-170 closure mechanism would be
   the "ship a shape under pressure" anti-pattern). The honest closure is a
   sibling stone tracked at the stamp's invariants list (the same shape the
   #151 endgame doctrine takes — gates are decisions, not reflexes).
3. **DEFER (arc-170 reanimation)**: F2/F3 (9 dark files) + the complectens 8
   L2 (sibling-deftest gaps in the same files). The composition wards cannot
   be earned while the concurrency layer leaks; they ride the arc-170
   reanimation arc. Named, scoped, not silent.

## After the strike

Gates → vigilatum stamp on `wat/test.wat` (the warded unit is the framework;
the corpus is its demonstrated surface) → 245.N INSCRIPTION (FM-11 wrap-proof
grep) → arc 245 CLOSED. The follow-on `#181-followon: arc-170 ignore-removal
gate` is cited at the stamp's invariants list (F4) and inscribed in the 245
INSCRIPTION's affirmative-scope bounds.
