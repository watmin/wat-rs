# SCORE — the type registry holds the BUILTIN types

Brief + Expectations: `BRIEF-STONE-…` / `EXPECTATIONS-STONE-the-type-registry-holds-the-builtin-types.md`.
Ruled E (consumption — the existing door) implemented by C (storage). Scored against my own re-run.

## Mode A. And the one STOP that fired is the one I named in advance.

| # | row | my result |
|---|---|---|
| 1★ | the DOOR answers `Type` for `:wat::core::i64` | PASS, through `registrations` — not the new field |
| 2 | container · opaque · rust-backed | PASS (`Vector`, `kernel::Peer`, `crossbeam_channel::Sender`) |
| 3★★ | `registrations(":user::NoSuchType")` empty | PASS — the only row that can catch a `contains` that says yes to everything |
| 4★ | `get(":wat::core::i64")` is `None` | PASS — the asymmetry is now a test, not a comment |
| 5 | the derived gate over both consts | PASS |
| 6 | `TypeDef`/`Nature` untouched | **verified myself** — no `+` line adds a variant or a `Nature::` arm |
| 7 | THE DOOR untouched | **`git diff -- src/value/symbol_table.rs` is EMPTY.** The waist did not move. |
| 8 | floor | **RED on the first run — see below.** Green at 4866/4866 after the fix |
| 9 | clippy | 0 under `-D warnings` |

`contains` grew by exactly one `||`; `get`'s body is byte-identical (the diff touches only its doc
comment, which now states the membership/structure split at the site). `src/types.rs` is the only
file changed.

## ⛔ THE FLOOR WENT RED — the same wall as this morning, and my brief did not warn about it

```
     Summary [  76.221s] 4866 tests run: 4865 passed, 1 failed, 19 skipped
        FAIL [   0.063s] (  59/4866) wat::lint no_loose_string_assert::tests_carry_no_loose_string_assert

    🔥🔥🔥 LOOSE STRING ASSERTIONS — 2 site(s) assert a value with contains/starts_with/
    ends_with where an exact `assert_eq!` belongs.
    Drive it to ZERO. Offenders:

    src/types.rs:6804
    src/types.rs:6883
```

Both sites are `assert!(env.contains(":wat::core::i64"), …)` and `assert!(!env.contains(":wat::core::Never"), …)`.

★ **These are FALSE POSITIVES, and the lint cannot help it.** `TypeEnv::contains(&str)` is *registry
membership* — exact by construction, the opposite of loose. The lint is a text pass over `.rs` files
with no type information, and its own doc records the boundary it relies on: *"collection membership
(`vec.contains(&item)` — arg is not a string literal) never match[es]"*. That exemption holds only
while the argument is not a string literal. A registry keyed by `String` breaks it: the call is
textually indistinguishable from `some_string.contains("x")`.

**FILED as a finding, not fixed here:** `no_loose_string_assert` has a false-positive class —
*membership on a string-keyed registry, asserted with a literal key.* It will fire on anyone testing
`TypeEnv`, `MacroRegistry`, or `rust_deps`, and the failure text tells them to reach for an `.edn`
golden, which is wrong advice for a `bool`. This is the recurring shape of the whole arc: **a
text-shape test that cannot tell two mechanisms apart** — the same defect as `defsurface`
discriminating on node kind, and the same one 255's own note rules out as the "B3 forgery."

### The disposition — through the door, not around the lint

I did **not** add a `rune:lint(loose-assert)` exemption. The rune's stated purpose is a *legitimately
loose* assertion; ours is not loose at all, so the marker would have told every future reader
something false about the site.

Nor did I distort the test to satisfy a check — `[[feedback_a_guard_drawn_too_tight_makes_the_honest_path_noncompliant]]`
is exactly this trap, and compliance is what usually gets built.

Instead both assertions now go **through THE DOOR**, the way rows 1 and 3 already did and the way
this stone's own ruling says every consumer should:

```rust
let regs = sym.registrations(":wat::core::i64");
assert!(regs.contains(RegistryKind::Type), …);   // enum arg — nothing for a text lint to mistake
```

That is strictly better on the merits, independent of the lint: my own EXPECTATIONS demanded row 1 be
asked through the door and I never applied the same rule to row 4. The lint's false positive is what
made me notice.

After the rewrite: 7/7 `stone_255b` tests pass, `no_loose_string_assert` passes, floor re-run below.

⚠ **And the part that is squarely mine: I hit this identical wall this morning**, on the `:peers`
negative controls, wrote a whole SCORE section about it — and then wrote the next brief without a
word of warning. A lesson learned at 09:00 and not propagated into the 19:00 brief is a lesson that
cost twice.

## ⛔ THE DEFECT IN MY BRIEF — my verification command pointed at 3.4% of the corpus

The brief told the rider to justify every group-3 name with
`grep -rn "<name>" --include=*.wat wat/`. Measured:

```
wat/            52 files      ← what my command searched
wat-scripts/   398
wat-tests/      81
tests/         979            ← 64% of the corpus, excluded by my command
                     total  1527
```

**My instruction selected 3.4% of the `.wat` corpus while the brief's prose said "the corpus".** The
rider hit it on `:wat::holon::Vector` — zero hits under `wat/`, found immediately at
`tests/collection/vector_first_class.wat:19` once re-scoped — read my intent over my command,
broadened, registered it, and flagged the discrepancy.

★ **Had it obeyed literally, STOP-2 would have fired on a name that is perfectly real.** My brief
would have produced a false refusal and I would have scored it as diligence. This is
`[[feedback_state_what_the_instrument_can_see_before_quoting_it]]` in its purest form — the
instrument selects a population, I wrote one population and meant another, and the gap only surfaced
because the executor checked the intent against the result.

## STOP-2 — `:wat::core::Never`, refused, and I verified the refusal

The rider registered 23 of the 24 group-3 names and refused one. I re-ran its evidence:

```
$ grep -rn "wat::core::Never" --include=*.wat .        (minus target/)
wat-scripts/scratch-pad/probe-selectables-homogeneity.wat:14:;;   … `after` now infers `I = :wat::core::Never`,
```

**One occurrence in 1527 files, and it is a `;;` comment.** `Never` exists only Rust-side — a
programmatic `TypeExpr::Path` for an inferred bottom, plus an `is_subtype` rule. It is never written
by a wat author, so registering it would teach the future wall to accept a name no program may use.
Correctly refused, and the rider added `stone_255b_never_is_deliberately_unregistered` asserting
`!contains(":wat::core::Never")` so the omission stays deliberate rather than drifting back in.

EXPECTATIONS predicted this exact name: *"I expect this to be the one that fires, if any does."*
`:wat::core::Value` cleared the same bar with real `.wat` type-position uses; `Never` did not. The
pair is what makes the judgement legible — the rule discriminated rather than rejecting both.

I spot-checked two more citations against the disk: `wat/seq.wat:240` `coll <- :wat::core::List<T>`
and `wat/service.wat:56` `after <- :wat::time::Duration`. Both genuine type positions.

## STOP-1 and STOP-3 — neither fired, and the diligence was real

STOP-3 (a contains-then-`get` caller) had exactly one production caller outside the new code:
`types.rs:3376`, an early-return collision guard on `::Op`/`::Reply` synthesis, never followed by a
`get`. That independently confirms the DESIGN's "zero contains-then-unwrap-get" measurement, which I
had taken by grep and labelled as such.

STOP-1 (the floor moving because something depended on the registry's silence): the rider traced
every `registrations` caller — `closure_extract.rs` reads only the `Macro` facet; the registry census
builds its name set from `TypeEnv::iter()` over `self.types`, which is unchanged, so `builtin_names`
cannot inflate it. Consistent with the green floor.

## What this stone bought

`registrations(":wat::core::i64")` now answers `{Type}` through the door that already existed. The
type-reference wall parked on `arc109-type-refs-parked` can delete `known_builtin_leaf_types()` — the
hand-list that was the whole reason that branch is parked — and ask the registry instead. W1's
capability keys and reflection are served by the same change without knowing it happened.

No new door, no new variant, no new predicate. `src/value/symbol_table.rs` has an empty diff, which is
the narrow-waist claim discharged mechanically rather than argued.

## ⚠ And row 8's EXPECTATION was wrong before the floor even ran

EXPECTATIONS said *"floor UNCHANGED at 4859/4859 … that is the unusual part"*, and made a point of
it. The floor is **4866**. The delta is exactly the 7 unit tests this stone ships, which I counted
from the diff rather than accepting: `stone_255b_row1…row5`, plus
`group3_opaques_are_membership_without_structure` and `never_is_deliberately_unregistered`.
4859 + 7 = 4866, accounted.

★ **"The floor is unchanged" is not a well-formed expectation for any stone that ships tests.** I
conflated *no behaviour change* with *no count change*. The well-formed row is: **no pre-existing
test changes outcome, and the count moves by exactly the number of tests added.** As written, a
correct stone would have failed the row — and on seeing 4866 the temptation is to hunt a regression
that does not exist, which is the opposite of what the count line is for.

## Postscript — I wrote 4866/4866 into this document before the measurement landed

The first draft of this SCORE carried *"floor 4866/4866, 0 FAIL"* in the acceptance table. I had read
a progress line showing `(1676/4866)` — the RUNNING TOTAL of a floor still in flight — and wrote the
verdict from it. The run finished RED.

Nothing was published on it; the number was corrected in the same session by the run itself. But the
mechanism is worth naming, because it is the one this arc keeps paying for: **a partial reading of an
instrument still in motion, written down as a result.** The floor prints its denominator on every
line; only the Summary is an answer. `[[feedback_state_what_the_instrument_can_see_before_quoting_it]]`
