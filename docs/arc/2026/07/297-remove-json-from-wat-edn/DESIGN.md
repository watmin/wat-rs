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

## The proof + the mechanism (builder, 2026-07-01): codec-parameterized IPC
> *"we prove it with ipc … we extend the process locus to having a codec param."*

The way we PROVE JSON is unneeded — and that protobuf is the real bridge — is not by argument but by **IPC**: extend the
**process locus** (the `spawn-process` / peer model) with a **codec param**, so the wire is a CHOICE, not hardcoded EDN.

- **The wire becomes pluggable.** A locus is spawned with a codec; its IPC send/recv encode/decode through THAT codec.
  **EDN is the default codec** (the self-describing spine); **protobuf is a selectable codec** (293 R9 — a surface is a
  `.proto`, purity = eligibility). The encode/decode boundary that today hardcodes EDN (`to_wire_edn` / the comms wire in
  `src/comms/` + the process-died path; the peer `send'`/`recv'` serialize point) gains a codec seam. *(Ground the exact
  plug-point — spawn opts + the comms encode/decode + the peer verbs — at arc start.)*
- **The proof is an IPC round-trip over the non-EDN codec.** Two loci (or a wat process + a foreign `protoc`-generated
  client) round-trip a record with the **protobuf** codec selected — no EDN parser on the foreign side. That demonstration
  IS the proof: once the wire is codec-parameterized and protobuf rides IPC cleanly, **JSON is a demonstrably redundant
  third codec** — nobody's bridge, nobody's spine — and 297 removes it.

**Scope note:** the codec-param + protobuf codec is BIGGER than "remove JSON" and likely its own arc (the protobuf-bridge
build); 297 (remove JSON) is the SMALL cut that the codec-param work justifies + subsumes. Sequence: codec seam →
protobuf codec proven over IPC → JSON removed (it was only ever a stand-in for "a machine face," which protobuf now is).

## Pairs
- 293 R9 *MVNDI CONCVRRVNT* (the surface kit is the schema; protobuf is the bridge) · `296/IDEA-surface-as-schema-protobuf.md`.
- Sits atop the 296 diagnostics work (`--check-output edn` is the surviving machine face).
