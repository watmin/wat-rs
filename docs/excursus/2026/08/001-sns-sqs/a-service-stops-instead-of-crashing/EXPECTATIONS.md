# EXPECTATIONS — a service stops instead of crashing

Written **before** the strike so the result cannot move the goalposts. Graded on the orchestrator's
**own** re-runs, never from the report.

Baseline, this tree (`79179f62f`, clean): floor **5245**/5245, 22 skipped, 0 FAIL, **528.052 s** (my
own run). clippy `--all-targets -D warnings` **0**. Circuit happy path `distinct=8000;dup=0`,
~22.5–25 s.

## The gate — relations and observations, not thresholds

⭑ Rows 1–4 are the acceptance gate and they are **relations**, deliberately: a threshold couples a
gate to a parameter another stone may move, and that cost a floor red this week.

| # | what | command | expected |
|---|---|---|---|
| 1 | ⭑ **the innocent client survives** — the headline | `./target/release/wat wat-scripts/scratch-pad/probe-a-handler-raise-kills-the-service.wat` | `b-` line reports a **served call**, no longer `connect-REFUSED` |
| 2 | **non-vacuity: the service still works** | same run | `a-control=Ok` — unchanged. If this regresses, nothing else on this card means anything |
| 3 | ⛔ **the fault is still reported, not swallowed** | same run | `a-boom` reports a FAILURE outcome (`Lost`/`Closed`/`TimedOut`) — **never `Ok`**. A success here is the stone failing in the most dangerous direction |
| 4 | ⛔ **a substrate bug still crashes** (negative control, executor writes it) | a handler tripping a genuine Rust panic, not `assertion-failed!` | the service DIES, as today. A wall that swallows interpreter bugs is worse than no wall |
| 4b | **the projection actually ran** (or STOP-2 was reported) | the SCORE's own words | it says whether durable state was projected via `hibernate-project`, or reports that it could not be — never silent |
| 5 | ⭑ **the seam's value is `nil`-typed** (STOP-1's premise, re-derived not trusted) | read `wat/service.wat:3498` and `:3937` | `(defn ~serve-name ~serve-params -> :wat::core::nil …)`. If it is anything else the route is refuted again |
| 6 | tests compile (the build does NOT compile them) | `cargo nextest run --release --no-run` | clean |
| 7 | **the floor** | `./scripts/floor.sh` → **Summary line** | `5245 passed`, 0 FAIL. A test count that GREW is fine and must be said; a count that SHRANK is a finding |
| 8 | lints | `cargo clippy --all-targets -D warnings` | 0 |
| 9 | the circuit is unchanged | `./scripts/capped.sh --limit 8g ./target/release/wat wat-scripts/fanout/circuit.wat 2000 4 3 8192 true 1000` | `distinct=8000;dup=0` |
| 10 | ⚠ **per-dispatch cost — REPORT, do not gate** | floor wall clock vs the 528.052 s baseline | report the delta. The happy path adds no work (the new branch is reached only on a caught panic), so ~0 is expected — **say the number either way**. No threshold: it is an observation the builder rules on |
| 11 | **scope is stated, not overclaimed** — REPORT | read the SCORE's own headline | it says *"a raise in an op handler is a graceful stop"* and NOT *"a service cannot crash"*. `:init`/`:hibernate`/`:stop`/the serve-loop frame are still fatal |
| 12 | one of the 19 real arms behaves | pick one `"redial failed — peer is dead"` arm and drive it | the service stops gracefully instead of dying. ⚠ If no existing injector can reach one, **say so** — that is D3-a's whole point and the absence is the finding, not a gap to paper over |

## Runtime prediction

**90–150 minutes.** The Rust branch is small and the precedent is exact; the cost is concentrated in
(a) the arity change rippling through `check.rs`'s inference and the intrinsic declaration, and
(b) the STASH-DANCE, because `wat/service.wat` is frozen into the binary and a Rust change ships
alongside it. Plus one floor run at ~9 minutes.

⛔ **The floor's TIMEOUT WALL is the number to respect, not the runtime** — three tests sit at
19–25 s against a 30 s terminate wall, ~1.2× headroom, and a floor already timed out at 40 s under
contention this week. If row 10's delta is material, that headroom is where it lands first.

## Trap-door risks, ranked

1. ⭑⭑ **Row 3 inverted.** The most likely way to "pass" this stone wrongly is to make the caller see
   `Ok` — a graceful stop that reports success. That is a failure collapsed into a success and it
   would be worse than today's crash, because today's crash is at least loud.
2. **The pre-op / post-op state confusion.** Using anything the panicking body produced is the one
   way to turn this into silent state corruption. The DESIGN pins it; the SCORE must say which value
   was actually passed.
3. **The trampoline** (STOP-3). Returning a value from a `catch_unwind` that must stay in tail
   position is the subtlest part of the diff.
4. **Per-dispatch closure allocation** (STOP-6). Cheap to get right up front, expensive to discover
   in a floor timeout — this seam runs on every message.
5. **Row 12 may be unreachable.** Every chaos knob we own is *momentary* — suppress a reply, oversize
   a frame — and the one terminal knob (`store-die-bp`) points at the store. Reaching a real
   `"redial failed"` arm may need the 6-edge injector, which is D3-a's next stone. **Reporting that
   absence is a pass for row 12**; inventing a bespoke fixture to fake it is not.
