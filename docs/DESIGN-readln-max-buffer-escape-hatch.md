# DESIGN — readln `:max-buffer-bytes` escape hatch (prime + macro)

**STRIKE-READY.** RED probe verified: `tests/nursery/probe_readln_max_buffer_kwarg.rs` —
`readln_max_buffer_bytes_kwarg_type_checks` FAILS at HEAD (kwarg shape doesn't check),
`readln_plain_form_still_type_checks` PASSES (backward-compat baseline).

## Why
The value-framing accumulator caps at `DEFAULT_MAX_FRAME_BYTES` (512 KiB, committed `49cbe8ee`).
A caller with a legitimately larger single message needs an opt-in. Per doctrine, the opt-in surface
is **kwargs, and kwargs is ALWAYS a macro** over a lean positional primitive.

## The surface (locked, builder co-design)
```
(:wat::kernel::readln -> :T)                                  ;; unchanged → 512 KiB default
(:wat::kernel::readln :max-buffer-bytes (* 2 1024 1024) -> :T) ;; opt-in → 2 MiB cap
```
The unit is in the name (`-bytes`); the value is plain i64 arithmetic (no `2-gb` literal). Backward
compatible: existing `(readln -> :T)` is untouched.

## The architecture — the prime convention (recv'/send'/select' lineage)
- **`:wat::kernel::readln'`** — the kernel-restricted positional **primitive**, `#[restricted_to(
  ":wat::kernel::readln'", ":wat::kernel::")]` (model: `spawn-thread'`/`close'`). Optional leading
  max-bytes so the DEFAULT lives in **exactly one place** (here), never duplicated into the macro:
  - `(readln' -> :T)`        → cap = `DEFAULT_MAX_FRAME_BYTES`
  - `(readln' <i64> -> :T)`  → cap = `<i64>`
  Recast from the current `eval_kernel_readln` (the existing readln intrinsic body).
- **`:wat::kernel::readln`** — the exposed interface = a **`defmacro`** wrapping readln':
  - `(readln -> :T)`                        → `(readln' -> :T)`
  - `(readln :max-buffer-bytes N -> :T)`    → `(readln' N -> :T)`
  Must **forward the `-> :T` annotation** intact (the checker infers readln's polymorphic return from
  the call-site arrow — see the readln scheme note in check.rs ~17291). Model the macro on the
  arg-handling macros in `wat/core.wat` (`format`, `cond`, `->`).

## Plumbing the cap to the reader
`readln'` carries the chosen cap through the StdInService round-trip to the frame reader:
```
readln'(cap) → StdInService::Req {thread-id, max-buffer-bytes}   ;; Req gains the field
            → StdInService/handle → (:wat::io::IOReader/read-frame in cap)
            → read_framed_edn(..., cap)                          ;; the param committed in 49cbe8ee
```
- **`wat/kernel/services/stdin.wat`**: `StdInService::Req` gains `max-buffer-bytes <- :wat::core::i64`;
  `StdInService/handle` reads it off the Req and passes it to `read-frame`.
- **`src/io.rs` `eval_ioreader_read_frame`**: accept a positional max-bytes (`(read-frame in <i64>)`),
  passing it to `read_framed_edn` instead of the hardcoded `DEFAULT_MAX_FRAME_BYTES`. (The default now
  enters via readln' → Req → here, so read-frame becomes 2-arg; it has exactly one caller — the
  StdInService handle — so this is a clean signature change. Confirm by grep.)

## STOP triggers (surface, don't improvise)
1. If the `readln` macro cannot forward `-> :T` cleanly through the macro-eval engine (the polymorphic
   return stops inferring) — STOP and report. This is THE wrinkle; get it right or surface it.
2. If recasting `readln` → `readln'` breaks existing readln callers (the ambient-stdio tests) for any
   reason beyond the macro indirection — STOP and report.
3. If `read-frame` turns out to have more than the one StdInService caller — STOP, report (the
   signature change needs all of them).

## Gate (run these yourself — you HAVE permission to run cargo wat)
1. `cargo build --release --bin wat` — clean.
2. `cargo test --release -p wat --test nursery readln_max_buffer` — BOTH probes green (kwarg + plain).
3. **Restriction test** (add it): a user-namespace fn calling `:wat::kernel::readln'` directly must be
   a `restricted-to` error (not unresolved). Model on an existing restricted-to probe.
4. The existing ambient-stdio readln path still works (single-line readln) — run the relevant
   `wat-tests` / nursery stdin tests; a single line is a Complete frame, so it must still read.
5. `cargo test --release -p wat --lib` → `953 passed; 36 failed; 1 ignored` (identical baseline).
6. `cargo test --release -p wat --test nursery` → no NEW failures beyond the 4 pre-existing RED-by-design.
7. `cargo clippy --release -p wat` → no new warnings on touched files.

Do NOT commit. Leave the tree dirty for review.

Runtime prediction: 45–75 min (the readln'→macro recast + Req field + read-frame 2-arg + the macro is
the hard part). Trap-door: the macro forwarding `-> :T` (STOP-1).
