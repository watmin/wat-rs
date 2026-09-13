# BRIEF — the rope can be looked at

**Read `DESIGN.md` beside this first.** It carries the reason this stone is **half** of the ranked item
(the other half reverses a ruling and is retired), the contract decision, and the six trap-doors.

## The work, in one paragraph

Nothing in wat can observe how a spawned lineage ended, so `CloseOutcome::Signaled` and
`CloseOutcome::Failed` have never been seen from wat. The verb that could show them, `:wat::kernel::close`,
is restricted to `:wat::kernel::` callers by a **ruling** (arc 259 S2d, *"the user never holds the rope"*)
and consumes the peer besides. Add a **non-consuming, unrestricted** `:wat::kernel::lineage-status` that
returns `(Option :- [CloseOutcome])` — `None` while running, `Some(outcome)` once ended — and make both
variants fire from wat with a probe.

## The rooms

| where | why you are going there |
|---|---|
| `src/types.rs:2104–2125` | `CloseOutcome`'s registration and its ruling comments. ⭑ Read all of it: `Closed [exit <- Option<i64>]` (None = thread, Some = process exit), `Signaled [signal <- i64]` — **"TERMINATED, never merely stopped"** — and `Failed [cause <- Failure]` carrying join-panic / wait-fail / **stopped-not-terminated**. Also `:2118–2121`: it is `Purity::Pure` *because* the peer is consumed elsewhere. |
| `src/intrinsic/kernel/resource.rs:524` | `#[wat_intrinsic(":wat::kernel::close")] eval_peer_close_prime` — ⭑ **THE PROVEN SHAPE.** It already builds every `CloseOutcome` variant from a lineage. Copy how it reads the tier and constructs the outcome; **do not copy the consume**, and do not copy the `:restricted-to`. |
| `src/process/clone.rs:171` | `try_wait() -> io::Result<Option<ExitStatus>>` — exactly *"has it ended, and how"*, non-blocking, non-consuming. The process tier's engine. |
| `src/intrinsic/mod.rs:749–771` | the registered kernel intrinsic list. Your verb goes here, alphabetically, **without** a caller restriction (`connect`, `send`, `recv` are the unrestricted neighbours to match). |
| `src/intrinsic/kernel/identity.rs:330–338` | `peer-process` — a non-consuming **Projection** returning `Option<Process>`. ⭑ This is how userland legitimately gets a lineage handle from a peer, and it is the shape of an observation verb here: unrestricted, total, no teardown. |
| `tests/process/signal_kill_produces_close_outcome_signaled.wat:11–24` | ⭑ **READ THIS HEADER.** A prior self ran this whole crawl and wrote down the ruling, the `ReservedPrefix` dead end (*"verified empirically"*), and the two sanctioned futures — one of which is this verb. It also says the co-located `.rs` reads `Process::wait()` one layer below. **Do not retarget that fixture** (DESIGN §Out of scope). |
| `wat-scripts/scratch-pad/probe-crash-surface-*.wat` | the campaign's probe idiom. ⭑ `probe-crash-surface-gate-gaveup.wat:26` is the cited way to park a handler; the `after`/`recv` idiom there is the one that type-checks. |

## Implementation sketch

```
#[wat_intrinsic(":wat::kernel::lineage-status")]           // NO :restricted-to
pub(crate) fn eval_lineage_status(peer: &WatAST, …) -> … {
    // same tier dispatch as eval_peer_close_prime, but:
    //   process: try_wait()  → None => Option::None
    //                          Some(status) => Some(CloseOutcome::{Closed|Signaled|Failed})
    //   thread:  is_finished() → false => Option::None
    //                            true  => Some(CloseOutcome::{Closed|Failed})   (no OS exit code)
    // the peer's cell is LEFT INTACT — read, never take
}
```

⚠ **The sketch is a convenience; `eval_peer_close_prime` is the contract.** Match how it classifies an
`ExitStatus` into `Closed`/`Signaled`/`Failed` rather than re-deriving the mapping — two of my sketches
have been wrong in this campaign and the executor was right to follow the disk both times.

## ⭑⭑ The proof — both variants, from wat, in one probe

A green floor cannot give this: nothing in the corpus calls the new verb. Write
`wat-scripts/scratch-pad/probe-lineage-status.wat` and print all of:

| cell | how | expected |
|---|---|---|
| still running | spawn/obtain a live lineage, peek | **`None`** |
| `Closed` | let it exit cleanly, peek | `Some(Closed …)` — thread `None` exit, process `Some(code)` |
| ⭑ `Signaled` | `:wat::kernel::signal` Kill on a process child, then peek | **`Some(Signaled 9)`** |
| ⭑ `Failed` | a child that **panics**, then peek | **`Some(Failed …)`** |
| not consumed | peek **twice**, and use the peer after | second peek agrees; the peer is still usable |

★ `:wat::kernel::signal` is unrestricted and the Kill fixture already drives it — that is the existing,
sanctioned way to *cause* `Signaled`, and it is why this verb is the missing half rather than a new
mechanism.

## Verify

- `cargo build --release` **and** `cargo nextest run --release --no-run` — the build does NOT compile tests.
- `./scripts/floor.sh`, read the **Summary line** → **5237 passed, 0 FAIL**, no `ARM.txt`.
- ⛔ **Never a piped exit code** — a type-error run this session reported `$?` = 0 through a `| head`; the
  true exit was **3**, on stderr.
- `cargo clippy --release --workspace --all-targets -- -D warnings` → **0**. ⚠ This stone reaches `src/`;
  clippy matters more than usual.
- Happy path `circuit.wat 2000 4 3 8192 true 1000` → `distinct=8000;dup=0`; shipped chaos
  `… 50 2 2 8192 true 1000 0 0 7 0 0 0 0 0 500` → exit 0.
- Everything heavy through `./scripts/capped.sh --limit 8g`.

## STOP triggers

1. **STOP-1 — if the verb cannot avoid consuming the peer**, STOP and say why. A consuming peek is `close`
   renamed and inherits arc 259's reasoning; I would rather have the gap than a hole in that wall.
2. **STOP-2 — do not add a `:restricted-to`** to the new verb. If you believe an observation needs a caller
   whitelist, STOP and argue it; do not ship it restricted, because that silently rebuilds the wall.
3. **STOP-3 — do not touch `close`'s restriction, its whitelist, or its consume.** Reversing arc 259 S2d is
   a doctrine ruling and explicitly out of scope.
4. **STOP-4 — if the thread tier cannot be observed non-blockingly**, STOP and report. Do **not** ship a
   process-only verb that raises on a Thread; half a verb is worse than none, and the `Peer`-vs-lineage type
   wall already caught one wrong assumption here.
5. **STOP-5 — if `Signaled` cannot be distinguished from a merely STOPPED child**, STOP.
   `src/types.rs:2113` is explicit: *Signaled means terminated, never merely stopped* — a SIGSTOP'd child is
   `Failed`'s stopped-not-terminated case.
6. **STOP-6 — do not retarget `tests/process/signal_kill_produces_close_outcome_signaled.{wat,rs}`.** Its
   header invites it and this stone is the trigger, but it is a separate change to an arc-170 gate.

## Shape to copy

`the-owner-wait-has-a-deadline/SCORE.md` — the last stone that added a kernel intrinsic and proved it with
a probe because the floor could not. And `src/intrinsic/kernel/resource.rs`'s own header for how a
resource-touching intrinsic documents its determinism and totality.
