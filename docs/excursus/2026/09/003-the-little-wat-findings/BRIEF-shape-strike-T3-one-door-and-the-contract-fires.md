# BRIEF — shape strike T3: one decode door, and a wrongly-shaped frame replies `:RequestMalformed`

Excursus 003. Design: `AUDIT-the-shape-of-an-error.md`, § "Strike T + T2 landed" and its § RULING
2026-10-01. This builds on `f24e16037` (T + T2) and `3140de773` (the ruling).

**Read `wat-rs/CLAUDE.md` in full first.** Every citation below is a claim. Verify each one before
relying on it, and STOP on a contradiction. Where the call is small and the intent is clear, make it
and name it.

## The defect, driven (not read)

T2 restored arc 278's `:RequestMalformed` with a **lenient re-decode**. When the strict decode refuses
a frame for its shape, `decode_client_message_event` (`src/kernel/message.rs:~1431`) throws away the
strict error and calls `decode_trusted_wire_lenient`. That is the pre-T decode, carried by a
`strict: bool` threaded through `edn_to_value_caps`, `tagged_to_value` and all four `reconstruct_*`.
The result is handed to the per-op `:wat::edn::validate` guard.

The orchestrator drove it with a temporary probe, which was not kept:

```
wire     #t.edn2.Bag/Op.Put {:req #t.edn2.Bag/PutRequest {:items ["a"] :stray 1}}
strict   Err(UnknownField { type_path: ":t::edn2::Bag::PutRequest", key: "stray" })
event    ServiceEvent::Message
validate sees  #t.edn2.Bag/PutRequest {:items ["a"]}
validate Ok (ACCEPTED)
```

The lenient pass drops the undeclared key, so `validate` never sees it, and the request is
**accepted**. Strike T's hole is open again on the process-tier service wire. The lenient door is
also a second decoder: two answers to one question.

## Target

1. **One decode door.** Delete `decode_trusted_wire_lenient` and the `strict` flag. Every decode is
   T's checked decode.
2. **A wrongly-shaped frame replies the op's `:RequestMalformed`.**
   - **The trigger.** When a client frame's strict decode is refused for shape, the frame's EDN still
     *parsed*: only typed reconstruction failed, so the outer tag (`#<ns>/Op.<Variant>`) is in hand.
     A shape refusal is `is_shape_defect()`: `FieldTypeMismatch`, `UnknownField`,
     `UndeclaredFieldType` or `UnknownStructField`.
   - **The event.** In that case, `decode_client_message_event` produces a new event that carries
     `idx`, the op's variant name, and the refusal's `path`/`expected`/`got`. For example
     `ServiceEvent::RequestMalformed`. Declare it beside `ServiceEvent`'s other variants
     (`wat/spawn.wat:~195`).
   - **The reply.** The serve-loop codegen (`wat/service.wat`, the per-op arms that build
     `shape-guarded`, around `:1790`) handles that event by sending
     `(~reply-variant-kw {:resp (~rm-ctor-kw {:path … :expected … :got …})})` for the named op. That
     is the **same constructor** the `validate` arm already uses: reuse it, do not build a second one.
   - **The server keeps serving**, exactly as the `validate` arm does.
   - **Other failures stay generic.** A frame whose outer tag is not a known op, or that does not
     parse, stays `ServiceEvent::Malformed` → `Reply::Failed` with T2's structured cause.
3. **One rendering of a shape defect.** Today the same defect renders two ways:

   | | path | got |
   |---|---|---|
   | `validate` (`edn_to_typed_value`) | `.items.[0]` | `Integer` |
   | T's refusal | `items.[0]` | `:wat::core::i64` |

   Clients already see `validate`'s format through `:RequestMalformed`, so the contract's format wins.
   Make T's `Mismatch` render through **one shared formatter** that both doors call. Do not
   hand-convert at the reply site.
   - For `UnknownField` and a missing declared field, use whatever rendering `coerce_struct_path`
     (which T taught to refuse undeclared keys) already produces. If it produces none, define one,
     name it, and use it in both doors.
4. **`:wat::edn::validate` stays** (defence in depth, per T's measurement). It now shares the
   formatter.

## Gates (each mutation-proven in RELEASE)

Build them at the Rust unit level, like T2's `excursus_003_t2_gates`: real wire bytes through the
real decoder. There is still no honest raw-socket door, as T2 measured.

- **GT3a, an undeclared key is refused on the wire.** The orchestrator's probe frame above (valid
  `items`, plus `:stray 1`) must yield the `RequestMalformed` event, not `Message`.
  - Mutation: restore a lenient retry. RED.
- **GT3b, a wrong field type gives the contract.** T2's `malformed_put_wire` (`items` given integers)
  must yield the `RequestMalformed` event with exactly the `path`/`expected`/`got` that `validate`
  would have produced for the same value.
  - Assert equality between the two doors' renderings by calling the shared formatter from both
    paths. Do not compare two hard-coded strings.
  - Mutation: break the formatter on one side. RED.
- **GT3c, the codegen replies `:RequestMalformed`.**
  - Drive one real `defservice` serve loop (in-process, with the event injected through the real
    `poll` path if it can be reached honestly) and assert that the client receives `:RequestMalformed`
    and that the server keeps serving a second client.
  - If no honest in-process door reaches the serve loop with a constructed event, say so and prove
    the codegen another way that runs it. Do not fake it.
  - Mutation: send the event to the generic `Reply::Failed` arm. RED.
- **GT3d, one door.** Prove no lenient decode exists. A grep proves nothing, so drive it: T's GT1/GT2
  must still pass, and every decode site goes through one function.
- T2's GT2a/GT2c stay green. **Retire GT2b** (it asserts the lenient retry), and say so in the
  commit.

## Goldens

Expected: none move. If any do, read them and report why.

## Scope fence

- **IN:** items 1–4 and the gates.
- **OUT:** a raw-socket test door (still missing, flagged by T2); B2; C, D, E, F; the stdlib-freeze
  excursus.

## Discipline

- **`cargo nextest run --release` ONLY, never `cargo test`.** One cargo process at a time: check with
  `pgrep -x cargo` and `pgrep -x cargo-nextest`, never `pgrep -f`.
- **Never use a git worktree with a shared `CARGO_TARGET_DIR`.**
- **Never edit files under `tests/`, `wat/` or `wat-scripts/` while any run is in flight.**
- **Never hand back while a build or floor runs.** Wait in the foreground with
  `until grep -qE '^ *Summary' <log>; do sleep 30; done`, repeating past the tool's 10-minute cap.
- The floor:
  - Run `scripts/floor.sh`: 0 failed, 0 timed out.
  - On any red: surface it before any re-run, verbatim, with the arm named.
  - On a timeout: surface it with its history; do not widen `.config/nextest.toml`.
  - **Never commit a red floor.**
- Stage by name BEFORE the floor. Never `git add -A`. No `cargo fmt`.
- Commit prefix `EXCURSUS(003):`, ending with
  `Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>`. Push to
  `origin/reason/little-wat-findings`.
- If your budget runs short, stop at a clean boundary and report where.

## Report

- the event's shape;
- the codegen arm;
- the shared formatter and the `UnknownField`/missing-field rendering;
- GT3c's door, or what was missing;
- each gate's mutation RED;
- the floor `Summary` line, verbatim;
- the SHA.
