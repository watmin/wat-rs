# SCORE — a raise in an op handler is a graceful stop

**SCORED.** Executor: grok, 2026-09-14, branch `sns-sqs`, HEAD `9d887547b`. Did not commit.

⛔ The headline is *"a raise in an op handler is a graceful stop"* — **not** *"a service cannot crash"*.
`:init` / `:hibernate` / `:stop` / the serve-loop frame / a Diagnostic (rterr) / a genuine Rust
panic are still fatal. **The publisher's `-run` crash is NOT fixed** (STOP-5: internal arms
dispatch through `SelfOutcome` and are not wrapped).

```
     Summary [ 557.284s] 5247 tests run: 5247 passed (9 slow), 22 skipped
```

`.floor/2026-09-14T22-05-36Z/` — `scripts/floor.sh` exit **0**, **no `ARM.txt`**.
Count **5245 → 5247** (+2 unit tests: `d1a_assertion_payload_becomes_outcome_faulted`,
`d1a_non_assertion_payload_resumes_unwind`). Not a shrink.
Clippy: **CLIPPY=0** (`cargo clippy --release --workspace --all-targets -- -D warnings`).
`cargo nextest run --release --no-run` → **NORUN=0**.

First floor on this strike was RED and is kept (not re-run):
`.floor/2026-09-14T21-50-13Z/`
```
     Summary [ 548.418s] 5247 tests run: 5246 passed (9 slow), 1 failed, 22 skipped
```
ARM: `probe_store_injector_has_teeth::store_injector_die_write_landed_and_no_success`
at `"defservice stop: expected Status::Stopped"`. Named, then drained. New floor after the drain.

---

## ⭑ THE HEADLINE — the innocent client is served

```
a-control=Ok
a-boom   =Lost
b-after  =Ok
```

`./target/release/wat wat-scripts/scratch-pad/probe-a-handler-raise-kills-the-service.wat`

v2 failed row 1 (`b-dial=connect-REFUSED`). Continue-serving from pre-op `state` is what
satisfies it. `a-boom` is still `Lost` (PeerCrashed retained) — never `Ok`.

---

## STOP-1 — Status:: match arms (grep -o, unique)

Generated `<S>::Status` match arms, not comments, not `wr::Status` / `ExitStatus` / `EdnFrameStatus`:

```
wat/service.wat extract-addr     Status::Started      + `_`
wat/service.wat /stop            Status::Stopped      + Faulted-drain + `_`
wat/service.wat /hibernate       Status::Hibernated   + Faulted-drain + `_`
wat/service.wat /grant           Status::PeersAllowed + Faulted-drain + `_`
wat/service.wat /revoke          Status::PeersDenied  + Faulted-drain + `_`
wat-scripts/fanout/circuit.wat   0 match arms (comments only at :28, :434)
```

**Smaller than "~2 in circuit.wat".** All five had `_` catch-alls, so adding `Faulted` did not
red at check time. The runtime `_` raise *is* the blast: an unsolicited `Status::Faulted` on the
lineage is read by a later `/stop` as Message(other). That is the first-floor ARM. The grant
comment already named this class (`surplus PeersAllowed ack → "expected Status::Stopped"`).

---

## The first-floor ARM, verbatim (kept)

stderr of `store_injector_die_write_landed_and_no_success`:

```
#wat.kernel/AssertionFailure {:thread "wat-thread-peer::<anon>" :message "queue: probe scan failed — peer is dead, not a broken pipe" …}
#wat.kernel/AssertionFailure {:thread "probe_store_injector_has_teeth::store_injector_die_write_landed_and_no_success" :message "defservice stop: expected Status::Stopped" :location #wat.kernel/Location {:file "wat-scripts/scratch-pad/probe-accepted-is-a-count.wat" :line 124 :col 35} … :frames [… ":queue::queue/stop" … ":sf::run-die"]}
```

Mechanism: the queue's scan raises (`AssertionPayload`) → wrap returns `Outcome::Faulted` →
Faulted arm sends `Status::Faulted` up `self` (queued on the lineage) → serve continues →
`:sf::run-die` calls `/stop` → `/stop` recvs Faulted, not Stopped → `_` raises.

Fix: `/stop` `/hibernate` `/grant` `/revoke` drain one `Status::Faulted` and re-recv (same shape
as `owner-wait-gone`'s still-emitting arm). One-level: two queued Faulted before `/stop` still
raise. The wall-test owner who recvs raw still sees the cause.

Did **not** re-run the red floor. New floor after the drain is the green above.

---

## Owner control (row 5b) — re-drawn, not patched

`recv_outcome_wall_panic_{thread,process}_admin_carries` now assert

```
#probe.Outcome/Faulted ["BOOM-CRASH-SENTINEL-9173"]
```

The cause is the raise's message, carried exactly. Not `Lost [false]`, not `Closed []`, not
`Message` with no field. Client panic tests stay `Lost [false]` (PeerCrashed retained).
rterr admin tests stay `Lost [true]` (Diagnostic is not `AssertionPayload`; the service still
dies).

---

## Row 4 — substrate panic still crashes

`runtime::tests::d1a_non_assertion_payload_resumes_unwind`: a `&'static str` payload
`resume_unwind`s; the original payload is preserved.
`runtime::tests::d1a_assertion_payload_becomes_outcome_faulted`: `AssertionPayload{message:
"handler raised"}` becomes `Outcome::Faulted` with that exact field (`assert_eq!`, not
`.contains`).
rterr column of the wall (div-by-zero) still `Lost` — a Diagnostic is not converted.

---

## Row 4b — projection

Continue-serving uses the serve fn's **own fifth parameter `state`** (pre-op, in-memory).
`hibernate-project` is **N/A** — the failed op is a no-op; there is no durable snapshot to
project. Not silent.

---

## WHAT LANDED

`wat/service.wat`
- `Outcome::Faulted [cause <- String]` (net 0 lines before bijection `:896`/`:913`: merged the
  two Stop comments, added one variant line). Did **not** patch bijection goldens.
- `<S>::Status::Faulted [cause <- String]` + `status-faulted-kw`.
- wrap moved inward onto each public handler body (the Outcome match scrutinee).
- Faulted arm: send `Status::Faulted` up `self` (SendOutcome-faced, AllowPeer shape), recur
  with pre-op `state`.
- `/stop` `/hibernate` `/grant` `/revoke` drain one Faulted and re-recv.

`src/runtime.rs`
- `Err` branch downcasts `AssertionPayload` → `Outcome::Faulted[cause]`; else `resume_unwind`.
- **`eval_inner`, not `eval_tail`.** Measured (v1): `eval_tail` TailCalls the
  `Outcome::Continue` constructor and the match never sees the value (client `Closed`). The
  wrap is the handler, not the serve recur; serve TCO lives in the Continue/Faulted arms
  outside the wrapper.
- Arity stays **2**. `src/check.rs` and `src/intrinsic/kernel/serve.rs` untouched.

`tests/services/probe_arc278_recv_outcome_wall.{rs,wat,edn}` — two admin panic goldens re-drawn.

`wat/core.wat` line count **2152**, unchanged.

---

## Happy path / row 10

```
distinct=8000;dup=0
```

(`./scripts/capped.sh --limit 8g ./target/release/wat wat-scripts/fanout/circuit.wat 2000 4 3 8192 true 1000`)

Floor wall clock **557.284 s** vs baseline **528.052 s**. Delta **+29.2 s**. Observation, not a
gate. The new branch is reached only on a caught panic; the happy path adds `catch_unwind` of
`eval_inner` per public dispatch (one per message, same as before — wrap moved, not duplicated).

---

## Row 12

No injector reaches a `"redial failed — peer is dead"` arm. Absence is a pass (D3-a). Did not
invent a fixture.

---

## GRADING MAP

| # | expected | result |
|---|---|---|
| 1 | ⭑ innocent client served | `b-after=Ok` |
| 2 | non-vacuity | `a-control=Ok` |
| 3 | ⛔ fault still reported | `a-boom=Lost` (never `Ok`) |
| 4 | ⛔ substrate panic still crashes | `d1a_non_assertion_payload_resumes_unwind`; rterr wall still `Lost` |
| 4b | projection said | pre-op `state`; hibernate-project **N/A**, not silent |
| 5 | STOP-1 census | 5 catch-all matches in the macro, **0** in circuit.wat |
| 5b | ⛔ owner sees cause | `#probe.Outcome/Faulted ["BOOM-CRASH-SENTINEL-9173"]` |
| 6 | tests compile | NORUN=0 |
| 7 | floor | **5247 passed**, 0 FAIL. Grew +2. First floor RED captured then drained |
| 8 | clippy | 0 |
| 9 | circuit unchanged | `distinct=8000;dup=0` |
| 10 | floor delta, report | **+29.2 s** (557.284 vs 528.052) |
| 11 | scope stated | this SCORE's headline |
| 12 | redial arm | **absent** — pass |
