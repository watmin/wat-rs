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

## The one failure class, and the evidence for it

**Across two full refreshes, every single failure resolved to name drift. Not one was a
semantic conflict between the two branches' logic.** They have composed cleanly every time.

| | 2026-08-28 first refresh | 2026-08-28 second refresh |
|---|---|---|
| floor cycles to green | 6 — 3182 → 63 → 57 → 39 → 10 → 8 → 0 | **1** |
| conflicts | 13, all hand-resolved | 13 → **6** (rerere replayed the rest) |
| stale names | found a family at a time, over hours | **33, before the first build** |

The mechanism: main renames across 1000+ files; grok-rete writes new code against the old
names; a **textually clean merge** ships the result broken. `git merge-tree` reported CLEAN
for the first union and it took 3182 of 5103 tests red on ONE panic.

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

## Two traps that cost real time here

- **Backgrounding a build hides its verdict.** `nohup cargo … &` returns the wrapper's exit
  code, not cargo's; a "build succeeded" that was really `echo` succeeding produced one wrong
  claim. Likewise `cmd | grep -c '^error' && next` — `grep -c` exits 1 on zero matches, so
  `next` never runs. Read the Summary line, never a piped exit code.
- **"is not pure" is a documented lie.** `src/rete/purity.rs:89` says so outright: the op IS
  pure — it is refused only because the name is not a registered *rete* name. Read it as
  "not from rete", or you will chase purity for an hour.

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
