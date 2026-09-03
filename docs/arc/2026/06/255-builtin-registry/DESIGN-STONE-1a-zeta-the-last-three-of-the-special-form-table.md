# DESIGN — STONE 1a-ζ: `do` · `ann-form` · `stream::lazy` — the last registerable rows of `special_forms.rs`

> Phase 1a of `[[DESIGN-CAMPAIGN-the-registry-becomes-the-sole-authority]]`, final family.
> Selected ahead of every rete remainder by the corpus re-derivation of 2026-09-03, on USE.

## Why these three, now

The corpus experiment re-derived to **71** names. Weighed by call site rather than by name:

```
                names   call sites
:wat::core::       33        4,260     ← 87% of all remaining unresolvable calls
:wat::rete::       30          278     ← 42% of the names, 6% of the use
```

★ Two of this stone's three rows are the **2nd and 7th largest names in the entire remainder**:
`:wat::core::do` at **609** call sites and `:wat::core::ann-form` at **244**. With
`:wat::stream::lazy` (15) that is **868 corpus call sites for three authored rows** — the best
ratio left anywhere on the worklist, and it was sitting under a heading the campaign had been
reading by name-count.

## The measured ground

```
row                    special_forms.rs   eval arm        tail arm    check arm   registered
:wat::core::do              @165          rt:2260         rt:998      ck:2881         no
:wat::core::ann-form        @142          rt:2267         rt:1009     ck:3238         no
:wat::stream::lazy          @244          rt:2279         (none)      ck:3304         no
```

`special_forms.rs` holds **30** rows (not the 35 the SEAM carries — stale), of which **24** are
registered. After this stone: 27 registered, and the three that remain are the three no stone can
take — `defstruct` (a stdlib macro, no declare-time fn to name) and `unquote`/`unquote-splicing`
(punctuation; a containment fact, not a row — an open fork).

★★★ **This stone therefore ENDS Phase 1a.** Every row in `special_forms.rs` that a stone can
register will be registered.

## ⛔ THE ARMS COME OUT, AND THAT IS NOT OPTIONAL

`registry_first_door_owns_every_handler_row_no_literal_arm_survives` filters on
`entry.handler.is_some()`. Registering `role = eval` gives each row a handler, and the gate then
**demands its literal arm inside `dispatch_keyword_head_value` be deleted** — `rt:2260`,
`rt:2267`, `rt:2279`.

⚠ This is written into the brief up front rather than left to be discovered. Last time a stone in
this family landed, the brief said *"don't touch the eval arms"* and **the gate outranked the
STOP** — correctly. A STOP is a claim about the world; the floor outranks it. The gate measures
`handler.is_some()` rather than naming rows precisely so the demand is automatic.

**The tail arms are a separate slot and a separate question.** `tail_handler` is its own field,
consulted only by `eval_tail`'s own guard (STOP-3 of the tail-door stone), and the no-literal-arm
gate's span is bounded to `dispatch_keyword_head_value` — so `rt:998`/`rt:1009` are outside it.
The precedent is `if`/`let`/`match`/`and`/`or`: each carries `role = tail`, and each arm was
retired when the registry-first tail door began answering first. `do` and `ann-form` follow that
precedent; `stream::lazy` has no tail arm and gets no `role = tail`.

## Acceptance — DERIVED

Each row read off a ledger or off the count of rows added. None is arithmetic on a nearby number.

```
                  before   after   why
registry rows       523     526    +3 attribute sites (count ANCHORED to `^\s*#\[wat_…`)
GAP_A                60      60    none of the three has a CheckEnv scheme, so none is on GAP_A
GAP_B                71      68    all three are on GAP_B — measured, not assumed
DEBT                103     106    +3: each has a literal check ARM, never an `env.register`
                                   scheme, so `check_env.get` returns None and the type gate
                                   skips them. The honest cost, and it lands in the CENSUS
                                   half of DEBT (a rank-1 scheme is the wrong shape for a
                                   special form), not the owed half.
KNOWN_UNREVIEWED     20      20    each row argues its own `@Totality`; none may be Unreviewed
floor          5127/5127  5127/5127  registering a row mints no `#[test]` fn
the corpus 71        71      68    −868 call sites, to be RE-DERIVED not predicted
```

## Out of scope — CUT

- `defstruct`, `unquote`, `unquote-splicing` — the three `special_forms.rs` rows no stone can
  take. Each is an open fork with its own NOTE; none is deferred work, all three are affirmatively
  outside this stone.
- The `:wat::core::` 33. `do` and `ann-form` are in that population by prefix but in Phase 1a by
  membership — they are `special_forms.rs` rows with declared arities, which is what makes them
  cheap. The other 31 are not, and nine of them carry live dispatch arms (Phase 1c).
- `special_forms.rs` itself. It keeps all 30 rows. This stone makes the registry able to answer;
  deleting the table is Phase 4a.
