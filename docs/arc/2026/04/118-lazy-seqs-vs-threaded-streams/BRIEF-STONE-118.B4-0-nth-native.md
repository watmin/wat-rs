# BRIEF — STONE 118.B4-0 · `nth` becomes the native kernel; the wat clause becomes its oracle

`nth` is the only positional accessor that is not a Rust intrinsic, and that is now a blocker: a
`defmacro` program body evaluates through `dispatch_keyword_head`, so it can call `first`/`drop`/`get`
but not `nth`. B4-ii's codemod took the stdlib from loading to `StartupError` on exactly that.

Make `nth` native. Keep B4-i's wat clause, renamed, as the **oracle** that proves the native honest.

## Read in order

1. **`src/runtime.rs:5646–5654`** — the `first` / `second` / `third` dispatch arms. Each is one line
   calling `eval_positional_accessor(…, op, index)` with a **constant** index. `nth` is the same
   call with the index taken from `args[1]`.
2. **`src/runtime.rs:15432`** — `eval_positional_accessor(args, list_span, env, sym, op, index:
   usize)`. Note two things: it is gated by `container.indexable()`, and its inner match is
   **exhaustive over the closed `StreamContainer` enum, no wildcard** — that exhaustiveness is the
   drift guarantee, keep it.
3. **`src/check.rs:~9180`** — `infer_positional_accessor`, the mirror of the above on the checker
   side, same `indexable()` gate, same `index` parameter, same exhaustive shape.
4. **`src/collection/seq_container.rs`** — the capability waist: `of_type` / `of_value` and the
   capability methods (`indexable`, `has_tail`, `mappable`, `measurable`, `gettable`, …). This is
   where the new capability goes.
5. **`wat/rete.wat:1508`** — the oracle exemplar: `insert-all-spec` (wat oracle) / `insert-all'`
   (native kernel) / `insert-all` (public). *"the native kernel is the fast impl, the spec keeps it
   honest."* Here the public name IS the native, so two names, not three.
6. **`wat/core.wat:1393–1435`** — B4-i's four-arm clause and `nth-walk`. **The header's
   total-CONTRACT / partial-FUNCTION argument is correct and stays** — extend it, do not rewrite it.

## The strike path

**1 — mint the capability.** `indexable()` is wrong to reuse: B4-iii flips it to `false` for Stream,
which would silently close `nth` on lazy seqs three stones later. `gettable()` is already `false` for
Stream. Add a third method on `StreamContainer` for *general positional lookup by index*:

```
Vector · PersistentVector · List · WatAstList · Stream   →  true
Tuple    (heterogeneous — a runtime index cannot be typed) →  false
HashSet  (unordered — no positional meaning)               →  false
```

Fill both classifiers and the capability-matrix doc comment at the top of the file.

**2 — the native.** A dispatch arm beside `first`/`second`/`third`, arity 2, index from `args[1]`
(an `i64`, evaluated). Route through the same exhaustive per-container shape. The indexable
containers use their O(1) path; **Stream walks** — `realize` one cell at a time, `i+1` forces for
index `i`, raising by name at exhaustion. Message stays exactly `"nth: index out of range"`.

**3 — the checker.** `nth` needs its own inference. Mirror `infer_positional_accessor`'s shape but
take the index from the second argument and require it to be `i64`. ★ This adds an arm to
`infer_list`'s hand-written keyword block — the population arc 255 is working to shrink. That cost is
known and accepted; **say so in your report**, do not hide it.

**4 — the oracle.** Rename the wat clause `:wat::core::nth` → `:wat::core::nth-spec` (and
`nth-walk` → `nth-spec-walk`). **Existing callers keep saying `nth` and now reach the native** —
that is the point, not an accident. Sites: `wat/bracket.wat:592,594,737`, `wat/fix.wat:1041`,
`wat/service.wat:1236`. Leave them alone.

**5 — the allow-list.** Add `":wat::core::nth"` to `is_pure_total` (`src/macros/eval.rs:~512`,
beside `first`/`rest`/`last`). This is legitimate **only now**: the list is "the pure-total subset of
`dispatch_keyword_head`", and after step 2 `nth` is finally in that population. It reads no state and
performs no effect; it raises on out-of-range, which is not disqualifying — `i64::/` is admitted at
the top of that same list precisely because div-by-zero is a deterministic located abort.

## Tests to add

- **A macro-body probe.** A `defmacro` whose program body calls `(nth (ast->children …) 1)`, mirroring
  the shape at `wat/service.wat:468`. This is the whole reason the stone exists — it must go from
  impossible to green.
- **A differential**, `nth` vs `nth-spec`, over Vector / PersistentVector / List / Stream, at index 0,
  middle, last, and past the end. Same values, same raise, same message. Include a **non-vacuity
  control**: perturb one side, watch the differential go red, revert it byte-identical, and say so.
- **A force count**: `nth` on a Stream at index `i` realizes exactly `i+1` cells. Build the generator
  from `wat-scripts/scratch-pad/probe-118B4-forces-per-element-by-walk-shape.wat` — its `:user::gen`
  prints one line per realization.

## Blast radius

`src/runtime.rs`, `src/check.rs`, `src/collection/seq_container.rs`, `src/macros/eval.rs`,
`wat/core.wat`, plus new tests. **No changes to `wat/bracket.wat`, `wat/fix.wat`, or
`wat/service.wat`** — their existing `nth` calls are supposed to keep working untouched.

## STOP triggers — each is "ship nothing further, report the gap"

**STOP-1** — the new capability cannot be added without changing which containers `indexable()`
accepts. Report what forced it. Do not widen `indexable()`; B4-iii depends on being able to flip it.

**STOP-2** — the differential disagrees anywhere. Report both sides verbatim: input, native output,
oracle output. A disagreement is the finding, not a thing to reconcile by editing one side.

**STOP-3** — Stream `nth` at index `i` does not realize exactly `i+1` cells. Report the count. A
higher number means the native drains where the oracle walks, which reintroduces the retention B3
deleted.

**STOP-4** — any existing test changes its result. Name it and give both outputs.

## Out of scope, and it matters

**Do not rule the `(nth s 0)` question.** Once `nth` accepts a Stream, `(nth s 0)` is a
`first`-equivalent, which interacts with the wall B4-iii will build. That is B4-iii's ruling and the
design stone says so. **B4-0 is a representation change: identical semantics, proven by the
differential.** If you find yourself changing what `nth` *means* on a Stream, stop — that is the
wrong stone.

## Verification

Run everything in the FOREGROUND and block on it — your turn ends when the numbers are in your hands,
not when a command is launched.

```
cargo build --release
systemd-run --user --scope -q -p MemoryMax=12G -p MemorySwapMax=0 timeout 1500 scripts/floor.sh
cargo clippy --release --all-targets -- -D warnings
```

Read the floor's **Summary line** from `.floor/latest/clean.log`, never a piped exit code. On any red:
do not re-run — copy the failing test's whole block verbatim, name the assertion that fired, stop.

## Prior result to copy for shape

`DESIGN-STONE-118.B6-native-foldl-over-seqable.md` and its tests — same arc, same native-plus-oracle
split, and a differential design worth mirroring closely.
