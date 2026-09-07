# BRIEF — the match arm is a BRACKET CLAUSE carrying a MAP PATTERN

> Read `NOTE-match-cond-clause-brackets.md` **in full, including its 2026-09-06 amendment**, which
> retires the flat positional clause and records why its warrant failed twice. The probe is
> committed and RED: `tests/wat_lang/probe_arc109_match_arm_is_a_map_pattern.rs` (4 rows).

## THE WORK

```clojure
;; TODAY                                    ;; TARGET
((:probe::Pair::Two a b) body)              [:probe::Pair::Two {:a a :b b} body]
((:probe::Pair::Empty) body)                [:probe::Pair::Empty {} body]
```

Two changes that are ONE change: the clause becomes a **bracket**, and the variant arm's middle
element becomes a **map pattern**. They are not separable — a flat 3-element arm in a paren clause
(`(Variant {:a a} body)`) is indistinguishable from a call; the bracket is what makes it a clause.

**`match` ONLY.** `cond` is explicitly out (see below).

## ⛔ SCOPE — cond is NOT in this stone

The NOTE records that `cond` **needs a new name** (*"do not name the bracket-claused conditional
form `cond`"* — a Clojure reader expects flat `test expr` pairs), that **intueri is OWED** for that
name, and that whether `match` and the test-dispatch form unify is **OPEN, decide at draw time**.
None of that is ruled. Builder, 2026-09-06: *"we are not working on cond right now… we're doing
match now."* Leave `cond`'s 62 forms in 37 files untouched.

## THE SURFACE — measured 2026-09-06

```
2333  match forms in .wat        (H-2's entire migration was 539 sites)
4310  variant arms
 653  files
 374  unit-variant arms (bare-keyword today) -> `{}`
   7  wildcard `(_ body)` arms
```

**This is a wat-fix codemod (R21), not hand edits.** A form-tree rewrite at this scale is exactly
what `wat/fix.wat` exists for; `wat-scripts/fixes/` holds the recorded migrations to copy.

## READ IN ORDER

```
NOTE-match-cond-clause-brackets.md          the grammar assertion + BOTH amendments. THE SPEC.
src/runtime.rs:8420  eval_match             the arm reader. Reaches EDN ZERO times — it binds from
                                            the runtime Value, so this stone depends on arc 296 G′
                                            (EnumValue.names carried), NOT on H-2's wire.
src/runtime.rs       eval_match_tail        the tail twin; both move together
src/runtime.rs:4265  the `{:keys …}` / `{var :field}` destructure (arc 257.2) — READ IT AND DO NOT
                                            ROUTE THROUGH IT. That is `let`'s BINDER-first order
                                            (`{a :x}`); a match arm is a PATTERN and takes
                                            core.match's KEY-first order (`{:x x}`). Same
                                            delimiter, different operation. Measured: `{:x a}` in a
                                            `let` is refused today, and that is correct.
wat/fix.wat + wat-scripts/fixes/*.wat       the codemod framework and its recorded migrations
```

## THE GRAMMAR ASSERTION (unchanged from the NOTE)

An arm is exactly one of — and the **head is the context-free discriminator**:

```
[_ body]                      wildcard        2 elements
[<bare-symbol> body]          binding         2 elements
[<Variant> <map-pattern> body] variant        3 elements, namespaced head
```

Only the middle element's shape changes (destructure vec → map pattern). Nothing else moves.

## STOP TRIGGERS

- **STOP-1 — you routed the map pattern through `let`'s destructure.** Binder-first (`{a :x}`) is
  `let`; key-first (`{:x x}`) is a pattern. Sharing the reader imports the wrong order, and the
  first draft of this very ruling made that exact mistake by reaching for the `let` relative.
- **STOP-2 — binding by position survives.** Row 2 of the probe is the whole stone: keys written in
  reverse order must still bind by name. If it passes only because the fixture's order matches the
  declaration, the row has been defeated — check that `{:b b :a a}` yields `-1`, not `+1`.
- **STOP-3 — both grammars are accepted at the end.** Row 4 requires the retired `(pattern body)`
  clause to be REFUSED. An interim that takes both leaves the corpus silently in two grammars and
  nothing proves the migration finished. If a dual-read is genuinely needed to land 653 files,
  STOP and report — that is a builder decision, not a strike decision.
- **STOP-4 — a `.wat` arm the codemod cannot rewrite.** Report the shape (R21); do not hand-edit.
- **STOP-5 — `cond` moves.** Out of scope, and its NAME is unruled.

## PRIOR ART

`SCORE-296-H2b` — a 539-site corpus migration by codemod with layout preserved, including the
lesson that a rewriter must never invent a payload key it cannot read from the declaration.
