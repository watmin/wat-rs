# claude-compute — the integration branch

An **observation post**, not a destination. It exists so `main` and `grok-rete` can keep
diverging while someone watches what their union actually does. It is **never merged back
into either parent.** When the three finally converge, delete it and cut a fresh one.

```
claude-compute = origin/main + origin/grok-rete + integration fixups
```

## Refresh it with the tools, not by hand

```
~/opt/bin/wat-sync.sh --status     divergence report; touches nothing
~/opt/bin/wat-sync.sh              fetch, merge both, drift gate, build, floor
~/opt/bin/wat-drift                report retired-name drift (exit 1 if any)
~/opt/bin/wat-drift --fix          .wat via the recorded codemods; Rust line-scoped
```

Both live OUTSIDE the repo on purpose, so this branch stays a pure merge-of-both plus
recorded fixups. `rerere` is enabled (`rerere.enabled`, `rerere.autoupdate`).

## The failure class — DRIFT, and it is not only names

**The branches' LOGIC has composed cleanly at every refresh. Not one failure has ever been a
semantic conflict.** Every one has been drift: main changes the corpus-wide truth, grok-rete
writes against the truth main replaced, and a **textually clean merge** ships it broken.

| | 2026-08-28 first | 2026-08-28 second | 2026-08-30 third |
|---|---|---|---|
| floor cycles to green | 6 — 3182 → 63 → 57 → 39 → 10 → 8 → 0 | **1** | 3 — 41 → 38 → **0** |
| conflicts | 13, all hand-resolved | 13 → **6** (rerere replayed the rest) | **0** |
| what drifted | retired NAMES | retired NAMES | a retired **FORM** |

⛔ **CORRECTED 2026-08-30. This section used to read "across two full refreshes, every single
failure resolved to NAME drift", and the gate was built to exactly that shape.** The third
refresh's 41 reds were one cause and it was not a name: arc 109's ONE PARAM-SPEC wall
(`284cd7c93`) made the bare `(Head T)` param-spec unrepresentable, and grok-rete's files were
written against it. `wat-drift` reported **clean** — correctly, by its own terms, because it
reads a table of retired NAMES and a retired FORM has no row in it.

Read the generalisation, not the instance: **the drifting thing is whatever main last made
corpus-wide-illegal.** Twice that was a name. Once it was a form. Next time assume it is
neither, and go and look.

### The hole this leaves, and the shape of the fix (NOT DRAWN)

`wat-drift` cannot be taught forms one at a time without becoming the hand-kept list it exists
to avoid. The root fix is to stop enumerating and let the migrations BE the gate: there are 80
recorded codemods in `wat-scripts/fixes/`, the CLI already carries a `--grep` census mode
(`src/distribution/argv.rs:109`), and **any recorded migration that would still emit an edit on
this branch's files IS drift, by construction.** That closes names and forms and everything
after them, and it cannot go stale, because a new migration arrives already gated.

## What `wat-drift` knows that a careful reader does not

It reads main's `RETIREMENT_TABLE` (`src/remedy/retirement.rs`) — never a hand-kept list of
"which families moved". That list was WRONG here once. Five design points, each learned by
getting it wrong first:

1. **Line-level, not file-level.** Only lines this branch ADDS over `origin/main`. A retired
   name on a line main also has is main's business: the retirement table itself,
   `wat-scripts/fixes/*` (which carry old names as DATA), and
   `255-stone-a-i-both-*-spellings.wat` (which asserts both ON PURPOSE).
2. **Qualified names only.** Four table rows are bare wat constructors (`Some`, `:None`,
   `Ok`, `Err`); substring-matching those found Rust's own `Ok`/`Err`/`Some` everywhere —
   327 hits, ~20 real. (When those become `wat.type/Option.Some` etc., re-check the
   `:wat::` prefix filter against the final spelling.)
3. **Both spellings.** Source `:wat::rete::core::i64::+` AND rendered `:wat.rete.core.i64/+`.
   The second hid in a test assertion and survived a pass that only knew the first.
4. **The rete mirror.** A rete row's name is its `core_name` with `rete::` inserted after
   `wat::`, so every core retirement implies a rete retirement the table never states.
   Missing this made 7 failing tests read as a "classify these rows" DECISION when they were
   plain drift — `reachability.rs`'s ledger keys sat on `:wat::rete::core::PersistentVector/get`
   after main renamed the row to `:wat::rete::vector::get`. The operand data was never
   missing; it was filed under the row's old name.
5. **CODE vs PROSE, and `--fix` is LINE-SCOPED.** A retired name in a comment is usually a
   deliberate citation. A whole-file `str.replace` rewrote an explanatory note that cited the
   pre-rename spellings on purpose, inverting its meaning. Prose is reported, never rewritten.

## Fixing `.wat` — the recorded codemods, never hand-edits

`CLAUDE.md` is the doctrine; this is the branch-specific shape. Sweep **every**
`wat-scripts/fixes/rename-*.wat` in census (`--grep`) mode over the changed files, not the
ones you think apply — the second refresh's hits were `vectors`, `set-and-list`, `maps`,
`string-verbs`, none of which were guessable. Count **occurrences**, not lines: the finder
emits one long line and `grep -c` silently undercounts. Comments are never rewritten (the
codemod walks the form tree), so prose is a separate manual pass.

**BOOTSTRAP.** `wat/gen.wat` is baked into the binary via `include_str!`, so when it is the
broken file the tool cannot boot to fix it. Invert the stash-dance in `wat/fix.wat`'s header:
comment out gen.wat's `WatSource` in `src/load/stdlib.rs`, `cargo build --release --bin wat`,
run the codemods, restore, rebuild.

## Recurring conflicts and their standing resolutions

- **`.wat` files** — take grok-rete's wholesale, then re-run the codemods. The migration is
  *derived*, not merged, so it cannot accumulate against grok-rete's next edit.
- **`probe_diagnostic_value_snapshot_*.edn` (5) + `probe_arc293_*.edn` (2)** — `:line` drift
  only. `normalize_rust_source_span_lines` (`src/lib.rs:186`) zeroes the `:line` of any
  `.rs`-filed span before comparing, so **neither number can fail a test**. Take main's.
  Verify line-only mechanically each time rather than assuming.
- **`tests/rete/datamancer.rete.edn`** — a compiled artifact with an `:abi` hash. Both sides
  produce different hashes and neither is right post-merge. **Regenerate**:
  `./target/release/wat tests/rete/datamancer.src.wat`. Required after ANY `RETE_OPS` change.
- **Rust conflicts** — resolve as (grok-rete's LOGIC) × (main's NAMES), per builder ruling.
  Never pick a side wholesale.

## Traps that have cost real time here

- **A NARROW CENSUS IS A FALSE ALL-CLEAR — and the narrow one is the one you write.**
  2026-08-30: grepped `Vector|vec` for bare param-specs, found zero, reported "no bare form
  remaining in code". `one-param-spec.wat`'s arity table covers SEVEN parametric heads; the
  survivors were `HashSet`, and the floor said so by going 41 → 38 instead of 41 → 0.
  **Run the codemod over the whole corpus as both CONTEXT and TARGET and read ITS report.**
  It is idempotent, its table is the authority, and it reports what it deliberately skips
  (`tuple-ambiguous`, `bracket-unknown-head`). A hand-grep of heads is a guess wearing a
  census's clothes.
- **A codemod cannot reach wat source held in a Rust string literal.** It walks the FORM TREE
  of `.wat` files. 2026-08-30: four sites in `src/rete/reachability.rs` (a grok-rete-only file)
  held `.wat` snippets that `format!` into `<entry>` programs. They are invisible to every
  recorded migration and must be swept by hand, line-scoped. When an error's span reads
  `:file "<entry>"`, the source is Rust, not the corpus — go grep `.rs`.
- **Backgrounding a build hides its verdict.** `nohup cargo … &` returns the wrapper's exit
  code, not cargo's; a "build succeeded" that was really `echo` succeeding produced one wrong
  claim. Likewise `cmd | grep -c '^error' && next` — `grep -c` exits 1 on zero matches, so
  `next` never runs. Read the Summary line, never a piped exit code.
- **"is not pure" is a documented lie.** `src/rete/purity.rs:89` says so outright: the op IS
  pure — it is refused only because the name is not a registered *rete* name. Read it as
  "not from rete", or you will chase purity for an hour.
- **`wat-sync.sh` used to exit 0 on a RED floor — FIXED 2026-08-30.** Its last line was
  `./scripts/floor.sh 2>&1 | tail -20`, whose status is `tail`'s, and the script's final
  command was an `echo`. `set -o pipefail` did not save it: the status was never consulted.
  A 41-failure floor reported `SYNC_EXIT=0`. The floor's status is now the script's status,
  so no path exits 0 on red. This is FM 20 in `docs/COMPACTION-AMNESIA-RECOVERY.md`, committed
  by the person who wrote the gate.

## Deferred, and where the reasoning lives

- **`RETE_MODULES` + `infer_rete_form` are hand-kept caches of derivable sets.** Both are
  parity problems a derivation would delete. Handled at the three-way sync, by builder ruling
  — see `docs/arc/2026/06/278-rules-engine/NOTE-rete-modules-is-a-hand-computed-cache-of-a-derivable-set.md`.
- **The `.edn` goldens still STORE a `:line` the comparator zeroes** — it cannot fail a test
  but conflicts on every refresh. Writing `:line 0` ends the class. grok-rete reached the same
  conclusion independently while recapturing goldens main had already made unnecessary.
- **`char` has no rete type segment.** `rete_type_segment_of` covers i64/f64/string/bool/
  keyword/enum only, so a `\c` operand is not derivable in a rule. `CharLit` is placed with
  Rational/BigInt/Nil accordingly. No forcing function yet.
- **`wat::cli retirement_table_reachable`** — 18.7s alone, TIMEOUTs at the 30s cap under full
  floor load. Attributed: main alone touched `src/remedy/retirement.rs` and that test since
  the merge-base. Main's, and it will intermittently red main's floor on a loaded box.

## Current state — grok-rete is HELD (refreshed 2026-08-30)

`claude-compute` @ `76b807231` carries main to `9d4976f1d` and grok-rete only to `1facc1f94`
(2026-08-28). Floor GREEN: `5201 tests run: 5201 passed (9 slow), 17 skipped`.
**A green floor right now means MAIN is green — NOT that the union is.** That distinction is
the whole reason this branch exists, so it is stated rather than implied — and it is exactly
why the 41 reds this refresh were arc-109 form drift and NOT the 160-error module-tree port
the hold is actually about. That port has still never been attempted.

**Why:** arc 255's HOME campaign relocated main's flat module tree into directory homes
(`string_ops`→`string/`, `wat_edn_bridge`+`edn_shim`→`edn/`, `hologram`+`sigma`→`holon/`,
`stdlib`→`host/`). grok-rete's commits are built against the layout main replaced. Measured:
keeping grok-rete's tree = 160 build errors; keeping main's = 327. Neither is a repoint — every
error is a hand decision about where a function lives and what API it exposes. No codemod
reaches this class, which is why `wat-drift` reports clean while the merge is unbuildable.

**The trigger is grok-rete taking main's module tree** — not main finishing, which it already
has (HOME-9..13 all landed, the last 98 commits back; main is on arc 109 now). Until grok-rete
moves, the port would be redone at every refresh.

`wat-sync.sh` honours this via `HOLD_GROK_RETE=1` and will SKIP the grok-rete merge with a
warning rather than blunder into a 300-error tree. Clear the flag when the hold lifts.

### ⚠ rerere replays BAD resolutions as faithfully as good ones

Measured here: a throwaway probe branch resolved `src/lib.rs` by taking grok-rete's side
wholesale. rerere recorded it, and on the NEXT merge silently auto-applied it — a
`git checkout --ours -- src/lib.rs` then no-opped (the path was no longer unmerged) and the
measurement that followed was against the wrong tree, unnoticed for two rounds.

**Never resolve crudely on a branch that shares `.git` with the real one.** If you already
have: `git merge <branch>` to re-trigger, `git rerere forget <path>`, `git merge --abort`.
Check afterwards that the file is what you think — `rerere` leaves no trace in `git status`.
