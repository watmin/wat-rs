# DESIGN — STONE 1c-a-ii: `get` · `apply` · `contains?` · `conforms?` · `Tuple`

> Second half of 1c-a, split by MODULE per `[[DESIGN-STONE-1c-a-i-the-collection-transforms]]`.
> 1c-a-i took the six that live in `src/collection/transform.rs`. These five are the remainder:
> four handlers in `src/runtime.rs` plus the one genuine extraction.

## The measured ground

```
name                    corpus   handler                              shape
:wat::core::Tuple          271   INLINE, 18 lines -> eval_tuple_ctor  ⚠ extraction
:wat::core::get            169   eval_get(args, list_span, env, sym)  canonical
:wat::core::apply           26   eval_apply(args, env, sym, Span)     ⚠ Span BY VALUE
:wat::core::contains?        7   eval_contains(…)                     canonical
:wat::core::conforms?        1   eval_conforms(…)                     canonical
                          ────
                           474
```

All five sit on **GAP_A and GAP_B**, none on **DEBT** — the 1b-i/1c-a-i shape: registering drains
two ledgers and pays nothing, and `doc_arg_ret_types_match_checker_scheme` actively verifies every
`@arg`/`@ret` against the scheme each already has. **The stone's central content is
machine-checked.**

⚠ Three of the five — `apply`, `conforms?`, `Tuple` — are ALSO on `KNOWN_UNREVIEWED`
(`src/rete/purity.rs`), so that ledger drains too. **This is predicted, not discovered.** Twice
now I have written "KNOWN_UNREVIEWED unchanged" into an acceptance table and been corrected by a
rider (1a-ζ: 20→18; 1c-a-i: 18→17). The coupling is `intrinsic_meta`'s registry-first consult: a
registered row with declared axes becomes classified, and leaving its name on the ledger then
fails that ledger's own STALE check.

## ⛔ THE THREE COMPLICATIONS, each named before the strike

**① `Tuple` is the one real extraction.** Its arm is 18 lines guarding a documented
**check-says-no / runtime-says-yes divergence**: `(Tuple :- [A B])` declared but unpopulated is an
arity mismatch that `check.rs`'s `infer_tuple_constructor` rejects, and answering it with an empty
tuple at runtime would re-create *"the exact class step ①b's Room 3 was found by"* (the arm's own
words). The `inner.is_empty() && rest.is_empty()` guard is what prevents it. **That guard moves
verbatim into the extracted fn** — this stone changes no behaviour.

**② `eval_apply` takes `list_span: Span` BY VALUE**, not `&Span`. `#[wat_intrinsic]` sniffs a
context tail of `&Environment` / `&SymbolTable` / `&Span` and rejects anything else at
macro-expand time. So `apply` needs a thin delegate carrying the canonical signature — the same
move `quote.rs`, `fn_form.rs` and `stream_lazy.rs` already make. **The live fn is not reshaped.**

**③ Registering `apply` fires a retirement a prior self already ruled on.**
`tests/diagnostics/probe_substrate_symmetry_list_span_threading.rs`'s `MUST_FIND` has exactly one
anchor left, and its own comment decides the case in advance:

> *"When the last one goes, this positive control has nothing left to anchor on and should be
> DELETED rather than re-anchored — a parser-sanity check for a match that no longer exists is a
> test asserting the absence of its own subject."*

★★★ **What retires is the `MUST_FIND` const and its `for` loop — NOT the test.** The substantive
assertion below them (`classify_arm` over every arm, panicking on any arm that calls `eval`
without threading `list_span`) is the test's actual job and stays untouched. A rider that deletes
the function would remove a live symmetry gate to satisfy a comment about a control.

⚠ And the sibling half needs no action: 1c-a-i **retired the `arms.len()` magnitude** rather than
lowering it a fifth time, so the five deletions here cannot trip it. Had it been lowered to 40 as
first written, this stone would have driven the count to 39 and tripped it again immediately.

## THE FOUR QUESTIONS — `MUST_FIND`'s disposition

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **delete the `MUST_FIND` const + its loop; keep the symmetry assertion** | YES | YES | YES | YES | ✅ |
| re-anchor `MUST_FIND` on some other arm | YES | YES | **NO** | — | ⛔ |
| delete the whole test function | YES | YES | **NO** | — | ⛔ |
| keep `apply` unregistered so the control survives | **NO** | YES | **NO** | — | ⛔ |

- **re-anchor — Honest NO**, and the file already ruled it: the control names *"arms the carve
  CANNOT take"*, and the campaign is taking all of them. A fresh anchor would be a claim we
  already know the campaign intends to falsify.
- **delete the function — Honest NO**: it removes a live gate (every arm threads `list_span`) to
  satisfy a comment about a *control* on that gate's instrument.
- **keep `apply` unregistered — Obvious NO**: preserving a test by refusing to do the work the
  test exists to measure is the tail wagging the dog.

## Acceptance — DERIVED

```
                  before   after   why
registry rows       532     537    +5 attribute sites (ANCHORED count)
GAP_A                54      49    all five are on it
GAP_B                62      57    all five are on it
DEBT                106     106    ⬅ UNCHANGED. Each resolves in CheckEnv already.
KNOWN_UNREVIEWED     17      14    apply · conforms? · Tuple are on it — PREDICTED this time
the corpus 59        59      54    −474 call sites, to be RE-DERIVED not predicted
literal arms deleted  —       5
MUST_FIND             1       0    the const and its loop retire; the test does not
floor          5127/5127  5127/5127
```

★ After this stone **Phase 1c-a is complete** and the eleven `:wat::core::` names that already had
schemes are all registered. What remains in `:wat::core::` is the twenty with **no** scheme — a
different deliverable that trades GAP_B for DEBT.

## Out of scope — CUT

- The 20 schemeless `:wat::core::` names. Different ledger arithmetic, own stones.
- `eval_tuple_ctor` itself, and every handler body. Extraction moves the arm's logic verbatim;
  nothing is rewritten.
- The `arms.len()` bound — already retired at 1c-a-i, not to be touched again.
