# DESIGN — STONE ③: the last ten, and why the census CANNOT reach zero

After stone ② the gate census stands at **96 files · 10 names**. Every one is measured below, and
they do **not** share a disposition. Three of the ten name verbs that **do not exist**.

## ⛔ THE HEADLINE — the census must NOT be driven to zero

Two of the ten must **stay refused**. A stone that registers all ten to make a number green would
bury the two findings the gate exists to produce. `DESIGN-the-blanket-dies-in-three.md` said this
about the four type names; it is now literally true of the endgame.

## ⛔ ③ DECOMPOSES BY SHAPE, NOT BY COUNT

```
③a  the eval CLUSTER — 10 names, one dispatch arm, one family rationale
③b  :wat::core::stream->pvec — has a scheme AND an arm
③c  :wat::core::i64/to-string — has an arm and NO scheme; needs a real CONTRACT, not membership
```

Three different shapes, so three stones. ③a is the population; ③b and ③c are singletons that
differ from it and from each other.

## A · MEMBERSHIP ROWS — the ③a/③b population

★ **The arc has already ruled what a row means for a verb that carries a scheme.**
`src/check.rs:5823`, in its own words:

> *"A row that DOES carry a `TypeScheme` never reaches this branch at all (it took the `Some(s)`
> arm above) — the scheme is **strictly stronger and stays the one authority** for that row
> (STOP-6: one authority per question)."*

and `Kind::SpecialForm` is excluded from row-arity checking by the same passage. So for these
names a registry row contributes **membership**, and restating the contract would create the second
authority the arc exists to end. This is not the cheap option; it is the ruled one. It extends
①′'s precedent exactly: *a population whose contract is carried elsewhere can still be a member.*

**The eval cluster — register all TEN, not the four the census surfaced.**
`src/runtime.rs:2956` dispatches them in ONE arm, and all ten carry hand-written schemes in one
contiguous block, `src/check.rs:19494–19663`:

```
:wat::eval-ast! (337) · :wat::eval-with-defs! (6) · :wat::eval-step! (1) · :wat::eval::walk (2)
:wat::eval-edn! · :wat::eval-file! · :wat::eval-digest! · :wat::eval-digest-string!
:wat::eval-signed! · :wat::eval-signed-string!                        ← six the corpus never calls
```

⛔ Registering only the four a census surfaced is **disposition-by-count**. The cluster is one
population; it enters as one.

**`:wat::core::stream->pvec` (5)** — carries a scheme (`src/check.rs`), a dispatch arm
(`runtime.rs:2582`) and a `const OP` (`collection/transform.rs:1046`). Membership.

**`:wat::core::i64/to-string` (1)** — ⚠ NOT the same shape. It has a live dispatch arm
(`runtime.rs:2704`) and **no scheme**. It is a genuine contract-less verb and needs a REAL row,
not membership. Named separately so it is not swept in with the six above.

## B · ⛔ NOT PART OF ③ — THEY ARE STONE ④, AND ④ GATES THE BLANKET

⚠ **CORRECTED.** These were first written up as "two-line corpus fixes." **Both premises were
measured and refuted:**

```
:wat::kernel::panic!   raise! expects :wat::core::Error — `(raise! "a string")` is a
                       TypeMismatch. Not a substitution; the call site's SHAPE changes.
:wat::string::=        its file is a FORMATTER FIXTURE consumed by wat-scripts/fmt/run-*.wat
                       and gated by `every_wat_scripts_file_loads`.
```

★★★ And that exposes the real class. `cond-overflow.wat` type-checks **only because of the
blanket** — the same as the `>X` witness in § C, for a different reason. So this is not hygiene:

> **STONE ④ — the corpus's blanket-dependents. THREE files that type-check only because the
> blanket accepts any `:wat::*` head, and that will turn `every_wat_scripts_file_loads` RED the
> moment it dies.** It is a HARD PREREQUISITE of the blanket's death, not a cleanup, and it will
> fail in a way that reads as an unrelated floor red.

```
wat-scripts/scratch-pad/arc109-type-equal-acceptance.wat   :wat::kernel::panic!   phantom
wat-scripts/fmt/fixtures/cond-overflow.wat                 :wat::string::=        phantom
wat-scripts/scratch-pad/probe-f64-comparator-bogus-head.wat :wat::rete::f64::>X   premise expired
```

Three files, three DIFFERENT reasons to change — which is why ④ is its own stone and not a rider
on ③, and why its brief cannot be written until the `Error` shape and the fixture's semantics are
measured.

## C · STAYS REFUSED BY DESIGN — 2 census names

**`:wat::rete::f64::>X` (1)** — a DELIBERATE witness
(`wat-scripts/scratch-pad/probe-f64-comparator-bogus-head.wat`), proving a bogus rete head
type-checks. It must keep failing; that is its job.

> ⚠⚠ **AND ITS CONTAINMENT PREMISE EXPIRES WITH THE BLANKET.** The file's own header:
>
> *"That is why it is safe to keep as an ORDINARY `.wat` under the loader gate:
> `every_wat_scripts_file_loads` only parses + type-checks (`startup_from_source`), it never runs
> `main`, so a body that raises at RUNTIME does not rot the gate."*
>
> That safety rests **entirely** on `--check` not validating `:wat::*` heads. The blanket's death
> is exactly what removes that, so this file stops type-checking and
> `every_wat_scripts_file_loads` goes RED. **It must be re-housed BEFORE the blanket dies** — as a
> fixture whose expected verdict is failure, not as an ordinary loader-gated `.wat`.
> `[[feedback_a_rulings_premise_expires_but_the_ruling_stands]]`

**`:wat::core::Option.Some` (2)** — THE DOT SPELLING
(`wat-scripts/scratch-pad/keyword-accessor-vs-enum-map-ctor.wat`). It is the migration's target
notation and is not live yet. It cannot be registered (the flip has not happened) and it cannot be
"fixed" (the file exists to measure precisely this). It is refused, correctly, until the dot flip —
which the seam's ordering caveat puts AFTER the blanket dies.

## The MECHANISM — per-site declaration, never a hand-list

①′ folded `RETE_OPS` because it is a TABLE. The eval cluster has no table: its names live in a
`match` arm and in a block of `env.register` calls, neither enumerable. **A literal list of ten
names in the registry builder would be a NEW HAND-LIST** — `[[feedback_a_gate_over_two_hand_lists_is_a_hand_list]]`.

`#[wat_special_form(":name")]` on a unit struct is the shape that avoids it: each name declares
itself at its own site and `inventory` collects them — how the 52 rete aliases already enter
(`src/intrinsic/special/rete_alias.rs`). Measured requirements per declaration: prose, `@added`,
`@Category`, `@Purity`, `@Determinism`, `@Totality`, `@ExpandTime`, `@ret`, and ≥1
`@example`/`@example-norun`. `@arg` is OPTIONAL — `:wat::core::let` carries `@syntax` and `@ret`
and no `@arg`, because its arguments are not a fixed list. The eval family is that same shape.

★ The axis candidate, by analogy to `if`/`let` (`src/intrinsic/special/control_flow.rs:26`): the
eval family runs whatever form it is handed, so `Preserving` on Purity / Determinism / Totality is
the defensible reading, and the `!` suffix marks that the evaluand MAY be effectful. **Argued, not
assumed — and if it does not hold for a member, that member is a STOP, not a guess.**

⚠ `@example-norun` is the accepted escape where an input cannot be synthesized inline
(`:wat::kernel::raise!` uses it), but `[[NOTE-a-norun-example-asserts-nothing]]` stands: prefer a
runnable example, and say why when you cannot.

## The arithmetic

```
10 census names  =  6 membership (the eval TEN + stream->pvec)
                 +  1 real row      (i64/to-string — arm, no scheme)
                 +  2 corpus fixes  (panic! → raise! · string::= → core::=)
                 +  2 refused by design  (>X the witness · Option.Some the dot spelling)
```

After A and B the gate census reaches **2 names** and stops there. That is the floor, and it is the
correct floor.

## Out of scope = REJECTED

- **Retiring the ten hand-written eval schemes.** They are the strictly-stronger authority per
  STOP-6; the row does not replace them.
- **The dot flip.** After the blanket, per the seam's measured ordering.
- **The 354 hand-registered schemes in `check.rs`.** The contract-unification campaign is the arc's
  larger body of work, not this stone.
