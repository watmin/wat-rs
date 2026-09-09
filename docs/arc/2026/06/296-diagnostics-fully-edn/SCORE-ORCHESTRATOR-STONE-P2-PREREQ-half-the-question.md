# SCORE (orchestrator) — STONE P-2 PREREQ: half the question, and the half is right

**Independent re-run.** Floor and clippy central.

```
FLOOR EXIT=0    Summary [ 192.179s] 5260 tests run: 5260 passed, 18 skipped
CLIPPY          the same 5 pre-existing dead_code items. No new ones.
```

## ★★★ THE MEASUREMENT THAT REFRAMES THE STONE — three positions, three answers

Same name, same program, **no user `use!`**:

```
resolve   (call head)     :rust::sqlite::Connection::open   REFUSED
                          ":rust::* reference not covered by any (:wat::core::use! ...) declaration"
is-type?  (this stone)    :rust::sqlite::Connection         false      ← AGREES with resolve
the WALL  (annotation)    [c <- :rust::sqlite::Connection]  ACCEPTED   ← the OUTLIER
```

**`is-type?` and `resolve` now agree. The annotation wall is the one out of step** — and it is out
of step because RELAND-1 had to fold *stdlib* `use!` into `use_decls` to kill the 838-file scream.
The wall consults a merged set that `resolve` has never had.

So EXPECTATIONS row 2 was right, and the collapse was impossible for a reason neither the DESIGN nor
the rider could have avoided: `:rust::sqlite::Connection` is simultaneously the subject name AND a
stdlib `use!`, and the rider verified no `:rust::*` in wat-rs defaults is `use!`-able without
already being `use!`d by the stdlib. **There was no other name the isolation could have used.**

## The rider: correct at every step

- **STOP-3 fired and it restored the arm.** Verbatim red captured, `covers` put back,
  `grep -c use_decls` reported honestly as `3`, not `0`.
- **STOP-1 held and the fixture was NOT adjusted** — the first sketch (seed all of `use_decls`)
  flipped `no_use_is_not_a_type` to `true`; it narrowed the seed to user residue instead of moving
  the bar. That is the exact discipline the two-fixture isolation exists to enforce.
- `register_use_declared_leaf` is `pub(crate)`, idempotent by early return, membership-only.
  `register_builtin_leaf` and both its `debug_assert!`s untouched. STOP-2 held. STOP-4 held.

## ⛔ MY BRIEF, TWICE

**① I pinned the WORDING and not the QUESTION.** The contract read *"seeded from THIS PROGRAM'S
`use!` declarations."* I never decided whether the **stdlib** is part of "this program." `resolve`
says no; the wall says yes. The rider named it exactly:

> *"The DESIGN's 'Where' cites the residue block and does not mention the `stdlib_post_types`
> collection ten lines above."*

`[[feedback_an_acceptance_row_is_a_pin_unless_it_derives_its_bar]]`

**② I briefed ONE of the two stores my own NOTE had already named.**
`NOTE-is-type-shares-the-blindness-the-P1-floor-exposed.md` lists `UseDeclarations` **and**
`subtype_edges`. The DESIGN and BRIEF I wrote an hour later addressed only the first. Measured:

```
is-type? :wat::spawn::Spawned   ->  false        and the WALL accepts it
```

A lesson found, written down, and dropped between two documents I authored the same afternoon.
`[[feedback_a_lesson_learned_and_then_dropped]]`

## The floor's one red, and why no exemption rune was used

```
🔥 LOOSE STRING ASSERTIONS — 2 site(s)      src/types.rs:6831, :6842
```

Both were `assert!(env.contains("<literal>"))` — **`TypeEnv::contains`, a set-membership predicate
returning `bool`.** The lint's stated rationale ("passes on reordered fields, malformed maps,
appended garbage") cannot apply: nothing is being partially matched.

★ **The lint's discriminator is "is the argument a string literal."** Its own header says collection
membership never matches *"— arg is not a string literal"*. That separates `vec.contains(&item)`
correctly and fails for a collection whose ELEMENTS are strings. Proof it is the literal and not the
call — same file, same method, opposite verdicts:

```
src/types.rs:6842   assert!(env.contains(":wat::core::i64"));        FLAGGED
src/types.rs:6901   assert!(env.contains(name), "{name:?} …");       passes
```

⛔ **I did not reach for `rune:lint(loose-assert)`.** That rune records *"this is legitimately
loose"*, and these assertions are not loose at all — the reason would not earn its standing, which
is precisely what `excusare` exists to catch. Instead the seeded name is now **bound once**, which is
better code on its own terms: the test's whole point is that the name REGISTERED and the name
QUERIED are the same name, and two separate literals cannot guarantee it. A typo between the two
`:wat::core::i64` literals would have made that half of the test vacuous.
`[[feedback_a_negative_control_that_can_be_kept_must_be_kept]]`

## Rows

| # | expected | actual |
|---|---|---|
| 1 | subject `true` | ✓ |
| 2 | ⛔ over-reach detector stays `false` | ✓ held — and the first sketch flipped it |
| 3 | hand-list control `true` | ✓ |
| 4 | phantom `false` | ✓ |
| 5 | `test(p2prereq)` 4 passed, 0 skipped | ✓ |
| 6 | `test(p1_annotation)` 10 passed, 0 skipped | ✓ |
| 7 | P-1 phantom refused, names the path | ✓ |
| 8 | the collapse | ⛔ **STOP-3.** `use_decls` count 3, not 0 — reported, not hidden |
| 9 | `get` stays None | ✓ own test; membership without structure, idempotent both ways |
| 10 | hand-list untouched | ✓ crossbeam rows intact |
| — | **FLOOR** | ✓ 5260/5260 after the loose-assert fix |

## Disposition

**LANDED.** The stone did what it could do correctly, and its failure to collapse is a measurement,
not a miss: it proved the two stores are NOT equivalent and named exactly why.

## ⬜ The follow-on — two halves of ONE thing

Both are *"one question, one answer"*; neither belongs to this stone.

```
the wall's SCOPE-BLINDNESS   coverage is per-DECLARING-SCOPE, not per-program. A stdlib annotation
                             is covered by stdlib use!; a user annotation by user use!. That is
                             what resolve already does by walking user residue only. The rider's
                             split of `user_use` from the merged set is the material this needs.
store 4 in the verb          `is-type?` must ask `is_subtype_parent` too — a one-liner, no plumbing.
                             ⚠ It carries a real question: `derive` never validates its MARKER
                             (types.rs:3410-3421), so store 4 admits any name anyone derived from.
                             One-question-one-answer still says the verb must agree with the wall;
                             the unvalidated-marker door is `derive`'s defect and closing it there
                             fixes both consumers at once.
```
