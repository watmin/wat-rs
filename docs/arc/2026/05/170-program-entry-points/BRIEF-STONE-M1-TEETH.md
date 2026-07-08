# BRIEF — Arc 170 M1-teeth: deterministic revoke-refusal (a .rs+.wat test pair)

**You are building a test that proves the capability circuit's revoke actually BITES:** a pid GRANTED into
a service's allow-set is *admitted* on a real dial, and after REVOKE the **same live pid** is *refused* —
deterministically (no race). Build it as a `tests/services/` `.rs`+`.wat` pair, exactly how the corpus does
it (Rust test slurps the `.wat` off disk via `startup_from_file`, evals `(:user::compute)` in a frozen world,
asserts `Ok`/`Err`).

## Read in order (the rooms — these are your copy-references)

1. **`tests/services/probe_arc209_c0b3bb_bounced.rs` + `probe_arc209_c0b3bb_bounced_bounced.wat`** — THE
   TEMPLATE. Study: the harness (`startup_from_file` → `parse_one!("(:user::compute)")` → `eval_in_frozen`
   → assert `Err` == the dialer was refused and DIED); the `spawn-program'` stranger that receives a leaked
   addr, `connect'`s, `send'`s, `recv'`s → on refusal the `recv'` EOFs → the child RAISES → dies → the owner's
   `recv'` on that child RAISES → `compute` raises → the Rust test asserts `Err`. Also note its TWO-test
   structure (`owner_served_via_birth_seed` + `stranger_is_bounced`). Note the `#[test]`s FORK and run
   `--test-threads=1`.
2. **`scratchpad/probe-m1-grant-admits.wat`** — GREEN, already proven (`./target/release/wat` → `"echo:hi"`).
   COPY its idioms verbatim: (a) the `:probe::Echo` defsurface + `:probe::echo'` defservice; (b) the prober
   spawned via `(:wat::kernel::spawn-program' (:wat::spawn::process) (:wat::core::forms …))` whose forms
   **RE-DECLARE the `:probe::Echo` surface** (the child evals in a FRESH world — omit it and the child dies
   with `UnresolvedReferences`); (c) `(:wat::kernel::peer-pid prober)` → `(:wat::core::Some p)` (the pid);
   (d) `(:probe::echo'/grant eh (:wat::core::Vector :wat::core::i64 p))` to grant.
3. **`scratchpad/probe-m1-service-pid.wat`** (green) + **`scratchpad/s2s-revoke-probe.wat`** — for the
   `(:probe::echo'/revoke eh (:wat::core::Vector :wat::core::i64 p))` call form. **DO NOT copy s2s-revoke's
   `caller2` refusal approach — it RACES** (grant-then-revoke vs the child's connect'); M1 must be
   deterministic (see below).
4. **`src/capability/policy.rs::only_my_peers_admits_lineage_and_refuses_everyone_else`** — the predicate is
   ALREADY unit-tested. DO NOT re-prove it; M1 proves only OUR grant/revoke drive the allow-set.

## What to build

**Two `.wat` fixtures + one `.rs` with two tests** (mirror the template's two-proof structure).

### `tests/services/probe_arc170_m1_teeth_admitted.wat` — the admit-via-grant control
This is `scratchpad/probe-m1-grant-admits.wat` promoted: change `:user::main -> :wat::core::nil` /`println`
to **`:user::compute -> :wat::core::String`** returning the echoed reply (`out`). The prober dials A once,
sends the reply up; the owner grants (via `peer-pid` → `/grant`), sends the addr, `recv'`s the reply, returns
it. **`compute` returns `"echo:hi"`.**

### `tests/services/probe_arc170_m1_teeth_revoked.wat` — the teeth
A TWO-PHASE prober. Its `spawn-program'` forms (re-declaring the `:probe::Echo` surface):
```clojure
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [self (:wat::program::self-peer :wat::core::String
             :wat::kernel::Address'<probe::Echo::Op,probe::Echo::Reply>)
     addr (:wat::kernel::recv' self)                                   ; A's addr (down)
     c1   (:wat::kernel::connect' addr)
     er1  (:probe::Echo/echo c1 (:probe::Echo::EchoRequest "hi"))       ; dial #1 — ADMITTED
     _    (:wat::kernel::send' self (:probe::Echo::EchoResponse/reply er1))  ; report "echo:hi" UP
     _sig (:wat::kernel::recv' self)                                   ; BLOCK for re-dial (a 2nd addr)
     c2   (:wat::kernel::connect' addr)
     er2  (:probe::Echo/echo c2 (:probe::Echo::EchoRequest "hi"))       ; dial #2 — after revoke: BOUNCED → RAISE → die
     _    (:wat::kernel::send' self (:probe::Echo::EchoResponse/reply er2))]  ; ← LOAD-BEARING (added 2026-07-08): dial #2 reply UP, ONLY reached if ADMITTED. WITHOUT it the test is VACUOUS — a clean prober exit ALSO disconnects the channel → recv' raises → Err either way. See DESIGN-STONE "The vacuity trap".
    nil))
```
Owner `:user::compute -> :wat::core::String`:
```clojure
[eh (:probe::echo'/start :locus (:wat::spawn::process) :record (:probe::echo'::Record))
 ea (:probe::echo'::Handle/addr eh)
 prober (:wat::kernel::spawn-program' (:wat::spawn::process) (:wat::core::forms …))
 ... (:wat::core::match (:wat::kernel::peer-pid prober) -> :wat::core::String
       ((:wat::core::Some p)
         (:wat::core::let
           [_  (:probe::echo'/grant  eh (:wat::core::Vector :wat::core::i64 p))  ; ack'd PeersAllowed
            _  (:wat::kernel::send' prober ea)                                    ; give addr → dial #1
            r1 (:wat::kernel::recv' prober)                                       ; "echo:hi" (dial #1 admitted)
            _  (:probe::echo'/revoke eh (:wat::core::Vector :wat::core::i64 p))   ; ack'd PeersDenied — pid GONE
            _  (:wat::kernel::send' prober ea)                                    ; re-dial signal (AFTER revoke ack)
            r2 (:wat::kernel::recv' prober)]                                      ; dial #2 → prober dies → RAISES
           r2))                                                                    ; unreached; compute raises
       (:wat::core::None (:wat::kernel::assertion-failed! "peer-pid None on process prober"
                           :wat::core::None :wat::core::None)))]
```
**DETERMINISM (the whole point):** the re-dial signal (`send' prober ea` the 2nd time) is sent only AFTER
`echo'/revoke` returns (it blocks on the `PeersDenied` ack — the pid is provably gone). So revoke
happens-before the re-dial happens-before dial #2. NO race. `compute` RAISES (dial #2 bounced).

Note the prober's self-peer down-type is `Address'<…>` and the re-dial signal is a **second `Address'`**
(`send' prober ea` again) — one type on the channel, used as the go-signal. It sends `String` up.

### `tests/services/probe_arc170_m1_teeth.rs`
Two `#[test]`s, copying the template's harness exactly:
- `granted_prober_is_admitted` → `startup_from_file("tests/services/probe_arc170_m1_teeth_admitted.wat")` →
  `eval_in_frozen (:user::compute)` → assert `Ok(Value::String)` == `"echo:hi"`.
- `revoked_prober_is_bounced` → `…_revoked.wat` → assert the result is `Err(_)` (the revoked dial #2 bounced,
  the prober died). Together: grant admits (test 1 + test 2's dial #1) / revoke refuses (test 2's raise).

## Blast radius
`tests/services/` ONLY — 1 `.rs` + 2 `.wat`. NO `src/` changes. NO new intrinsics (`peer-pid`, `echo'/grant`,
`echo'/revoke` all exist + are proven). NO changes to any existing test.

## STOP triggers (halt + report; do NOT improvise around)
- **STOP-1** — if the prober's dial #2 (after revoke) is still SERVED (not bounced), STOP and report: the
  revoke did not gate the new accept. This is a real substrate finding, not a test bug to paper over.
- **STOP-2** — if `(peer-pid prober)` returns `:None` (should be `Some` — proven green), STOP and report.
- **STOP-3** — if the checker rejects a surface/service form and the fix is not obvious from the
  copy-references, STOP and report the exact checker message (it teaches the correct form; do not thrash).

## How to work / iterate
Run from the crate root. Iterate with the harness, FOREGROUND-blocking (never background-and-poll):
```
cargo nextest run --release -p wat --test probe_arc170_m1_teeth --test-threads=1
```
Build without `--features simd`. When both tests pass, run the whole floor ONCE to confirm no regression:
```
cargo nextest run --release
```

## Expectations (report each with its real result)
| what | command | expected |
|---|---|---|
| admit-via-grant control | `cargo nextest run --release -p wat --test probe_arc170_m1_teeth -E 'test(granted_prober_is_admitted)' --test-threads=1` | 1 passed (`Ok "echo:hi"`) |
| the teeth | `cargo nextest run --release -p wat --test probe_arc170_m1_teeth -E 'test(revoked_prober_is_bounced)' --test-threads=1` | 1 passed (`Err` — bounced + died) |
| whole floor | `cargo nextest run --release` | 0 NEW failures (report the summary line) |

Return: the two test outcomes (with the actual `Ok`/`Err` values you observed), the floor summary line, the
final file contents, and any STOP you hit.
