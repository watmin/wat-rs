# EXPECTATIONS — the rope can be looked at

**Written BEFORE the strike**, from `15ddc5e35`.

## The floor cannot give this stone

Nothing in the corpus calls the new verb, so green proves it compiles. **Rows 3 and 4 are the stone** and
they exist only because they were written before the strike — the third time this session the default gate
measures something else.

| # | what | check | expected |
|---|---|---|---|
| 1 | the verb exists, unrestricted | `grep` the intrinsic list | registered; **no** `:restricted-to` |
| 2 | `None` while running | probe, live lineage | `None` |
| 3 | ⭑⭑ `Signaled` fires from wat | probe: `signal` Kill a process child, then peek | **`Some(Signaled 9)`** |
| 4 | ⭑⭑ `Failed` fires from wat | probe: a panicking child, then peek | **`Some(Failed …)`** |
| 5 | `Closed` fires, both tiers | probe: clean exit | process `Some(Closed (Some code))`, thread `Some(Closed None)` |
| 6 | ⛔ does NOT consume | peek twice, then use the peer | second peek agrees; peer still usable |
| 7 | both tiers work | probe covers Thread **and** Process | neither raises |
| 8 | `Signaled` ≠ stopped | a SIGSTOP'd child | **not** `Signaled` — `Failed` stopped-not-terminated |
| 9 | ⛔ `close` untouched | `git diff` on its restriction/consume | **no change** to `close` |
| 10 | `CloseOutcome` still Pure | `grep` the registration | `Purity::Pure` |
| 11 | floor | `./scripts/floor.sh` → **Summary line** | **5237 passed, 0 FAIL**, no `ARM.txt` |
| 12 | tests compile | `nextest --release --no-run` | clean |
| 13 | clippy | `-D warnings` | exit **0** — this stone reaches `src/` |
| 14 | happy / chaos | the two commands | `distinct=8000;dup=0` · exit 0 |
| 15 | the FINDING is corrected | read it | item 3 records the split; two cells retired as **unreachable by design** |

## ⚠ What I will accept as a null, and what I will not

**Accept:** STOP-4 firing — the thread tier genuinely cannot be observed non-blockingly, reported with the
reason, and the stone landing process-only **only if the builder rules**. A half-verb shipped silently is
not acceptable; a half-verb *reported* is a finding.

⛔ **Reject:**
- **A consuming peek** (row 6). It is `close` renamed, and it inherits the arc-259 reasoning I spent this
  crawl establishing.
- **A `:restricted-to` on the new verb** (row 1 / STOP-2). It would rebuild the wall this stone routes
  around honestly, and the coverage gain would be zero.
- **Any change to `close`** (row 9 / STOP-3). Reversing arc 259 S2d is a doctrine ruling, not a coverage fix.
- **`Signaled` reported for a stopped child** (row 8). `src/types.rs:2113` is explicit.
- **Rows 3–4 claimed from a green floor.** The floor never calls this verb.

## Runtime prediction

**50–80 minutes.** A Rust intrinsic plus tier dispatch is the bulk; the probe needs a panicking child and a
signalled child. Floor ~490–520 s, clippy ~2–4 min.

## Trap-doors, ranked

1. ⛔ **Consuming** (row 6) — silently turns this into the thing the ruling forbids.
2. ⛔ **Shipping restricted** (row 1) — zero gain, wall rebuilt.
3. **Thread tier** (row 7 / STOP-4) — `try_wait` is process-only.
4. **Stopped vs terminated** (row 8) — the one classification that looks right and is wrong.
5. **`src/` reached** — clippy and the floor both carry more weight than in the last four stones.
