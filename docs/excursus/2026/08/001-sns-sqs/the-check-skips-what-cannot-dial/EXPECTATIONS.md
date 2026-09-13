# EXPECTATIONS — the check skips what cannot dial

**Written BEFORE the strike**, from `49bdf2b41`.

## What this stone is graded on

**A number, and a ruling made on it.** Correctness is a *read* (the construction proof); value is a *run*. The
stone succeeds if the measurement is trustworthy — **including if it says revert.**

| # | what | check | expected |
|---|---|---|---|
| 1 | ⭑ the guard is a construction no-op | read the diff | guarded on `addr-fields` emptiness only; the collector's result for that case was already empty |
| 2 | ⭑⭑ ≥3 runs WITHOUT the guard | quoted verbatim | three Summary lines, quiet box |
| 3 | ⭑⭑ ≥3 runs WITH the guard | quoted verbatim | three Summary lines, same box state |
| 4 | ⭑⭑ the ruling follows the number | the SCORE | **keep** iff the mean saving clearly exceeds ~6 s; **revert** otherwise, explicitly |
| 5 | ⛔ the refusal still fires | `nextest -E 'test(dial_declares)'` | **2 passed** |
| 6 | ⛔ the three corpus controls | `--check` sqs, circuit, sns-fanout | exit **0** each |
| 7 | ⛔ semantics untouched | `git diff` | **if KEPT:** only `:1029`'s expression wrapped; siblings and collector unchanged. **if REVERTED:** `git diff -- wat/service.wat` **EMPTY**. ⚠ This row must not fire on a correct revert |
| 8 | floor green at the right count | Summary line | **5241** |
| 9 | clippy | `-D warnings` | exit 0 |
| 10 | no `src/` | `git diff --stat -- src/` | **EMPTY** |
| 11 | happy path | `2000 4 3 8192 true 1000` | `distinct=8000;dup=0` |
| 12 | ⚠ box state stated | the SCORE | *quiet*, explicitly — this is a timing measurement |

## ⭑ Row 4 is the stone, and "revert" is a pass

I have committed in advance: **if the saving does not clear the band, the guard comes out.** A strike that keeps
it on a 3 s mean difference has failed row 4, not passed it. ⭑ And a strike that *reverts* with three clean
pairs quoted has **succeeded** — that is a measured null, which this campaign has learned to value (`p`, `j` and
`inbox-cap` are all recorded dead ends that cost real time to establish).

## ⚠ What I will reject

- **Fewer than three runs a side** (rows 2–3). The band is ~6 s; one pair proves nothing.
- **A timing number from a busy box** (row 12). This campaign published a wrong *rate* that way already.
- **Keeping the guard on an unmeasurable saving** (row 4 / STOP-2).
- **Any semantic change** (row 7 / STOP-1, STOP-3).
- **A service count quoted as evidence.** DESIGN rejects it: my cheap instrument (97 of 235 files) over-counts.

## Runtime prediction

**70–100 minutes, almost all of it floor runs** — six floors at ~525–570 s each is ~55 min of pure waiting. The
edit is one `if`.

## Trap-doors, ranked

1. ⛔ **Too few runs** (rows 2–3) — the whole stone is the number.
2. ⛔ **Keeping an unmeasurable win** (row 4).
3. **A busy box** (row 12).
4. **Over-guarding** and silently disabling the check (row 5 catches it).
5. **Reporting 5239** as the floor count (row 8) — two stones have moved it.
