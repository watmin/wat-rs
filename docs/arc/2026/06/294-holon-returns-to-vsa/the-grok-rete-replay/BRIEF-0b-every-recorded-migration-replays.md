# BRIEF 0b — every recorded migration replays: the ledger goes to ZERO

Anchor: `/home/john/work/holon/wat-rs`. Verify with `pwd`. **Never use worktrees.** Do not touch
`~/work/holon/` (the frozen root).

## ⛔ TREE STATE — read this first

```
branch   replay/grok-rete   <-- YOU ARE HERE
HEAD     0d51a1305          0a landed · floor 5382/5382 · clippy 0 · pushed
gate     tests/cli/every_recorded_migration_replays.rs  — 13 fixtures · 83 FROZEN_LEDGER · 0 runes
```

## The work, in one paragraph

0a made the 11 grep-rules codemods work on today's binary, and it landed a gate that replays
recorded migrations against fixtures. 0b finishes the job the builder ruled: **all 96 recorded
migrations** replay. Every `FROZEN_LEDGER` entry becomes one of two things:
- a fixture (a history-derived oracle, with near-misses), or
- the ONE permanent exemption, `rune:replay(unreadable-preimage)`, carrying the reader's verbatim
  error.

Along the way:
- A codemod found ROTTED (it fails, or it disagrees with its history, on a pre-image today's reader
  CAN read) is REPAIRED, never exempted.
- Every codemod declares its `SCOPE`.
- The ledger ends empty, and the constant is deleted.

Read `DESIGN-step-0-recorded-migrations-work-on-todays-binary.md` (contract decisions 2 and 3) and
`SCORE-0a-rewriters-read-what-is-written.md`. The first is the law; the second is the shape to copy.

## Two batches, one handoff between them

**Batch 1 — the 17 chain codemods.** The replay pilot needs exactly these, in landing order: step 2
`edits-carry-the-old-text`, then 13–28: `one-param-spec`, `mandatory-typed-quasiquote-residual`,
`rename-sort-prime-to-native`, `bare-symbol-shorthand-to-fqdn`,
`rename-slipped-core-heads-to-their-homes`, `repoint-retired-heads-to-live-spellings`,
`bare-none-keyword-to-fqdn`, `break-kind-string-to-enum`, `node-kind-string-to-enum`,
`fmt-head-fqdn-to-clojure`, `variant-vector-to-tagged-map`, `match-arm-to-bracket-map-pattern`,
`assertion-failed-to-kwargs`, `positional-ctor-to-map`, `bare-variant-to-qualified`,
`variant-separator-to-dot`.
- Fixture each one, and declare SCOPE on each AND on the 11 from 0a.
- Commit on green: the gate, then `scripts/floor.sh` (read the Summary), then clippy 0.
- **Do not push.** Write `SCORE-0b-batch-1.md`, then `pulsare_yield kind=scored`.
- **STOP there.** The orchestrator verifies batch 1 and yields back before batch 2 begins, so two
  agents never work one tree.

**Batch 2 — the other 66, the repairs, the exemptions, and the gate's final shape.** Commit in
batches of your choosing, each on green. Write `SCORE-0b-batch-2.md`, then `pulsare_yield kind=scored`.

## The oracle rule (unchanged from 0a — the heart of it)

`after.post` comes from HISTORY or from the codemod's header spec. **NEVER from running the codemod.**
- **History:** whole top-level forms from a real file its landing commit changed. `before.pre` is
  the file at `<landing>^`, `after.post` is the file at `<landing>`. Choose forms that commit changed
  ONLY by this codemod.
- **Five landing commits changed no `.wat` at all:** `bare-none-keyword-to-fqdn`,
  `parametrics-take-a-type-vector`, `rename-sourcefile-to-source-file`, `sweep-lint-fixes`,
  `to-faithful-clojure-net`. For these, take the first later commit that applied the codemod to the
  corpus (`git log -S'<a token it rewrites>'`). If there is none, use the header spec, and say which
  in the SCORE.
- **Every documented rewrite case** in the codemod's header (for a rules codemod, every `defrule`)
  is exercised by at least one line.
- **Every fixture carries at least one near-miss** that must stay byte-identical. Name it in the
  SCORE.
- **The table in `bootstrap/step0-probes/0b-oracles.txt`** has each stem's landing commit and how
  many `.wat` files that commit changed.

## Repair, exemption, STOP — the three dispositions for a fixture that will not pass

Run the codemod on its readable pre-image, then:
- **The reader refuses the pre-image** (a parse error before any rule or walk runs). Add the header
  rune `;; rune:replay(unreadable-preimage) — <the reader's verbatim error>`. This is the only
  exemption. Likely candidates to TEST, not assume: `angle-brackets-to-binder`,
  `tuple-parens-to-binder`, `rename-record-def-to-defrecord`, `strip-match-ascription`,
  `strip-expect-ascription`.
- **The reader accepts it, and the codemod exits nonzero or disagrees with history.** It is ROTTED.
  Repair it with the smallest change that restores its DOCUMENTED behaviour on today's binary. Put
  the root cause and the change in the SCORE.
  - Known: `to-faithful-clojure-net` (`:81` `:fix::has-ns?`, `:88` `:fix::type-shaped?`) and
    `to-faithful-clojure-rete` (`:79` `:fix::head-keyword-str?`) call user fns inside a
    `:wat::rete::where`. Main's fence admits only `:wat::rete::` ops
    (`compile-condition: where expr is not a rete primitive`, rc=2). Express each test in rete
    primitives. The fence's own message names the method.
- **The repair needs a `src/` change, changes what the codemod is documented to do, or exposes a
  missing substrate primitive.** STOP. That is a finding, not a repair.

## SCOPE — decision 3, made checkable

- One header line per codemod: `;; SCOPE: <entry> …`. An entry is either:
  - `corpus`, meaning every tracked `*.wat` outside `wat-scripts/fixes/` (a tool is never its own
    input), or
  - a repo-relative path or glob.
- Derive it from the header's usage line and the landing commit's file set. 0a's two codemods show
  the explicit-path form.
- `fmt-head-fqdn-to-clojure`, `break-kind-string-to-enum`, `node-kind-string-to-enum` and
  `edits-carry-the-old-text` migrated TOOLING (fmt rules, grep programs, codemods), not the corpus.
  Their SCOPE must say so.
- **The gate asserts:**
  - every fixtured or runed stem has exactly one non-empty SCOPE line;
  - every glob entry matches at least one tracked file.
- By the end of batch 2 that covers all 96.

## The gate's final shape (batch 2)

- `FROZEN_LEDGER` and every code path that reads it are deleted.
- Coverage becomes: every stem has a fixture XOR a rune, and a SCOPE.
- The stale, orphan and multi-category checks stay, wherever they still apply.
- Prove the new arms can fail before trusting them. Each goes RED, then is reverted, with the
  verbatim text in the SCORE:
  - (e) a stem with no SCOPE line;
  - (f) a SCOPE glob that matches nothing;
  - (g) a rune with an empty reason;
  - (h) a rune on a stem that also has a fixture.

## ⛔ STOP triggers — each is a REJECTION. Ship nothing; report the verbatim evidence.

- **STOP-1:** a repair needs `src/`, changes a codemod's documented behaviour, or exposes a missing
  primitive.
- **STOP-2:** an `after.post` cannot come from history or spec. Name the codemod.
- **STOP-3:** a codemod's input only exists as a file type or name the fixture's `<stem>.wat` temp
  file cannot carry. (Measured: none today.)
- **STOP-4:** a SCOPE cannot be determined from the header or the landing commit.
- **STOP-5:** the floor goes red for a reason outside this stone's files. Per the floor rules: do
  not re-run; capture the whole block.

## Blast radius

- `wat-scripts/fixes/*.wat` (SCOPE lines; repairs on rotted codemods only)
- `wat-scripts/fixes/replay/`
- `tests/cli/every_recorded_migration_replays.rs`
- this directory's SCOREs

**No `src/`.**

## Tier

Commit per batch on green. **Do not push. Do not touch main.** Yield after batch 1 and wait for the
orchestrator. Yield again after batch 2.
