# AMEND-3 — STONE 255.89: one image, one identity

**Drawn 2026-10-04.** **Executor: grok via pulsare, working solo** (it runs the floor). Continues at `edc0804be`. Commit
locally on `main`; **do not push**.

## Accepted from AMEND-2

One door in `wat-reader` (three copies folded into one, the slash-rule unit test); rete heads, the alpha key
(`CondKey`, data rather than text), purity guards, vocabulary, eval-insert, stratify, the expression lowerer, `matches?`
in the checker **and** the runtime, and nested variants all decide by identity, each proved by a both-spellings probe
that asserts a value. Floor `.floor/2026-10-04T01-11-11Z` (my read of its Summary: 6431 run, 6277 passed, 154 failed):
MISSING 0, NEW_FAIL 0, CURED 159. Clippy clean, ignores 18.

**One correction.** The SCORE says `wat-scripts/perf/grid/where-inline-computed.wat` is `Unrecognized` "for both
spellings of that head". The census says that file was rc 0 before the conversion, and its pre-image (`9d85da5a5`,
line 147) is `(:wat::rete::core::cond ((:wat::rete::i64::> :k 100) true) (:else false))` **inside a fact form**, so the
keyword spelling was recognized somewhere. That site decides by spelling. Find it (it is not `classify_rete_clause`, which
sees the `:when` item, not the inline constraint).

## The work: the identity families, in this order

### 1. Rete's last two raw reads, and the inline constraint above

`wat.rete/negated-types-under` (`wat/rete/oracle/stratify.wat:114`) and `wat.rete/rule-negates-in` (`:151`) go through
`wat.rete/head-identity`. The two stratify-number tests are the witnesses. Fix the inline-cond site the same way.
Both-spellings probes, as before.

### 2. The member join: R-a says one image, so one identity

R-a (2026-10-02): *a `/` join means member of a type; a faithful symbol has one `/`, so `:a::b/c` and `:a::b::c` share one
image.* The converted corpus now holds that image, and the floor shows the two keyword spellings still keyed apart:
`service-cache-lru` (`:wat::cache::Cache::GetResult` against `:wat::cache::Cache/GetResult`), the defservice dispatch
(`:my::Counter::Op` against `:my::Counter/Op`), the accessor and foreign-record purity tests (`Log/level`,
`ForeignRecord/get`), `:u::Demo::Has::has`, and `wat.core.i64/to-string` canonicalizing to the retired
`:wat::core::i64::to-string` while the keyword `:wat::core::i64/to-string` it came from is live.

**Measure first, then cure, and STOP-2 if the data says the ruling cannot hold:**
- List every declared name the registry holds under a `/` member join (`T/m`) and, for each, whether a `::` twin
  (`T::m`) is also declared. A pair where **both are declared as different things** is a STOP-2 row (two declarations
  one image claims). Report the count; if it is 0, R-a holds and the cure is one key.
- **The cure, if R-a holds:** the keyword member join and its `::` twin are **one identity** at the door, so a name
  declared in either keyword spelling, or written as the symbol image, resolves to the same entry. Say which key the
  registry holds and why, with the 255.86 member rekey (`rekey_type_member_functions`) in view. The early return in
  `canonical_identity` that keeps `:wat::cache::Cache/GetResult` distinct from `::` is the line this decides. Whatever you
  change there, the `wat-reader` slash-rule unit test changes with it, row by row, in this commit.
- **The retired-name collision:** if `:wat::core::i64/to-string` is live and its symbol image lands on a **retired**
  path, the retirement table and the member key disagree about one identity. Measure which retired rows collide with a
  live member (`:wat::core::<prim>/<m>` against a retired `:wat::core::<prim>::<m>`). Each collision is a STOP-2 row
  unless the G1 remedy (`wat.<prim>/<m>`, e.g. `:wat::i64::to-string`) is already the live name, in which case the
  corpus spelling is retired and the cure is the recorded G1 codemod (`wat-scripts/fixes/one-name-per-operation.wat`)
  run on those files, as its own commit, never a hand-edit. Say which.

### 3. The 28 unresolved references

`src/resolve/normalize.rs:991` is where the resolver gives up, not the cause
(`[[feedback_an_error_names_where_it_gave_up_not_what_is_missing]]`). Group the 28 by **what was declared and how**
(a `def` name such as `:t::pi` or `:my::my-answer`, a member join, a quasiquote marker such as `:foo::bar`, a
macro-built name), with the declaring site for each group. The member-join rows go to item 2; cure the rest by
mechanism.

### 4. The enum-variant controls

`a_real_enum_variant_in_a_rete_constraint_matches` prints 0 against 1, and `legitimate_keyword_constants_are_still_keywords`
1 against 3. The wall from `e969cf456` must refuse a nonexistent variant **and admit a real one**, in both spellings. A
real variant matching nothing is a wall that over-refuses. Find the deciding site.

## Not in this amend

The goldens and test-text kinds (the 56 EDN mismatches, the verbatim-`::` printer, the head-spelling fixture, the
field-span caret, the Law-A diagnostic naming `:wat::core::=`, the comm purity golden), the acronym LAW, sift, peers, and
the six native/oracle diffs whose line is not isolated. They are the next amend, after this floor shows what remains.

## Then

The floor, in the foreground, `git status` clean. Report the name set against `.floor/2026-10-03T13-09-11Z` (MISSING 0)
and the fail set against `.floor/2026-10-04T01-11-11Z` (NEW_FAIL 0, CURED n). A red caused by this amend's own change is
captured verbatim, cured, and followed by a **new** floor; any other new red is a STOP. Clippy, ignores 18. Append to the
SCORE, commit, **do not push**.
