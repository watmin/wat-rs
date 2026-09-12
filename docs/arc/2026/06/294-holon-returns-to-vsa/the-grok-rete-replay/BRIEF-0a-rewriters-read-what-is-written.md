# BRIEF 0a — rewriters read what is WRITTEN: the 11 grep-rules codemods work on today's binary

Anchor: `/home/john/work/holon/wat-rs`. Verify with `pwd`. **Never use worktrees.** Do not touch
`~/work/holon/` (the frozen root).

## ⛔ TREE STATE — read this first

```
branch   replay/grok-rete   <-- YOU ARE HERE. Off main a3218644d plus carried main-side tooling.
HEAD     0388b8468          clean · floor 5373/5373 GREEN · clippy 0 · pushed to origin
main     a3218644d          frozen. Do not check it out.
```

`target/release/wat` on this branch BOOTS (main's stdlib). You need no bootstrap binary.

## The work, in one paragraph

Main folded `:wat::grep::Named` to clojure form (`0b5742cc7`), and 11 recorded grep-rules codemods
(40 rules) went dead on today's binary. They match nothing, or they splice a folded name the source
does not contain. Fix the class, not the 11:
1. `:wat::grep::Written` gains `text`, the verbatim `ast-name`. Rewriting rules join `Written`;
   readers keep joining `Named`.
2. The 11 codemods move onto `Written`.
3. The 45 folded literals `0b5742cc7` wrote into five of them are restored to their verbatim spellings.
4. A gate lands that REPLAYS recorded migrations against fixtures, so this can never rot silently
   again.

Read `DESIGN-step-0-recorded-migrations-work-on-todays-binary.md` (this directory) first. It holds
the contract decisions and why the alternatives were rejected.

## Rooms, in order

1. **`wat/grep.wat:79–92`, `Written`.** The record you extend. Its comment states the split you are
   completing: `Named` = WHAT it is called, `Written` = AND IT IS SPELLED HERE.
2. **`wat/grep.wat:249–266`, the `written?` guard and `Written`'s ONE construction site.** Add
   `:text (:wat::core::ast-name node)`. `ast-name` is already called under the same guard two lines
   above, so this is safe.
3. **`wat/grep.wat:1–10`, the header CONTRACT.** It lists `Node · Named · Span` and has never named
   `Written`. Make it true.
4. **`wat-scripts/fixes/rename-core-string-to-string.wat:48–90`, the rule shape all 11 share:**
   `Node` + `Named` + `Span` + `Source`, then `where` clauses on `?n`. Also `:124–147`
   (`edits-of`): old-text is the FIRST capture's value. After the move, that value is the verbatim
   spelling, and `fix-text-apply`'s claim check becomes real again.
5. **`wat/fix.wat:231–242`, `fix-text-span-text`'s doc:** *"the WRONG door for a RENAME."* It says
   why old-text must come from the name, not from slicing the span.
6. **`wat-scripts/fixes/fmt-head-fqdn-to-clojure.wat`, the STRING-node rewriter.** Codemod B below is
   its exact inverse, driven by an explicit table. Copy its shape.
7. **`tests/cli/grep_programs_still_match.rs`,** the shape of a test that drives the real binary
   (`env!("CARGO_BIN_EXE_wat")`, stdin EDN). `tests/cli/mod.rs` is an `include!` stub over
   `build.rs`'s generated `cli_mods.rs`, so dropping the new `.rs` beside it IS the registration.
8. **`tests/lint/no_bare_is_err.rs:40–60, 233–260`,** the house `FROZEN_ALLOWLIST` idiom: named
   identities with reasons. Your ledger follows it, and it also turns a STALE entry red.
9. **`wat-scripts/scratch-pad/probe-written-admits-fqdn-keyword-tokens.wat`, the orchestrator's
   probe.** `Written` exists for an FQDN keyword token in head and argument position.

## The proven rule shape (probe 2, run end-to-end; mirror it exactly)

`?n` stays the variable name, so every `where` and `:then` below the join is unchanged:

```
(:wat::rete::defrule :rn::core-string
  :when [(:wat::grep::Node    (?id <- :id) (?k <- :kind))
         (:wat::grep::Written (?id <- :id) (?n <- :text) (?l <- :line) (?c <- :col) (?el <- :end-line) (?ec <- :end-col))
         (:wat::grep::Source  (?f <- :file))
         (:wat::rete::where (:wat::rete::core::enum::= ?k (:wat::grep::NodeKind.Keyword {})))
         (:wat::rete::where (:wat::rete::string::starts-with? ?n ":wat::core::string::"))]
  :then [ …unchanged… ])
```

On `(:wat::core::defn :u::a [] -> :wat::core::String (:wat::core::string::interpolate "x"))` plus
`(:wat::core::defn :u::b [] -> :wat::core::nil (:u::take :wat::core::string::concat))`:
- Both tokens were renamed.
- A second run changed nothing.
- Today's unconverted copy on the same binary renamed nothing.

## The strike

1. **`wat/grep.wat`:** add `text <- :wat::core::String` to `Written`; add `:text (:wat::core::ast-name node)`
   at its construction; update the `Written` comment and the header contract. Rebuild, and confirm
   the binary's mtime moved.
2. **Codemod B — `wat-scripts/fixes/restore-verbatim-literals-after-the-fold.wat`.**
   - **The table:** 45 (folded → verbatim) STRING pairs. Derive it from
     `git show 0b5742cc7 -- <the 5 files>`: per file, zip the string literals on the `-` lines with
     those on the `+` lines, in order, and keep the pairs that differ.
   - Measured: 7/7, 14/14, 5/5, 30/30 and 9/9 literals; 45 distinct changed pairs; each folded key
     occurs exactly once in today's file.
   - Rewrite STRING nodes (never comments) whose value equals a key. Span-faithful, idempotent.
   - The 5 files: `rename-four-families-to-their-homes`, `rename-core-vectors-to-their-homes`,
     `rename-core-set-and-list-to-their-homes`, `rename-string-verbs-to-their-home`,
     `rename-math-stat-seq-to-their-homes`.
   - Header: `;; SCOPE: <those 5 paths>`.
3. **Codemod A — `wat-scripts/fixes/rewriting-rules-join-written.wat`.**
   - In every `:wat::rete::defrule` whose `:when` holds `(:wat::grep::Named (?I <- :id) (?N <- :name))`
     AND `(:wat::grep::Span (?I <- :id) (?L <- :line) (?C <- :col) (?EL <- :end-line) (?EC <- :end-col))`
     on the same id variable: delete the `Named` pattern's whole line, and turn the `Span` pattern
     into `(:wat::grep::Written (?I <- :id) (?N <- :text) (?L <- :line) (?C <- :col) (?EL <- :end-line) (?EC <- :end-col))`.
   - Span-faithful, idempotent.
   - Run it over EXACTLY the 11, and never over itself: `rename-core-string-to-string`,
     `rename-four-families-to-their-homes`, `rename-core-numerics-to-their-homes`,
     `rename-rete-numerics-to-their-homes`, `rename-core-bigint-rational-to-their-homes`,
     `rename-core-maps-to-their-homes`, `rename-core-vectors-to-their-homes`,
     `rename-core-set-and-list-to-their-homes`, `rename-keyword-to-its-home`,
     `rename-string-verbs-to-their-home`, `rename-math-stat-seq-to-their-homes`.
   - Header: `;; SCOPE: <those 11 paths>`.
   - Each of the 11 headers says the finder reads `Named`; that prose becomes false. Correct it,
     comment lines only, and list every such edit in the SCORE.
4. **The gate — `tests/cli/every_recorded_migration_replays.rs`.**
   - Enumerate `wat-scripts/fixes/*.wat` (top level; directories are not codemods). That is 96 once
     A and B exist.
   - Each stem must be EXACTLY one of:
     - **a fixture dir, `wat-scripts/fixes/replay/<stem>/`:** `before.pre`, `after.post`, and
       optionally `stdin`. `stdin` is a template in which `{FILE}` is replaced by the temp file's
       absolute path; the default is `["{FILE}"]\n`. `one-param-spec` reads TWO path vectors.
       These extensions keep fixtures out of `wat_scripts_fixes_load.rs`'s `.wat` walk and out of
       `every_tracked_wat_parses.rs`'s `git ls-files '*.wat'`.
     - **a `FROZEN_LEDGER` entry `(stem, reason)`.**
     - **a header rune `;; rune:replay(unreadable-preimage) — <reason>`.** This is the ONE category,
       and the reason must be non-empty.
   - Per fixture:
     - copy `before.pre` to a fresh temp dir as `<stem>.wat`;
     - run `CARGO_BIN_EXE_wat <abs codemod path>` from the repo root, and assert rc 0;
     - assert the result == `after.post` byte for byte;
     - run again, and assert nothing changes (idempotence);
     - assert `before.pre` != `after.post` (non-vacuity).
   - Ledger:
     - An entry whose stem has a fixture, or no longer exists, is RED (stale).
     - A stem with none of the three is RED.
     - Collect every violation and assert once, naming each offender.
   - Shard the fixture runs across 8 `#[test]` fns (sorted stem index `% 8`), via a small
     `macro_rules!`. Keep the ledger and coverage check in its own fast fn.
   - The measured cost of one replay is ~0.25 s, except `positional-ctor-to-map` at ~9.6 s. All 94
     on a trivial file take ~33 s serially.
5. **Fixtures in 0a: 13** (the 11, plus A and B). The other 83 go in `FROZEN_LEDGER` by name, each
   with the reason `stone 0b: fixture pending`. `to-faithful-clojure-net` and `-rete` instead get
   `stone 0b: ROTTED — rete where-fence refuses a user fn (:fix::has-ns? / :fix::head-keyword-str?), rc=2`.
   - **The oracle rule, which is the heart of this stone:** `after.post` comes from HISTORY or from
     the codemod's header spec. NEVER from running the codemod.
   - For the 11: take whole top-level forms from a real file changed by the codemod's landing commit.
     `before.pre` is the file at `<landing>^`, `after.post` is the file at `<landing>`. Choose forms
     that commit changed ONLY by this rename. Landing commits: `23efc6056`, `acc95652d`, `ae2330bc1`,
     `870d59898`, `1a3f6a703`, `110335bd5`, `0d303f780`, `9dd54e58a`, `22453b9b6`, `315bbf546`,
     `29f350365`.
   - **Every one of the 40 rules is exercised** by at least one line, and each fixture carries at
     least one near-miss that must stay byte-identical. Examples of near-misses: the type
     `:wat::core::String` next to `:wat::core::string::`, a string literal holding the old prefix,
     or a comment holding it.
   - A and B: the fixture comes from their spec, hand-written.
6. **Prove the gate can fail before trusting its pass.** Demonstrate each of the following RED, then
   revert it, and put the verbatim failure text in the SCORE:
   - (a) one byte changed in one `after.post`;
   - (b) a scratch stem with no fixture, ledger entry, or rune;
   - (c) a ledger entry for a stem that has a fixture;
   - (d) one of the 11 temporarily restored to its `Named` join (`git stash` of that one file):
     its fixture must go red.
7. `scripts/floor.sh`: read the Summary line. `cargo clippy --release --all-targets`: zero lines.

## Blast radius

- `wat/grep.wat`
- the 11 codemods (via A and B, plus header comment prose)
- 2 new codemods
- `wat-scripts/fixes/replay/` (13 dirs)
- `tests/cli/every_recorded_migration_replays.rs`, plus its registration
- this directory's SCORE

**No `src/` change.** If one looks necessary, see STOP-5.

## ⛔ STOP triggers — each is a REJECTION. Ship nothing; report the verbatim evidence.

- **STOP-1:** a rule in the 11 does not hold exactly one `Named` and one `Span` pattern on the same id
  variable. Codemod A's shape does not fit it. Name the rule.
- **STOP-2:** the table is not 45 pairs, or a folded key maps to two different verbatim values, or a
  key does not occur exactly once in its file.
- **STOP-3:** an `after.post` cannot be taken from history or the header spec. Name the codemod.
- **STOP-4:** a converted codemod renames nothing, or `fix-text-apply` refuses, on its fixture. Name
  the codemod and paste its whole output.
- **STOP-5:** adding `text` changes the outcome of any existing test, or needs a `src/` edit. Name the
  test or site.

## Acceptance

`EXPECTATIONS-0a.md` (this directory) is the scorecard. It was written before the strike, and the
SCORE answers every row.

## Tier

Do the work, and leave it UNCOMMITTED on `replay/grok-rete`. Write
`SCORE-0a-rewriters-read-what-is-written.md` beside this brief. The orchestrator re-runs the rows and
commits. **Do not commit. Do not push. Do not touch main.** Call `pulsare_yield` only when the SCORE
is written and nothing is running. The prior shape to copy is
`../the-codemod-resolves-once-not-per-file/SCORE-c-resolve-only-what-this-file-declares.md`.
