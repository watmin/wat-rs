# REVIEW — the STOP was right; the diagnosis is not an engine defect

> Weighed against my own reading of the fixture.

## Accepted, and the discipline was exactly right

- **Row 1 proved the no-op before the fix.** `assert_ne!` on the unfixed rewrite went RED with
  *"the rewrite must change the constant"* — so *"it never ran"* is demonstrated, not asserted.
- **Row 4 reported both numbers untuned**, kept the assertion at `Ok(0)`, and refused to accommodate
  the `Err`.
- **STOP-1 was taken correctly**: no cure attempted, no green floor claimed over a red assertion.
- **The keyword fixture answers `Ok(0)`** — discrimination genuinely holds there, and a check that
  had never executed now does.

That is the right shape for an unexpected result and I would rather have this than a green.

## ⛔ But the enum result is a FIXTURE limit, not "the comment's predicted engine defect"

```
(:wat::core::defenum :probe::E :wat::enum::Pure :A :B)
```

**The enum has exactly two faces**, and the two inserted facts are `:probe::E::A` (hit) and
`:probe::E::B` (miss). So **a valid `:probe::E` constant matching NEITHER fact does not exist.**

`:zeta` is not "a constant matching neither" — it is not a `:probe::E` value at all. A bare keyword
in that position resolves as a field reference (which is the rule `FIELD_WINS` exists to pin), there
is no field `:zeta`, and the engine refuses it **at compile time**. That is correct behaviour.

**The asymmetry with the keyword fixture proves the point rather than contradicting it:** there `:v`
is a `keyword`-typed field and `:alpha` is a keyword *constant*, so `:zeta` is an equally valid
keyword that matches nothing → `Ok(0)`. A keyword field accepts any keyword; an enum field accepts
only that enum's faces.

So the comment's warning — *"the operand is being evaluated but not compared"* — is **not** what we
observed. Nothing reached comparison because nothing valid was offered.

## What to change

**Give `:probe::E` a third face and use it.**

```
(:wat::core::defenum :probe::E :wat::enum::Pure :A :B :C)
…
let never = src.replacen("::= :v :probe::E::A", "::= :v :probe::E::C", 1);
```

`:C` is a valid `:probe::E`, matches neither inserted fact, and the row can finally assert what it
was written to assert. Expect `Ok(0)`; if it is anything else, **that** is the engine finding the
comment predicted, and STOP-1 applies for real.

Keep the keyword fixture as it stands — it already passes honestly.

## And correct the SCORE's claim

The SCORE says this is *"the comment's predicted engine defect … sitting behind a check that had
never executed."* It is not. It is a two-variant enum unable to express "matches neither", which the
dead `if` hid for as long as the row existed. **A wrong diagnosis recorded in an arc doc is inherited
by everyone after us** — this arc has spent nine strikes on exactly that failure mode.

## Not asking for

Any engine change. Any sweep of other rewrites — you confirmed there are none left in this file, and
the five siblings already use `assert_ne!`.
