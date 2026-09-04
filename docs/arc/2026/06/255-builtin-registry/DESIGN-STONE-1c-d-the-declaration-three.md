# DESIGN — STONE 1c-d: `extend-type` · `derive` · `defclause` — the third regime, and a fork closes

> Phase 1c's last `:wat::core::` family. **212 corpus call sites**, and a regime this campaign has
> not touched since 1a: forms that are consumed at freeze time and never evaluate.

## Why these three are one stone

All three share a shape no other remaining name has:

```
                     dispatch arm   declare-time entry            check arm   special_forms.rs
extend-type   157        NONE       parse_extend_type_form :937      :2647          no
derive         47        NONE       parse_derive_form     :1273      :2666          no
defclause       8        NONE       parse_defclause_form   :690      :2635          no
```

**No dispatch arm at all.** They are not verbs the evaluator reaches — they are declarations the
freeze pipeline consumes. That is `@Purity Unevaluated` and `role = declare`, the regime minted at
Stone 1a-β-0 for `def`/`defmacro`/`defalias` and used again at 1a-δ for the loaders.

Each already has a named, `pub(crate)` declare-time fn to point `role = declare` at, and a checker
arm to point `role = check` at. **Nothing needs extracting and nothing needs writing** — this is
wiring two pointers per row onto code that already exists.

★ `role = declare` **stacks** (`src/config.rs:327-328` carries two annotations on one fn) because
it emits source text only — the asymmetry `[[NOTE-role-eval-cannot-stack-and-the-error-does-not-say-so]]`
records. Nothing here needs it, but it means a shared declare-time fn would not be a blocker.

## ★★★ AND IT CLOSES A FORK THE SEAM HAS CARRIED SINCE THE HAND-ROLLED-ARMS STONE

```
SEAM:214   defclause   lost its named refusal this session; it has no registry row. Register it.
```

That stone retired the hand-rolled `def`/`defclause` arms in favour of the registry-first door's
`Unevaluated` guard. `def` had a row, so it kept a named refusal. **`defclause` had none, so it
lost one** — `runtime.rs:2054-2058` and `:2250` record the loss at the site.

★ Registering `defclause` restores it **by construction, not by an arm**: the moment the row exists
with `@Purity Unevaluated`, `dispatch_keyword_head`'s guard answers
`DeclarationInExpressionPosition` for it. That is the campaign's whole thesis in one row — the
refusal comes back because the registry can finally answer, not because someone re-added a
special case.

## The axes — the regime decides most of them

`@Purity Unevaluated` is not a choice here; it is what the registration gate keys on
(`every_special_form_carries_check_and_eval_impls`: an `Unevaluated` row may name `declare` alone,
every other row must name `check` AND `eval`). A row with no dispatch arm and no handler has no
eval impl to name, so `Unevaluated` is the only grade the gate admits — and it is true: all three
are consumed by `freeze`/`register_types`/`parse_*` before evaluation begins.

⚠ **`@Category` is the one that must be argued, not assumed.** `:Declaration`'s own prose is
"registers a program-level entity … visible to everything after it", which fits all three on its
face — but 1a-δ proved that reading wrong once already: the loaders looked like declarations and
turned out to be `:Splice` (a load registers nothing; it replaces itself with N forms). Each of
these three must be read for what it actually *does* to the program, not what its name suggests.

## Acceptance — DERIVED

```
                  before   after   why
registry rows       546     549    +3 attribute sites (ANCHORED count)
GAP_A                49      49    none of the three is on it
GAP_B                48      45    all three are on it
DEBT                115     118    ⬅ +3 — no CheckEnv scheme exists for any of them
KNOWN_UNREVIEWED     14      13    ⬅ −1: `derive` IS on it, the other two are NOT.
                                   CHECKED against the constant, not assumed — this is the
                                   ledger two acceptance tables got wrong by predicting
                                   "unchanged", and one got right by measuring.
literal arms deleted  —       0    ⬅ THERE ARE NO ARMS. Unlike every stone since 1c-a, the
                                   no-literal-arm gate has nothing to demand here.
defclause's named refusal   RESTORED, by the Unevaluated guard rather than by an arm
floor          5129/5129  5129/5129
the corpus 43        43      40    −212 sites
```

★ **Zero arm deletions is the tell that this is a different regime**, and it is why the stone is
cheap despite 212 call sites: there is no dispatch to migrate, only a declaration to record.

## Out of scope — CUT

- `=`/`not=` — held at `Partial` on a named door (bounded generics). Not this stone, not soon.
- `str` · `None` — ordinary verbs, the small follow-on that finishes `:wat::core::`.
- `defstruct` · `unquote` · `unquote-splicing` — the three `special_forms.rs` rows no stone can
  take. Unchanged.
- Every implementation body. This stone annotates and registers; it rewrites nothing.
