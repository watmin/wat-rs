# BRIEF — STONE: `wat/grep.wat`

DESIGN: `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-wat-grep-is-a-feature.md` — read it whole,
first. It carries the contract, the file's shape, and why each name is what it is.

Companion record (read for the shapes and the reasoning, NOT for scope — its home is superseded):
`DESIGN-STONE-the-grep-match.md`.

## Your role

You are a rider, not the orchestrator. **Ending your turn ENDS you** — nothing wakes you, no
notification is coming. Run every command in the FOREGROUND and block on it. Your turn ends when the
numbers are in your hands, not when a command is launched.

**You may not spawn sub-agents.**

Anchor: `/home/john/work/holon/wat-rs`. `pwd` first. You do not commit, push, stash, revert, or
checkout — leave your work uncommitted and report.

## ⚠ THIS STONE EDITS THE STDLIB, WHICH CHANGES HOW YOU VERIFY

`wat/*.wat` is baked into the binary by `include_str!` at RUST-compile time. **A stdlib edit is
invisible to `target/release/wat` until the crate rebuilds.** So:

- **You MUST run `cargo build --release` after editing `wat/grep.wat`, before any `--check` or run.**
  Measured this session: **19s** incremental. Cap it:
  `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 timeout 900 cargo build --release`
- **Do NOT run `cargo nextest`, `scripts/floor.sh`, or clippy.** Those are the orchestrator's and are
  taken centrally on a quiescent tree. The build above is yours because without it you would be
  writing blind; the floor is not.
- A `--check` that contradicts your edit is the stale-binary symptom. Rebuild, then re-read.

## The work in one paragraph

Create `wat/grep.wat` — the stdlib home for wat-grep's vocabulary — and register it in
`STDLIB_FILES`. Most of its content is a **MOVE of proven code**: the walk in
`wat-scripts/scratch-pad/rules-corpus-03-source-to-facts.wat` already turns real source into facts
and its numbers are shipped and measured. The migration is `:fx::` → `:wat::grep::`, `:fx::Acc`
becomes `:wat::grep::Facts`, and the four inline `Option/expect` calls collapse into one door,
`extent-of`. Then corpus-03 drops its declarations and consumes the stdlib verbs.

## The rooms — read in this order

1. **`wat-scripts/scratch-pad/rules-corpus-03-source-to-facts.wat`** — whole, 194 lines. This is the
   source of the move. `:fx::Node`/`:fx::Named`/`:fx::Span` (`:28-47`), `nameable?`/`structural?`
   (`:49-60`), `:fx::walk` (`:60-104`), `:fx::empty-acc` + `:fx::extract` (`:105-120`),
   `:fx::report` (`:121-130`). The four `Option/expect` calls at `:92-99` are what `extent-of`
   replaces.
2. **`wat/fix.wat:179-193`** — `fix-text-offset-of`. The canonical
   `(Option/expect (HashMap/get loc :line) "…")` chain your door absorbs, and the `-of` precedent.
3. **`src/stdlib.rs:34`** — `STDLIB_FILES`. Read the header comment: foundational → derived, a file
   precedes another only if the later one has no eval-dep on it. `wat/core.wat` is first (`:40`),
   `wat/fix.wat` at `:286`, `wat/lint.wat` at `:326`, `wat/rete.wat` at `:362`.
4. **`wat/rete/syntax.wat`** — where `defquery` and friends live, and a worked example of a stdlib
   file whose placement was ruled by the load-order gate rather than by tidiness. Its header records
   why.
5. **`wat-scripts/scratch-pad/probe-rhs-builds-core-span.wat`** — the foot records what a rete
   `:then` can and cannot construct, measured. Read it before writing the proving rule.

## What to build

The DESIGN's "THE FILE" section gives every declaration. Spell every FQDN in full. Beyond the
declarations, two functions:

- **`:wat::grep::extent-of [node <- :wat::WatAST] -> :wat::grep::Extent`** — calls `ast-span` and
  `ast-end-span`, consumes all four `HashMap/get` Options in ONE place. Mirror
  `fix-text-offset-of`'s shape; give each message enough to locate the failure. After this exists,
  **no other site anywhere unwraps a span** — that is what the name promises and row 5 measures it.
- **`:wat::grep::facts-of [src <- :wat::core::String] -> :wat::grep::Facts`** — corpus-03's
  `extract` + `walk`, moved. Keep the pre-order id assignment and the `Named` guard EXACTLY as they
  are: the guard is the file's central lesson (`ast-name` is partial; the absence IS the guard), and
  `Span` is emitted UNCONDITIONALLY beside `Node` because `ast-span` is total.

Then **corpus-03** drops its own `Node`/`Named`/`Span`/`Acc`/`walk`/`extract` and calls
`:wat::grep::facts-of`. Its rules and its report stay — retargeted at the stdlib fact types. It
remains a probe, and it is this stone's regression check.

⚠ **`Span`'s declaration carries a one-line cross-reference to `Extent`** naming the dependency
between their field lists — nothing pins them together, so a later rename must be made in both by
hand. intueri asked for this explicitly.

## The proving rule

In corpus-03, alongside its existing rules, one rule that asserts a real `:wat::grep::Match`. It
binds coordinates from `:wat::grep::Span`, supplies `file` as a literal in the RHS, and builds a
non-empty `captures` vector. Query it with `:wat::grep::q-match` and print the result.

⚠ **Two things measured this session that will otherwise cost you a cycle:**
- The vector constructor is **`:wat::rete::core::PersistentVector`**, NOT `:wat::core::PersistentVector`.
  Core's fails the fence with *"is not total"*.
- A **record** constructor takes kwargs; a **tagged enum variant** takes positions. Both can appear
  in one `:then` and they look identical at the call site.

## The acceptance rows YOU run

Rebuild first. Then:

- **Row 1 — the load-order gate accepts the placement.** Run `(:wat::deporder::verify-stdlib)` and
  report its output. `[]` is the pass. **The gate is the authority on where the file goes** — if it
  names a violation, MOVE the file, do not argue with it.
- **Row 2 — corpus-03 reports the same numbers.** `wat/fix.wat Node=4316 … Span=4316`,
  `neg-consumer 435`, `probe_do_splice 33`, `Named` strictly below `Node` in each. Verbatim.
- **Row 3 — a rule builds a complete `Match` in one RHS.** Print the fact; report it verbatim.
- **Row 4 — no `Option` in the Match's rendered EDN.** Grep your own row-3 output for `Option/` and
  report what you find.
- **Row 5 — `extent-of` is the ONLY site that unwraps an `ast-span` HashMap.** Census
  `Option/expect` + `HashMap/get` across `wat/grep.wat` and corpus-03; report the count and every
  site.
- **Row 6 — `--check` exits 0** on corpus-03 and on any other `.wat` you touched.

Report each row's command and output **verbatim** — never a summary, never a `| head`/`| tail`
window. A row you could not run is reported as not-run, never as passed.

## Blast radius

- `wat/grep.wat` — created
- `src/stdlib.rs` — one `WatSource` entry
- `wat-scripts/scratch-pad/rules-corpus-03-source-to-facts.wat` — edited

Nothing else under `src/`. Nothing in `wat-scripts/lib/`. No other stdlib file.

## STOP triggers — each ships NOTHING and surfaces the gap

1. **The load-order gate names a violation you cannot resolve by MOVING the file** — e.g. a genuine
   two-way dependency between `wat/grep.wat` and something else. STOP and report the gate's exact
   output; do not restructure another stdlib file to make room.
2. **Row 2's counts move.** STOP. The move changed behaviour. Report both sets of numbers; do not
   adjust the walk to make them agree.
3. **A `Match` field cannot be constructed in a `:then`.** STOP and report the compiler's message
   verbatim plus which field. Do not flatten `captures` or drop a coordinate to get past it.
4. **Anything requires editing a `src/` file other than `stdlib.rs`, or a `wat/` file other than
   your new one.** STOP — that is a substrate finding for the orchestrator, not work for you.

A STOP means: leave the tree as it is, write the report, end your turn. It is never a licence to
ship a smaller version of a row.

## What you own that nobody can reconstruct

Your exact outputs, the row-5 census site by site, where the gate made you put the file and what it
said, and anything that surprised you — a construction that failed for a reason this brief did not
predict, a count you expected to move that didn't, a message that read wrong.
