# WEIGH — STONE 255.29: a thread address is data — ACCEPTED

**Executor: grok via pulsare, commit `22052781d`.** Weighed by the orchestrator against disk on 2026-09-25.

## Re-measured independently

| row | measured | result |
|---|---|---|
| tree · floor | `git status`; `.floor/2026-09-25T04-54-10Z` | clean; `6112 tests run: 6112 passed (9 slow), 22 skipped` |
| a Shared address in a pure record | `--check tests/kernel/probe_arc255_29_shared_in_pure_record.wat` | rc=0 (pre-stone rc=1, per SCORE) |
| the inverted 255.25 row | `--check …_address_shared_field.wat` | rc=0 |
| ⭐ a thread address to a process child | `wat tests/kernel/probe_arc255_29_thread_address_to_process.wat` | `["Rejected: thread address minted by process … dialed from process … — a thread address is dialable only inside its minting process" "Connected"]`; pre-stone the child died decoding |
| ⭐ **timing, A/B, interleaved in one session** | the B3 probe, pre-255.29 binary vs new, ×4 each | steady round trip **pre 112.7–115.4 µs · new 112.1–114.9 µs: no difference** |

On timing: the SCORE honestly reported steady runs of 118–120 µs, above B3's quoted band. **That band came
from another session.** The interleaved A/B in one session shows no difference, so the SCORE's numbers
reflect machine load, not the change. The SCORE was right to report them; the brief was wrong to set a
cross-session band as the bar.

Taken from the SCORE without re-running: clippy 0; census `no STOP-8` (215; 10 new paths all rc 0); delta
NEW 2 / RECOVERY 0; ledger 211; three floors with the two reds fixed at their causes, not re-run to
green; unit tests pinning the round trip, the dangling id and a foreign minter with `assert_eq!`.

## What landed

- `ThreadAddressWire {:minter-pid :id}`: **no nonce, no auth**, per the ruling. It goes through the
  socket tier's trusted door; general decode still refuses it.
- A weak registry (dropping the last `Address` still closes the listener).
- Another process gets `Rejected`, a routing fact. A dangling id is `Refused`, with the outcome kind
  unchanged, as the brief required.
- `portable_form` stays socket-only; `address-wire?` stays false for a thread address.
- A **`Shared` address is pure.**

## Decisions the builder should see

- ⚠ **ZERO-MUTEX: the registry is a `Mutex<HashMap<u64, Weak<Sender>>>`.** The brief asked for lock-free,
  or a report with the reason. Reported: `std`/`crossbeam` have no lock-free map of `Weak`, and the
  stone adds no crate. The lock is taken **only at mint and at trusted decode**, never on `connect` with
  a live sender and never on a send. **The builder's call:** accept, add a crate, or redesign.
- **A recycled pid:** without a nonce, a later process that inherits the minter's pid **and** still has
  that id live would dial. Acceptable under *"auth on threads is nonsense"*; recorded.

## ⚠ A consequence of making Shared pure, carried

**Six 255.28 rows were inverted** (Shared through a generic is now pure, correctly under the ruling). Its
remaining refused rows are a generic `Struct` and a generic `Impure` enum, and both are decided by the
**declared nature**, not by argument substitution. **Unmeasured:** whether any refused row still
exercises 255.28's substitution path. The only argument-dependent purity in the tree was `Address`+`Shared`,
which is now gone. If none does, the substitution branch has no non-vacuity row. It will need either a
row that uses a real argument-dependent case, or to be named for `purgare`. **For a future stone.**
