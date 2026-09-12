# BRIEF — the gate methods face an outcome

**Read `DESIGN.md` beside this first.** It contains one contract decision whose *reason is not
symmetry with the previous stone*, and a twin you must not leave behind.

## The work, in one paragraph

`<S>/grant` and `<S>/revoke` return `nil` and raise on all four non-`Message` outcomes — two of them
in messages that say the peer is alive. Give both a `GateOutcome` and the same wall-clock-bounded
**re-recv** `stop`/`hibernate` now use, so an owner faces a value. This closes the owner-method class:
stop ✓ hibernate ✓ grant · revoke.

## The rooms

| where | why you are going there |
|---|---|
| `wat/service.wat:3056` `grant-method-body` | the primary strike. `peer-process` tier check, then four raising arms. |
| `wat/service.wat:3106` `revoke-method-body` | **the twin.** Identical shape, `Status::PeersDenied`. Same change. |
| `wat/service.wat:2330` | ⛔ READ FIRST: the serve `AllowPeer` arm. It **continues serving** and **never retries its ack**. Both facts drive the contract decision. |
| `wat/service.wat` `owner-recv-loop` | the loop stone 2 built. Reuse it; do not write a second one. |
| `wat/service.wat` `StopOutcome` / `stop-faced` / `require-stopped` | the shape and the call-site helpers to mirror for the gate. |
| `wat-scripts/fanout/circuit.wat` | 18 of the 68 call sites, and the post-spawn grant ordering the doc-comment calls load-bearing. |

## Implementation sketch

```
(:wat::core::defenum :wat::service::GateOutcome :wat::enum::Pure
  :Applied []
  :Gone    [cause <- :wat::kernel::LociDiedError]
  :GaveUp  [waited-ms <- :wat::core::i64  last <- :wat::core::String])
```

`grant-method-body` becomes:

```
(match (peer-process handle)
  ((Some _) → send AllowPeer[pids] once (keep the four-arm SendOutcome tolerance verbatim),
              t0, then the SAME bounded re-recv owner-recv-loop already does, mapping:
                Message(Status::PeersAllowed) → Applied
                Message(other)                → KEEP today's raise (protocol violation)
                Closed / Lost(c)              → Gone …
                TimedOut / Stopped / Malformed → re-recv while budget remains, else GaveUp{elapsed,last})
  ((None) → Applied))        ;; thread tier: the handle IS the grant. No wait, no pretence.
```

`revoke` is the same with `Status::PeersDenied`. ⭑ **Reuse `owner-recv-loop`** — it already returns
`StopOutcome`, so either generalize it over the success constructor or add a thin `gate-recv-loop`
beside it that shares the bound logic. State which you chose in the SCORE and why; **do not duplicate
the bound.**

Call sites: mirror `stop-faced` / `require-stopped` with gate equivalents so a caller that only needs
"it happened" and a caller that treats failure as a local defect both have a named helper, and the
generated method stops deciding for everyone.

## Blast radius, measured

```
(:ns/grant …)   68 occurrences / 33 files   (circuit.wat 18, sns-fanout.wat 5)
(:ns/revoke …)   4
.rs / .jsonl     0   (verified, not assumed)
```

Count with `grep -o … | wc -l`; `grep -c` counts LINES and reported 1 where there were 11 today.

## STOP triggers

1. **STOP-1 — if you find yourself re-SENDING `AllowPeer`**, stop and re-read the DESIGN's contract
   decision. `AllowPeer` *is* idempotent and the service *does* survive it, so re-ask looks right and
   is wrong: a surplus ack left on the lineage peer is read by a later `stop` as
   `Message(other) → "expected Status::Stopped"`, which is today's crash class, silent until it
   detonates elsewhere. If you believe the argument is wrong, say so — do not quietly switch.
2. **STOP-2 — if `revoke` turns out NOT to share the shape**, report it rather than forcing it.
3. **STOP-3 — if the census disagrees with 68/4/0**, report the real number.
4. **STOP-4 — if the thread arm cannot return `Applied`** (e.g. the tier check is not where the DESIGN
   says), stop: the whole shape depends on the `(None)` arm being a no-wait success.

## Verify

- `cargo build --release` **and** `cargo nextest run --release --no-run`.
- `./scripts/floor.sh`, read the **Summary line**, never a piped exit code.
- Happy path: `./target/release/wat wat-scripts/fanout/circuit.wat 2000 4 3 8192 true 1000` →
  `distinct=8000;dup=0`. ⭑ This exercises the post-spawn grant on 18 sites; a broken grant shows up
  here as a dial that outruns its gate.
- Chaos: `… 50 2 2 8192 true 1000 0 0 7 0 0 0 0 0 500` → exit 0.
- ⭑ **Negative control**: hand-construct `GateOutcome::GaveUp` and pass it to the require-helper; it
  must raise and name its bound. A verdict nobody can trip is theatre — this check caught `GaveUp`'s
  reachability in stone 2 and the `Verdict` type in the gate.

## Shape to copy

`the-owner-faces-an-outcome/` — DESIGN, BRIEF, SCORE. Same class, one tier up, and its SCORE records
the `select`-refuses-mixed-tiers finding you will otherwise rediscover.
