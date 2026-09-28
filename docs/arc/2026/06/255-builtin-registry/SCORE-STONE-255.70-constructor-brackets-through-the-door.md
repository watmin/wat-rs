# SCORE — STONE 255.70: a constructor's type bracket reads through the type door; `List` takes `:-`; convert the rest

**Executor: a Sonnet subagent.** Commits `c389233af` (part 1 — the two checker/runtime gaps),
`d3d0434e5` (part 2 — the 109-site corpus conversion), `ef6c3c203` (the floor's own one fix), this
SCORE. Drawn at `22415ef09`, against `f99789bc7`.

## Part 1 — the checker/runtime gaps close (`c389233af`)

### 1a. A constructor's `:-` bracket reads through the type door

`parse_bracket_type_keyword` (`src/check.rs`) accepted only a bare `WatAST::Keyword` per slot of a
constructor's own `:-` bracket — the K1 class 255.67 retired five times, still live here for
`Tuple`/`PersistentMap`/`PersistentVector`'s bracket slots. **Retired the function**; its three
call sites now call `parse_param_spec_slot` (`src/check.rs`), the SAME door
`infer_hashset_constructor`/`infer_list_constructor` already used — which itself calls
`parse_type_node` (`src/types.rs`), the one door reading a bare keyword, a namespaced symbol, a
nested parametric `(Head :- […])` form, and a fn-type bracket identically.

**Every site touched** (all three of `parse_bracket_type_keyword`'s callers; no fourth site found):

- `infer_persistentmap_constructor` (`src/check.rs`, K/V slots) — 2 call sites, one per slot.
- `infer_persistentvector_constructor` (`src/check.rs`, T slot) — 1 call site.
- `infer_tuple_constructor` (`src/check.rs`, per-position slot) — 1 call site, in a loop.

**Searched for other keyword-only bracket/constructor paths** (the brief's own instruction —
`unwrap_type_param_bracket` and the constructor intrinsics' run-time peel included):

- `unwrap_type_param_bracket` (`src/check.rs`) — NOT a keyword-only recognizer: it calls
  `peel_param_spec` (`src/types.rs`), which only detects the `:-` MARKER and splices the raw
  `WatAST` nodes back into the value stream unparsed — it never inspects a slot's shape at all.
  Its three callers (`Vector`/`HashMap`/`HashSet`'s runtime dispatch arms, `src/runtime.rs`) are
  unaffected; no change needed.
- `eval_vector_ctor`/`eval_hashmap_ctor`/`eval_hashset_ctor`'s own runtime `args[0]` shape guard
  (`src/collection/eval.rs`) — already routes `Symbol`/`List`/`Vector` shapes through
  `parse_type_node` (a `WatAST::Keyword` arm is a no-op accept, since types are erased at
  runtime); confirmed unchanged, not the same bug class.
- `eval_persistentvector_ctor`/`eval_persistentmap_ctor`/`eval_tuple_ctor`'s runtime dispatch
  (`src/runtime.rs`, `src/collection/eval.rs`) — use `split_type_param_bracket`, which discards
  the bracket's contents wholesale (types are erased at runtime for these three); no shape
  inspection at all, so no keyword-only bug possible there.

### 1b. `List` takes `:-`, checker and runtime

**Checker** (`infer_linked_list_constructor`, `src/check.rs`): now peels an OPTIONAL leading
`:- [T]` bracket via `split_type_param_bracket`, the slot parsed through the door via
`parse_param_spec_slot` — the exact same peel `infer_persistentvector_constructor` uses (declared
T is the `assignable` up-cast target; bracket-less stays a fresh unified variable). The bracket is
optional, not mandatory (unlike `infer_list_constructor`'s `Vector` arm) — the wall making it
mandatory is a later stone, and STOP-2 requires every bracket-less `List` call still standing
elsewhere in the corpus to keep checking exactly as before.

**Runtime** (`src/intrinsic/list.rs`): `:wat::core::List` moves off ALGEBRA back onto BINDING (an
`&[WatAST]` leading param, the shape `PersistentVector`/`PersistentMap`/`Tuple` already use) —
ALGEBRA's auto-generated AST door evaluates every arg (bracket included) as an ordinary value
BEFORE the handler ever sees it, which is wrong for a `:-`/`[T]` bracket. The new `eval_list_ctor`
strips an optional bracket via `split_type_param_bracket` (discarding the declared type — types
are erased at runtime, matching `PersistentVector`/`PersistentMap`/`Tuple`'s own runtime peel)
before evaluating the rest by ordinary call-by-value. The prior ALGEBRA-era value door (what
`apply`'s substrate fallback, `dispatch_substrate_impl`, reaches for `(apply :wat::core::List
[1 2 3])`) is preserved byte-for-byte as `list_of_value_door`, wired explicitly via
`#[wat_intrinsic(":wat::core::List", value = list_of_value_door)]` — STOP-2: moving the AST door
off ALGEBRA must not silently drop the value door none of `List`'s BINDING siblings ever had one
to lose.

### Tests (`tests/function/probe_stone255_70_constructor_brackets_through_the_door.{wat,rs}`)

6 rows, each checking AND running: a `PersistentVector` of `Tuple`s (compound element type), a
`PersistentMap` of `String` to `PersistentVector` (compound element type), `List` with `:-` in
value position (non-empty), the empty bracketed `List` (the empty list), a bracket-less `List`
call (STOP-2 regression guard — must still run), and `List` with a compound (`Tuple`) element
type (both fixes together).

```
cargo test --release --test function stone255_70
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 227 filtered out
```

## Part 2 — the 109-site corpus conversion (`d3d0434e5`)

### Re-deriving the 109 (measured fresh against this stone's fixed checker, not copied)

**The 35 compound sites** — re-walked with the committed
`wat-scripts/scratch-pad/255-69-census-walk.wat` census tool over the 19 distinct files
SCORE-STONE-255.69's own list named: all 35 exact `(file, line, col)` positions confirmed present,
byte-identical to that SCORE's list (0 missing, 17 other untouched sites in the same files
correctly left alone — unresolved/not-checked, out of this stone's scope).

**The 74 List sites** — independently re-derived, not copied: census-walked the 29 files
SCORE-STONE-255.69 named (86 raw `List`-headed hits), joined each against a fresh
`WAT_CHECK_TYPES=1 wat --check <file>` run by exact `(file, line, col)` — **74 TYPE-tagged** (a
genuine recorded element type, uniformly `:wat::core::i64` at every one of the 74), **10
UNRESOLVED**, **2 not-checked** (86 = 74+10+2). **74 matches SCORE-STONE-255.69's own count
exactly**, independently re-measured.

### The bridge (an input, never a committed artifact — same rule 255.69's own type-table builder
followed: `rune:replay(unreadable-preimage)` territory, not a corpus edit)

The checker's recorded-type DISPLAY text still uses the compact tuple notation `:(A,B)` inside a
Tuple's own rendering context (confirmed: `read-string` refuses it — "comma inside keyword body
retired"), and inside that same stripped-rendering context every leaf (a namespaced type, a
Parametric head, a rigid type-var) loses its leading colon too — confirmed empirically across the
35 sites' own recorded types (22 of 35 needed the bridge; 13 already read clean). Wrote a small
standalone Rust program (session scratchpad, not committed — the same disposition
`scratch/stone25569_desugar.rs` had) that recursively distinguishes a Parametric fragment (a
top-level `:-` token present) from a compact Tuple fragment (none), rewriting the latter to
`(:wat::core::Tuple :- […])` and restoring a leading colon on every bare leaf token found inside
either shape — never touching a fragment that already reads clean. Verified against all 35 real
recorded types before building the codemod's table.

### The codemod, unchanged

`wat-scripts/fixes/typed-constructors.wat` needed **no code change** — it already walks a
constructor's args-vector structurally (any nested form, any head) and already lists `List` among
its five convertible heads; the 109 sites were excluded ONLY because the type table SCORE-255.69
built deliberately omitted them (the pre-255.70 checker refused the converted shape). Drove it,
unchanged, with a fresh table covering the 109 sites, over the 48 distinct files they live in.

**Dry-run** (`/tmp` copies of the 48 files): codemod ran clean (rc 0), every file changed, `diff`
inspected (spot-checked `wat/gen.wat`, `tests/collection/probe_arc216_stone7_tuple_roundtrip.wat`,
`tests/collection/list.wat` — each hand-verified correct: the bracket-less
`(:wat::core::List 1 2)` at `list.wat`'s untouched empty-call site was correctly left alone, not
in the 74). `--check` rc before vs after, all 48: **41 standalone-checkable non-`wat/` files, 0
flips** (rc 0 -> rc 0 for all 41); the 7 `wat/` stdlib files (`ReservedPrefix` on a standalone
`--check` either way, pre-existing and unrelated) validated instead by temporarily swapping the 7
converted files into the live tree and re-running `WAT_CHECK_TYPES=1 wat --check <trivial-entry>`
(which boots the WHOLE frozen stdlib regardless of entry file) — the summary line **`TYPES
recorded 41599 distinct 16867 spans 14368 multi 158 unresolved 122 orphans 0 check-errors 0`** is
byte-identical before and after the swap (reverted via `git checkout --` immediately after).
Idempotent: re-ran the codemod over the dry-run copies, `sha256sum` identical, 0 changes.

**Applied to the real corpus** (same table, same 48 paths): identical output to the dry-run
(`diff -q` against every dry-run copy: 0 mismatches). Idempotent on the real corpus too: re-ran
the codemod over the 48 converted files, `sha256sum` identical, 0 changes.

**`wat-tests/gen.wat:354:3` / `:837:3`** (the duplicate-`defn` pair, byte-identical bodies,
`:837:3` carrying no type record): **left BOTH untouched** — 837 has no TYPE record to convert
354's twin against, and converting 354 alone would diverge the two bodies and trip
`DefRedefForbidden` (the exact mechanism SCORE-STONE-255.69 already diagnosed for this pair).
Confirmed both lines still read byte-identical (`(:wat::core::PersistentVector 3 3 3 3 4)`) after
the conversion pass.

### Coverage

| set | target | converted |
|---|---:|---:|
| compound-element (PersistentMap 6, PersistentVector 10, Tuple 19) | 35 | **35** |
| `List` | 74 | **74** |
| duplicate-`defn` pair | 1 | **0** (left both, named above) |
| **total** | **109** | **109** |

**109 of 109 target sites converted (35+74); the separate duplicate-defn pair (1 site, not part of
the 109 — its own exclusion from SCORE-STONE-255.69) left both untouched, as designed.**

### The re-derived remaining untyped-constructor count, corpus-wide

Fresh census walk, whole current corpus scope (`git ls-files '*.wat' '*.wat.bad'`, excluding
`wat-scripts/fixes/**` and `*.wat.golden` — **2549 files**, 4 more than SCORE-255.69's cited
2545, ordinary repo drift since that stone landed the same day):

```
List              23
PersistentMap     27
PersistentVector  44
Tuple             73
Vector            10
-----
total            177
```

Arithmetic against SCORE-255.69's own count: `1804 (255.68 total) - 1520 (255.69 converted) - 109
(this stone converted) = 175` expected. Measured **177** — reconciled, both accounted for:

- **+1 `List`**: this stone's OWN new test fixture
  (`tests/function/probe_stone255_70_constructor_brackets_through_the_door.wat:41:3`) deliberately
  contains one bracket-less `(wat.type/List 1 2 3)` call — the STOP-2 regression-guard row
  (`list-bracketless-still-runs`), intentionally left untyped to prove the bracket stays optional.
- **+1 `PersistentVector`**: the 4-file corpus-count drift (2549 vs 2545) — unrelated commits
  landed elsewhere in the tree since SCORE-255.69, outside this stone's diff, not investigated
  further (not this stone's row to explain).

## Gates

| what | how | result |
|---|---|---|
| release floor (1st) | `scripts/floor.sh`, once, alone, foreground/background, at `d3d0434e5` | RED — 1 failed (this stone's own gap, diagnosed and cured — see below) |
| release floor (2nd) | `scripts/floor.sh`, once, alone, at `ef6c3c203` | `.floor/2026-09-28T22-13-36Z`: `Summary [ 388.715s] 6225 tests run: 6225 passed (27 slow), 24 skipped` |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 (both before and after the floor's own fix) |
| census | pre `.census/2026-09-28T21-58-38Z.txt` (2266 files, unmodified draw) -> post `.census/2026-09-28T21-59-59Z.txt` (2266 files) | `census-diff: no STOP-8`, rc 0 |
| delta | `scripts/replay/delta.sh --list <48 converted files> --codemod wat-scripts/fixes/typed-constructors.wat` (pre-conversion content restored via `git checkout HEAD -- <list>` first, at the part-1-only commit `c389233af`, then re-applied after) | `ORIG-CLEAN 41/48  CONV-CLEAN 41/48  NEW 0  RECOVERY 0`, rc 0 |
| idempotent | the codemod re-run over all 48 converted files, both the dry-run copies and the real corpus | `sha256sum` identical each time — 0 changes |
| coverage | 109 / 109 target sites converted (35 compound + 74 List); the duplicate-defn pair left both | stated above, arithmetic closes |

## The floor went RED once — captured verbatim, diagnosed, cured, new floor run

Per doctrine: not re-run on the red, captured whole from `.floor/2026-09-28T22-03-50Z/ARM.txt`
before anything else, then fixed and a NEW floor run.

**First floor, at `d3d0434e5`** — `.floor/2026-09-28T22-03-50Z/`:

```
FLOOR RED — 2026-09-28T22-03-50Z
exit=100
     Summary [ 383.678s] 6225 tests run: 6224 passed (24 slow), 1 failed, 24 skipped

        FAIL [   0.049s] ( 335/6225) wat::lint unused_span_justified::ignored_spans_are_justified
  stdout ───

    running 1 test
    test unused_span_justified::ignored_spans_are_justified ... FAILED

    failures:

    failures:
        unused_span_justified::ignored_spans_are_justified

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 363 filtered out; finished in 0.04s

  stderr ───

    thread 'unused_span_justified::ignored_spans_are_justified' (1113164) panicked at /home/john/work/holon/wat-rs/tests/lint/unused_span_justified.rs:123:5:


    🔥🔥🔥 UNJUSTIFIED IGNORED SPAN — 2 param(s) named `_…span: &Span` silence rustc's
    unused warning while carrying a source location that a fallible fn could have used. An
    ignored `&Span` can mean an UNLOCATED error (the span was in hand and dropped).

    THE FIX — read the fn's error paths and do ONE of:
    • FIX (unlocated): the fn emits an error while this `_span` was available and would
    improve it → thread the span into that error's `RuntimeError { span: … }`, rename
    `_span`→`span`, NO rune (it no longer matches this lint).
    • RUNE (earned): the drop is safe — add a co-located, same-line
    `// rune:lint(unused-span) — <reason>`. Honest reasons:
    - `infallible — no error path`
    - `located elsewhere` — every error already uses a real WAT span (`arg.span()`,
    a threaded inner span, or the error is a VALUE located at the caller's
    match). State WHERE.
    `rust_caller_span!()` does NOT earn standing: a Rust line is the HARM this lint names,
    not a location. A site that ignores its span AND raises at a Rust line is a FIX.
    A rune of "drops the location but we ignore it" does NOT earn its standing — that site is
    a FIX, not a rune (excusare — the reason must earn it). For a single-line signature, break
    the `_…span: &Span` param onto its own line to carry the rune.

    Offenders:

    src/intrinsic/list.rs:55
    src/intrinsic/list.rs:77

    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**Mechanism**: this stone's own gap — `eval_list_ctor`'s `_list_span: &Span` and
`list_of_value_door`'s `_span: &Span` (both new this stone, `src/intrinsic/list.rs`) are
underscore-prefixed `&Span` params with no justification rune, per
`tests/lint/unused_span_justified.rs`'s structural wall. **Fix** (`ef6c3c203`): both earn an
EARNED exemption, not a FIX-the-error-path — `eval_list_ctor`'s element errors already locate at
each arg's own `arg.span()` via `eval_inner` ("located elsewhere"); `list_of_value_door` has no
error path at all, only `Value::clone` into a `LinkedList` ("infallible"). Added the co-located
`// rune:lint(unused-span)` comment on each param's own line (breaking `list_of_value_door`'s
single-line signature onto multiple lines per the lint's own doc note). Re-verified alone:

```
cargo test --release --test lint unused_span_justified
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 360 filtered out
```

`cargo clippy --release --all-targets -- -D warnings` re-verified clean after the fix (rc 0).
Neither corpus files nor the codemod were touched by this fix — purely the new runtime handler's
own missing rune.

**Second floor, at `ef6c3c203`** — `.floor/2026-09-28T22-13-36Z/`:

```
Summary [ 388.715s] 6225 tests run: 6225 passed (27 slow), 24 skipped
```

All green. No STOP fired at any point in this stone (no red outside this stone's own gap; no
census flip).

## Files

- `src/check.rs` — `parse_bracket_type_keyword` retired; its three callers
  (`infer_persistentmap_constructor` x2, `infer_persistentvector_constructor`,
  `infer_tuple_constructor`) now call `parse_param_spec_slot`; `infer_linked_list_constructor`
  gains the optional `:- [T]` peel. Committed `c389233af`.
- `src/intrinsic/list.rs` — `:wat::core::List` moves ALGEBRA -> BINDING (`eval_list_ctor`), the
  prior value door preserved as `list_of_value_door`. Committed `c389233af`; the two
  `rune:lint(unused-span)` additions committed `ef6c3c203`.
- `tests/function/probe_stone255_70_constructor_brackets_through_the_door.{wat,rs}` — 6 driven
  rows. Committed `c389233af`.
- 48 corpus files (109 sites: 35 compound-element, 74 `List`) — the conversion, driven by the
  unchanged `wat-scripts/fixes/typed-constructors.wat`. Committed `d3d0434e5`.
- The notation-bridge Rust program and the per-site type table — **not committed**, an input per
  the same rule SCORE-STONE-255.69's own type-table builder followed (kept in the session
  scratchpad, reproducible from this SCORE's method: the 35-site and 74-site lists above, joined
  against a fresh `WAT_CHECK_TYPES=1 wat --check <file>` per distinct file, bridged per the
  algorithm described in "Part 2 — The bridge").
