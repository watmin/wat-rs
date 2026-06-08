# NOTE — Slice 9 prerequisite: the stability-100 soak (builder directive)

**Filed 2026-06-07 (builder, mid Stone 6.3):** *"we need to use our scripts/
whatever tool to do a 100 rounds of tests to prove all the races are gone
before we close this arc out."*

## The gate

Before the Slice-9 INSCRIPTION (the triple-close of 214 + 253 + 254) may be
written, run:

```bash
./scripts/stability-100.sh          # 100 rounds (the default)
```

The tool exists (minted in an earlier stability era): each round runs
`cargo test --release --workspace --no-fail-fast`, logging per-run
pass/fail/build-failure counts to `/tmp/stability-100.log` + a summary.
**The bar: 100/100 clean rounds — zero flake runs, zero build failures.**

## Why this gate is the arc's own thesis, mechanically

The per-round command is the RAW WORKSPACE RUN-TIER — the exact tier the
standing discipline has BANNED all campaign ("NEVER run the raw
`cargo test --workspace` — it deadlocks/leaks on the old stack"; it forced
hand-kills twice in one session). The ban existed BECAUSE of the classes
214 set out to annihilate: the ambient-stdio-ProcessPeer deadlock, the fd/
orphan leaks, the hand-wired plumbing. Post Slices 6+8 those classes have
no living member — so the soak is the arc grading its own homework:

- **100/100 clean** = the races are PROVEN gone at soak scale, AND the
  envelope era (setsid+timeout per-binary) formally ENDS — the discipline
  docs + breadcrumb update to un-ban the raw tier (a separate small stone:
  rewrite the "HARD-WON DISCIPLINES" entries that codify the ban).
- **Any flake** = a survivor exists; it gets the full failure-engineering
  treatment (extirpare: stop, read, class-kill) BEFORE the INSCRIPTION —
  the arc does not close over a flaking suite.

## Sequencing

After the Slice-6 endgame (6.3 fork-death → 6.w double-ward) and Slice 7
(brackets) + the 8.4 ship-or-cut decision; immediately BEFORE drafting the
INSCRIPTION. Budget: ~100× the workspace wall-time — run it detached, read
the summary, attach the log's summary block to the INSCRIPTION as the
proof-of-soak.

## FM-11 standing

This note is the named tracker: the INSCRIPTION cites the soak's summary
verbatim (rounds / clean / flake / build-failures) or it does not ship.
