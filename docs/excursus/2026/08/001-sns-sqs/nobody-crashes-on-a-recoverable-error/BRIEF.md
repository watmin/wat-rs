# BRIEF — nobody crashes on a recoverable error (the census)

**Read `DESIGN.md` beside this first.** It carries the one contract decision (what counts as
recoverable), the variant table by direction, why `RecvOutcome::Stopped` is excluded, and five
trap-doors — the first of which is the whole validity of the result.

⛔ **REPORT-ONLY. This stone migrates NOTHING.** If you find yourself editing an arm, stop.

## The work, in one paragraph

Widen the struck D5 census from one variant to **every recoverable transport variant, in both
directions**, and report. The instrument already walks form trees, already classifies by `head=` and
`home=`, already recognises `assertion-failed!`/`raise!`/`panic!`, and already carries controls. Its
variant filter is a single predicate. Add a **direction** axis derived from the variant family
(`ServiceEvent::*` is service-facing-a-client; everything else is a caller facing its peer), give every
newly-covered family **its own PRESENT and ABSENT control**, and produce a ranked report the builder
can rule on.

## The rooms

| where | why you are going there |
|---|---|
| `wat-scripts/census-malformed-raising.wat:83` | ⭐ THE FILTER — one predicate, `(contains? (kw-name node) "RecvOutcome::Malformed")`. This is the change. |
| same file, `:100`–`:105` | what counts as a raise: `assert­ion-failed!` / `raise!` / `panic!`. Unchanged. |
| same file, `:14`–`:16` and `:420` | ⭑ THE CONTROLS — CONTROL A must be ABSENT, CONTROL B must be PRESENT. Read how they are asserted, then write one pair **per newly-covered family** (trap-door 1). |
| same file, `:107` | the non-raising body shape, and its own STOP-5 note about `let`/`do`/`if` bodies. |
| same file, header `:6`–`:9` | the stdin usage — a sorted EDN vector of paths, the standard migration harness shape. |
| `docs/excursus/2026/08/001-sns-sqs/the-malformed-census/FINDING-the-malformed-census.md` | D5's output and its two axes. Your report is this, widened by one axis. |
| `wat/service.wat:2539` / `:2549` | the two service-side arms. `:2549` WILL appear — flag it per the DESIGN; it is unreachable-by-unknown-kind, not rankable work. |
| `FINDING-the-client-vanished-arms-are-unreached.md` | why `:2549` is flagged rather than ranked, and why a client-side injector is coverage rather than a bug chase. |

## Implementation sketch

```wat
;; :83 becomes a family/variant table rather than one string
(:census::recoverable? node)   ;; RecvOutcome::{Lost,Closed,TimedOut,Malformed}
                               ;; SendOutcome::{Lost,Closed} · TrySendOutcome::{Lost,Closed,WouldBlock}
                               ;; CallOutcome::{Lost,Closed,DeadlineFired,Malformed}
                               ;; ConnectOutcome::{Refused,Rejected,Failed}
                               ;; ServiceEvent::{Closed,Lost,Malformed}
;; and a third reported axis, derived not guessed:
;;   direction = ServiceEvent::* ? "service-faces-client" : "client-faces-service"
```

Report rows unchanged in shape (`file:line:col head= home= raises=`) plus `variant=` and `direction=`.
Totals per (direction × family × home × head).

⭑ **`RecvOutcome::Stopped` gets its OWN bucket, reported and not folded in** — the DESIGN explains why.

## Blast radius

One census file under `wat-scripts/`. **Everything else: 0.** Confirm rather than inherit.

## STOP triggers

1. ⛔ **STOP-1 — if any newly-covered family cannot be given BOTH controls** (one arm that must be
   PRESENT, one that must be ABSENT), STOP and say which. A census without a control is a number, and
   five of this session's wrong claims came from exactly that.
2. ⛔ **STOP-2 — if extending the filter breaks `census-malformed-raising.wat`'s own controls**, STOP.
   That instrument is struck and D5's SCORE cites its numbers; extend a copy or parameterize so the
   D5 controls still pass **unchanged**.
3. **STOP-3 — if you are tempted to migrate an arm**, STOP. Report-only. The builder rules the sequence.
4. **STOP-4 — if `Stopped` cannot be separated from the recoverable set**, STOP and report. Folding
   shutdown paths into this population would sweep in work nobody ruled.
5. **STOP-5 — if a raising body is a `let`/`do`/`if` the walker cannot classify**, report it as
   `raises=UNCLASSIFIED` rather than guessing. The D5 instrument already has a note on this shape.

## Verify

- `cargo build --release` **and** `cargo nextest run --release --no-run` (the floor gate parses and
  type-checks **every** `.wat` under `wat-scripts/`).
- `./scripts/floor.sh`, read the **Summary line**, never a piped exit code.
- ⭑ **The D5 controls still pass** — run the original census unchanged and confirm its numbers.
- ⭑ **Every new family's PRESENT control fires and ABSENT control does not.** Name each in the SCORE.
- ⭑ **Quote exemplars, not just totals** — at least one real row per (direction × family) bucket. A
  count is not a finding until you have looked at the matches.
- ⚠ Report the **`head=`/`home=`** split so live-service arms are separable from probes and tests. D5's
  ratio was 544 total against ~59 in scope; a total with no split is not rulable.

## Shape to copy

`wat-scripts/census-malformed-raising.wat` itself, and
`the-malformed-census/FINDING-the-malformed-census.md` for how the output was written up so the builder
could rule a scope constraint off it.
