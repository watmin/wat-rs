# DESIGN — STONE H: every wire encoder is strict (stone G's loose ends)

**Drawn 2026-09-24.** Builder: *"let's deal with the loose ends"* — from stone G's report.

Stone G made the three `send`/`try-send` wire arms refuse, at the sender, a value the EDN writer
would render as `opaque_nil`. It did so through one strict encoder, `value_to_wire_edn_string`
(`src/edn/render.rs:~4489`), and `encode_for_wire` (`src/kernel/message.rs:167`). This stone closes
what it left.

## 1. Every OTHER encoder that feeds a wire — census, then strict

All `value_to_edn*` calls outside the writer, classified at `203e2726c`:

| site | wire? | disposition |
|---|---|---|
| `src/kernel/resource.rs:1128` — `after`, process tier: encodes `msg` into a frame `poll'`/`select'` decode as data | **yes** | strict, refuse at `list_span` |
| `src/comms/mod.rs:163` — `impl EdnRepresentable for Value::to_wire`, LENIENT and `None` registry; callers `src/comms/process.rs:336,462` | **unknown** | its own comment: *"anything still reaching here with a record is a plumbing gap."* **Prove unreachable from wat, or make it strict.** |
| `src/services/verbs.rs:347,386` — `println`/`eprintln` | printing | `nil` is correct — ⚠ unless a child's stdout IS its peer channel; confirm |
| `src/kernel/spawn.rs:987` — the `ps` label | no | *"DESCRIBES only, never ROUTES"* |
| `src/runtime.rs:9972` (edn validate), error chains, panic hook, `distribution/`, MCP | no | diagnostics — `opaque_nil` is correct (arc 294) |

⭐ **The class, not the case:** the goal is that NO path that ships bytes a peer decodes as data can
use the lenient writer. Name the invariant in the code where a future hand adding a wire encoder will
meet it; a gate that fails when a wire path uses the lenient writer is the top rung if the material
allows (say so if it does not).

## 2. `try-send` encodes before it knows the tier — a spurious refusal on an in-locus send

Stone G's executor: `try-send` encodes the payload even on the THREAD tier, where the encoding is
thrown away — so a thread-tier `try-send` of an unencodable value (e.g. a holon algebra) raises.
Under `RULING-purity-is-parametric.md`, **a thread peer carries any value.** A refusal there refuses
a legal program. **Measure it first**, then encode only in the socket arm.

## 3. Two writer outputs that are NOT `opaque_nil` — measure the round trip

`HandlePool` renders as a string body, `Value::Vector` with a `{:dim}` body. Strict mode does not
refuse them — correct under "ask the writer" ONLY if they genuinely round-trip over a wire. Measure
each: sent over a process wire, what arrives? If one arrives as something other than itself, the
writer is lying for it, and that is a finding to report (do not silently add it to the strict set).

## Out of scope — accepted as a limitation

- **The refusal names the class only (`:t::Box`), not `Box<Lru>`** — the runtime value does not carry
  its type arguments. Accepted: the message names the offending INNER type (`:rust::cache::Lru`),
  which is what the author needs.
- The receiver's "retired (arc 278 A.0)" wording — now reachable only from a peer outside this
  runtime.
