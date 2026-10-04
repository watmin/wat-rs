# AMEND — STOP 255.92: one name, and no text keys

**Drawn 2026-10-04.** **Executor: grok via pulsare, working solo** (it runs the floor). Continues at `e5de81ad7`. Commit
locally on `main`; **do not push**.

The STOPs were right to fire, and `NameMap`/`NameSet`/`NameBTree`, the one translation (`wat_keyword_to_clojure_symbol`
is now `Name::from_keyword`'s `Display`), the tightened macro test and the 52 rendered keys found are accepted. Four
rulings and four pieces of work follow.

## STOP-1: ruled. Two keyword joins of one name are one name

R-a (2026-10-02): *a faithful symbol has one `/`, so `:a::b/c` and `:a::b::c` share one image.* The builder's pair
ruling (2026-10-04): the only identity is `{namespace, name}`, and keyword spellings exist only in the transition. So
`:user::helper/of` and `:user::helper::of` **are** `{user.helper, of}`, and one entry answers both.

Two tests pinned the retired keyword grammar's opposite rule (255.8's "direction control"):
`tests/macros/probe_arc251_8d_macro_member_join_wrong_join.wat.bad` and row 4 of
`probe_arc255_14_namespace_join` (`:wat::spawn::process/runner-count` absent). **Rewrite each to assert today's
fact**, in the same commit, with the history in its doc comment (FM 34: a gate that pinned a rule is rewritten, never
quietly deleted). The `.wat.bad` becomes a `.wat` whose test asserts both spellings reach the **same** entry
(the same function value), and a registry row asserts **one** key for the pair, not two.

## The 328 failures are not "the same collapse" until shown

The SCORE files the `stdio.wat` `PatternMatchFailed` (`:215`, `:310`) under STOP-1 without showing the deciding site.
`stdio.wat:215` matches `wat.kernel.StdOut/WriteResponse.Ok`: a **variant** spelling. A variant's identity and how a
`match` compares a value's variant to an arm's pattern is a mechanism of its own. Find the site (likely a variant
path compared as text on one side and entered as a `Name` on the other) and cure it.

**And build the instrument that would have told us:** every insert into a `NameMap`/`NameSet` whose pair is already
present under a **different spelling** is recorded (pair, both spellings, whether the two values are equal). Run it on
`startup_bare` and on the floor's worlds (a counter or a startup report; not a test that gates). **Equal values: the
collapse is harmless. Different values: a real conflict, a STOP-2 row each.** Report the list.

## STOP-2: ruled. A composite key is structure, never text

The 52 rendered keys (`parametric_extensions`, `subtype_edges`, `subtype_parents`: `(wat.type/Vector :- [:T])`,
`(:wat::capability::Dialable :- [… :T])`) are type **applications**. Under "there are no strings", their key is the
structure: a head `Name` and its arguments (each a `Name`, a type variable, or a nested application), compared and
hashed as data. Build that key type from the `TypeExpr` the registration already holds (not by parsing the rendered
text back), and delete `NameMap`'s `rendered` side map. If a key cannot be built from a `TypeExpr` at its insert site,
name the site.

## The cost rose: 71.9 s → 85.2 s

`get(&str)` calls `Name::enter` on every lookup: a parse, and allocations, per call, on paths 255.90 measured in the
tens of millions. **A text boundary is for where a name enters, not where it is looked up.**
- The hot callers (255.90 §5: `canonical_type_key`, `TypeEnv::get`, `env_key`, the call-head path at
  `src/runtime.rs:949`, `match_arm.rs:110`) take and pass a `Name`. Where a hot site holds a keyword literal, hoist the
  `Name` out of the loop or build it once (a `static` built at first use is a frozen value, not a lock: `OnceLock` is
  the immutable tier).
- **Measure like against like:** the fuzz deftest alone, **six runs each**, on `4d4087f03` (before this stone) and on
  your tree, same machine, nothing else running; state both means. The gate is your tree's mean **≤** the before mean.
  The floor's single in-load number is reported too, and it is not the gate (FM 27).

## Also

- `NameMap` keeps "the inserted spelling" for iteration and diagnostics. A diagnostic prints the `Name`'s `Display`.
  Keep the retained spelling only where something reads it, and name each reader.
- `UseDeclarations::covers` is a **prefix of the stored spelling**: a string compare deciding identity. It compares
  namespaces as data (a declared `rust` namespace covers a `Name` whose namespace starts with those segments).

## Then

The floor, in the foreground, `git status` clean. **Test-name set against `.floor/2026-10-04T04-53-19Z`: MISSING 0.**
The collision list, the cost table (six and six), clippy, ignores 18. A red caused by this amend's own change is
captured verbatim, cured, and followed by a **new** floor; any other new red is a STOP. Append to the SCORE, commit,
**do not push**.
