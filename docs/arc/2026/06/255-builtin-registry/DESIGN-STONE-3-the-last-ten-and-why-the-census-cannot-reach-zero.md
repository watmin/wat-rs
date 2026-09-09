# DESIGN — STONE ③: the last ten, and why the census CANNOT reach zero

After stone ② the gate census stands at **96 files · 10 names**. Every one is measured below, and
they do **not** share a disposition. Three of the ten name verbs that **do not exist**.

## ⛔ THE HEADLINE — the census must NOT be driven to zero

Two of the ten must **stay refused**. A stone that registers all ten to make a number green would
bury the two findings the gate exists to produce. `DESIGN-the-blanket-dies-in-three.md` said this
about the four type names; it is now literally true of the endgame.

## A · MEMBERSHIP ROWS — 6 census names

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

## B · CORPUS FIXES — 2 census names, both PHANTOMS

These are not registration candidates. **Nothing implements them.** The blanket is why they
type-check.

```
:wat::kernel::panic!    grep -rn "kernel::panic" src/  →  EMPTY.  The verb is :wat::kernel::raise!
                        (src/intrinsic/kernel/abort.rs:73)
                        site: wat-scripts/scratch-pad/arc109-type-equal-acceptance.wat:16,
                              inside a live `match` arm

:wat::string::=         registered nowhere. The `:wat::string::*` surface has concat/contains?/
                        empty?/ends-with?/interpolate/join/length/split/starts-with? — and no `=`.
                        `:wat::rete::string::=` ALIASES TO `:wat::core::=` (vocabulary.rs:1114),
                        which is the real verb.
                        sites: wat-scripts/fmt/fixtures/cond-overflow.wat:8,9 — formatter
                              fixtures, never executed, which is why nobody noticed
```

★ This is arc 255's founding sentence collecting its scalps: *"the undefined-func class dies as a
SIDE EFFECT of fixing the real defect."* Two live corpus calls to functions that do not exist.

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
