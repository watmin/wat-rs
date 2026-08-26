# DESIGN — EDN value-framing over the pipe wire (multi-line safe)

**STRIKE-READY.** RED probe verified: `tests/nursery/probe_edn_value_framing.rs` fails at HEAD with
`DecodeError("...unclosed map")` — the reader gets only the first line `{`.

## Why
The pipe wire protocol (`project_pipe_protocol.md`) is **line-delimited EDN: one value = one physical
line.** Every decoder does `read_line` → `read_edn(that_one_line)`. So a multi-line EDN value — exactly
what `:wat::kernel::pprintln` emits, and any pretty/large value — cannot cross a pipe: the first line
(`{`) is incomplete EDN and the decode errors. This blocks `pprintln` over IPC and is a latent
correctness gap for any value whose compact form a future encoder might wrap.

## The contract (value-framing)
A frame is **one complete EDN value terminated by a clean newline, with no trailing data** (the
anti-smuggling invariant — nothing like an HTTP request-smuggle). The reader accumulates physical lines
until the buffer parses as a complete value, then stops.

## The two findings that make this easy (already in the substrate)
1. **The parser distinguishes incomplete from malformed.** `wat_edn::ErrorKind::{UnexpectedEof,
   UnclosedString, UnclosedList, UnclosedVector, UnclosedMap, UnclosedSet}` = "a token/delimiter opened
   and input ran out" → *read more*. Every other variant = a genuine syntax error → surface it. (The RED
   probe's `UnclosedMap` is this signal firing.)
2. **Anti-smuggling is free.** `wat_edn` `parse_top` (`crates/wat-edn/src/parser.rs:53`) parses exactly
   one value then **requires `Token::Eof`** — trailing data after the value is already rejected. So
   "complete value + nothing after" is enforced by the existing parser, not something to add.

Corollary: **`read_edn` already parses a multi-line string** (the lexer skips `\n`). The defect is
purely that readers feed it ONE line. The fix is read-loop accumulation, NOT the parser or `read_edn`.

## The build

### 1. Expose the incomplete signal (the only `wat_edn`/`edn::render` change)
- `crates/wat-edn/src/error.rs`: add `impl ErrorKind { pub fn is_incomplete(&self) -> bool }` — true for
  `UnexpectedEof | UnclosedString | UnclosedList | UnclosedVector | UnclosedMap | UnclosedSet`, false
  otherwise. (Also expose it off `Error` if that's what callers hold.)
- `src/edn/render.rs`: add `pub fn edn_frame_status(s: &str) -> EdnFrameStatus` where
  `enum EdnFrameStatus { Complete, Incomplete, Malformed(String) }`. It calls `wat_edn::parse_owned(s)`:
  `Ok(_) → Complete`; `Err(e) if e.kind.is_incomplete() → Incomplete`; else `Malformed(format!("{e}"))`.
  (Note: `read_edn` currently *stringifies* the parser error in `edn::render::read_edn_caps`, discarding the kind —
  do NOT rely on that path for the signal; use `parse_owned` directly in `edn_frame_status`.)

### 2. The shared accumulate helper (transport-agnostic)
A function that, given a "read one line" source, returns one complete frame:
```
enum FramedRead { Frame(String), Eof, Truncated(String) /* EOF mid-value */, Malformed(String) }
fn read_framed_edn(next_line: impl FnMut(Span) -> Result<Option<String>, RuntimeError>, span: Span)
    -> Result<FramedRead, RuntimeError>
```
Loop: read a line; `None` on the FIRST line → `Eof` (clean); `None` mid-buffer → `Truncated` (writer died
mid-frame — a real error). Append the line (re-add the `\n` stripped by `read_line`); call
`edn_frame_status(buf)`: `Incomplete` → read another line; `Complete` → return `Frame(buf)`; `Malformed` →
return `Malformed`. Each caller then decodes the returned frame string with its OWN decoder
(`read_edn` / `decode_trusted_wire` / typed), preserving current per-site decode semantics. Line-granular
accumulation is sufficient (no byte carryover) precisely because the contract guarantees a frame ends at a
newline and the next value starts on the next line; we stop the instant the buffer is `Complete`, so we
never over-read into the next value.

### 3. Route the decode read-loops through it
Known line-framed decode sites (the sonnet confirms the full set by grep; apply the helper to each):
- **`src/channel/transfer.rs`** `typed_recv` PipeFd arm (~217) — the one-`read_line`-then-`read_edn` is
  replaced by `read_framed_edn(|s| reader.read_line(s), span)` then `read_edn(frame, types)`. Keep the
  existing poll/shutdown multiplex wrapping each `read_line` (so shutdown still wins between lines).
- **The StdInService read loop** (`src/services/`) — `readln` delegates the fd read to the service loop,
  which reads lines off fd 0 and replies; it must accumulate a full frame before replying. Find its read
  loop and route it through the helper.
- **The comms recv path** (`src/comms/`) — IF it is line-framed (feeds `decode_trusted_wire(edn_str)` from
  a line read), route it too. If it uses a different framing (length-prefix), it is OUT OF SCOPE — say so.

## STOP triggers (surface, do not improvise)
1. If a decode site uses a non-line framing (length-prefix, fixed-size) — STOP, report it; do not force it
   through the line helper.
2. If `parse_owned` does NOT actually reject trailing data after a value (i.e. `parse_top`'s EOF check
   isn't the path `parse_owned` uses) — STOP; the anti-smuggling claim must hold or the contract is unsafe.
3. If routing a site needs an architectural change beyond "swap the read for the helper" — STOP and report
   the shape rather than carving deep.

## Expectations (independent scorecard — fixed before the strike)
| # | what | command | expected |
|---|---|---|---|
| 1 | the framing probe goes green | `cargo test --release -p wat --test nursery multiline_edn_value_frames` | 1 passed |
| 2 | pprintln round-trips over a pipe | a new test: pretty-print a map to a pipe, recv it, assert == the compact-decoded value | pass |
| 3 | single-line values still work (no regression) | `cargo test --release -p wat --test nursery` + `--test channel` (sender_receiver_from_pipe) | green; framing-suite green |
| 4 | trailing-data frame is rejected (anti-smuggle) | a test: write `{:a 1} {:b 2}\n` as one frame → Malformed, not silently one value | DecodeError/Malformed |
| 5 | lib floor holds | `cargo test --release -p wat --lib` | 953 / 36 / 1 (identical to baseline; 36 pre-existing) |
| 6 | clippy clean on touched files | `cargo clippy --release -p wat` | no new warnings |

Runtime prediction: 60–90 min (the helper + 2–3 read-loop sites + the `is_incomplete`/`edn_frame_status`
additions + tests). Trap-door: a read-loop that interleaves poll/shutdown per line must keep that
multiplex around EACH `read_line` inside the accumulate loop — losing it would break shutdown
responsiveness mid-frame.
