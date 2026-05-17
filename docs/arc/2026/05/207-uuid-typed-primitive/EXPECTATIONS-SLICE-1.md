# EXPECTATIONS — Arc 207 Slice 1

## Mode prediction

**Mode A — clean audit, decision reached, surface checklist produced (~70%).** Sonnet reads the 5 audit targets, names the existing-primitive patterns, picks (a) / (b) / (c) with four-questions-YES-YES-YES-YES, produces a concrete slice-2 checklist. ~35-45 min. Likely shape: option (c) wins (true distinct Value variant; mirrors wat-edn; matches Clojure's distinct-runtime-type) OR option (a) wins (lightest weight; substrate-aligned with how `:wat::core::Bytes` works per arc 062).

**Mode B — STOP-trigger fires (~20%).** Most likely STOP-trigger:
- Audit 2 reveals wat-edn's `Value::Uuid` requires substrate-level coordination beyond a simple variant addition
- Audit 3 reveals dispatch infrastructure needs explicit registration per-type (not automatic)
- Audit 4 reveals `parse_str` tolerance / canonical form has surprise semantics affecting user surface

Sonnet surfaces, orchestrator decides, sonnet completes the audit with the decision encoded.

**Mode C — fourth candidate shape surfaces (~10%).** Audit reveals a wat-specific substrate pattern the DESIGN didn't enumerate (e.g., struct-restricted-style opaque type, or a dispatch-table-only registration with no Value variant). Sonnet runs four questions inline on the new candidate; if it wins, surface to orchestrator for confirmation before SCORE.

## Shape-decision prediction

My orchestrator-side guess: **option (c) — new `Value::wat__core__Uuid(uuid::Uuid)` variant.**

Reasoning sonnet's audit should either confirm or overturn:
- Clojure precedent has UUIDs as distinct runtime values (java.util.UUID); wat-edn already has `Value::Uuid` at its layer
- Option (a) typealias loses runtime discrimination — `(= some-string some-uuid-as-string)` would unexpectedly return true at the Value layer
- Option (b) newtype adds construction ceremony every Uuid use site (wrap/unwrap); doesn't match the "Uuid is just an identifier" user mental model
- Option (c) is heaviest weight but lands the actual semantic ("a Uuid is a distinct thing"); enables the EDN read fix in slice 3 to be a direct `Edn::Uuid → Value::wat__core__Uuid` map

If audit reveals (a) is the substrate's pattern for typed primitives (e.g., `:wat::core::Bytes` is a typealias), the wat-shaped answer may converge on (a) anyway — same destination, different mechanism. Trust the audit.

## Workspace baseline expected

Sonnet should NOT run cargo test (no compile changes). Sonnet may run `cargo check` if needed to verify the substrate's current state, but shouldn't burn time on the test suite.

## Out-of-scope findings (surface, don't act)

These may surface during audit; surface in SCORE honest-deltas, don't act on them:

- Other typed primitives that could benefit from the same shape decision (e.g., if `:wat::core::Bytes` should be (c) instead of (a)) — out of arc 207 scope
- Bugs in arc 206's String-typed verbs — out of arc 207 (slice 4 retires them entirely anyway)
- Stale references in DESIGN.md that audit reveals incorrect — surface; DESIGN is living per FM 13
- Naming convention drift sonnet notices in adjacent verbs — out of slice 1 scope (naming for Uuid is settled; other verbs not arc 207's concern)

## Failure-mode catches

- FM 11 (deferral language): N/A for this slice — no INSCRIPTION yet
- FM 14 (surface retirement leaving internal identifiers): N/A — this is mint slice, not retirement
- FM 16 (no tool preamble): BRIEF doesn't mention Bash/cargo availability
- FM 17 (discipline-after-pushback): sonnet should fire FM checks before action — most relevantly FM 1 (grep before claiming) applies to every audit assertion
- `feedback_assertion_demands_evidence`: every audit claim cites file:line; if sonnet writes "the substrate registers types via X" without a file:line, that's the failure pattern to catch

## What "done" means for this slice

SCORE-SLICE-1.md exists; 3 SCORE rows YES; shape decision (a/b/c) named with four-questions verdict; slice 2 surface checklist concrete enough that BRIEF-SLICE-2 can be drafted from it without re-deciding shape.

## Calibration record

Slice 1 is the cheapest slice (audit, no code). If sonnet returns under 30 min that's calibration data for sub-slice future slicing. If sonnet hits the 60-min cap on a pure-read audit, something's structurally wrong with the BRIEF (most likely: I over-specified audits sonnet should narrow itself, or under-specified what file:line evidence looks like).

Orchestrator's expectation: 35-40 min. Sonnet should NOT touch any source file; if it does, that's a BRIEF-violation worth catching.

## Atomic commit shape

NO commit by sonnet. Orchestrator independently verifies SCORE + commits both BRIEF + EXPECTATIONS + SCORE in one commit (SCORE comes BACK from sonnet; BRIEF + EXPECTATIONS go OUT before sonnet runs; orchestrator commits the round-trip atomically when sonnet returns).

Actually that's wrong — BRIEF + EXPECTATIONS commit BEFORE spawn (per recovery doc § 7 pre-flight), SCORE commits AFTER sonnet returns. Two separate commits. Sonnet doesn't commit anything.

Sonnet: trust the audit; trust the disk; produce the checklist; return.
