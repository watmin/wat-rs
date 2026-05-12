# Arc 190 — Independent foundational test rebuild (src/ + wat/)

**Status:** stub opened 2026-05-13 per user direction.
**Gates on:** arc 188 (perf + Rust impl scrutiny) — or after the impl-scrutiny family closes.

## Motivation

> *"after that i want all unit tests to be rebuilt (while preserving the existing ones) for both src/ and wat/ -- i want the agent who works on this to assume no prior tests exists and we rebuild our foundational tests adjacent to the existing ones -- completely independent coverage tests"*

The existing test suite grew organically alongside the substrate. Each arc shipped its tests; each test verifies its arc's load-bearing concern. The tests work, but they're tests-as-arc-record, not tests-as-foundation-coverage.

This arc rebuilds the foundational tests **assuming no prior tests exist** — an agent picks up the substrate cold, looks at each module + each wat-level form, and writes the tests they would write FROM SCRATCH. The result is **adjacent** (preserves existing tests; ships alongside in distinct test files) and provides **completely independent coverage** (no shared fixtures, no inherited assumptions, no "but the existing test covers this" reasoning).

**Why preserve existing tests:** they're the arc record. Each arc's tests are what that arc considered load-bearing at the time. The new foundational tests are a SECOND vantage — both vantages running gives evidence the substrate's correctness doesn't depend on shared test-writing assumptions.

## Sketch

### Substrate-level (src/)

Per module (post-arc-187 modularization):
- Read the module's public surface
- Write tests for every public function / every state transition / every error path
- Pretend the existing test files don't exist; check coverage after via grep, not consultation
- New files live adjacent: `tests/{module}_foundational.rs` or similar

### wat-level (wat/)

Per wat-level form / per wat-stdlib file:
- Read the form's specification (docs + INSCRIPTION)
- Write tests for every behavior the spec describes
- Pretend the existing wat-tests don't exist
- New files live at `wat-tests/foundational/{topic}.wat` or similar
- Each test exercises the form FROM the user-facing surface, not its substrate plumbing

### Coverage equivalence + divergence audit

Once both suites exist + pass:
- Where do they EQUIVALENTLY cover the same behavior? Confirm; this is the strong evidence (two independent vantages converged)
- Where does one cover behavior the other misses? Surface — either:
  - Existing test caught a load-bearing detail the foundational forgot → annotate foundational test to add it (carefully — preserve independence)
  - Foundational test caught a behavior existing tests miss → ship the foundational; flag the gap in existing coverage; consider amending existing-arc record
- Where do they DISAGREE on behavior? **Substrate bug surfaced.** Diagnose.

## Why this is foundation work

The user's foundational-impeccable framing (cross-reference INTENTIONS.md):
> *"once 109 wraps up - we'll have what we believe to be an incredibly solid foundation to begin the next leg of work... i cannot begin any of that work until the foundation is impeccable."*

A test suite that grew with the substrate may have shared blind spots — assumptions that every arc inherited because the same person wrote each arc's tests. An independent foundational suite, written fresh from spec, eliminates that shared blind spot.

The strange-loop framing: the substrate teaches whoever picks it up cold. The foundational test suite is the test of that teaching — if the substrate's diagnostics + docs are good enough, a fresh agent reading them and writing tests from scratch should converge on equivalent coverage.

## Cross-references

- Arc 187 (modularization) — must close first (post-modularization boundaries are what a fresh agent walks)
- Arc 188 (perf scrutiny) — must close before this arc (perf changes might shift hot paths; foundational tests should reflect final perf-honest impl)
- Arc 189 (wat-edn streaming) — if it ships, foundational tests cover both the whole-document + streaming APIs
- INTENTIONS.md § "Why the disciplines compose with the platform" — the static-check + foundational-tests pair forms the verifiable substrate
