# DESIGN — STONE ④: the corpus's blanket-dependents

**The last thing standing between the tree and a dead `:wat::*` whitelist.** FOUR files under
`wat-scripts/` type-check ONLY because the blanket accepts any `:wat::*` head. The loader gate
(`tests/lint/wat_scripts_fixes_load.rs:36`) runs `collect_wat(Path::new("wat-scripts"))` and
requires `startup_from_source` to succeed for EVERY `.wat` beneath it — so the moment the blanket
dies, all four turn `every_wat_scripts_file_loads_on_the_current_runtime` RED, in a way that reads
as an unrelated floor failure.

**This is a hard prerequisite, not hygiene.**

## The set, derived as a DIFFERENTIAL — not from the raw census

⚠ The gate census reports **40 refusing files**, and **36 of them are baseline failures** that
already fail `--check` on clean main (35 `wat/*.wat` stdlib files, which are not standalone
programs, plus `wat-tests/core/unknown-call-head-panics.wat`, which fails by design). Reading 40 as
the worklist would be the same instrument error that cost this campaign a whole NOTE earlier today.

```
fail under the gate            40
fail on clean main             36
TRUE blanket-dependents         4      ← pass today, break the moment the blanket dies
```

```
wat-scripts/scratch-pad/arc109-type-equal-acceptance.wat      :wat::kernel::panic!      phantom
wat-scripts/fmt/fixtures/cond-overflow.wat                    :wat::string::=           phantom
wat-scripts/scratch-pad/probe-f64-comparator-bogus-head.wat   :wat::rete::f64::>X       witness
wat-scripts/scratch-pad/keyword-accessor-vs-enum-map-ctor.wat :wat::core::Option.Some   the flip
```

## ① `wat-scripts/scratch-pad/arc109-type-equal-acceptance.wat:16` — a PHANTOM

```
(:wat::kernel::panic! "form-of: malformed source")
```

`grep -rn "kernel::panic" src/` → **EMPTY**. Nothing implements it. The verb is
`:wat::kernel::raise!` (`src/intrinsic/kernel/abort.rs:73`).

⚠ **`raise!` is NOT a drop-in** — measured: it expects `:wat::core::Error`, so
`(:wat::kernel::raise! "a string")` is a `TypeMismatch`.

★ **`wat/deporder.wat:180` is the precedent, and it is the IDENTICAL situation** — a
`ReadOutcome::Malformed` arm in a form-reading fn:

```
[:wat::core::ReadOutcome::Malformed {:cause __cause}
  (:wat::kernel::assertion-failed! :message (:wat::core::Error/message __cause))]
```

**Measured on this file: the substitution type-checks, `--check` EXIT 0.** And it is strictly
better than what is there — the site currently BINDS `__cause` and DISCARDS it, hardcoding a
string, so the fix also surfaces the real parse error.

⚠ **ORTHOGONAL, PRE-EXISTING, OUT OF SCOPE — named so it is not mistaken for this stone's doing:**
the file already fails at RUNTIME (exit 1) at line 24, on arc 109's angle-bracket wall
(`keyword-node ":wat::kernel::Peer<A,B>"`). Measured BEFORE any edit. The gate only parses and
type-checks, so this does not affect it. Do not fix it here; do not let it read as a regression.

## ② `wat-scripts/fmt/fixtures/cond-overflow.wat:8,9` — a PHANTOM

```
:wat::string::=      registered NOWHERE.
```

The `:wat::string::*` surface is concat/contains?/empty?/ends-with?/interpolate/join/length/
split/starts-with? — and no `=`. `:wat::rete::string::=` **aliases to `:wat::core::=`**
(`src/rete/vocabulary.rs:1114`), which is the real verb. Its only sites are formatter fixtures,
never executed — which is why nobody noticed.

⚠⚠ **THE HAZARD, MEASURED AND CLEARED.** The fixture's own header is
*"Nested cond whose ALIGNED width exceeds 120"* — its entire purpose is a width threshold. The
names differ by **2 characters** (`:wat::string::=` 15, `:wat::core::=` 13), so a shorter head
could drop the aligned width under 120, make the formatter ALIGN, and leave a fixture that
silently tests nothing.

**Measured by formatting both versions through `wat-scripts/fmt/run-all.wat`: byte-identical
structure, both stay UNALIGNED.** The property survives. Safe.

★ And nothing would have caught it if it had not: the fixture has NO golden and no automated
consumer — `grep -rn cond-overflow wat-scripts/fmt/` finds only itself. It is invoked by hand.
The measurement is the only guard there was.

## ③ `wat-scripts/scratch-pad/probe-f64-comparator-bogus-head.wat` — the WITNESS, and its premise

`:wat::rete::f64::>X` never existed. This file is a DELIBERATE witness for arc 278's EXPECTATIONS
row 5, and it must keep failing — that is its job.

> ⚠ **ITS CONTAINMENT PREMISE IS THE BLANKET.** Its own header:
> *"That is why it is safe to keep as an ORDINARY `.wat` under the loader gate:
> `every_wat_scripts_file_loads` only parses + type-checks … so a body that raises at RUNTIME does
> not rot the gate."*
> The blanket's death is exactly what removes that.
> `[[feedback_a_rulings_premise_expires_but_the_ruling_stands]]`

**Disposition: MOVE it out of `wat-scripts/`, and give it the assertion it never had.** Today
NOTHING asserts the property this file exists to demonstrate; it merely sits on disk exhibiting it.
As a `tests/` fixture with a row it becomes a real witness:

```
--check   exit 0     ← the defect it witnesses (a bogus :wat::* head type-checks)
run       raises UnknownFunction naming :wat::rete::f64::>X
```

★★ **And that row is a RATCHET aimed at the blanket.** When the blanket dies, `--check` starts
exiting 1 and the row goes RED — at exactly the moment it should, in a test whose name says why.
The blanket's own stone then updates it, and that update IS the blanket's proof. A file that
silently changed meaning becomes a gate that announces the change.

## ④ `wat-scripts/scratch-pad/keyword-accessor-vs-enum-map-ctor.wat` — THE BLANKET'S OWN MECHANISM

Measured as a dependent, not assumed. And this file is not a fixture that happens to use a bad
name — **it is the recorded explanation of why the blanket must die before the dot flip.** Its own
comment:

> *"It is NOT a mis-built variant. `(K {map})` is the shape of BOTH the enum map ctor and the
> keyword-as-accessor fall-through. When K does not resolve to a verb, the accessor treats K as a
> KEY, misses, and returns None — a total lookup, never an error. **The `:wat::*` blanket is what
> lets a `:wat::core::` head reach that fall-through; a `:usr::` head is refused by resolve
> first.**"*

That is the evidence behind the seam's ordering caveat. When the blanket dies,
`(:wat::core::Option.Some {…})` stops reaching the accessor and is refused at resolve — which is
exactly the outcome the ordering argument predicts.

⚠ Only the two `:wat::core::Option.Some` lines change. The file's bare-keyword lines
(`(:anything-at-all {…})`, `(:value {…})`) need no blanket — the file says so itself — and must
stay legal.

**Disposition: MOVE and ASSERT, same as ③.** Its five printlns all succeed today; two of them must
be refused after. A row pinning today's behaviour becomes the ratchet that fires when the blanket
lands, and the blanket's stone updates it as its own proof.

## The arithmetic

```
gate census      40 refusing  −  36 baseline  =  4 true dependents
after ④           0 blanket-dependents  → the whitelist can die
of the four       2 are phantom calls FIXED · 2 are witnesses MOVED and ASSERTED
```

## Out of scope = REJECTED

- **Deleting the blanket.** ④ unblocks it; the deletion is its own stone, and it owns updating the
  witness row above.
- **The angle-bracket runtime failure** in ①'s file. Pre-existing, measured, orthogonal.
- **The dot flip.** After the blanket, per the seam's measured ordering — which ④'s file ④ below
  is the evidence for.
