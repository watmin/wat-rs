# Arc 214 Slice 1 — WARD PASS

Per the kernel-impeccability protocol (INTERSTITIAL § 2026-05-19 "Kernel impeccability via ward pass"): per-slice trust gate = BRIEF scorecard verification + ward pass before commit. This doc captures the ward round-trip for Slice 1.

## Round 1 — initial ward pass (after sonnet SCORE Mode A)

Targets:
- `src/comms/mod.rs` (~144 LOC)
- `tests/probe_comms_foundation.rs` (~52 LOC)

4 wards spawned in parallel per `/wards` skill convention (independent agents, single message).

### gaze — 5 findings (1 L1, 4 L2)

| Line | Level | Observation |
|---|---|---|
| probe:36-43 | L1 | `probe_slice1_error_types_construct` named "construct" but body is shape-only with `_`-bindings; either add assertions OR rename honestly |
| mod.rs:139-140 | L2 | `SelectOutcome::Recv(usize, Result<T, RecvError>)` — anonymous tuple field; make struct variant |
| mod.rs:70-73 | L2 | `close(self)` ambiguous (close pair? half-close? flush?); needs clarifying doc |
| mod.rs:88-89 | L2 | `CommReceiver::len` has no doc comment |
| mod.rs:107 | L2 | `TryRecvError` doc says "OR" repeating variants; should say WHY callers distinguish |

Spark check: positive (module-level cascade contract docs + blanket-impl rationale praised).

### forge — 3 L1 + 1 candidate-rune

| Line | Level | Observation |
|---|---|---|
| mod.rs:120 | L1 | `CloseError(pub String)` — pub field allows arbitrary external construction |
| mod.rs:127 | L1 | `WireError(pub String)` — same pub-field issue |
| mod.rs:138 | L1 | `SelectOutcome::Recv` bare `usize` — `ReceiverIndex(usize)` newtype would prevent count/offset confusion |
| mod.rs:89 | candidate-rune | `close(self)` consumes; `len(&self)` doesn't — typestate gap |

Well-forged sections noted: HolonRepresentable trait shape; SendError carrying unsent value; TryRecvError enum distinction; SelectOutcome's index-result pairing.

### reap — 1 finding

| Line | Observation |
|---|---|
| probe:40 | `TryRecvError::Disconnected` variant exists but probe only constructs `Empty` — coverage gap |

All other items (CommSender/CommReceiver, close, try_recv, len, CloseError/WireError, SelectOutcome variants, HolonRepresentable) confirmed alive with downstream consumers per DESIGN.md.

### sever — CLEAN

No braided concerns. Four sections properly delineated; cascade contract correctly scoped at module level with trait-level references; no inline domain logic in probe.

## Orchestrator design decisions (judgment calls)

**Decision 1: `close` name** — KEEP (don't rename to `disconnect`)

Four-questions both pass YES; tractability tiebreaker favors established convention (crossbeam/std/widespread); add clarifying doc to address gaze's concern about ambiguity. The doc explains semantic (signal end-of-stream; peer sees Disconnected on next op after ALL clones close).

**Decision 2: close/len typestate gap** — NOT A FINDING

The trait correctly models multi-handle semantics. `close(self)` consumes ONE clone; other Clone'd handles remain valid; `len()` on a still-alive clone is correct behavior. The forge candidate-rune misreads the multi-clone semantics. No change needed; no rune needed.

## Fix pass — orchestrator-direct (7 surgical edits)

Per the new protocol's "orchestrator addresses OR redirects sonnet": for these small mechanical edits + design decisions, orchestrator applied directly (no sonnet round-trip).

| # | Fix | Files |
|---|---|---|
| 1 | `CloseError(pub String)` → private field + `new()` + `message()` accessors | mod.rs:139-151 |
| 2 | `WireError(pub String)` → private field + `new()` + `message()` accessors | mod.rs:158-171 |
| 3 | Mint `ReceiverIndex(pub usize)` newtype; convert `SelectOutcome::Recv` to struct variant `{ index: ReceiverIndex, result: Result<T, RecvError> }` | mod.rs:176-201 |
| 4 | Add doc comment to `CommReceiver::len` | mod.rs:97-99 |
| 5 | Rewrite `TryRecvError` doc to explain WHY callers distinguish (retry-vs-bail-out logic) | mod.rs:107-111 |
| 6 | Add clarifying docs to `close` methods on both traits (semantic: signal end-of-stream; multi-clone behavior) | mod.rs:70-74, 100-104 |
| 7 | Probe: rename to `probe_slice1_error_types_construct_and_distinguish`; add `TryRecvError::Disconnected` construction; add real assertions (`assert_eq!`, `assert_ne!`, `assert!(matches!)`) | probe:35-91 |

Mechanical verification post-fix:
- `cargo build --release` clean (5 pre-existing dead_code warnings; zero new)
- `cargo test --release --test probe_comms_foundation` 3/3 PASS (including renamed test)

## Round 2 — ward re-pass

4 wards spawned in parallel against the fixed files.

### gaze re-pass — CLEAN

> "The five findings from the first pass are gone. No new findings introduced. The code speaks: names are the things, comments carry WHY not WHAT, structure mirrors the tier architecture, and the test file proves the exact distinctions the module doc promises. The spark is present."

Specifically validated:
- `ReceiverIndex` doc (lines 176-182): "nails the distinction"
- `CloseError`/`WireError` private-field rationale: "concise and honest"
- `CommReceiver::len` doc: "a BEWARE/FOR class comment that grounds an otherwise abstract method"
- Test names: "full sentences in imperative form... each tells exactly what it proves"

### forge re-pass — CLEAN

> "All three Level 1 findings from the first pass are addressed without regression. No `rune:forge()` markers present. No new issues introduced."

Hickey lens (values, not places): every item is trait/type/inherent-method; no `&mut self` mutations of shared state; close consumes ownership cleanly.

Beckman lens (types enforce): newtype `ReceiverIndex` prevents count/offset confusion; struct-variant fields prevent positional confusion; CloseError/WireError constructible only via `new(impl Into<String>)`; TryRecvError variants distinct by type.

### reap re-pass — CLEAN

> "No dead thoughts found."

Each new item verified alive:
- `ReceiverIndex` — constructed/matched in probe; future Slice 2/3 Select impls will construct it
- `CloseError`/`WireError` `new()`/`message()` — exercised in probe; mandatory for Slice 2/3 impl surfaces per DESIGN.md
- `SelectOutcome::Shutdown` — constructed + matched in probe; tier Select impls return on cascade
- `CommSender`/`CommReceiver` traits — DESIGN.md confirms Slice 2/3 implementations

First-pass coverage gap closed: `assert_ne!(Empty, Disconnected)` constructs both variants + asserts distinction.

### sever re-pass — CLEAN

> "Concerns remain properly separated after fix pass."

Specifically validated:
- `ReceiverIndex` co-located with `SelectOutcome` under select-outcome section divider (correct — one logical unit)
- `CloseError`/`WireError` impl blocks immediately follow their struct definitions in error-types section
- Doc scopes don't bleed (module-level → trait-level → method-level)
- Probe tests: each covers ONE coherent surface; multi-assertion error test is "a long function that does one thing" per SKILL.md exempt-list

## Verdict

**SLICE 1 IMPECCABLE — all 4 wards clean on re-pass.**

- gaze: code speaks; structure mirrors architecture; spark present
- forge: Hickey + Beckman both nod; types enforce contracts; values flow cleanly
- reap: zero dead thoughts; every item alive with downstream consumer planned
- sever: zero braided concerns; tier architecture is visible in the file structure

The kernel-impeccability protocol's per-slice trust gate fires GREEN: BRIEF scorecard Mode A (17/17 satisfied per sonnet's SCORE) + ward pass CLEAN (all 4 wards green on re-pass).

Slice 1 ready to commit. Slice 2 (thread tier) is the next stone.

## Cross-references

- BRIEF-214-SLICE-1-FOUNDATION-PRIMITIVES.md — work order
- EXPECTATIONS-214-SLICE-1-FOUNDATION-PRIMITIVES.md — 17-row scorecard
- SCORE-214-SLICE-1-FOUNDATION-PRIMITIVES.md — sonnet's Mode A report
- INTERSTITIAL § 2026-05-19 "Kernel impeccability via ward pass" — protocol doctrine
- `feedback_assertion_demands_evidence` — every ward finding is evidence; act on it
- `feedback_any_defect_catastrophic` — kernel defects intolerable; ward findings = defect candidates
