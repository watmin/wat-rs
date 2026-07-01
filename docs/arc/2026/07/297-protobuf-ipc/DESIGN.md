# Arc 297 — protobuf-IPC (codec-parameterized IPC + the protobuf bridge)

> **STATUS: STUB (2026-07-01) — planned, not started.** Grew from "remove JSON from wat-edn" — builder: *"that arc evolves
> into protobuf-ipc and removing json support is part of the objective."* Belongs after 296 closes (the derive gives the
> structural serializer this rides on). Pairs 293 R9 *MVNDI CONCVRRVNT* + `296/IDEA-surface-as-schema-protobuf.md`.

## Thesis
Make the IPC wire a **codec** on the process locus — **EDN is the default spine; protobuf is a selectable codec** — add the
**protobuf codec** (a surface IS a `.proto`; record ⇄ protobuf-binary; the same structural walk `#[derive(ToEdn)]` does),
and **prove it with an IPC round-trip** that a foreign `protoc`-generated client speaks *with no EDN parser and without
being a Lisp*. This is the bridge to the polyglot world (293 R9). **Removing JSON from wat-edn is part of the objective** —
once the wire is codec-parameterized and protobuf is the machine/interop face, a JSON codec is a redundant third wire that
falls out.

## Rationale (builder, verbatim)
> *"wat does not need a json library. protobufs are the bridge."*
> *"we prove it with ipc … we extend the process locus to having a codec param."*
> *"that arc evolves into protobuf-ipc and removing json support is part of the objective."*

## Objectives (in order)
1. **Codec-parameterized process locus.** Extend `spawn-process` / the peer model with a **codec param**; the IPC
   encode/decode boundary that today hardcodes EDN (`to_wire_edn` · `src/comms/` wire · the peer `send'`/`recv'` serialize
   point · the process-died path) gains a codec seam. **EDN = default codec.** *(Ground the exact plug-points — spawn opts +
   comms encode/decode + the peer verbs — at arc start.)*
2. **The protobuf codec.** `surface → .proto` (emit a message def from a surface's `:features`) + `.proto → surface`
   (import); `record ⇄ protobuf-binary`. The **purity axis (293.W) = proto-eligibility** (pure ⇒ serializable ⇒
   proto-able; impure struct ⇒ not — the wall we already built). The one genuinely new design: a **field-number policy**
   (protobuf keys on the field NUMBER, wat/EDN on the name — needed for wire identity + schema evolution; a
   `#[to_edn(field = N)]`-style tag or a registry).
3. **The IPC proof.** Two loci (or a wat process + a foreign `protoc` client) round-trip a record over the **protobuf**
   codec — no EDN parser on the foreign side. That demonstration IS the proof of the bridge, and the proof that JSON is
   redundant.
4. **Remove JSON** (subsumed — the small cut the above justifies). Delete **`crates/wat-edn/src/json.rs`** (`edn_to_json`,
   `to_json_string`, `to_json_string_pretty`, `from_json_string` + the serde_json plumbing); the **`lib.rs:85`** re-exports
   + `mod json;`; references in `vocab.rs` / `parser.rs`; retire the consumers — **`src/edn_shim.rs:105,166`** + the CLI
   **`--check-output json`** mode (`edn` stays as the machine face); drop **`serde_json`** from `crates/wat-edn/Cargo.toml`.
5. **(later) gRPC / services.** Map wat's peer/service verbs (`spawn-program'`, the service verbs) to protobuf `service {}`
   + RPC methods. Messages are the easy 80%; RPC is a further layer.

## The CLI is an IPC channel too — `curl | wat --codec protobuf` falls out for free (builder, 2026-07-01)
> *"curl https://some-protobuf | wat --codec protobuf file.wat … if we do it with ipc, curl into stdin is a thing we've
> just proven via ipc?"*

Yes — **stdin/stdout are fds = wires = IPC channels.** The recovery doc's `(stdout, stderr, exit-code)` triangle already
treats them as the process IPC boundary; **stdin is its inbound sibling.** So the codec param needs NO new mechanism for the
CLI — it is the SAME wire and the SAME codec seam as spawn-IPC. **Proving codec-IPC (objective 3) proves the CLI case for
free:**

```
curl https://some.endpoint/thing.pb | wat --codec protobuf file.wat
```
→ `wat` decodes stdin through the protobuf codec; the program receives a **structured record** (satisfying its surface), not
bytes. Encoded output goes back out stdout in the same codec. **wat becomes a protobuf-aware unix filter** — consume a
protobuf HTTP body as a typed record with zero glue, emit one back — the polyglot bridge (293 R9) at the shell.

This makes objective 3's proof concrete + cheap: the IPC round-trip and the `curl | wat --codec protobuf` demo are the same
seam exercised two ways. `--codec` (default `edn`) is the CLI surface of the process-locus codec param.

## Decisions to run at start (four-questions, not leans)
- **Any load-bearing `--check-output json` / `from_json` consumer?** Grep the workspace + the labs; if a real consumer
  exists, decide reroute-to-edn vs keep-a-shim vs the protobuf bridge.
- **The field-number policy** (objective 2) — how field numbers are assigned + stabilized for wire-compat + evolution.
- **Codec selection surface** — where the codec param lives (spawn opts? a peer/connection attribute?) + its default.

## Note on scale
This is BIGGER than a stub's worth of build — objectives 1–3 (the codec seam + protobuf codec + IPC proof) are the arc's
heart; objective 4 (JSON removal) is a clean subordinate cut. Decompose into strikes at arc start (examinare).
