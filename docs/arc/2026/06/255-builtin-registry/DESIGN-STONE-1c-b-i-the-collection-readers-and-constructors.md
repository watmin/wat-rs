# DESIGN — STONE 1c-b-i: `first` · `second` · `third` · `PersistentVector` · `PersistentMap`

> Phase 1c-b: the `:wat::core::` names with **no** `CheckEnv` scheme. Where 1c-a drained two
> ledgers and paid nothing, this half **trades GAP_B for DEBT** — a different deliverable, said
> plainly in its own acceptance row rather than folded into one number.

## The remaining population, re-measured

**16 names, 2,464 corpus call sites** (the cached census still listed four that this session's
codemod repaired — `println`, `edn::write`, `tuple-get`, `reduce-walk` — so the raw file reads 20
/ 2,470 and is stale by exactly those six sites). All 16 sit on **GAP_B only**: none on GAP_A,
none yet on DEBT.

They fall into five families by mechanism, not by prefix:

```
accessors      first · second · third                      597 sites   ← THIS STONE
constructors   PersistentVector · PersistentMap            705 sites   ← THIS STONE
comparison     = · not= · < · > · <= · >=                  812 sites
declarations   extend-type · derive · defclause            212 sites   ⚠ no door arm — freeze-time
misc           str · None                                  138 sites
```

## The stone, and why these five are ONE stone

Five rows, **1,302 corpus call sites** — and the seam is the mechanism, not the verb semantics.
Every one of the five is the same shape: **a dispatch arm that does a little pre-processing and
then delegates**, so the fix for every one is the same — extract a per-name wrapper, annotate it,
delete the arm.

```rust
":wat::core::first"  => eval_positional_accessor(args, span, env, sym, ":wat::core::first",  0)
":wat::core::second" => eval_positional_accessor(args, span, env, sym, ":wat::core::second", 1)
":wat::core::third"  => eval_positional_accessor(args, span, env, sym, ":wat::core::third",  2)

":wat::core::PersistentVector" => { split_type_param_bracket(args) …
                                    eval_persistentvector_ctor(values, …) }
":wat::core::PersistentMap"    => { split_type_param_bracket(args) …
                                    eval_persistentmap_ctor(values, …) }
```

## ⛔ THE TRAP, AND THIS ARC PREDICTED IT BY NAME

**The three accessors share ONE implementation**, parameterised by index — one runtime fn
(`eval_positional_accessor`) and one checker fn (`infer_positional_accessor`), each taking the
FQDN and the index as arguments.

`#[wat_intrinsic]` emits its shim as `format_ident!("__wat_intrinsic_shim_{}", fn_name)` — **keyed
on the function's identifier, not the FQDN.** Three annotations on `eval_positional_accessor`
would emit three shims of the same name and fail to compile.

★★★ `[[NOTE-role-eval-cannot-stack-and-the-error-does-not-say-so]]`, written by the 1a-ε rider,
says exactly this and forecast exactly this moment: *"It will bite again: 15 rows remain to
register and **shared arms are the norm**."* It is biting now, and the cure is named up front
rather than discovered in a compile error whose message names a mangled symbol.

**So all five get their own thin wrapper.** Not a workaround — it is the only shape that gives
five distinct FQDNs five distinct shims, and it leaves every shared implementation untouched.

## ⚠ AND THE DEBT THIS PAYS IS PARTLY MISFILED — the second concrete instance

All five land on DEBT (none has an `env.register` scheme). But `probe_can_doc_types_reconstruct_the_checker_scheme`
splits DEBT by `Kind`: `Kind::Intrinsic, no scheme` reads *"a scheme could exist and does not"*.

For the accessors that is **false**. `infer_positional_accessor`'s own doc states it:
*"Polymorphic over (Vector :- [T]) and tuple — both are index-addressed. **Rank-1 HM can't express
the union, so this is special-cased**."* A rank-1 `TypeScheme` genuinely cannot exist for them —
they belong in DEBT's **census** half, and the `Kind` discriminator will file them as **owed**.

★ This is the second measured instance of the defect already surfaced to the builder: **`Kind` is
stamped by the registration vehicle, not by the verb.** (The first: `:wat::rete::core::List`, an
alias to an ordinary function, filed under "wrong shape".) It is recorded here, not fixed here —
the ledger split is its own stone and the builder holds that ruling.

## THE FOUR QUESTIONS

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **the five, as one stone, cut on the wrapper mechanism** | YES | YES | YES | YES | ✅ |
| split accessors from constructors | YES | **NO** | YES | — | ⛔ |
| all 16 remaining `:wat::core::` in one stone | YES | **NO** | **NO** | — | ⛔ |
| take the comparison family first (812 sites) | YES | YES | YES | **NO** | ⛔ **later, not never** |

- **split — Simple NO**: it would cut on verb semantics (read vs build) while the *work* is
  identical in every row. 1c-a cut on module because that was where the mechanism changed; here
  it does not change.
- **all 16 — Honest NO**: the declarations have no door arm at all (freeze-time forms, the
  `Purity::Unevaluated` regime), so one number would report three different mechanisms.
- **comparison first — Good UX NO**, and this is a scheduling judgement, not a dismissal: `=` at
  688 sites is the single largest name left and it unblocks six rete aliases, but polymorphic
  equality is the hardest axis story in the remainder (what is `=` total ON?). Taking it after
  two clean wrapper stones means the pattern is proven before the hard argument starts.

## Acceptance — DERIVED

```
                  before   after   why
registry rows       537     542    +5 attribute sites (ANCHORED count)
GAP_A                49      49    none of the five is on it
GAP_B                57      52    all five are on it
DEBT                106     111    ⬅ +5, ALL FIVE. The honest cost. A rise of anything
                                   other than 5 means a different population was registered.
KNOWN_UNREVIEWED     14      14    none of the five is on it (only `derive` is, and it is
                                   not in this stone) — CHECKED, not assumed
literal arms deleted  —       5
floor          5127/5127  5127/5127
the corpus           —      −5 names, −1,302 sites, to be RE-DERIVED not predicted
```

## Out of scope — CUT

- The comparison six, the declaration three, `str`, `None`. Each its own stone.
- `eval_positional_accessor`, `infer_positional_accessor`, `eval_persistentvector_ctor`,
  `eval_persistentmap_ctor` — every shared implementation stays untouched. This stone adds
  wrappers and deletes arms; it rewrites nothing.
- The DEBT ledger split. Named above as a second instance; ruled elsewhere.
