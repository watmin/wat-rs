# Arc 189 — wat-edn streaming (maybe a sub-arc of 188)

**Status:** stub opened 2026-05-13 per user direction.
**Gates on:** arc 188 (perf + Rust impl scrutiny) — or ships as sub-concern of 188.

## Motivation

> *"maybe yet another arc for making wat-edn faster with streaming (we skipped it because it was good enough to move forward)"*

wat-edn was minted with whole-document parse/render semantics — fast enough for arc 092's needs at the time. Streaming was deferred as not-load-bearing for moving forward.

When arc 188 (perf scrutiny) sweeps wat-edn, streaming may surface as the highest-value remaining win:

- **Parse-time**: token-by-token streaming for large EDN documents (logs, telemetry batches, IPC payloads exceeding pipe buffer size)
- **Render-time**: chunked emit so producers don't materialize the whole document before write
- **Memory**: peak-resident drops for processing pipelines that don't need the whole document in memory simultaneously

If 188's profiling shows wat-edn's non-streaming shape is the bottleneck for any consumer (telemetry sqlite reader, IPC-over-pipes for spawn-process, etc.), 189 ships the streaming variant.

## Sketch

- **API**: `wat_edn::stream::Reader<R: Read>` yields `WatAST` per call; `wat_edn::stream::Writer<W: Write>` consumes `WatAST` per call without buffering the whole document
- **Compatibility**: keep the existing whole-document API; streaming is additive (consumers opt in)
- **Benchmark suite**: representative documents (small/medium/large) measure parse + render throughput before/after; peak-resident memory tracked separately
- **Re-test EDN roundtrip suite**: streaming must preserve byte-for-byte identity for the existing roundtrip tests

## Why "maybe"

If arc 188's profiling shows wat-edn's whole-document shape isn't a hot bottleneck for any consumer, this arc remains a stub. The streaming variant is real engineering but only ships if measured demand justifies the maintenance surface of a second API. Substrate-as-teacher applies — wait for the profiler to surface the case.

## Cross-references

- Arc 188 (perf scrutiny) — gates this arc; may fold this work in directly
- Arc 092 (wat-edn v4 minting + roundtrip test) — the original arc
- `crates/wat-edn/` — implementation
