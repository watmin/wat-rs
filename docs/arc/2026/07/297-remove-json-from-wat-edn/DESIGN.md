# Arc 297 — Remove JSON from wat-edn

> **STATUS: STUB (2026-07-01) — planned, not started.** Opened while building 296; belongs after 296 closes (or whenever).

## Thesis
Remove the JSON serialization layer from `wat-edn`. EDN is the self-describing wire; **protobuf** is the interop/perf
bridge to the polyglot world (293 R9 *MVNDI CONCVRRVNT*, `296/IDEA-surface-as-schema-protobuf.md`). A JSON codec is a
third, redundant, lossy-by-comparison wire we do not need to own.

## Rationale (builder, verbatim)
> *"wat does not need a json library. protobufs are the bridge."*

## Scope (grounded 2026-07-01)
Delete the JSON module + its exports + its consumers:
- **`crates/wat-edn/src/json.rs`** — the whole module: `edn_to_json`, `to_json_string`, `to_json_string_pretty`,
  `from_json_string` (+ the `JV`/serde_json plumbing).
- **`crates/wat-edn/src/lib.rs:85`** — the `from_json_string, to_json_string, to_json_string_pretty` re-exports (+ the `mod
  json;` declaration); references in `vocab.rs` / `parser.rs`.
- **Consumers (must be retired/rerouted first):**
  - `src/edn_shim.rs:105,166` — uses of `to_json_string` (find what they feed; reroute to EDN or delete).
  - **The CLI `--check-output json` mode** (`crates/wat-cli/src/lib.rs` — arc 296 wired `--check-output json →
    wat_edn::to_json_string`). Retire the `json` output mode; `--check-output edn` is the machine face. (Or, if a
    machine-readable-for-non-wat face is wanted, that is the protobuf bridge, not JSON.)
  - Any `--check-output json` / `from_json` tests (`crates/wat-cli/tests/wat_cli.rs`, others) — updated/removed.
- **`crates/wat-edn/Cargo.toml`** — drop the `serde_json` dependency (a nice side benefit: one fewer dep) once nothing uses it.

## Decision to run at start (four-questions)
- Is any `--check-output json` consumer load-bearing enough that dropping it is a breaking change users rely on? Grep the
  workspace + the labs; if a real consumer exists, decide reroute-to-edn vs keep-a-shim vs the protobuf bridge — via the
  four-questions, not a lean.

## Pairs
- 293 R9 *MVNDI CONCVRRVNT* (the surface kit is the schema; protobuf is the bridge) · `296/IDEA-surface-as-schema-protobuf.md`.
- Sits atop the 296 diagnostics work (`--check-output edn` is the surviving machine face).
