# Arc 170 — STONE M1-teeth: the deterministic revoke-refusal proof (2026-07-09)

> **The teeth.** The e2e (`probe-cap2-e2e.wat`) proved grant/revoke **fire**. This stone proves they
> **bite** — a granted pid is *admitted* on a real dial, and after revoke the **same live pid** is
> *refused*, **deterministically** (no race). It is the proof that was PROBANDVM in
> `REALIZATION-CAPABILITY-CIRCUIT.md`.

## Why (the room this lands in)

The only prior "refusal" evidence is `scratchpad/s2s-revoke-probe.wat` — and it is a **race**: its `caller2`
*"grants then immediately revokes its own pid, racing caller2's own :init connect'"*, and it admits *"if the
race did not land the revoke before caller2's connect (rare), whatever caller2 prints."* Non-deterministic by
its own words. M1 replaces that preview with a **deterministic** proof.

**Determinism comes from ordering by acks + explicit signals**, not timing:
```
grant  (ack'd: PeersAllowed)  ─┐
prober dial #1 → ADMITTED      │  happens-before
prober reports "admitted" ↑    │
owner recv's the report        │
revoke (ack'd: PeersDenied)   ─┤  the pid is PROVABLY gone from A's allow-set
owner signals "re-dial" ↓      │  sent AFTER the revoke ack → happens-before dial #2
prober dial #2 → REFUSED        ┘  bounced → EOF → raise → die
```
The revoke's `PeersDenied` ack happens-before the owner sends the re-dial signal, which happens-before the
prober's second `connect'`. There is no window for a race.

## The honesty boundary (grounded in the arc-272 precedent — DO NOT re-walk the dead-end)

Per `272/DESIGN-STONE-comms-policy.md:69-88`: we do **not** integration-test a "recycled pid," and we do
**not** test the cross-uid case — both only exercise **the kernel's `SO_PEERCRED`/pid-assignment axiom, not
our code**. *"You do not test your axioms."* The predicate (`admits` false for pid ∉ lineage) is **already
unit-tested** (`src/capability/policy.rs::only_my_peers_admits_lineage_and_refuses_everyone_else`). This stone
adds **only our code**: that *our revoke removes the pid* and *a real dial by that same-uid live pid is then
bounced at the live gate*. Same uid throughout; the only thing that changes is allow-set membership, driven by
our grant/revoke.

## What it is — two fixtures + two tests, mirroring `probe_arc209_c0b3bb_bounced`

The proven shape to copy is `tests/services/probe_arc209_c0b3bb_bounced.rs` (two tests:
`owner_served_via_birth_seed` + `stranger_is_bounced`; `startup_from_file` → `eval_in_frozen (:user::compute)`;
`Err` == the dialer was refused and died). M1 is its grant/revoke twin.

### Fixture 1 — `tests/services/probe_arc170_m1_granted_admitted.wat` (the admit-via-grant control)
The GREEN disconfirming probe `scratchpad/probe-m1-grant-admits.wat`, promoted: `:user::compute -> String`,
returns the echoed reply instead of `println`. Owner starts A (defservice, process), spawns a prober
(`spawn-program'` process that **re-declares the `:probe::Echo` surface** — the child evals in a fresh world),
reads `(peer-pid prober)` → grants it, sends A's addr down, the prober dials → **admitted** → sends the reply
up → owner returns it. **`compute` returns `"echo:hi"`.**

### Fixture 2 — `tests/services/probe_arc170_m1_revoked_bounced.wat` (the teeth)
The two-phase deterministic circuit above. The prober's self-peer is `<String, Address'<Op,Reply>>` (sends
`String` up, receives `Address'` down); the **re-dial signal is a second `send' prober ea`** (a second
`Address'` down — one type on the channel, used as the go-signal). Sketch:
```clojure
;; prober forms (spawn-program' process): re-declare :probe::Echo surface, then —
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [self (:wat::program::self-peer :wat::core::String
             :wat::kernel::Address'<probe::Echo::Op,probe::Echo::Reply>)
     addr (:wat::kernel::recv' self)                                   ; A's addr
     c1   (:wat::kernel::connect' addr)
     er1  (:probe::Echo/echo c1 (:probe::Echo::EchoRequest "hi"))       ; ADMITTED
     _    (:wat::kernel::send' self (:probe::Echo::EchoResponse/reply er1))  ; report up
     _sig (:wat::kernel::recv' self)                                   ; BLOCK for re-dial (2nd addr)
     c2   (:wat::kernel::connect' addr)
     er2  (:probe::Echo/echo c2 (:probe::Echo::EchoRequest "hi"))       ; after revoke: BOUNCED → RAISE → die (before the send below)
     _    (:wat::kernel::send' self (:probe::Echo::EchoResponse/reply er2))]  ; ← LOAD-BEARING: dial #2 reply UP, ONLY reached if ADMITTED
    nil))
;; owner (:user::compute -> String):
[eh (:probe::echo'/start :locus (:wat::spawn::process) :record (:probe::echo'::Record))
 ea (:probe::echo'::Handle/addr eh)
 prober (:wat::kernel::spawn-program' (:wat::spawn::process) (:wat::core::forms ...))
 ... (:wat::core::match (:wat::kernel::peer-pid prober) -> :wat::core::String
       ((:wat::core::Some p)
         (:wat::core::let
           [_  (:probe::echo'/grant  eh (:wat::core::Vector :wat::core::i64 p))  ; ack'd PeersAllowed
            _  (:wat::kernel::send' prober ea)                                    ; give addr
            r1 (:wat::kernel::recv' prober)                                       ; "echo:hi" (dial #1 admitted)
            _  (:probe::echo'/revoke eh (:wat::core::Vector :wat::core::i64 p))   ; ack'd PeersDenied — pid GONE
            _  (:wat::kernel::send' prober ea)                                    ; re-dial signal (after revoke)
            r2 (:wat::kernel::recv' prober)]                                      ; dial #2 → prober dies → RAISES
           r2))                                                                    ; unreached; compute raises
       (:wat::core::None (:wat::kernel::assertion-failed! "peer-pid None on process prober" ...)))]
```
**`compute` raises** — the revoked dial #2 bounced. Rust `revoked_prober_is_bounced` asserts `Err`.

### The two Rust tests — `tests/services/probe_arc170_m1_teeth.rs`
Copy `probe_arc209_c0b3bb_bounced.rs` verbatim in shape (`startup_from_file`, `parse_one!("(:user::compute)")`,
`eval_in_frozen`, `--test-threads=1`, these FORK):
- `granted_prober_is_admitted` → `Ok(Value::String("echo:hi"))`.
- `revoked_prober_is_bounced` → `Err(_)`.

**Together they are the teeth:** test 1 (and test 2's dial #1, which must succeed for the flow to reach the
revoke) prove grant *admits*; test 2's raise proves revoke *refuses* — same uid, same live pid, deterministic.

## Out of scope / rejected (affirmative cuts)
- **`PPID == owner` (no reparent) + the B←A bracket-pool realism** → the **M1-pool** follow-on strike (the
  prober's owner outlives it by construction — reparent risk lives in the *pool*, not the prober). Named, not
  deferred.
- **No recycled-pid test, no cross-uid test** — the arc-272 dead-end (tests the kernel). Predicate is
  unit-tested; we prove only our grant/revoke.
- **No race-based refusal** (the `s2s-revoke-probe` anti-pattern). If the design cannot order the revoke ack
  before the re-dial signal, STOP.

## The vacuity trap (found by measurement, 2026-07-08) — the prober MUST send dial #2's reply up

The first cut of the fixture ended the prober on `nil` after dial #2 (no send-up). **That test was VACUOUS:**
a `recv'` on a peer that exited *cleanly* raises `"process channel disconnected"` — the *same* `Err` as a
peer that *crashed on a bounce*. So `revoked_prober_is_bounced` asserted `Err` **whether or not the revoke
bit**: with revoke → dial #2 bounces → prober crashes → `Err`; without revoke → dial #2 admits → prober
exits clean → channel disconnects → `Err`. Green, but proving nothing.

**Caught by a counterfactual** (`scratchpad/probe-m1-cf-norevoke.wat`): the exact circuit with only the
`echo'/revoke` line removed *still raised* → the test did not discriminate. **Fix:** the prober sends dial
#2's reply UP (`send' self (EchoResponse/reply er2)`), so a *successful* dial is observable as `Ok`. Now:
without revoke → dial #2 admits → reply travels up → owner `r2 = "echo:hi"` → `Ok` (verified,
`probe-m1-fix-norevoke.wat` → `"echo:hi"`); with revoke → dial #2 bounces → prober crashes *before* the send
→ `Err` (verified, `probe-m1-fix-revoke.wat` → raise). The test is now **self-guarding**: a revoke
regression flips test 2 from `Err` to `Ok`, going RED. **General lesson: a pass/refuse test must make the
PASS observable, or `Err` cannot discriminate refuse from any-other-failure.**

## Expectations (scorecard — weighed by the orchestrator's OWN re-run, 2026-07-08)

The target is **`services`** (`tests/services/mod.rs`; files auto-registered by `build.rs` from presence —
`--test probe_arc170_m1_teeth` is NOT a target, a phantom that misled once).

| what | command | result (own re-run) |
|---|---|---|
| both teeth tests | `cargo nextest run --release -p wat --test services -E 'test(m1_teeth)' --test-threads=1` | ✅ 2 passed (`granted`→`Ok "echo:hi"`, `revoked`→`Err`) |
| discrimination | `./target/release/wat scratchpad/probe-m1-fix-norevoke.wat` (no revoke) vs `…-fix-revoke.wat` | ✅ `"echo:hi"` vs raise — revoke is load-bearing |
| the predicate (already ours) | `cargo nextest run --release -p wat --lib -E 'test(only_my_peers_admits_lineage)'` | (unchanged — not re-run) |
| whole floor | `cargo nextest run --release` | ✅ 0 new: only the expected-red `no_inlined_wat` lint + the `sigterm…` flake (PASS isolated `--test-threads=1`) |

## RESUME after this stone
M1-teeth green → **M1-pool** (B←A, granted bracket pool, workers→A→B, revoke-on-reap allow-set check +
`PPID == owner`), then the `map` arg-order flip (fn-first).
