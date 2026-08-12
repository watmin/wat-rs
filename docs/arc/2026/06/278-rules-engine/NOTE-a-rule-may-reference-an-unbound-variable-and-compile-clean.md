# NOTE — a rule whose `:then` names a variable the `:when` never bound COMPILES CLEAN

**Filed 2026-08-12, out of arc 278's `fn-forms`-reads-data-as-code fix. NOT fixed — builder-ruled
"another bug - we'll deal with it in time." Tracked here because it is a TOTALITY gap in the rete
row surface, the same class as #63 and #80, and because it was found by DISPROVING a claim made in
its defence.**

## The defect

`:wat::rete::compile` accepts a rule whose RHS (`:then`) references a pattern variable that the LHS
(`:when`) never binds. No error, no warning, no located diagnostic. The rule compiles.

## The confirmed instance

`wat-scripts/scratch-pad/probe-arc278-who-diagnoses-a-bad-rule.wat` — a control rule and a broken
one, identical but for a single unbound variable:

```clojure
;; CONTROL — `?c` is bound by the `<-` in :when and consumed in :then
(:wat::rete::defrule :usr::ok-rule
  :when [(:usr::Temp (?c <- :c) (:wat::rete::core::i64::> ?c 50))]
  :then [(:usr::Hot :c ?c)])

;; SUBJECT — `?missing` is consumed in :then and NEVER bound in :when
(:wat::rete::defrule :usr::bad-rule
  :when [(:usr::Temp (?c <- :c) (:wat::rete::core::i64::> ?c 50))]
  :then [(:usr::Hot :c ?missing)])
```

Run output, verbatim:

```
"CONTROL rule built"
"CONTROL compiled OK — the well-formed rule passes its own gate"
"BROKEN rule built — now compiling it"
"BROKEN COMPILED WITHOUT RAISING — the DSL did NOT diagnose the unbound variable; the claim is REFUTED"
```

The probe carries a **non-vacuity control by construction**: the two rules differ in exactly one
variable, so "both compiled" cannot be read as the instrument failing to exercise anything.

## How it was found — by disproving the claim made in its defence

This was not hunted. Deleting a guard in `closure_extract` (the walker that raised on symbols inside
a `quote`) required answering *"then who catches a genuinely broken rule?"* The answer offered was
that the DSL owns it — *"our rete solution will run compile and it will raise if compile faults and
the user is given a detailed message on the mistake."* **Measured, and it does not.**

That refutation did not weaken the `closure_extract` fix — it strengthened it. The walker's raise
fired on `?c` (**valid**) and would have fired identically on `?missing` (**invalid**): it cannot
tell them apart, because it does not know what a rule is. It was never a guard for this; it was
noise that happened to be loud. But the objection it was answering turns out to have **no answer at
all**: nothing catches this today.

## ⚠ BOUNDED — what was NOT measured

**COMPILE only.** Whether *firing* the broken rule raises, or silently derives a corrupt fact, is
**unmeasured**. "Silent at compile, loud at fire" and "never diagnosed" are materially different
dispositions and no claim is made between them here. **That measurement is owed before any fix is
designed** — it decides whether this is a missing compile-time gate or a missing runtime one, and a
fix aimed at the wrong tier is wasted work.

## Why it is a class, not a one-off

- **#63** (closed) — *"a `:then` kwargs item may under-supply fields and SILENTLY construct a corrupt
  record."* Same surface, same silence, same tier. This is its sibling: not a missing FIELD but an
  unbound VALUE for a field that is present.
- **#80** (closed) — *"EVERY rete row must be TOTAL — 5 are not, and nothing stops a 6th."* The row
  vocabulary was made total; the BINDING discipline over those rows was not.
- **R41 / R55**, the no-hidden-failures LAW: a wrong program accepted silently is the mask shape the
  LAW annihilated everywhere else. R57 `IGNORANTIAM DELEMVS` is the standing correction — the LAW is
  completed by USE, and this is one more place USE reached that the declaration had not.
- **R29 `RVINA ERVDIT`** — the checker must RUIN a wrong form, located, where it was written. Here
  it ruins nothing.

## Cost, in tiers — the middle one is deliberately unsized

**Tier 0 — the owed measurement. Minutes.** Seed a fact, fire the broken rule, observe. Extend the
existing probe; it already builds both rules. **Do this first**; every tier below depends on which
tier the failure actually belongs to.

**Tier 1 — a compile-time bound-variable check. Small, IF Tier 0 says compile is the right tier.**
`make-rule` already receives `:when` and `:then` as quoted vectors; the bound set is derivable from
the `<-` binders in `:when`, and every `?`-prefixed symbol in `:then` must be a member. One located
error naming the unbound variable and the rule. **Do not implement this before Tier 0** — if the
fire path already raises well, a second gate is redundant machinery.

**Tier 2 — NOT SIZED, on purpose.** Whether the same silence exists for the accumulator fence,
`exists`, negation rows, and the `where` bodies is unknown; nobody has enumerated the binding
positions across the row families. Anyone who sizes this without doing that enumeration is guessing
— an error this arc recorded four separate instances of in one day (R60).

**Tier 3 — structural.** Make an unbound RHS reference **unrepresentable**: the rule constructor
takes an LHS whose binder set is a value, and the RHS is built against it, so "reference a name
nothing bound" has no form. Top of the extirpare ladder; cost depends entirely on Tier 2.

## Reproduction

`wat-scripts/scratch-pad/probe-arc278-who-diagnoses-a-bad-rule.wat` — run it:

```
target/release/wat wat-scripts/scratch-pad/probe-arc278-who-diagnoses-a-bad-rule.wat
```

It lives under `wat-scripts/` legitimately: it **loads and runs clean**, which is the whole finding.
(A probe that must FAIL to load could not live there — the `every_wat_scripts_file_loads` gate would
go red; see `109/NOTE-a-malformed-definition-must-not-vanish.md`, which puts its repro in `/tmp` for
exactly that reason.)

## Cross-references

- `wat/rete.wat:2359-2400` — `make-rule` + `defrule`; `defrule` QUOTES `:when`/`:then` verbatim, so
  the binder information is present and unexamined at compile.
- `docs/arc/2026/06/278-rules-engine/FINDING-the-closure-walker-reads-data-as-code.md` — the fix
  whose defence-objection produced this measurement.
- Task **#63** (closed), task **#80** (closed) — the two siblings in this class.
- `109/NOTE-a-malformed-definition-must-not-vanish.md` — the same signature one layer down: a wrong
  form accepted silently, discovered far from its cause.
