# BRIEF — STONE 1c-f: `reduce` becomes an alias for `foldl`

You are a **rider**, not the orchestrator. **Ending your turn ENDS you** — it does not suspend you,
and nothing will wake you. There is no notification coming. Work in the FOREGROUND and block on
every command: your turn ends when the results are in your hands.

Anchor: **`/home/john/work/holon/wat-rs`**. Run `pwd` first. Use `git -C /home/john/work/holon/wat-rs`
for any git read. You may not spawn sub-agents. Do not commit, push, stash, or revert — the
orchestrator owns all of that. Do not run the full floor; the orchestrator runs it centrally.

## The work, in one paragraph

`:wat::core::reduce` is a two-arm `defclause` whose 3-arity arm is the bare body
`(:wat::core::foldl f init coll)` — it is `foldl` wearing a second name. Replace it with a
`defalias`. One thing must land first: `defalias` derives its signature from `foldl`'s retained
`CheckEnv` TypeScheme, and that scheme's collection param is still `Vector` from before `foldl` was
widened to walk any seqable — so widen it to `Seqable` in the same stone. Then augment the single
2-arity caller, and measure whether two now-shadowed `reduce` arms in the rete purity residue have
become unreachable.

Read `DESIGN-STONE-1c-f-reduce-is-an-alias-for-foldl.md` (sibling) first — it carries the probe
results this brief is built on, including the exact `--check` output you should expect to see.

## Rooms, in order

1. **`src/check.rs:20547-20564`** — `foldl`'s retained TypeScheme. The third entry of `params` is
   `vec_of(t_var())` at `:20560`. Its two comment lines above (`:20548-20549`) already say the
   scheme exists *for alias derivation*; this stone makes that true for Seqable inputs.
2. **`src/check.rs:20462-20464`** — the note reading *"a static TypeScheme cannot express 'any
   Seqable'"*. Measured false on 2026-09-03. Correct the note at its site; leave
   `zip`/`window`/`remove-at` alone.
3. **`wat/seq.wat:318-329`** — the `defclause`. The whole form, including its 2-arity arm.
4. **`wat-scripts/scratch-pad/probe-118B2-rider-verification.wat:59-70`** — the comment at `:59`
   naming *"3-arity and 2-arity Stream arms"*, and the 2-arity call beginning at `:67`.
5. **`src/rete/purity.rs:557`** and **`src/rete/purity.rs:652`** — the two `":wat::core::reduce"`
   hand-list arms. Measurement targets, not assumed edits.

## Implementation sketch

```rust
// room 1 — src/check.rs, inside foldl's TypeScheme params
TypeExpr::Parametric { head: "wat::core::Seqable".into(), args: vec![t_var()] },
```

```wat
;; room 3 — wat/seq.wat, replacing the whole defclause form
(:wat::core::defalias :wat::core::reduce :wat::core::foldl)
```

Keep the explanatory comment block that sits above the form (`wat/seq.wat:~295-317`) — update its
prose so it describes an alias rather than a two-arm defclause, and record that the 2-arity
seed-from-first arm is gone and why.

Room 4: the probe's 2-arity call becomes a 3-arity call, and the comment at `:59` stops claiming a
2-arity arm exists. Preserve everything the probe was built to exercise on the Stream path.

## The instrument — the stdlib is compiled in

`wat/seq.wat` is `include_str!`ed at `src/load/stdlib.rs:68`. **A stale binary cannot see your edit.**
`cargo build --release --bin wat` after touching `wat/seq.wat`, every time, before any `--check`.

**The aimed-probe canary.** Before you trust any green, write a 2-arity call to a scratch file and
`--check` it — it MUST come back RED with `expected 3 argument(s); got 2`. A green there means your
binary is stale and every other result you have is meaningless. This exact false-green happened
during the lair study.

## Rooms 5 — the measurement, done properly

After rooms 1-4 build clean:

1. Delete the `":wat::core::reduce"` arm at `src/rete/purity.rs:557` **and** at `:652`.
2. `cargo build --release` and run the scoped rete/purity suites:
   `cargo nextest run --release -E 'binary_id(wat::rete)' 2>&1 | tail -20`
3. **Green** → the arms were shadowed by `head_ok`'s `sym.has_function` door; keep them deleted and
   say so with the evidence.
   **Red** → restore both arms exactly, and report which test named which arm. Either answer is a
   successful stone; report what you measured, never what you expected.

## Blast radius

`src/check.rs` (one param, one comment) · `wat/seq.wat` (one form + its comment block) ·
`wat-scripts/scratch-pad/probe-118B2-rider-verification.wat` · `src/rete/purity.rs` (two arms,
conditional on the measurement). **No new types. No new files. No other verb touched.**

## STOP triggers — each rejects; none permits a smaller delivery

- **STOP-1** — if widening the scheme to `Seqable` does not clear
  `tests/collection/probe_arc278_0d_transform_dispatch_parity.wat`, STOP and report the verbatim
  `--check` output. Do not reach for a custom `infer_reduce` arm; that is a different stone.
- **STOP-2** — if any file other than
  `wat-scripts/scratch-pad/probe-118B2-rider-verification.wat` goes red on the arity change, STOP
  and report it. The census found exactly one 2-arity caller; a second means the census was wrong
  and the orchestrator must re-derive it before you continue.
- **STOP-3** — do not delete any `.wat` file, and do not delete the probe's Stream coverage.
  Augment it. Builder ruling, 2026-09-03: *"deletions must clear a high bar... we augment as they
  need."*
- **STOP-4** — if the 2-arity canary comes back green, STOP. Your binary is stale; nothing you have
  measured is trustworthy.

## What to report

The verbatim `--check` result for each of the eight call-site files; the canary's result; the
rete-suite Summary line for the room-5 measurement and your disposition of the two arms; the exact
diff you made to the probe; and anything that surprised you.
