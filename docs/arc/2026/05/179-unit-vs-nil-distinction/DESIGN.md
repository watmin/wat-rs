# Arc 179 — `nil` is the unit value; `()` is not a value

**Status:** DESIGNED 2026-07-28. Supersedes the 2026-05-12 stub
(*"TBD — user holds the design in their head; this stub is the persistence handle."*).
The design was held for 2.5 months; it is now written down.

## Motivation (the builder, 2026-05-12 — unchanged and still exact)

> *"i want to make () != :wat::core::nil ... they both represent Rust's Unit... but... i
> saw a bunch of new additions who declared a ret val of :wat::core::nil and then their
> final body statement was ()"*

Re-raised 2026-07-28 as *"we need a mass rewrite to remove this heresy"* — then, on
seeing that a rewrite alone is a convention that regrows: *"how about.. nil != () ...."*

It recurs because nothing prevents it. It recurred **this session**, by the apparatus's
own hand — `()`-as-nil copied out of an old fixture into two new files, caught by the
builder, not by the substrate.

## THE RULING

**`nil` is the sole unit value. `()` ceases to be a value expression.**

`()` survives ONLY as **syntax** — the empty parameter list — where it is not a value at
all:

```clojure
:wat::core::Fn()->wat::core::Record              ; wat/spawn.wat:51,86 — empty PARAM LIST
(:wat::core::lambda (() -> :wat::core::i64) …)   ; tests/function/fn_rename_multi_lambda.wat:3
```

After this arc, *"nil != ()"* holds in the strongest available sense — not "two different
values", but **`()` is not a value, so it cannot be one.**

### Why NOT "two distinct value types" (the stub's first sketch — REJECTED)

The stub proposed `()` become *"a distinct empty-shape (empty tuple? empty collection
literal?)"*. Rejected: two ways to say *nothing* is the synonym anti-pattern, and it is
the same reason `:()` was already retired from TYPE position (arc 109 slice 1d →
`BareLegacyUnitType`). Minting a second no-value would re-commit at the value layer the
exact error we corrected at the type layer.

Nothing needs a 0-tuple *value*. `nil` is the unit value; `[]` is the empty vector; and
EDN already encodes exactly those two under the uniform rule — record → `{map}`, enum
variant → `[vec]`, `nil` → the unit value only. `()` has no slot in that model.

## Grounded — the fusion, at two layers

**Type layer** — `src/types.rs:820-832`:

```
typealias :wat::core::nil = :()
```

→ `TypeExpr::Tuple(vec![])` (`src/check.rs:2300`, `:2370` — declarations return it).
`:()` in type position **already errors**: `src/check/error.rs:455` emits the located
`BareLegacyUnitType` remedy steering to `:wat::core::nil`. **That half is already a wall
and this arc does not touch it.**

**Value layer** — the unfixed half:

- `nil` → `WatAST::NilLit(span)` (`crates/wat-reader/src/ast.rs:97`; arc 244 canonicalized
  it from `Symbol("nil")` — `src/closure_extract.rs:1774`) → `Value::Unit`
  (`src/runtime.rs:4298`).
- `()` → an empty List literal (`crates/wat-reader/src/parser.rs:286`: *"distinct from
  `()`, which is Unit / empty List"*) → **also** `Value::Unit`.

Two spellings, one value, nothing for the checker to object to. That is the whole defect.

## The strike

**The corrected law enumerates its own violators** (arc 278 R52 `QVOD LEX ACCENDIT`). Do
NOT hand-build a worklist by grep — this arc's own 2026-07-28 survey proved why: a text
sweep cannot distinguish a value `()` from `Fn()`'s empty parameter list, and
hand-classification left three stdlib sites and six subject tests one careless codemod
away from corruption.

1. **Make an empty list in VALUE position a located check error**, with a remedy naming
   `nil` — same shape as the existing type-position `BareLegacyUnitType` message.
   Empty-parameter-list syntax is unaffected *because the checker knows the position*.
   That distinction is precisely what a grep cannot make and what the checker gets free.
2. **Build. The screamers are the worklist.** Fix each to `nil`.
3. **Weigh the floor** by your OWN `cargo nextest run --release` re-run (Summary line,
   ANSI-stripped, never a piped exit).
4. If the churn is wide enough to want a tool, the migration is a recorded
   `wat-scripts/fixes/*.wat` wat-fix codemod driven by the CHECKER's site list — never a
   text glob, never python/sed. Dry-run on a `/tmp` copy, `diff`, prove idempotency.

### Sites known in advance (2026-07-28 survey — indicative, NOT the worklist)

19 non-comment value-position sites across 14 files: `wat-tests/test.wat`,
`wat-tests/core/{result-expect,struct-to-form,core-arithmetic,record-def,option-expect}.wat`,
`tests/cli/{wat_cli__check_good,wat_cli__programs_are_atoms,wat_cli__sigterm_polling_loop,
wat_cli__wrong_arg_type_main,wat_cli__presence_proof,synthetic_battery__alpha,
synthetic_battery__beta}.wat`, `wat-scripts/scratch-pad/probe-repl-durable-forms.wat`.

**Expect the checker to find sites this list does not have.** Inline wat inside Rust test
strings is invisible to any `.wat` sweep — the class-4 blind spot (arc 278 24d) — and only
a build/run surfaces it.

## SUBJECT TESTS — these assert the OLD behaviour; re-point, do not "fix"

Their subject IS `()`-as-nil. A codemod that rewrites them destroys the coverage:

- `tests/wat_lang/wat_arc153_nil_rename.wat:5,8` — functions named `probe-nil-paren` and
  `nil-form-paren`. **Arc 153's own regression gate for this exact form.** Under this arc
  they must INVERT: assert that `()` in value position is now REJECTED.
- `tests/resolve/probe_arc251_keyword_to_type_form.wat:16` — `":()"` inside a *string*,
  testing keyword→type-form conversion. Type layer; untouched by this arc. **Leave.**
- `tests/macros/probe_arc249_threading_witness_t{f,l}_empty.wat:2` — ground whether their
  `()` is a value or an empty threading target BEFORE touching either.
- `wat-tests/edn/render.wat:31` — `(:wat::edn::write ())` asserts how `()` renders as EDN.
  Under this arc there is no `()` value to render. Re-point or retire.
- `tests/types/structs_builtin_redeclare.wat.bad:4` — a NEGATIVE fixture. **Leave.**

## Out of scope — affirmatively cut, not deferred

- **The type-position spelling.** `:()` → `:wat::core::nil` is arc 109 slice 1d + arc 153,
  already walled and already emitting a located remedy. This arc is the VALUE layer only.
- **`wat-scripts/*.wat.disabled`** — 61 `:()` and 21 `()` sites in disabled scripts that
  nothing loads. A graveyard; it should be DELETED rather than migrated, as its own cut.
- **`docs/arc/**` historical `.wat`** — inert (nothing loads it; `every_wat_scripts_file_loads`
  gates `wat-scripts/` only) and inscribed. What is inscribed is inscribed.
- **Renaming `:wat::core::nil`**, and revisiting `nil` ≠ `None` ≠ `false` ≠ empty-list —
  settled in arc 153.

## Cross-references

- **arc 109 slice 1d** — minted the unit type; retired `:()` at type position.
- **arc 153** — renamed unit → nil (the user-facing keyword swap); settled `nil` ≠ `None`.
- **arc 244** — canonicalized bare `nil` to `NilLit` (not `Symbol("nil")`); the literal
  this arc makes sole.
- **arc 165** — tuple PascalCase rename; informs the "empty Tuple" framing this arc rejects.
- **arc 278 R52 `QVOD LEX ACCENDIT`** — the corrected law lights every existing violator;
  the method this strike uses instead of a grep.
- **arc 278 24t** — *"GROUND EACH CASE INDIVIDUALLY BEFORE THE VERDICT"*; the discipline
  that produced the subject-test list above.
