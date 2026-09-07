# EXPECTATIONS — curing the oracle's `rule-produces`, written BEFORE the strike

## Scorecard

| what | command | expected |
|---|---|---|
| the defect, before | `cargo run --release --bin wat -- wat-scripts/scratch-pad/arc278-produced-type-userfn-facts.wat` | `ORACLE facts [0 1 0]` — quote it, it is the "before" |
| the defect, after | same command | **`ORACLE facts [0 1 1]`**, matching native and Clara |
| oracle strata, after | same command | `Rate` raised — no `a2::mk-rate` key |
| names agree, after | `cargo run --release --bin wat -- wat-scripts/scratch-pad/arc278-produced-type-userfn-head.wat` | MEASURE `["pt::Rate"]`, was `["pt::first-rate"]` |
| ANCHOR unmoved | same | `["pt::Rate"]` — unchanged. If this moves the cure is wrong |
| the gate flips | `cargo nextest run --release -E 'test(produced_type_userfn)'` | 2 passed, both asserting AGREEMENT, messages rewritten |
| ⛔ the whole floor | `scripts/floor.sh` | 5475, **0 fail** — and if not, a captured red is the deliverable |
| clippy | `cargo clippy --all-targets --release -- -D warnings` | rc=0 |

## Runtime prediction

40–70 min. The edit is small; the `wat/` rebuild and a full floor are most of it, and the floor is
the part that matters because oracle behaviour moved.

## Trap doors, named in advance

- **⛔ A fallback to the colon-strip.** The single worst outcome. It would make the cure work on the
  fixture and silently keep the old answer everywhere resolution is imperfect — the defect
  surviving inside its own fix. STOP-1 exists for this.
- **Un-breaking a test that was asserting the wrong answer.** Oracle behaviour is changing on
  purpose. Any red is a question — *was this test pinned to the oracle's mistake?* — not a chore.
  Answering it wrongly silently re-pins the defect somewhere new.
- **The anchor moving unnoticed.** `:pt::plain` is the only thing standing between "the engines
  agree" and "the extractor returns nothing on both sides".
- **A stale binary.** `wat/` is `include_str!`'d; `~/.cargo/bin/wat` is NOT rebuilt by cargo. Use
  `cargo run --release --bin wat`. One measurement has already been retracted in this arc for this.
- **Messages left stale after the flip.** An assertion that now demands agreement under a message
  explaining why they diverge is the exact class this arc has spent the session removing.
