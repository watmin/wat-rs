# EXPECTATIONS — the sender grows a ring

Written **before** the strike. **This stone changes `src/` on the transport's hot path.** The floor is
the gate, and two named probes are the load-bearing arms.

Rows state what must be true, not where to look.

## Rows

| # | what must be true | how it is checked | expected |
|---|---|---|---|
| 1 | ★ **the probe that FOUND the bug still passes, fast** | `probe_arc278_partial_frame_residue` | **PASS ~3 s**, not 20 s. This is the arm that hung; it is the regression oracle |
| 2 | ★ **its innocent sibling still passes** | `probe_arc278_send_poll_arm` | PASS, ~0.03 s — the full-pipe shutdown arm |
| 3 | ★ **the tie-break survives as a property** | a new probe | with room available **and** a stop pending, the send **still writes**; with the pipe full and a stop pending, it returns `Shutdown` |
| 4 | ⛔ **`NonblockGuard` is gone from the ring path** | read the diff | the send fd is **blocking** when the Write is submitted — every measurement behind this stone was made on a blocking fd |
| 5 | **no partially-delivered write is ever cancelled** | the new probe reports it | a cancel only ever follows a Write that delivered **0** bytes |
| 6 | **`SendError` is unchanged** | read the diff | variants, errno mapping (`EPIPE` → `Disconnected`), and message strings identical |
| 7 | **`try_send` is untouched** | `git diff` | non-blocking by contract; no wait to multiplex |
| 8 | **`Sender` is still not `Clone`** | compile + read | single-writer, `PIPE_BUF` atomicity. A ring must not smuggle a `Clone` in |
| 9 | **every construction site handles ring failure** | read the diff | all four (`:614`, `:2055`, `:2108`, `:2132`) — matching each site's own signature |
| 10 | **the bootstrap fallback holds** | the new probe or a unit test | `SHUTDOWN_BROADCAST_READ_FD == -1` → submit the Write alone; no panic, no hang |
| 11 | **`EINTR` on `submit_and_wait` retries** | read the diff | mirrors the Read path at `process.rs:1315-1320` |
| 12 | **the header stops lying** | read line 10 | it currently says `libc::write`; it must name the ring |
| 13 | **the floor holds** | `scripts/floor.sh` | **read the Summary line**: 5226 passed (5225 + the new probe), 22 skipped, **0 FAIL, 0 TIMEOUT**, quiet box |

## The rows that carry this stone

⚠ **Row 1 is not a formality.** `probe_arc278_partial_frame_residue` is the arm that went red and
exposed the whole finding. If this change is wrong, that is where it shows — and a 20 s PASS is a
failure wearing a pass's clothes. **Read its duration, not just its verdict.**

⚠ **Row 3 is the point of the stone.** The tie-break — *writable wins; a dying process must still be
able to utter its last words* — is today a hand-written branch order guarding a hand-written comment.
After this it should be a property of the multiplexer. **Prove both halves**: it still speaks when it
can, and it stops when it cannot.

⚠ **Row 4 is the trap that would silently invalidate everything.** All four probes measured a
**blocking** fd (`fill_until_eagain` restores the original flags before submitting). With
`O_NONBLOCK` still set, io_uring returns `-EAGAIN` instead of parking, and none of the measured
behaviour transfers.

⚠ **Row 5 is why this stone was drawable at all.** Measured: room `> 0` completes immediately with
the short count; room `== 0` parks having delivered nothing. If a cancel ever follows a delivered
count, the premise is broken — **STOP and surface it, do not work around it.**

## Runtime prediction

**90–150 minutes.** Larger than anything in this line: a struct field, four construction sites, a
rewritten wait, a removed guard, and a new probe. The floor is the long pole.

## Trap-doors named in advance

- **The `framed` buffer must outlive every `submit_and_wait`** — same discipline the Read site
  documents in its SAFETY comment.
- **Two SQEs, one ring, capacity 4** — the Receiver's proven size (Read 1 + POLL_ADD 2, headroom).
- **`user_data` is the only way to tell completions apart.** Two tags, checked — never assumed by
  order.
- **A cancelled `PollAdd` must be drained** when the Write wins, or the next iteration sees a stale
  CQE.
- **`AsyncCancel` may return `ENOENT`** if the Write completed first — that is not an error.
- **The broadcast arm re-arms every iteration.** A `PollAdd` consumed on one pass is not still armed
  on the next.

## What this stone does NOT claim

⚠ It does **not** touch `src/io.rs` — cut by the four questions, its own stone once ring ownership
has an answer.
⚠ It does **not** fold in `signalfd` — that is `a signal is an fd`, and it retires the handler and
the wake pipe, **not** the broadcast.
⚠ It does **not** make `Sender` cloneable.
