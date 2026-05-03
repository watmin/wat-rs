---
name: vocare
description: Call the test to its caller. The datamancer vocares the tests — does this verify what the caller sees, or has the test reached past the interface into the implementation?
argument-hint: [test-file-path or directory]
---

# Vocare

> All code is measurable from the caller's perspective. That's the interface to confirm.

The other wards check the code. Vocare checks **the tests**.

A test is a call. It calls the interface and reports what came back. Tests are the substrate's voice to a fresh reader — they show the recommended way to use a thing. A test pattern lifted by the next reader becomes the next consumer's code.

When a test calls past the interface — into protocol mechanics, internal state, channel allocations a consumer wouldn't touch — it speaks for the implementer, not the caller. A reader who lifts that pattern learns the wrong shape.

## The principle

Every line of substrate has a caller. Service consumers, service implementers, framework users, library users — each occupies a vantage. The caller's vantage IS the interface. Anything else is mechanism.

Vocare asks: **from WHOSE vantage does this test verify?**

- A consumer-API test stands where a consumer stands.
- A substrate-primitive reference test stands where a service implementer stands.
- A Rust unit test stands where another Rust caller stands.

A test that stands NOWHERE — that has cracked open layers no real caller would touch — is at the wrong vantage. It vouches for nothing the caller sees.

## What vocare flags

### Tests calling past the recommended surface

When a helper verb / public API exists for a use case, but the test bypasses it:

```scheme
;; A consumer of HologramCacheService calls
;; (HologramCacheService/get req-tx reply-tx reply-rx probes).
;;
;; This test bypasses the helper and hand-builds the wire protocol:
(:wat::kernel::send req-tx
  (:wat::holon::lru::HologramCacheService::Request::Get k reply-tx))
;; Then manually unwraps the Result<Option<T>, ThreadDiedError> chain.
```

This test verifies the wire protocol. Wire-protocol pedagogy lives in `wat-rs/wat-tests/service-template.wat` (per `SERVICE-PROGRAMS.md` § "Audience boundary"). A test in a consumer crate's `wat-tests/` should call the consumer surface, not the wire under it.

### Tests with patterns USER-GUIDE wouldn't recommend

If the test's call shape doesn't appear in the worked examples in USER-GUIDE / README / doc-comments — and a worked example for the same use case exists — the test is at the wrong layer.

### Tests verifying state the caller can't observe

A test that pokes at internal counters, private fields, intermediate channel states, or other invisible-to-caller details. The test passes when the visible behavior is broken; fails when the invisible mechanism changes. Both directions are wrong.

### Tests structurally indistinguishable from the implementation

If the test could be deleted and replaced by a copy-paste of the production code without losing coverage, the test isn't measuring anything observable — it's restating the implementation.

## What vocare does NOT flag

- Tests in **wire-protocol reference files** (e.g., `wat-rs/wat-tests/service-template.wat`). Those ARE caller-vantage tests; their caller is the service implementer using substrate primitives.
- **Rust unit tests in `src/*.rs`**. Their callers are other Rust modules. The test's vantage matches its actual caller's vantage.
- **Tests that mirror worked examples** in USER-GUIDE / README. The example is the recommended pattern; the test that mirrors it confirms the recommendation.
- Tests that **expose a defect by setting up a state the caller can produce** — even if the setup is intricate. The intricacy is the bug surface, not a wrong vantage.

## How to read a test through vocare

For each test in scope:

1. **Identify the caller.** Who calls this code in production? Consumer, implementer, internal Rust caller — pick one.
2. **Read the test as that caller.** Does the call shape match what that caller would write? Or does it reach past their vantage?
3. **Check the worked examples.** Does a USER-GUIDE / README / doc-comment example show this shape? If yes, the test is at the right vantage. If no, ask why.
4. **Check the public surface.** Is there a helper / API the test could have called instead of building the call by hand? If yes, the test should call the helper.

## Reporting format

For each flagged test, report:

- File path + test name
- The vantage the test PRESENTS as testing (consumer, implementer, etc.)
- The vantage it ACTUALLY tests from
- The recommended-surface call the test should be using instead
- One-sentence judgment: real defect surfaced, or scenario coverage that needs rewriting at the right layer

## The rune

Some tests intentionally test from a non-consumer vantage — they
exist as substrate references, protocol fixtures, or vantage-bypass
tests by design. For these cases, the test gets a **rune** that
declares the vantage exempt with a justified reason:

```scheme
(:wat::test::deftest :wat::kernel::test-spawn-program-stdout-piping
  ;; rune:vocare(substrate-primitive-reference) — this test documents the spawn-program substrate primitive's stdout-piping contract; the implementer's vantage IS the canonical vantage here
  (:wat::core::let* (...) ...))
```

Format: `;; rune:vocare(<category>) — <reason>`

Mirrors the lab's ward-rune convention (`~/work/holon/holon-lab-trading/.claude/skills/`):
positional category in parens, em-dash separator, free-text reason after.

**Categories:**

- `substrate-primitive-reference` — test exists to document the substrate primitive itself (e.g., `wat-tests/service-template.wat`). The implementer's vantage is the canonical vantage at this site; consumer-vantage doesn't apply.
- `protocol-fixture` — test exercises protocol mechanics as setup for downstream tests; consumer doesn't see this layer but it's load-bearing for the next layer's coverage.
- `vantage-bypass-test` — intentional internals test (parser tests touching the token stream, runtime tests probing internal state) that validates substrate machinery a consumer wouldn't reach.
- `defect-exposure-fixture` — test sets up a state the caller can produce; the setup intricacy is the bug surface being demonstrated, not a wrong vantage.

Placement: on the line immediately above the deftest body
(typically inside the deftest form, before the let*).

The reason field is required. A rune with an empty reason fails
the spell — the rune's job is to capture the WHY so the next
reader understands the exemption rather than guessing.

When vocare encounters a rune, it skips the test and records the
exemption in its report. Recognize `rune:vocare(...)` runes.

## Cross-references

- `docs/CONVENTIONS.md` § "Caller-perspective verification" — the principle the spell defends.
- `docs/SERVICE-PROGRAMS.md` § "Audience boundary" — separates wire-protocol pedagogy (service implementers) from consumer-API pedagogy (service consumers).
- `wat-rs/wat-tests/service-template.wat` — the canonical wire-protocol reference.
