# SCORE — STONE: a trailing run of pairs

No commit. Floor and clippy left to the orchestrator. Level 2 still open.

## The five shapes, one rule

**`(assoc HashMap-ctor :k x)`** — compound prefix breaks; pair whole on one line:

```
(:wat::hashmap::assoc
  (:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64])
  :k x)
```

**`(get HashMap-ctor :k)`** — odd trailing run is not a run; compound breaks; `:k` own line:

```
(:wat::hashmap::get
  (:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64])
  :k)
```

**Four-pair record** — no regression, values aligned:

```
(:wat::grep::Unreadable
  :file   path
  :reason msg
  :line   n
  :col    c)
```

**defservice / atom positional** — still rides, via leading-atom (the positional is an atom), not a special case:

```
(f :wat-tests::recorder
  :satisfies :wat-tests::Recorder
  :durable   []
  :ephemeral []
  :impls     [])
```

**Type-args then one pair:**

```
(:wat::core::HashMap :- [:wat::core::keyword :wat::core::String]
  :some-kw "some-string")
```

All `IDEMPOTENT=true`. `--check` clean on every new fixture before the floor.

## The 132-column line is gone

`compound-then-kw.wat` before this stone:

```
(:wat::hashmap::get (:wat::hashmap::assoc (:wat::core::HashMap :- [...])
                      :k x) :k)
```

After:

```
(:wat::hashmap::get
  (:wat::hashmap::assoc
    (:wat::core::HashMap :- [:wat::core::keyword :wat::core::i64])
    :k x)
  :k)
```

## What changed in the rule

`is_kwargs` from `src/check.rs:13444` applied to a **suffix**: length ≥ 2, length even, every even-offset element a keyword. Unique smallest start. `:-` / `->` are never keys. Prefix of the run uses ordinary leading-atom (first compound and everything after it in the prefix). The previous "positionals ride" special case is gone.

Pair-width counts only a broken child whose **next** sibling has no Break (a key with a riding value). Prefix compounds do not set the pad.

`grep -c 'col' rules/*` → 0 in every file.

## Row 14 — 614 examples

| | |
|---|---|
| run | **614** |
| changed | **342** |
| lines still over 120 | **0** |
| worst remaining | **104 cols** |

The previous run's single 132-column line **is gone**. Worst remaining, verbatim:

```
(:wat::edn::ForeignRecord/get
  (:wat::core::match (:wat::edn::read-foreign "#some.unknown/Rec {:kind #some.unknown.Kind/Click [42]}")
    ((:wat::edn::ReadForeignOutcome::Value fr)
      fr)
    ((:wat::edn::ReadForeignOutcome::Malformed _)
      (:wat::kernel::assertion-failed! "bad fixture"
        :wat::core::None :wat::core::None)))
  :kind)
```

That is `(get X :k)`: compound on its own line, trailing odd keyword on its own line. Under 120.

## Walls / comments / load

Disagreeing-kind sabotage still raises `fmt: conflicting Breaks for node 11 — block vs align`. Deleted after. `ClaimedUnder` 0. `io.wat` **COMMENTS=28**. `every_wat_scripts_file_loads` **1 passed**. Existing fixtures idempotent; `generic-fn` ret-spec still both tokens one line.

No Rust. `AlignPairs` / `Break` / `Claim` unchanged as records.

---

## ORCHESTRATOR VERDICT — 2026-09-06

**ACCEPTED. No edit.** ★★★ **And the priority target is MET.**

| what | result |
|---|---|
| ★ row 2 — `(assoc … :k x)` | compound prefix breaks; **`:k x` whole on one line** |
| ★★ row 3 — `(get X :k)` | odd trailing run is not a run; `:k` on its own line; **nothing compound rides** |
| ★ rows 4/5/6 | four-pair record aligned · `defservice`'s atom positional rides · type-args + one pair |
| ★★★ row 14 — the 132-column line | **GONE** |
| walls | disagreeing-kind sabotage raises; `ClaimedUnder` 0; `grep -c 'col' rules/*` **0** |
| floor | **5179 run, 5179 passed, 0 FAILED, 18 skipped** · clippy **0** |

### ★★★ ROW 14, RE-RUN INDEPENDENTLY WITH A DIFFERENT EXTRACTION

I did not take the strike's number. I re-extracted every `@example` expression myself — a narrower
predicate than the strike's (only lines whose expression begins `(`), giving **476 forms** where the
strike counted 614 — concatenated them into one file, and formatted it in a single pass:

```
SOURCE      476 example forms   max line 1153 cols    71 lines over 120
FORMATTED   1401 lines          max line  106 cols     0 lines over 120     IDEMPOTENT=true
```

**Two different extractions, the same conclusion: zero over 120.** That is the strongest form of
agreement available here — the numbers differ, the verdict does not.

## ⭐ WHAT THIS MEANS FOR THE ARC

`[[PARKED-the-migration-waits-on-wat-fmt]]` parked arc 255's doc migration with this reason:

> *"The sweep would bake 609 one-line examples (median 67 cols, p90 188, MAX 1515) into the new form
> and force a re-sweep."*

**That reason is now discharged.** The examples format to a max of 106 columns, under the builder's
ruled 120, idempotently. **The blocker that parked the migration is gone**, and the priority the
builder named — *"starting with rust metadata-maps first"* — is unblocked.

⚠ **Not the same as "the migration is ready to run."** What is proven is that the FORMATTER produces
acceptable output for these expressions. The migration also needs the `#wat.doc/Row` printer to emit
through it, and that has not been wired.

## The rule, and why it is one rule

`is_kwargs` from `src/check.rs:13444` applied to a **suffix** rather than the whole argument list —
length ≥ 2, even, every even-offset element a keyword. **The formatter uses the substrate's own
definition of a kwarg call**, so it cannot disagree with the checker or the evaluator about what one
is. Everything before the run obeys the ordinary leading-atom rule; the "positionals ride" special
case is deleted, and `defservice` still works **because its positional is an atom** — which is row
5's point and the proof that the special case was the bug.

## Not disputed

`:-` / `->` are never treated as keys. Pair-width counts only a key whose value rides, so a prefix
compound cannot set the pad. Every new fixture `--check`s clean before the floor (row 12, added
after the last stone lost a floor run to a mistyped fixture). `io.wat` **COMMENTS=28**.
`generic-fn`'s ret-spec still both tokens on one line.

## ⬜ WHAT REMAINS, and none of it blocks the priority path

```
level-2 cross-sibling-call alignment    named since the kwargs stone; still open
the emitter's comment-indent defects    CORPUS-ONLY — no @example can contain a `;;`
`fn` / long-`defn` arg-spec one-per-line  visible in corpus output, not in doc examples
R15 (120) as a lint                     the budget is now MET by construction, not enforced
wat fmt --check joining the floor        the eventual gate
```
