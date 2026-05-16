# Arc 170 Slice 4c-α-i BRIEF — migrate readln-echo to Layer 2 (run-hermetic-with-io)

**Task:** #319
**Phase:** Slice 4c-α first sub-stone of the 4c-α decomposition (i → ii → iii → iv).
**Predecessors:** All of 4a-α (mint) + 4a-β (sweep) + 4a-γ chain (audit + decorate + flip) shipped. The deftest macro now defaults to run-thread; hermetic semantics opt-in via `:wat::test::deftest-hermetic`. One Layer-2 escalation deferred from 4a-β: `wat-tests/kernel/services/ambient-stdio.wat:110` (readln-echo helper using `:wat::test::run-hermetic-ast` with stdin pre-seed for stdin-driven readln). This slice migrates that single site to Layer 2.

## Goal

Migrate the `:test::run-readln-echo` helper (defined inside the `:deftest-ambient` make-deftest-hermetic prelude at `wat-tests/kernel/services/ambient-stdio.wat:108-117`) AND its `:deftest-ambient` consumer at lines 183-187 from the legacy `:wat::test::run-hermetic-ast` to the modern `:wat::test::run-hermetic-with-io` (Layer 2). After this slice: zero wat-level callers of `:wat::test::run-hermetic-ast` (the legacy define survives in test.wat with zero callers; deleted in 4c-α-iv).

## Current shape (lines 108-117 + 183-187)

```scheme
;; HELPER (inside the :deftest-ambient prelude)
(:wat::core::define
  (:test::run-readln-echo -> :wat::kernel::RunResult)
  (:wat::test::run-hermetic-ast
    (:wat::test::program
      (:wat::core::define
        (:user::main -> :wat::core::nil)
        (:wat::core::let
          [echoed (:wat::kernel::readln -> :wat::core::String)]
          (:wat::kernel::println echoed))))
    (:wat::core::Vector :wat::core::String "\"echo me\"" "")))

;; CONSUMER deftest (Layer 4)
(:wat::test::time-limit "15000ms")
(:deftest-ambient :wat-rs::test::test-ambient-stdio-readln-echo
  (:wat::test::assert-stdout-is
    (:test::run-readln-echo)
    (:wat::core::Vector :wat::core::String "\"echo me\"")))
```

The legacy form's stdin Vec has TWO elements (`"\"echo me\""` + `""`) so `(string::join "\n" stdin)` produces a trailing newline — the substrate's IOReader/read-line in the stdin service blocks until `\n` arrives.

## Target shape (Layer 2)

```scheme
;; HELPER (inside the :deftest-ambient prelude)
(:wat::core::define
  (:test::run-readln-echo -> :wat::test::RunResultIO<wat::core::String>)
  (:wat::test::run-hermetic-with-io
    :wat::core::String                                  ;; input-type
    :wat::core::String                                  ;; output-type
    (:wat::core::Vector :wat::core::String "echo me")   ;; native String input (Sender/from-pipe handles EDN encoding)
    (:wat::core::let
      [echoed (:wat::kernel::readln -> :wat::core::String)]
      (:wat::kernel::println echoed))))

;; CONSUMER deftest (Layer 4) — assertion shape changes
(:wat::test::time-limit "15000ms")
(:deftest-ambient :wat-rs::test::test-ambient-stdio-readln-echo
  (:wat::test::assert-eq
    (:wat::test::RunResultIO/outputs (:test::run-readln-echo))
    (:wat::core::Vector :wat::core::String "echo me")))
```

## What changes

1. **Helper return type:** `:wat::kernel::RunResult` → `:wat::test::RunResultIO<wat::core::String>`
2. **Spawn macro:** `(:wat::test::run-hermetic-ast (:wat::test::program (define :user::main ...)) stdin-Vec)` → `(:wat::test::run-hermetic-with-io :I :O inputs body)`
3. **Wire format:** stdin pre-seed `(Vec :String "\"echo me\"" "")` (raw EDN string with trailing newline hack) → typed inputs `(Vec :String "echo me")` (native String; Layer 2's Sender/from-pipe wraps Process/stdin and EDN-encodes the typed value)
4. **Body unwraps:** the legacy `(:wat::test::program (define :user::main ...))` wrapper RETIRES; the body becomes the direct let-expression that `run-hermetic-with-io` wraps in `(fn [] -> :nil ...)` at expansion time
5. **Assertion:** `(:wat::test::assert-stdout-is r (Vec :String "\"echo me\""))` → `(:wat::test::assert-eq (:wat::test::RunResultIO/outputs r) (Vec :String "echo me"))`. The native String comes back (Layer 2's Receiver/from-pipe decodes the EDN wire form), so the expected vec uses native `"echo me"` (no EDN quotes).

## Why test purpose is preserved

The test's purpose is to verify the ambient-stdio readln → println round-trip through the trio services in a forked child. The BODY of the inner program is unchanged: still calls `(:wat::kernel::readln -> :wat::core::String)` and `(:wat::kernel::println echoed)`. The trio orchestrator still routes the calls through the bootstrap services. The hermetic OS-process boundary still applies (`run-hermetic-with-io` uses spawn-process underneath, same as `run-hermetic` Layer 1).

What changes is the WIRE-FORMAT BOUNDARY between parent and child:
- **Before:** parent constructs raw EDN string + trailing newline; child's `readln` parses it from fd 0
- **After:** parent passes native typed value; Layer 2's Sender/from-pipe EDN-encodes onto fd 0; child's `readln` parses from fd 0 (same code path); child's `println` EDN-encodes onto fd 1; Layer 2's Receiver/from-pipe decodes back to native typed value

The test continues to exercise:
- Ambient `readln` reading from stdin via the StdInService
- Ambient `println` writing to stdout via the StdOutService
- The trio orchestrator's routing logic
- The hermetic OS-process boundary (fork + pipe + EDN marshalling)

What it DROPS exercising:
- The Layer 0 "raw string with trailing newline hack" wire format (which was a legacy substrate detail per the existing comment)

This is a substrate-honest improvement: the typed channels are the modern wire format; the test now uses them.

## Substrate edits — NONE

No `src/` Rust changes. No edits to deftest macro / run-hermetic / run-hermetic-with-io / spawn-process / etc. Pure consumer migration in `wat-tests/kernel/services/ambient-stdio.wat`.

## Constraints (HARD)

- DO NOT commit. Orchestrator commits atomically after independent verification.
- DO NOT edit any file other than `wat-tests/kernel/services/ambient-stdio.wat`.
- DO NOT touch the `:wat::test::run-hermetic-ast` define in `wat/test.wat` — that lives until 4c-α-iv.
- DO NOT touch the 4 other helpers in the `:deftest-ambient` prelude (`:test::run-println-string`, `:test::run-println-i64`, `:test::run-eprintln-string`, `:test::run-println-twice`) — they use Layer 1 `:wat::test::run-hermetic` correctly per 4a-β.
- DO NOT touch the 4 other deftest consumers (Layer 0-3) — they're correct as-is.
- DO NOT touch INSCRIPTIONs / past SCOREs / DEFERRAL-VIOLATIONS / SUPERSEDED BRIEFs / AUDIT.

## Scorecard (5 rows, YES/NO with grep/build/test evidence)

| Row | What | Evidence |
|-----|------|----------|
| A | `wat-tests/kernel/services/ambient-stdio.wat` no longer contains `:wat::test::run-hermetic-ast` | `grep -n "run-hermetic-ast" wat-tests/kernel/services/ambient-stdio.wat` returns 0 lines (or only comment references) |
| B | The readln-echo helper uses `:wat::test::run-hermetic-with-io` with native `"echo me"` input | grep + visual confirms the new shape per § "Target shape" above |
| C | The Layer 4 deftest consumer reads `RunResultIO/outputs` (not `stdout`) | grep `RunResultIO/outputs` returns at least one match in this file |
| D | `cargo build --release --workspace --tests` clean | build output Finished, zero errors |
| E | The `test-ambient-stdio-readln-echo` test passes; workspace failure count ≤ post-flip baseline (2 failed) | cargo test confirms `deftest_wat_rs_test_test_ambient_stdio_readln_echo ... ok`; total failures ≤ 2 (or in rotation band ≤ 11 to account for variance) |

## STOP triggers

- Build fails after the migration → STOP; the Layer 2 shape isn't matching the macro signature; surface the error.
- The readln-echo test fails post-migration → STOP; the wire-format shift broke the round-trip; investigate whether the EDN encoding/decoding works as expected.
- Workspace failure count regresses significantly (>11) → STOP; surface regression class.

## Update the doc-comment too

The current comment block at lines 87-107 describes the legacy Layer 0 wire-format hack ("Test seeds stdin with one EDN line... The stdin vec uses TWO elements..."). After migration, replace this with a description of the Layer 2 wire format: parent passes native typed value; Layer 2's Sender/from-pipe wraps stdin for EDN-over-pipe semantics; child's readln/println still operates on ambient stdio routed through the trio services; the EDN encoding/decoding is symmetric at the channel boundary.

Also update the layer doc-comment at lines 29 ("Layer 4 :test::run-readln-echo → run-hermetic-ast { readln → println }") to reflect `run-hermetic-with-io`.

Also update lines 13-14 ("Each helper uses :wat::test::run-hermetic-ast which forks a child via :wat::kernel::fork-program-ast") — that's now stale (Layer 0-3 already migrated to `run-hermetic` Layer 1 in 4a-β; Layer 4 migrates here). Rewrite to reflect the post-migration mixed state: Layer 0-3 use `:wat::test::run-hermetic` (Layer 1); Layer 4 uses `:wat::test::run-hermetic-with-io` (Layer 2).

## Implementation protocol

1. Verify cwd + tip + clean working tree.
2. Read the current `wat-tests/kernel/services/ambient-stdio.wat` fully (188 lines).
3. Make the helper migration (lines 108-117 area).
4. Make the consumer migration (lines 183-187 area).
5. Update the doc-comments (lines 13-14, line 29, lines 87-107).
6. Build + test. If green, write SCORE.
7. STOP at first red.

## Time-box

Predicted 15-30 min orchestrator-direct OR sonnet-spawn (this slice is a small redesign — could go either way). EXPECTATIONS will set the band.

## On completion

Write `SCORE-SLICE-4C-ALPHA-I-LAYER2-READLN-ECHO.md`. 5 rows YES/NO with evidence. Honest deltas — especially any substrate behavior surfaced by the wire-format shift (the typed EDN-over-pipe encoding/decoding through Sender/from-pipe / Receiver/from-pipe).

After this slice: zero wat-level callers of `:wat::test::run-hermetic-ast`. The legacy define in `wat/test.wat:253` still exists but is orphan; 4c-α-iv deletes it atomically with the other 2 wrappers + sandbox.wat + hermetic.wat (once the Rust-side migrations in 4c-α-ii + 4c-α-iii also clear).
