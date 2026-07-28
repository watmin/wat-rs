# BRIEF — the stdin read joins the lock-step (arc 170)

## The work, in one paragraph

Everything in wat waits by SELECTING over its pipes. One reader does not: `RealStdin`
reports `as_raw_fd_for_poll() -> None` even though it wraps **fd 0**, so the stdin
service's read is a bare blocking `read(2)`. It cannot observe a stop request, and
because the stdio services are held for the process lifetime by design, that parked
thread **pins the whole process alive until stdin EOFs** — a `wat` program cannot be
stopped while a human sits at its prompt. This brief brings that read into the
multiplex the substrate already implements elsewhere, and carries the resulting
"a stop was requested" truthfully up six layers, each of which currently reports it
as EOF.

The acceptance test already exists, is deterministic, and is `#[ignore]`d:
`tests/cli/wat_cli::sigterm_reaches_a_program_blocked_on_stdin`. Un-ignoring it and
seeing it pass is the definition of done.

## THE EXEMPLAR — copy this, do not invent

`src/channel/transfer.rs:200-250` already does exactly the poll this needs:

```rust
let pipe_fd_opt = reader.as_raw_fd_for_poll();
let broadcast_fd = crate::runtime::SHUTDOWN_BROADCAST_READ_FD.load(Ordering::SeqCst);
enum LineResult { Line(String), Eof, Shutdown, Disconnected }
let read_one_line = || -> LineResult {
    if let (Some(pfd), true) = (pipe_fd_opt, broadcast_fd >= 0) {
        loop {
            let mut fds = [ libc::pollfd { fd: pfd,          events: POLLIN|POLLHUP, revents: 0 },
                            libc::pollfd { fd: broadcast_fd, events: POLLHUP,        revents: 0 } ];
            let n = unsafe { libc::poll(fds.as_mut_ptr(), 2, -1) };
            if n < 0 { if last_os_error().kind() == Interrupted { continue } break }
            if n == 0 { continue }
            if fds[1].revents != 0 { return LineResult::Shutdown }   // shutdown wins ties
            if fds[0].revents != 0 { break }
        }
    }
    /* …then the ordinary read_line… */
};
```

Read it in full before writing anything. It is the correct shape, including the
EINTR retry and the shutdown-wins-ties rule.

## The rooms, in order, and why each

1. **`src/io.rs`, `impl WatReader for RealStdin`** (`as_raw_fd_for_poll`, ~:62 default and
   the RealStdin impl ~:126-160). It returns `None` because a doc comment calls RealStdin
   "non-FD-backed" — but it wraps `std::io::Stdin`, which is fd 0. Make it `Some(0)` and
   correct that comment; it is the claim that took stdin out of the system.
2. **`src/io.rs`, `read_framed_edn`** (~:1479). Its accumulate loop ends
   `Ok(None) | Err(_) => { … FramedRead::Eof … }` — a wildcard that will report a stop as a
   clean EOF. This is the SAME defect class the arc just removed from `kernel/peer.rs`; do
   not leave it in place while adding the poll above it.
3. **`FramedRead`** (`src/edn_shim.rs` ~:1437, variants `Frame|Eof|Truncated|Malformed|TooLarge`)
   — needs a variant meaning "a stop was requested; nothing is wrong with the stream."
4. **`eval_ioreader_read_frame`** (`src/io.rs`, the `:wat::io::IOReader/read-frame` verb,
   currently `-> Option<String>`) — `Option<String>` cannot express the new outcome. Its
   scheme is in `src/check.rs`.
5. **`:wat::kernel::StdIn::ReadLineResponse`** (`wat/kernel/services/stdio-primes.wat`,
   the surface's `:messages`) and the `stdin-svc` `read-line` impl beside it.
6. **`:wat::kernel::ReadFrameOutcome`** (`src/types.rs`, currently `Frame [text] | Eof`) —
   the caller-facing outcome the REPL matches, plus `stdio-read-frame`
   (`stdio-primes.wat`, the wat helper) which maps into it.

## The contract decision, pinned

**A stop request is NOT an EOF and NOT an error.** At every layer above it must be its own
named outcome. Naming at layers 3–6 is `intueri`'s to rule and is NOT yours to pick — use
a clearly-provisional placeholder and say so in your report; the orchestrator casts the ward
and renames before commit. What matters here is that the distinction SURVIVES each hop.

## Blast radius

`src/io.rs` · `src/edn_shim.rs` (the `FramedRead` enum + its match arms) · `src/types.rs`
(the outcome enum) · `src/check.rs` (the two verb schemes) ·
`wat/kernel/services/stdio-primes.wat` · `wat-scripts/demos/repl/repl.wat` (one new match arm).
No new files. No changes to `substrate_on_stop_signal`, `trigger_shutdown`, or anything under
`src/kernel/` — those are the NEXT stone and are deliberately out of scope.

## STOP triggers — ship nothing and report

- **STOP-1.** If `read_framed_edn`'s callers cannot be updated without touching a file outside
  the blast radius above, STOP and report which file and why.
- **STOP-2.** If `StdIn::ReadLineResponse` cannot take a new variant because `defservice`
  mandates a fixed variant set for serviceable op-Responses, STOP and report the exact
  checker error. Do not work around it by reusing `Eof`.
- **STOP-3.** If at any layer the only way to carry the stop is to reuse an existing variant
  that means something else (`Eof`, `Disconnected`, `Malformed`), STOP. Reporting a stop as
  an EOF is the precise defect this brief exists to remove; recreating it one layer up is
  worse than not shipping.

## Gate

`cargo build --release --all-targets` — clean, zero warnings. That is your gate.
The orchestrator runs the test floor centrally and weighs it.

## Definition of done

`cargo build --release --all-targets` clean, and a written report naming: each layer's new
outcome (with your provisional names flagged as provisional), any STOP you hit, and whether
`tests/cli/wat_cli::sigterm_reaches_a_program_blocked_on_stdin` still carries its `#[ignore]`
(leave the attribute in place — the orchestrator removes it after weighing).
