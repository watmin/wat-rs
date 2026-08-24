# NOTE — the guides are not merely mis-spelled, they are NOT EXECUTABLE

**Filed 2026-08-23**, from findings two prose riders reached independently while fixing the angle
spelling in `docs/USER-GUIDE.md` and the cheatsheet.

The angle sweep makes the guides *less wrong*. It does not make them right. Measured:

```
:wat::core::define IS RETIRED   — "(Stone 241.11; eval-time residue completed Stone 241.16)"
   docs/USER-GUIDE.md      32 mentions   ← the guide's PRIMARY teaching form for defining a function
   docs/WAT-CHEATSHEET.md   3
   README.md                3
```

Both riders hit it, flagged it, and correctly did **not** fix it — out of their brief's scope. Their
other findings, all pre-existing and all independent of arc 109:

- **`let`'s binding shape changed** — now a flat `[name expr …]` vector, not nested `((name expr) …)`
  pairs. Confirmed by `--check`: *"let bindings must be a flat vector"*. Guide and README still teach
  the pairs.
- **Bare short names as shorthand** — `:String`, `:i64`, `:Vec`, `:Option` without their
  `wat::core::` paths, used pervasively. Already non-compiling before the sweep.
- **Content drift** — `crates/wat-lru/`, `crates/wat-holon-lru/` and `wat/stream.wat` do not exist;
  `ChunkStep`, `KeyedChunkStep`, `ReqTxPool` appear nowhere in the corpus or `src/`.
- **A stale claim preserved deliberately** — the guide states `CommResult<T>` is
  `Result<Option<T>, ThreadDiedError>`; the live typealias in `wat/kernel/channel.wat` is
  `(:wat::core::Result :- [(:wat::core::Option :- [T]) :wat::kernel::ThreadPanics])`. The rider fixed
  the spelling and kept the claim, which was right — correcting content silently inside a spelling
  sweep is how a sweep becomes unreviewable.

★ **The honest summary, in the rider's own words: *"many snippets in the guide — before and after my
edit — aren't literally paste-and-run."*** The document teaches a retired special form 32 times, a
retired `let` shape, and type names that do not resolve.

## Why this is the same defect as the doc validator, one level out

`@arg`/`@ret` used to be validated by `starts_with(':')` — a shape test standing in for a parse — and
that is now fixed: the doc validator asks the reader, and an inexpressible type fails the BUILD.

**The guides have no validator at all.** Their fenced code blocks are prose to the toolchain. Nothing
has ever asked whether a single example in `USER-GUIDE.md` parses, let alone runs — which is exactly
why a form retired in arc 241 is still the primary thing it teaches in arc 109.

A sweep fixes today's spelling. **The available structural move is to extract fenced `wat` blocks from
the Markdown and `--check` them in the test suite** — the same shape as `every_wat_scripts_file_loads`,
which is precisely why `wat-scripts/` cannot rot into a graveyard. A guide whose examples are gated
cannot teach a dead form, and the gate's first run is the census.

⚠ **That gate cannot land as-is.** It would go red on every one of the defects above, and probably many
more — which is the argument for it, not against it, but it means the gate and the repair are one
stone, sequenced, not a lint dropped on a red corpus.

## Scope

Out of the prose sweep's scope — that stone is spelling only, deliberately. Not tracked elsewhere;
this NOTE is the record.

Kin: `DESIGN-STONE-rip-the-heresy-from-the-prose.md` (the validator that now guards `@arg`/`@ret`),
`[[feedback_a_green_test_can_prove_nothing]]` — an example nothing runs is a claim, not a proof.
