# EXPECTATIONS — a peer wait can be bounded too

Written **before** the strike. Graded on the orchestrator's **own** re-runs.

Baseline (`e7816e543`): floor **5251**/5251, 0 FAIL, clippy 0.
Probe today: dies with *"recv-by-deadline: expected owner handle ((Thread :- [I O]) | (Process :- [I O])),
got :wat::kernel::Peer"*.

⚠ **Floor delta as a BAND, never a number** — this box's noise floor is ≥ ±16 s.
⛔ **`timeout -k` on anything that might block** — SIGTERM does not stop a blocked `wat` (125 s measured).

| # | what | command | expected |
|---|---|---|---|
| 1 | ⭑⭑ **the refusal is gone** | the committed probe | no *"expected owner handle"*. Its CONTROL prints **`TimedOut`** — the variant is producible on a `Peer` for the first time |
| 2 | ⭑⭑ **the differential still holds** | same probe, SUBJECT half | a **bare** recv still blocks. If both halves now return, the probe has stopped discriminating and proves nothing |
| 3 | ⛔ **a live peer still answers** | a peer that sends inside the deadline | `Message`, not `TimedOut`. A bound that always fires is a broken clock |
| 4 | ⭑ **`child-main` needed NO arm change** (STOP-1) | the diff | one line: `recv` → `recv-by-deadline` + the constant. Any arm edit ⇒ report it |
| 5 | ⛔ **the tier comes from `peer-wire?`** (STOP-2) | the diff | not from env/peer-kind. `select` refuses mixed tiers |
| 6 | ⛔ **`inert` has a safe source** (STOP-5) | the SCORE | a sentinel a peer could never legitimately send, or a stated parameter |
| 7 | ⭑ **every service still starts** | `./scripts/floor.sh` | `5251 passed` or higher, 0 FAIL. `child-main` is in every service, so the floor IS this test |
| 8 | ⛔ **`:deadline-ms` not revived** (STOP-3) | the diff | no clause of that name; D4 refuses it by macro error |
| 9 | ⛔ **only `child-main` converted** (STOP-4) | the diff | 258 other bare recvs untouched |
| 10 | ⚠ **the startup constant, justified** | the SCORE | the number **and why** — not an unexplained literal |
| 11 | clippy | `--all-targets -D warnings` | 0 |
| 12 | tests compile | `cargo nextest run --release --no-run` | clean |
| 13 | scope stated | the SCORE's headline | a hang became a **named death**. Recovery is the deferred supervision ruling |

## Runtime prediction

**120–180 minutes.** Widening an intrinsic reaches `message.rs`, `check.rs` and `runtime.rs`; the wat
side is one line. The cost is the tier path and the `inert` decision.

## Trap-door risks, ranked

1. ⭑⭑ **Row 2 collapsing.** If the probe's SUBJECT half stops blocking, the differential is inert and we
   would have "proved" the fix with a control alone — the exact failure this session hit eight times.
2. **Row 3 inverted.** A bound that fires on a healthy peer would break every service's startup, loudly,
   on the floor — which is why row 7 is the real gate.
3. **Row 5.** A mixed-tier `select` is a runtime refusal, and the env is unavailable at this site.
4. **Row 6.** A timer payload a peer could also send makes a real message indistinguishable from a
   timeout — a silent wrong answer, the worst class this session found.
