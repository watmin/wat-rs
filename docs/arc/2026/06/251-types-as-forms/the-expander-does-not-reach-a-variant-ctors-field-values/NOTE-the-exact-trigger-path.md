# NOTE — the exact trigger path, read from the expansion

The builder: *"don't you have an exact trigger path?"* — **I did not.** I had an error message and a
story built around it (*"now that `:arms` is genuinely inferred…"*). This is the traced path, every
step either measured or read from the expansion.

## Why the story was untestable

Four minimal repros were written and **all four passed**:

```
a variant value in a plain arg slot                                   exit 0
a variant inside a RECORD's type argument   (Box :- [Op])             exit 0
variant -> Vector -> record parameterised by the ENUM's own param     exit 0
the real Alarm shape, direct slot AND vector slot                     exit 0
```

★ Every one of them had a **declared** expected type — a function parameter — so inference had
direction. The defect needs the expected type to be ABSENT, which no hand-written repro produced.

## The trigger, from `macroexpand` on the `defservice` form

The `:impls` arm body is spliced as a **`match` SCRUTINEE** (`wat/service.wat:1621`, `:1682`):

```wat
(:wat.core/match
  (:wat.core/let [s state ctx (…)]
    (:wat.service/Outcome.ReplyAndArm {:state s :reply (:probe.Tick2/StartResponse.Ok {})
       :arms [(:wat.service/Alarm :after (:wat.time/Millisecond 5)
                                  :op (:probe.tick2/Op.-Tick {}))]}))
  [:wat.service/Outcome.Reply {…} …]
  …)
```

**A match scrutinee has no declared type.** So:

```
① the arm body is inferred with NO expected type
② `:arms` infers bottom-up  ->  Vector<Alarm<Op.-Tick>>
③ that binds Outcome's O := Op.-Tick — the narrow type escapes UPWARD
④ the generated fold declares   alarm <- (:wat.service/Alarm :- [:probe.tick2/Op])   (read from
   the expansion — the generated side is CORRECT)
⑤ same-head parametric args are INVARIANT (arc 278 Stone 2)  ->  mismatch
```

```
expected  [… (:wat::service::Alarm :- [:probe::tick2::Op.-Tick]) :-> …]
got       [… (:wat::service::Alarm :- [:probe::tick2::Op])       :-> …]
```

★★ **Before the expander fix, step ② produced a FRESH VAR** — the ctor head was unexpanded, so it
carried no scheme, `infer` returned `:?N`, and nothing constrained `O`. It unified with whatever the
arms needed. **The service type-checked because a value had no type.**

## Where the fix belongs — NOT the checker

`infer_component_against` already checks a value against an expected type, and
`check_compound_against_expected` already does this for vector/map/set literals and for the
`:wat::core::Tuple` CONSTRUCTOR. The checker is fine. It can only direct when someone hands it a
type, and here the **generator** is what dropped it.

`wat/service.wat` is holding the very type it fails to ascribe — its own comment at `:1271` says so:

> *"(the poll' set) is typed with the superset O; **the O flows into `(Outcome :- [S R O])` /
> `(Alarm :- [O])`**."*

So the scrutinee it emits should be ascribed with that op's declared `(Outcome :- [State Reply O])`,
via `:wat::core::ann-form` — the substrate's existing ascription form, which `src/check.rs:15890`
describes as driving the *"ann-form-directed per-position up-cast (`assignable`)"*. With direction
present, the narrow literal meets `Alarm<Op>` through the `Variant <: Enum` edge that already
exists — measured `exit 0`, three ways, above.

★★★ **Option C was right in principle and wrong in location.** I would have briefed a checker
change. The checker needed nothing; the macro needed to stop discarding a type it already had.

## ⛔ Two process failures on the way here, both mine

**An A/B that measured the same binary twice.** `git stash push src/macros/expand.rs` stashed
NOTHING, because that file was already committed — and the rebuild reported `Finished in 0.19s`,
which I read past. The follow-on `git stash pop` then popped a MONTHS-OLD stash from another
session (`f854c4967`), conflicting 17 files. `git reset --hard` cost nothing because everything was
committed. `[[feedback_i_committed_on_a_non_quiescent_tree]]` inverted: committing often is what
made the mistake free.

**A causal claim with no trace.** *"Now that `:arms` is genuinely inferred"* was narration. The
valid A/B — actually deleting the Map/Set arms and rebuilding — came only after the builder asked
for the trigger path:

```
WITHOUT the expander fix   control exit 0
WITH    the expander fix   control exit 1
```
