# Stone 214.5.1 — the channel substrate flip: memory channels re-back onto comms::thread

> Slice 5 opens. The scout (task #194): ~251 wat call sites / 19 files on the old
> verbs. THE INSIGHT: they don't migrate one by one — the wat `Sender`/`Receiver`
> VALUES wrap swappable inner enums (`SenderInner`/`ReceiverInner`,
> typed_channel.rs:88/113), so the backing flips UNDERNEATH all callers at once.

## Scope (don't-patch-the-grave applied)

- **IN: the Crossbeam tier.** `make-channel` + `sender/receiver_from_crossbeam`
  construct comms::thread endpoints; `typed_send`/`typed_recv`/`typed_try_recv`/
  `sender_close`/old-`select` delegate their memory-channel arms to comms.
  Capacities already agree: comms::thread::pair = bounded(1) = the 254.0
  make-channel collapse.
- **OUT (affirmative): the PipeFd tier.** Its constructors (`from_pipe`,
  `make_pipe_channel_pair`) serve the OLD stdio/Process plumbing whose
  consumers are REBUILT in Slice 8 (services universe-resident) and whose
  carcass is deleted in Slice 6. Re-plumbing them now is grave-tending. The
  old select's PipeFd rejection stays — its callers move to peers + `select'`
  (already live, io_uring).

## The flip

1. `SenderInner::Crossbeam{sender, closed}` → **replaced** by
   `SenderInner::Comms{sender: comms::thread::Sender<Value>, closed: AtomicBool}`;
   `ReceiverInner::Crossbeam(rx)` → `ReceiverInner::Comms(comms::thread::Receiver<Value>)`.
   HARD CUT — the Crossbeam variants do not survive alongside.
2. `make-channel` eval + `sender/receiver_from_crossbeam` construct via
   `comms::thread::pair::<Value>()` (the constructors rename to `from_comms` or
   absorb; intueri picks).
3. `typed_send`/`typed_recv`/`typed_try_recv` memory arms delegate to the comms
   methods — callers GAIN the cascade contract (comms recv wakes on substrate
   shutdown; the old bare-crossbeam arm did not).
4. Old `select` (`eval_kernel_select`): the memory-channel extraction
   (`try_as_crossbeam_receiver`) re-points at the Comms inner; the wait goes
   through `comms::thread::Select` (cascade-aware). PipeFd arm: unchanged
   (still rejected — graveyard, dies with its consumers).
5. `spawn-thread`'s hand-wired channel construction flips the same way
   (it mints crossbeam pairs today — same constructor swap). This is 5.2 if it
   doesn't fit; same stone if it's the same three lines.

## Why this shape wins (four questions)

- **Obvious?** YES — one backing enum, one constructor, the same verbs.
- **Simple?** YES — delegation arms; no caller changes; no new surface.
- **Honest?** YES — the old stack loses its memory-channel tenants from BELOW;
  Slice 6's delete becomes an eviction notice, not a migration. The PipeFd cut
  is affirmative (Slice 8 rebuilds its consumers), not deferral.
- **Good UX?** YES — 251 call sites keep working, now cascade-aware.

## The gates

- **Probe (FM-2-bis):** a Rust nursery test asserting the make-channel
  receiver's inner Debug renders as the Comms backing — RED today
  ("Crossbeam"), GREEN after; compiles in both worlds (string assert on Debug,
  deliberately, so the HARD CUT doesn't break the probe's compile).
- **THE regression gate: the corpus.** `scripts/integration-run.sh` (the 33
  binaries / 147 wat tests) — all green before and after; this IS the proof
  that 251 call sites never noticed. Plus lib band + the peer-verb integration
  probes (no interference) + clippy.
