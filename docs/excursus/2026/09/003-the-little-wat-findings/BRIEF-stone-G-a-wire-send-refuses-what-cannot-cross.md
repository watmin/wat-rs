# BRIEF — STONE G: a wire `send` refuses a value that cannot cross, at the sender

Read `DESIGN-stone-G-a-wire-send-refuses-what-cannot-cross.md` first.

## First — measure what the SENDER sees today

The orchestrator's probe discarded the child's `SendOutcome`. Before changing anything, capture it for
the handle case (e.g. have the child send the outcome's variant name back, or write it to a file).
That is the "before" row.

## The work

1. Find every arm of `send` / `try-send` that encodes a value to EDN for a WIRE peer
   (`src/kernel/message.rs`; the child's `self-peer` type path may differ from
   `PROCESS_PEER_TYPE_PATH`). Report the list.
2. Give the encoder a STRICT mode in which reaching `opaque_nil` is an error, not a `nil`. The set
   must be the writer's own — `src/edn/render.rs` (`opaque_nil`, ~`:4673`), with
   `crate::capability::encode_capability` deciding which opaques DO cross.
3. The wire arms use strict mode and, on refusal, RAISE a wat runtime error with `list_span` (the
   user's call), naming the op, that the value is not pure, and the offending type.
4. Every non-wire encode path is UNCHANGED.

## Rooms

- `src/kernel/message.rs:~185-260` — the `send` arms; note the timer arm's misuse-raises precedent.
- `src/edn/render.rs` — `value_to_edn_with`, `opaque_nil` (~`:4673`), and the recv-side decode error
  at `:3587`.
- `src/capability/` — `encode_capability`, for the opaques that legitimately cross.
- The spawned-process shape: `tests/comms/probe_arc272_6c2_record_ipc_derisk.{rs,wat}`.

## STOP triggers

1. **A strict mode cannot be added without changing a non-wire caller's output.** Report the caller.
2. **A legitimate wire send in the corpus carries an opaque the writer nils** (the floor will say).
   Report it — do not widen the strict set to make it pass.
3. **The raise's `:location` is a `.rs` file.** The span must be `list_span`; report if it is not.
4. **Any gate reddens that you did not add** — capture whole, name the arm. ⛔ Do not re-run first.

## Prove it

A probe modelled on `probe_arc272_6c2`: (a) generic fn, T = `Lru` → the CHILD's send raises a wat
error at the user's span, naming the type; (b) T = `i64` → received whole, `#t/Box {:x 42}`; (c) a
Wire `Address` crossing as before (positive control for `encode_capability`); (d) `:wat::edn::write`
of a handle outside the wire still prints `nil` (arc 294, unchanged).
**Mutation:** make the wire path non-strict again → (a) goes back to a receiver-side `Lost`.

## Mechanics — ⛔ read

- Floor: `nohup scripts/floor.sh > <file> 2>&1 &`, then repeated foreground
  `until grep -qE '^ *Summary' <that file>; do sleep 30; done` blocks. Do not end your turn while it
  runs. Never `pgrep -f 'cargo …'`. Read `Summary` lines, never piped exit codes.
- `cargo fmt` reformats the whole workspace — `rustfmt <file>` or neither.
- `git add` BEFORE `git ls-files`-based gates; new `tests/` files answer to `no_loose_string_assert`,
  `no_inlined_edn`, `no_inlined_wat_in_tests`, `every_tracked_wat_parses`,
  `every_wat_bad_fixture_actually_fails`.
- `:wat::core::the` does not exist, and `concat` is not for strings — both bit the orchestrator's probe.
