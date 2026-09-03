# DESIGN — STONE 1c-a-i: the six collection transforms, and why 1c-a splits by MODULE

> Phase 1c of `[[DESIGN-CAMPAIGN-the-registry-becomes-the-sole-authority]]`.
> Crawl: `[[DESIGN-CAMPAIGN-1c-the-lair-study-before-any-strike]]`.

## The crawl corrected the lair study twice

**① The "inline blocks" are not inline logic.** The study counted five `:wat::core::*` arms as
inline `=> { … }` blocks needing extraction. Read: four of the five —
`foldl`, `filter`, `stream->vec`, `find-last-index` — are **wrapped one-line delegations** to
`crate::collection::transform::*`. The braces are rustfmt, not logic. Only `:wat::core::Tuple`
(18 lines, type-directed through `split_type_param_bracket`) carries real inline work.

So the true shape of 1c-a's eleven: **ten delegate to a named fn and can be annotated in place;
one needs extraction.**

**② The seam is the MODULE, not the arm shape.** Six of the eleven live in one module and share
one identical signature:

```rust
pub(crate) fn eval_vec_map | eval_mapv | eval_stream_to_vec
             | eval_vec_find_last_index | eval_vec_foldl | eval_filter (
    args: &[WatAST], call_span: &Span, env: &Environment, sym: &SymbolTable,
) -> Result<Value, EvalBreak>
```

That is exactly `#[wat_intrinsic]`'s variadic shape with a context tail the macro sniffs in any
declared order. **No delegate, no extraction, no signature reshaping — six annotations in place.**

## The stone

Register the six `crate::collection::transform` verbs — `foldl` · `map` · `filter` ·
`stream->vec` · `mapv` · `find-last-index` — as `#[wat_intrinsic]` rows on their existing handler
fns, each with an argued doc block including all five closed-domain axes. Then delete their six
literal arms from the eval door, which the registry-first door will answer instead.

★ **463 corpus call sites**, `foldl` alone at 380 — the largest single name left after `do`.

## ⛔ THE ARM CASCADE IS THE POINT, NOT A SIDE EFFECT

`registry_first_door_owns_every_handler_row_no_literal_arm_survives` filters on
`entry.handler.is_some()`. `#[wat_intrinsic]` mints a shim, so the gate **requires** each of the
six arms be deleted from `dispatch_keyword_head_value`. This has already outranked one of my STOPs
in this campaign; it is stated up front rather than discovered.

## Why this is the 1b-i shape, and why that matters

All six sit on **GAP_A and GAP_B**, and **none on DEBT** — measured, not assumed. Each already has
a `CheckEnv` scheme, so registering drains two ledgers and pays nothing:

```
GAP_A  60 → 54        GAP_B  68 → 62        DEBT  106 → 106
```

★★★ And DEBT holding is the acceptance row that **cannot be satisfied sloppily**:
`doc_arg_ret_types_match_checker_scheme` (`src/intrinsic/mod.rs:2254`) actively compares every
`@arg` and `@ret` string against that scheme and reds with both spellings side by side. Unlike
1b-ii — which had no such gate and needed a hand audit — **this stone's central content is
machine-checked.** A mis-transcribed type cannot ship.

## THE FOUR QUESTIONS — the decomposition of 1c-a's eleven

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **split by module: 6 transforms, then 5 runtime/Tuple** | YES | YES | YES | YES | ✅ **PICKED** |
| all 11 in one stone | YES | **NO** | **NO** | — | ⛔ |
| split by arm shape (delegating vs inline) | **NO** | **NO** | YES | — | ⛔ |

- **all 11 — Simple NO**: eleven authored rows is past this campaign's own measured rider ceiling
  (6 authored rows + 11 edits ≈ 375K tokens). **Honest NO** too: it would bundle `Tuple`'s
  extraction and `apply`'s test cascade with six mechanical annotations, and report one number
  for three different kinds of work.
- **split by arm shape — Obvious NO**: the shape is an artifact of rustfmt. Four of the five
  "inline" arms are one-line delegations. A cut on that axis divides nothing real, and the study
  that proposed it was reading braces as logic.

## ⚠ What 1c-a-ii inherits, named now so it is not a surprise

- **`:wat::core::Tuple`** — the one genuine extraction. Its arm guards a documented
  check-says-no/runtime-says-yes divergence (`(Tuple :- [A B])` with no values is an arity
  mismatch, and answering it with an empty tuple would re-create the exact class step ①b found).
  That guard must move verbatim.
- **`:wat::core::apply`** — registering it fires a pre-written retirement.
  `tests/diagnostics/probe_substrate_symmetry_list_span_threading.rs`'s `MUST_FIND` has exactly
  **one** anchor left, and the test's own comment rules on it in advance: *"When the last one
  goes, this positive control has nothing left to anchor on and should be DELETED rather than
  re-anchored — a parser-sanity check for a match that no longer exists is a test asserting the
  absence of its own subject."* ★ A test whose retirement condition a prior self wrote, about to
  fire.
- **`eval_apply`'s signature is `(args, env, sym, list_span: Span)`** — `Span` BY VALUE, not
  `&Span`. It will not match `#[wat_intrinsic]`'s context-tail sniff and needs a thin delegate.

## Acceptance — DERIVED

```
                  before   after   why
registry rows       526     532    +6 attribute sites (count ANCHORED to `^\s*#\[wat_…`)
GAP_A                60      54    all six are on it — each has a scheme, no row
GAP_B                68      62    all six are on it
DEBT                106     106    ⬅ UNCHANGED. Each resolves in CheckEnv already.
                                   A rise means a row was mis-transcribed.
KNOWN_UNREVIEWED     18      18    each row argues its own Totality; none may be Unreviewed
the corpus 65        65      59    −463 call sites, to be RE-DERIVED not predicted
floor          5127/5127  5127/5127  registering a row mints no `#[test]` fn
literal arms deleted   —      6    the gate's demand, not an extra
```

## Out of scope — CUT

- The other five of 1c-a (`get` · `apply` · `contains?` · `conforms?` · `Tuple`) — 1c-a-ii.
- The 20 `:wat::core::` names with no scheme — they trade GAP_B for DEBT, a different deliverable.
- `crate::collection::transform`'s other verbs. Only the six with a live door arm AND a scheme.
