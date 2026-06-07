# BRIEF — Stone 214.5.1: the channel substrate flip (memory tier → comms::thread)

> DESIGN: `DESIGN-STONE-5.1-CHANNEL-SUBSTRATE-FLIP.md`. Probe (committed, RED):
> `tests/nursery/probe_arc214_stone51_channel_substrate_flip.rs` — asserts the
> make-channel Sender/Receiver inners Debug-render as the Comms backing.

The wat `Sender`/`Receiver` values wrap swappable inner enums. Flip the MEMORY
tier's backing from bare crossbeam onto `comms::thread` — underneath all ~251
old-verb call sites, which stay untouched and gain the cascade contract.
**The PipeFd tier is OUT** (affirmative: its consumers rebuild in Slice 8; the
old select's PipeFd rejection stays).

## The rooms (read in order)

1. `src/typed_channel.rs:88-160` — `SenderInner`/`ReceiverInner` + the
   `from_crossbeam` constructors. THE FLIP: `Crossbeam{sender, closed}` →
   `Comms{sender: crate::comms::thread::Sender<Value>, closed: AtomicBool}`;
   `Crossbeam(rx)` → `Comms(crate::comms::thread::Receiver<Value>)`. HARD CUT —
   the Crossbeam variants are deleted, not kept alongside. Constructors take the
   comms endpoints (rename `from_crossbeam` → `from_comms`; sweep their callers).
2. `src/typed_channel.rs` — `typed_send` (~203), `typed_recv` (~295),
   `typed_try_recv` (~407), `sender_close` (~259): the memory arms delegate to
   the comms methods. **PRESERVE THE OUTCOME SURFACE EXACTLY** — the
   `SendOutcome`/`RecvOutcome` shapes the wat layer sees must not change
   (comms `SendError` → the same outcome the old crossbeam disconnect produced;
   comms `RecvError` → the old disconnected outcome; comms try_recv `None` →
   the old empty outcome). The `closed: AtomicBool` check stays in front,
   exactly as today.
3. `src/typed_channel.rs:487` — `try_as_crossbeam_receiver` (the old select's
   extraction): re-shape to hand back what `comms::thread::Select` needs
   (`&comms::thread::Receiver<Value>` registration); rename to match
   (intueri). `src/runtime.rs:~18600` — `eval_kernel_select`'s memory path
   waits via `comms::thread::Select` (cascade-aware). The PipeFd arm stays
   rejected exactly as today.
4. `src/runtime.rs:~17938` — make-channel's construction site
   (`sender_from_crossbeam(tx)` etc.): construct via
   `crate::comms::thread::pair::<Value>()` (capacity already agrees —
   bounded(1) both sides since 254.0).
5. `src/runtime.rs:~19052` — `eval_kernel_spawn_thread`'s hand-wired channel
   construction: same constructor swap (it mints crossbeam pairs today).
6. `src/comms/thread.rs` — `pair`, `Sender::send/close`, `Receiver::recv/
   try_recv`, `Select`: the delegation targets. comms recv integrates the
   shutdown broadcast internally — the flip GAINS cascade-awareness without
   surface change.

## STOP triggers (rejection criteria — ship nothing for that part; report)

- STOP-1: if any `typed_recv`/`typed_send` memory-arm behavior cannot map 1:1
  onto the comms methods (extra outcome states, timeout/deadline arms, ordering
  guarantees), STOP and report the exact mismatch — do NOT approximate.
- STOP-2: if `eval_kernel_select`'s memory path cannot be expressed over
  `comms::thread::Select` without changing its wat-visible result shape, STOP
  with the shape difference.

## Verify (report exact numbers)

- `cargo test --release --test nursery probe_arc214_stone51_channel_substrate_flip` → **2 passed**
- `cargo test --release --test nursery probe_arc214` → **54 passed**
- `cargo test --release --lib -p wat` → ~943/0/1 band
- Corpus spot-gates (plain cargo, these binaries are thread-tier):
  `cargo test --release -p wat --test channel` (if present) and TWO more wat
  corpus binaries that exercise make-channel/send/recv/select heavily — pick
  them by grepping `tests/` for the old verbs; report which you chose and the
  numbers. (The orchestrator runs the FULL `integration-run.sh` at score.)
- `cargo clippy --release` → no new warnings in touched files.

Do NOT commit — the orchestrator scores (including the full corpus run) and
commits.

## Expectations (orchestrator scorecard)

| # | Claim | Check |
|---|---|---|
| 1 | flip probe 2/2 (both inners Comms-backed) | re-run |
| 2 | arc214 nursery 54/0; lib band green | re-run |
| 3 | **FULL corpus green — integration-run.sh, all binaries** | orchestrator runs |
| 4 | Crossbeam variants DELETED (HARD CUT); no dual backing | read diff |
| 5 | Outcome surface identical (SendOutcome/RecvOutcome unchanged) | read diff |
| 6 | PipeFd arms untouched | read diff |
| 7 | no new clippy; tree dirty | clippy + git status |

Runtime band: 25–40 min (enum flip + 5 delegation arms + 2 construction sites).
