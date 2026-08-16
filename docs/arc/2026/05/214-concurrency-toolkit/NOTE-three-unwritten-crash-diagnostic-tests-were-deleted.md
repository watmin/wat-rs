# NOTE (arc 214) — three UNWRITTEN crash-diagnostic tests were deleted, and this is what they were for

**Filed 2026-08-16. A POINTER, not a decision.** Builder: *"remove the unwritten tests.. leave a
NOTE-<slug>.md in their respective arcs.... we'll deal with them when we go to close the arcs... not
a now thing."*

## What was deleted, and why it was not a test

Three `#[test]` functions whose entire body was `unimplemented!()`:

| was | file |
|---|---|
| `alpha_recv_prime_autoraises_child_crash_reason` | `tests/kernel/probe_arc214_alpha_crash_autoraise.rs` (whole file, 47 lines) + its co-located `.wat` fixture |
| `spawn_program_prime_process_error_emits_diagnostic` | `tests/kernel/spawn_program_prime_process.rs` |
| `spawn_program_prime_process_runtime_error_emits_diagnostic` | `tests/kernel/spawn_program_prime_process.rs` |

Each carried `#[ignore]`, and each ignore's own reason said the quiet part:

> *"UNWRITTEN (arc 214 1b-ii-α): the body is `unimplemented!()` — running this out-of-band panics, it
> does not measure. Not a concurrency quarantine."*

**They were placeholders wearing a test's clothes.** No arc closing could ever turn them green,
because there was nothing to turn green — the unlock was "someone writes the body". They sat in the
`#[ignore]` count as three permanent rows that no work could retire, which is exactly the population
Stone K (arc 296) split out: an `#[ignore]` must mean *blocked or broken*, and "never written" is
neither.

## ★ THE DESIGN CONTENT THEY CARRIED — this is why a NOTE and not a silent delete

The alpha file's header held a real, grounded account of an open substrate gap. Preserved verbatim in
substance:

**The dogfood claim.** io_uring is meant to be the substrate's ONLY io-select loop, and the 3-fd
cross-process IPC (`in` / `Ok` / `Err`) in a cap-4 ring is the proving point that dogfoods the
autoscaling TCO loop. Stone `1b-ii-α` folds the **`Err` channel** — the child's fd 2, *today a
SEPARATE plain `libc::pipe`* drained by `ProcessPeerBundle::take_crash_reason` — into the io_uring
receiver as a **third `POLL_ADD` arm**.

**Why that matters to the surface.** Once `Err` is an arm of the ring, a crashed child's
`#wat.kernel/ProcessPanics` reason arrives through `recv` itself, so `recv'` can **auto-raise** it.
That closes Q1: the substrate raises on the user's behalf — no user-facing crash verb, and no second
`take_crash_reason` call.

**The gap as it stands at HEAD** (this is the part worth keeping — it is a live description of
current behaviour, not a prediction):

> The `Err` channel is NOT an io_uring arm. A crashed child closes its stdout (fd 1); the comms
> `Receiver` sees EOF; `bundle.peer.recv()` returns a **bare `RecvError` with no reason**; and
> `eval_peer_recv_prime` maps that to a generic
> `MalformedForm { reason: "recv failed: process channel disconnected" }`. The actual cause — e.g.
> `DivisionByZero` — is reachable ONLY via the separate plain-pipe `take_crash_reason`, never through
> `recv'`.

**The crash path the other two were written against.** Child hits a runtime error → panic →
`catch_unwind` → `finish_forked_child` → `emit_structured_exit` writes the `ProcessPanics` envelope to
fd 2 → parent reads it via `bundle.recv()` → `Crashed(reason)`.

**What they would have asserted, on unlock:** the EXACT crash-diagnostic EDN — `assert_eq!` on a
`DivisionByZero` `#wat.kernel/ProcessPanics` envelope, for the io_uring-raised case (α), the process
crash case, and the runtime-error case.

## ⚠ A STALE CLAIM THE DELETED FILE CONTAINED

Its header said:

> *"Companion HEAD-behavior tests (the same crash, read via `take_crash_reason`):*
> *`spawn_program_prime_process_runtime_error_emits_diagnostic` (already green)."*

That test was **also `unimplemented!()`**. It was never green; it never ran. One unwritten test cited
another unwritten test as its passing companion, and the citation sat there unchallenged.
`[[feedback_a_green_test_can_prove_nothing]]` — here in its purest form, since there was not even a
test to be hollow.

## What closing arc 214 has to decide

Not "un-ignore these" — they are gone. The decision is a **ruling on the capability**:

1. **Is `1b-ii-α` still the intent?** Folding `Err` into the ring as a third `POLL_ADD` arm, so
   `recv'` auto-raises. If yes, the tests get WRITTEN then, against a mechanism that exists, and they
   assert the exact EDN.
2. **Or is the plain-pipe `take_crash_reason` the settled design?** Then the gap described above is
   not a gap, and what is owed is a test of the CURRENT path — that a crashed child's reason reaches
   the parent at all — which is a different test from the three deleted.

Either way the three placeholders were not the artifact that answers it. **Do not resurrect them from
git; write what the ruling calls for.**

## Measured at deletion

```
#[ignore] before  20      after  17
floor             re-run and green; see the commit
```

`tests/kernel/mod.rs` declares no modules — `build.rs` auto-generates the module list from sibling
`*.rs` files into `OUT_DIR`, so deleting the file needed no declaration change. (Checked before
deleting, not after.)

## Kin

- `docs/arc/2026/06/296-diagnostics-fully-edn/DESIGN-STONE-K-ignore-means-one-thing.md` — the stone
  that says `#[ignore]` means ONE thing. These three were a fourth kind it did not name: **not
  written**. Worth adding to K's taxonomy if the question ever recurs.
- `[[feedback_a_green_test_can_prove_nothing]]` · `[[feedback_nothing_blocks_it_is_not_a_work_item]]`
