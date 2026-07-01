# IDEA (for later — builder, 2026-07-01) — the surface kit ≈ protobuf, with EDN as the wire

> Parked mid-296. NOT current work. Builder's question: *"did we just make protobuf-as-edn very simple? our surface
> record requirement — it can be parsed to a protobuf ref?"*

## The observation
The 293/294 surface kit + the 296 `#[derive(ToEdn)]` are structurally the same machine protobuf is:

| protobuf | wat |
|---|---|
| a `message` schema (`.proto`) — named typed fields | a **surface** (`defsurface` — a named set of `field <- :Type`) |
| a message instance | a **record** satisfying the surface |
| generated serialize/deserialize | **`#[derive(ToEdn)]`** (structural, by construction) |
| the wire (tag-numbered binary) | **EDN** (self-describing, tagged) |
| `repeated` | `Vector<T>` |
| `oneof` | an enum / sum type |
| `optional`/presence | `Option<T>` |

So a **surface is a schema**, a **record is a message**, and the derive is the codegen — we built all three for the error
work. Protobuf interop would be a **new backend on the same kit**, not new architecture:
- **surface → `.proto`** (emit a message def from a surface's `:features`), and **`.proto` → surface** (import a proto as
  a wat surface). The "surface record requirement" the builder means = the field contract = the message schema.
- **record ⇄ protobuf-binary** as an alternate wire codec beside the EDN one (the same structural walk the derive does).

## The wrinkles (why it's *simple*, not *trivial*)
- **Field numbers.** protobuf's wire identity is the field NUMBER (tag), not the name; wat/EDN keys on the name. A
  surface→proto mapping must assign/stabilize field numbers (a `#[to_edn(...)]`-style `field = N` attribute, or a
  registry) for wire-compat + forward/backward evolution. This is the one genuinely new concern.
- **Scalar type set** (proto's int32/int64/sint/fixed/…): a mapping table; wat's scalars are fewer.
- **Presence/default semantics** differ (proto3 no-presence vs Option).

## Why it's cheap now
The hard parts — structural typing (surfaces), a schema contract (satisfaction), a derive-generated structural
serializer (`#[derive(ToEdn)]`), one canonical field walk — are DONE for 296. A protobuf codec/`.proto` bridge is "a
second wire + a field-number policy" over the existing kit. Pairs the clj↔wat bridge vision (EDN as the spine; a proto
face is another head). Revisit after 296 closes.

## Facets the builder pushed on (2026-07-01) — this is the bridge to the rest of the world

- **Perf.** protobuf binary (varint + field-tags) is far smaller + faster to parse than EDN text. The SAME record now has
  three faces: `Display` (human), EDN (self-describing/holon spine), **protobuf (perf + interop)**. High-throughput IPC /
  cross-service uses the binary face; nothing else changes.
- **Vend clients that are not wat and not a Lisp.** A `.proto` + `protoc` generates native client code in ~20+ languages.
  A Go / Java / Python / C++ service consuming a wat service needs ONLY the `.proto` — it speaks protobuf, never touches
  EDN, never knows wat is a Lisp. **The surface IS the `.proto`.** This decouples wat's INTERNAL model (Lisp, EDN, holon)
  from its EXTERNAL contract (a language-agnostic schema). The polyglot world consumes wat without adopting any of it.
- **The purity axis ALREADY defines proto-eligibility.** 293.W: a PURE record crosses the wire; an IMPURE struct (holds a
  resource, in-locus) cannot. That is EXACTLY "what can be a protobuf message." The purity wall we built for the wire is
  the schema-eligibility gate for free — pure ⇒ serializable ⇒ proto-able; impure ⇒ not. No new boundary needed.
- **The further layer (honest scope).** Message serialization is the easy 80%. Full protobuf-RPC (gRPC) means mapping
  wat's peer/service model (`spawn-program'`, the service verbs) to protobuf `service {}` + RPC methods — a real added
  design, not just a codec. Messages first; services later.

**Reframed bridge vision:** EDN is the spine, a Clojure head is one face, and **a protobuf face is the door to every other
language** — built almost accidentally by the surface kit + the purity axis + the 296 derive. Broader than the Lisp-family
"clj↔wat" framing ([[project_clj_wat_bridge_vision]]) — the bridge is polyglot.
