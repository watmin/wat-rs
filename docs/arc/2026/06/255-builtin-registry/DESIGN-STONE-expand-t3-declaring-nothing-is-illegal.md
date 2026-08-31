# DESIGN — STONE expand-T3: declaring nothing becomes ILLEGAL, and the compiler is the census

> Builder: *"how do we make declaring nothing illegal? we did this trick earlier with totality."*

Exactly that trick. `@ExpandTime` joins `@Purity`, `@Determinism`, `@Totality` and `@Category` as
REQUIRED. Absence becomes `DocError::MissingExpandTime`, and the registration macro refuses to
expand.

```
crates/wat-doc/src/lib.rs:712    `parse`               .unwrap_or(Unreviewed) -> .ok_or(Missing…)?
crates/wat-doc/src/lib.rs:1045   `parse_special_form`  the same
```

```
433  registration sites must answer   (429 #[wat_intrinsic] + 4 #[wat_special_form])
  1  already does                     :wat::core::fresh-symbol — @ExpandTime Legal (T2)
432  do not                           every one becomes a compile error
```

## ★ THE CENSUS IS THE COMPILER — proven at totality's T3, and it is why this route was chosen

Do not count your way to "every site declares." Delete one directive and rebuild:

```
error: #[wat_intrinsic] :wat::i64::/: doc comment is missing a required `@Totality <Variant>`
       directive (known: Total, Partial, Preserving, Unreviewed)
```

It names the offending verb. **A clean build IS the proof**, and no search pattern of mine can be
wrong about it — which mattered at totality's T3, where my predicted site count was wrong and the
rider's measured one was right.

## The one contract decision, pinned: EVERY SITE GETS `Unreviewed`

**No verb is adjudicated by this stone**, even though 202 answers already exist in
`macros/eval.rs`'s allow-list. Those transcribe in T4a, as a separate reviewable act with
attribution — the shape that worked for totality's 27.

★ **Why not seed from the list:** it would make T3 judgement-bearing, and a mis-seeded verb would be
invisible — an entry silently declaring `Legal` that nobody ruled. A mechanical sweep has no
judgement to get wrong. Two passes over the same files is the price, and it is worth paying.

⛔ Expect to read doc blocks for verbs you *know* are on the allow-list. Write `Unreviewed` anyway
and note it. `@ExpandTime` becomes MANDATORY here; it becomes ANSWERED in T4a.

## ⚠ THE SCOPE ERROR THIS STONE MUST NOT REPEAT

Totality's T3 acceptance rows read *"`cargo test -p wat-doc -p wat-macros`"*. **The `tests/` tree
belongs to the `wat` package, not those crates**, so my own criteria could not see three reds that
this exact change produced — two in `tests/reflection/probe_arc255_axes_are_declared_not_derived.rs`
(a shared doc fixture failing `MissingTotality`) and one lint red on new `contains`-style assertions.

**Measured now, before the sweep: `tests/` holds 6 fixtures carrying `@Totality`.** They will need
`@ExpandTime` too. The acceptance rows below run the FULL floor, not two crates.

★ And that probe is named `axes_are_declared_not_derived` — the file whose thesis is *"the axes are
DECLARED"*. When it broke on totality it was because it had stopped declaring the newest axis. **It
will break again here, and the fix is to extend its CLAIM to assert `expand_time`, not merely to
add the directive to its fixture.** A new axis nothing there asserts is a new axis that file's
thesis has quietly stopped covering.

## Method

~432 doc blocks is past hand-editing: a surgical Rust Cargo tool under repo-local `tools/`, deleted
before the commit. Read file → insert one line after the `@Totality` line of each block lacking
`@ExpandTime` → write, every other byte untouched.

**Placement: after `@Totality`, before `@Category`** — so the four property axes read as a block.

★ **Per-file non-ASCII counts verified unchanged.** This repo has silently lost 5,720 non-ASCII
characters to a whole-file round-trip while the suite stayed green; content integrity is a separate
axis from tests-green.

## Out of scope = REJECTED

- **Answering `@ExpandTime` for any verb.** T4a.
- **`is_expand_time_legal`.** It keeps its 202-name hand-list until T4.
- **`:wat::core::fresh-symbol`'s `Legal`.** T2 set it deliberately; it stays and is the control.

## Calibration

Predicted 40–70 min. The compiler names every site, so the sweep is diagnostic-driven rather than
census-driven — which is the entire reason for this route.
