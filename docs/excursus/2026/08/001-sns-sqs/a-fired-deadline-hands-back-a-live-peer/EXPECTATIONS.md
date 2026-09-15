# EXPECTATIONS — a fired deadline hands back a live peer

Written **before** the strike. Graded on the orchestrator's **own** re-runs.

Baseline (`ecd95e371`): floor **5249**/5249, 0 FAIL, **535.833 s** (orchestrator's own, quiet box).
clippy 0. Happy path `distinct=8000;dup=0`.

⚠ **The executor's floor wall clock is not comparable to this baseline** — four stones running:
executor +29.2 / +29.6 / +18.0 / +23.2 s where the orchestrator measured +7.5 / +0.96 / +0.24 /
+0.025 s on the same trees. Report the number; the orchestrator's run grades row 6.

| # | what | command | expected |
|---|---|---|---|
| 1 | ⭑⭑ **the process-tier desync is GONE** | `probe-a-slow-peer-desyncs-the-next-call.wat` | every call gets **its own tag** back. Today: `Answered(tag=11) sent=22` |
| 2 | ⭑⭑ **the thread-tier desync is GONE** | `probe-a-slow-peer-desyncs-thread-tier.wat` | same. ⛔ This row is why the thread dialer must remember — a pass on row 1 alone is half a stone |
| 3 | ⛔ **a dead peer answers `Lost`, never `DeadlineFired`** | kill the service, fire a deadline | `Lost`. A redial that cannot succeed must not be reported as a repair |
| 4 | ⛔ **the original handle still works** | use the same binding after the swap | it serves. The caller never learns a new name for its peer |
| 5 | ⛔ **the swap is observable** (STOP-1) | the SCORE | says how a reader learns a redial happened. Silence here means a corruption was traded for an invisible reconnect |
| 6 | floor | `./scripts/floor.sh` → **Summary** | `5249 passed` or higher, 0 FAIL. A SHRINK is a finding |
| 7 | clippy | `cargo clippy --all-targets -D warnings` | 0 |
| 8 | tests compile | `cargo nextest run --release --no-run` | clean |
| 9 | **the 13 match arms are untouched** | `git diff --stat` | `wat/service.wat` + `src/` only; no probe or corpus arm rewritten to accommodate a new shape |
| 10 | happy path unchanged | the record invocation | `distinct=8000;dup=0` |
| 11 | ⚠ **redial cost — REPORT, do not gate** | the deadline path | a round trip on a path that just lost one. Say the number |
| 12 | ⚠ **`recv-by-deadline` stated, not assumed** | the SCORE | says it is **unmeasured**. Do not fix it, do not vouch for it |
| 13 | scope stated | the SCORE's headline | `call-by-deadline` repairs its handle. Stone 3 (the publisher) untouched |

## Runtime prediction

**90–150 minutes.** The wat change is one function; the cost is the Rust-side swap intrinsic, the
thread-dialer completion across its `from_thread` sites, and two acceptance fixtures on two tiers.

## Trap-door risks, ranked

1. ⭑⭑ **Row 2 skipped.** Passing row 1 and shipping is the likely failure: the process tier is the one
   everybody tests. A primitive that repairs half its callers silently is worse than one that repairs
   none, because nobody can tell which half they are in.
2. **Row 3 collapsed into `DeadlineFired`.** If a failed redial still reports `DeadlineFired`, the new
   promise is a lie and the defect moves up one level.
3. **Row 5 unanswered.** Hidden mutation is accepted here on the condition it is visible. An
   unobservable reconnect is a new problem wearing the old one's fix.
4. **Row 4.** If the caller's binding stops working after a swap, every existing call site breaks —
   loudly, at least.
