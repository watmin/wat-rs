# Arc 233 — Substrate diagnostic-richness: errors as teaching values

**Status:** ACTIVE (opened 2026-05-23 evening). Strategic pivot from arc 232.

**Driver direction (2026-05-23):**

> *"we believed we had remarkable errors - we don't - we need to raise the bar"*

## Why now (the strategic pivot)

Arc 232 surfaced — over ~30-50 minutes of investigation cost in a single session — that wat's substrate errors LOSE INFORMATION at exactly the moments when richer diagnostics would teach fastest:

- `NotCallable { got: "wat::core::keyword" }` — names the TYPE but loses the VALUE. Required 3 diagnostic probes to figure out what keyword had been attempted.
- Type-mismatch errors with `&'static str` fields — same shape.
- The bracket-syntax trap door (`[-> :T]` invented vs canonical `-> :T`) was a substrate-naming gap; a richer error message could have surfaced "expected `->` symbol, got `Vector(...)` containing..." in one line.

The FM 2-bis probe discipline is partly a **workaround for the diagnostic gap**. We teach ourselves what the substrate should be teaching us.

This tax compounds: every substrate-dev session pays ~30-50 min. The remaining work shape is substrate-heavy (defprotocol → MTG → Truth Engine → trading-lab v2 → wat-MCP horizon). The ROI of fixing the diagnostic layer NOW — before the consumer-side wave hits the gap — is high.

Per [[failure-engineering]] + [[any-defect-catastrophic]]: a structural problem that costs us 30-50 min per substrate session, propagated across N sessions, is a real liability, not polish. We raise the bar.

## What "remarkable errors" means (the target)

The idealized form has four pieces:

1. **Errors are structured values, not formatted strings.** Errors carry typed expected/actual + causal chain + structured context. Rendering is presentation, not identity. Pattern: Clojure `ex-info` + `ex-data`, Rust `source()` chain, Erlang `{error, Reason, Stacktrace}`.

2. **Value snapshots replace `&'static str`.** Where the error names "what was given," it holds a `ValueSnapshot { type_name, rendered, provenance }` — not just `"wat::core::keyword"`. Lazy-render; cheap to construct; honest about content.

3. **Provenance tracking on Values.** Every Value remembers WHERE it came from — literal-source position, symbol-binding chain ("`k` was bound from `:bad` at line 5"), or runtime-builder ("from `keyword/from-string s` at line 12 where `s` = …"). This is the WHO of the diagnostic.

4. **Errors-as-EDN wire protocol.** Structured errors flow over IPC boundaries (cross-thread, cross-process, cross-language) as EDN, not strings. Tools consume errors as data. Seeded by arc 211b panic-as-EDN; extend to all `RuntimeError` variants.

## Stone plan (three substrate stones + INSCRIPTION)

| Stone | Purpose | Size | Status |
|---|---|---|---|
| 233.1 | **ValueSnapshot** — mint `ValueSnapshot { type_name, rendered, provenance: Provenance::Unknown }` type. Sweep `RuntimeError` variants: promote `got` / `expected` / similar `&'static str` fields → `ValueSnapshot`. Update construction sites to render the actual Value at error-construction time. Provenance always `Unknown` for v1 (no tracking yet); the `rendered` field alone closes the inline-value gap. | medium (mechanical sweep across many sites) | PENDING |
| 233.2 | **Provenance on Values** — extend `Value` (or shadow via parallel structure) with `Provenance` tracking. Every Value-construction site attaches provenance: `Literal { span }` / `SymbolBound { binding_site, head_site }` / `RuntimeBuilt { producer, head_site }` / etc. Threads through eval, let-binding, `keyword/from-string`, EDN reader, mailbox payloads. ValueSnapshot's provenance field fills with real data. | LARGE (semantic substrate change; whole-Value-surface) | BLOCKED on 233.1 |
| 233.3 | **Errors-as-EDN extension** — generalize arc 211b's AssertionPayload pattern across all `RuntimeError` variants. Errors serialize to EDN over IPC boundaries (cross-thread, cross-process, cross-language). Tools consume as data. Aligns with arc 092 wat-edn + arc 217 Clojure-IPC. | small-medium (extends existing seed) | independent of 233.2 (can ship after 233.1) |
| 233.4 | INSCRIPTION + USER-GUIDE chapter + cross-references | paperwork | blocked on 233.1 + 233.2 + 233.3 |

### Stone ordering rationale

- **233.1 first** — highest immediate payoff. ValueSnapshot closes the immediate frustration (we'd have caught the `NotCallable` and bracket-syntax issues faster). Mechanical sweep; calibratable scope.
- **233.2 second** — the load-bearing substrate change. Provenance tracking is the structural addition that turns the diagnostic into a teaching artifact. ValueSnapshot WITHOUT Provenance is "include the rendered value" — useful but limited. Provenance fills in the WHO.
- **233.3 third or parallel** — consumer-tooling enabler. Wire-protocol extension. Builds on existing arc 211b seed; doesn't gate the substrate-layer work.

Each stone is separable, honestly-bounded, independently shippable in some order (233.2 strictly after 233.1; 233.3 anywhere after 233.1).

## Out of scope for arc 233

- Logging / structured logging frameworks (separate concern; these are errors, not logs)
- Tracing spans (separate concern; arc 091 / wat-telemetry already covers timing-spans)
- Error recovery patterns (`try/catch` etc.; arc 108 typed-expect already covers Result-unwrap)
- Stack trace rendering format (the substrate already has arc 113 cross-thread backtrace; 233 augments per-error, not the trace shape itself)
- Performance: the snapshot/provenance overhead is part of the design; if performance becomes load-bearing in a hot path, the v2 design adds a "production mode" that disables provenance tracking. Not v1.

## Predecessors

- arc 064 — assert-eq renders values + surfaces location (precedent for value-render in diagnostics)
- arc 113 — cascading runtime error messages (precedent for causal chain awareness)
- arc 116 — phenomenal cargo debugging (Failure → Diagnostic; precedent for printing infrastructure)
- arc 138 — errors carry point-in-code coordinates (precedent for spans on every error)
- arc 211b — panic-as-EDN (the AssertionPayload EDN serializer; seed for 233.3)
- arc 217 — Clojure-IPC bridge (consumer of structured errors over EDN)
- arc 092 — wat-edn v4 minting (the EDN serialization substrate)

## Relationship to arc 232

Arc 232 (defprotocol + extend-type) is PAUSED at Stone 232.0a. Stone 232.0a substrate work has NOT shipped — only the probe + DESIGN are committed.

When arc 233 completes (or completes 233.1 at minimum):
- Arc 232 resumes from Stone 232.0a
- defprotocol BRIEF (Stone 232.1) gets authored against richer diagnostics
- defprotocol's own dev cycle benefits from improved error messages (consumer-side validation of arc 233's substrate work)

The pivot is strategic: build defprotocol with the new diagnostic substrate in place rather than retrofit later.

## Cross-references

- [[failure-engineering]] — the doctrine arc 233 enforces
- [[any-defect-catastrophic]] — the discipline that justifies the pivot
- [[substrate-as-teacher]] — the doctrine arc 233 raises the bar on
- [[wat-llm-first-design]] — LLM co-authors need structured errors; this arc makes wat better for its target audience
- `docs/arc/2026/04/109-kill-std/INVENTORY.md` § O — the backlog entry this arc takes the work from
- `docs/arc/2026/05/232-defprotocol-extend-type/DESIGN.md` — paused arc; resumes after 233 lands
- `docs/SUBSTRATE-AS-TEACHER.md` — the canonical doctrine doc

## Trap-door audit lessons (carry forward from arc 232)

Per `feedback_sonnet_writes_substrate` + FM 2-bis:

- ValueSnapshot's shape needs an empirical probe BEFORE BRIEF (verify the rendering path works for primitives, Vec, fn, Bind, etc.)
- Provenance's struct shape needs design dialogue + a probe showing it composes through let-bindings (the hardest case)
- Errors-as-EDN needs a probe demonstrating round-trip through wat-edn

Every BRIEF cites: existing primitive signatures verbatim, canonical inline `-> :T` syntax (no `[-> :T]` brackets), grep evidence for every named primitive.

Orchestrator does NOT edit substrate code directly. Sonnet writes; orchestrator briefs + scores + commits. The protocol is the proof of communication.
