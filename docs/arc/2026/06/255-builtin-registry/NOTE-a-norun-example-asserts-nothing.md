# NOTE — a `@example-norun` expected value asserts NOTHING, and the contract mandates 118 of them

**Filed 2026-08-23**, at the builder's request, from a line spotted in passing:

```
/// @example-norun (:wat::io::IOReader/read-line reader) #=> (Some "hello")
```

> *"we need to validate ret types in no-run are logical?.. idk... this will be
> `(wat.core/Option.Some "hello")` in our new syntax... not something to fix now"*

**Filed in arc 255** because 255 built `crates/wat-doc` and owns the doc contract — the gate below is
labelled `Arc 255.1b-firm` in its own comment. Arc 141 defined the directive vocabulary; 255 made it
enforceable. **A reminder, not a ruling.**

## What is actually true — measured, not reasoned

The builder's guess was closer than the doc, and still not right. Measured on the live binary:

```
the doc says     (Some "hello")
the guess was    (wat.core/Option.Some "hello")
IT ACTUALLY IS   #wat.core.Option/Some ["hello"]        ← an EDN-tagged value
```

★ **Neither of us would have got it from reading.** That is the whole argument for validating it: the
expected value of an example is exactly the kind of claim a human cannot check by eye and a machine
can check trivially.

## The structural defect — the contract REQUIRES unverifiable assertions

`src/intrinsic/mod.rs::purity_mandated_examples`, in its own words:

> *"Arc 255.1b-firm: pure+det intrinsics MUST carry ≥1 runnable `@example`; **non-pure-det intrinsics
> MUST carry ≥1 `@example-norun` and NO runnable `@example`**."*

```
@example        86   run: true    — executed, compared, kept honest
@example-norun 118   run: false   — NEVER executed; the `#=>` value is compared to NOTHING
```

**The majority of documented examples carry an expected value that nothing can ever falsify.** And they
are not an oversight — they are *mandatory*: an impure or nondeterministic intrinsic is FORBIDDEN a
runnable example, so its only permitted example is the unverifiable kind.

The requirement manufactures the rot. `[[feedback_a_green_test_can_prove_nothing]]` — except here there
is not even a green; there is a printed claim with no instrument behind it.

## Why the reason for `-norun` does not extend to the expected VALUE

`-norun` exists because the CALL cannot be executed at doc-check time — it reads a file descriptor,
forks, asks the clock. That is a fact about the **call**.

It is not a fact about the **rendered form of the answer**. `#wat.core.Option/Some ["hello"]` is a
value shape the substrate can produce and render without performing any I/O at all. So a `-norun`
example's `#=>` is unverifiable *by current construction*, not by nature.

## ⛔ AMENDED — the ENUM WIRE REP IS CHANGING, and it decides the shape of the fix

The builder, on reading the above:

> *"we have a pending change to the edn rep of enums... for options they'll become
> `#wat.core/Option.Some {:value "hello"}` … what matters is that we don't forget to do the no-run
> challenging the ret val must be an instance of the ret-type the doc declares — we get this when we
> run, but no run has non-determinism so it'll be nuanced"*

It is filed as **`docs/arc/2026/06/296-diagnostics-fully-edn/DESIGN-STONE-H-variants-are-maps.md`** —
*"a variant is a tagged map, like everything else"*, **STATUS: DRAWN, NOT BUILT**, ruled 2026-08-15.
Its wire form:

```clojure
#wat.telemetry/Numeric.I64 {:val 42}        ;; tag #<ns>/<Enum>.<Variant>, body a map keyed by binder
```

So the value in the offending doc line is on its **third** spelling, and only one of them has ever
been written down correctly:

```
the doc says            (Some "hello")                          ← never right
renders TODAY as        #wat.core.Option/Some ["hello"]
renders AFTER 296 H as  #wat.core/Option.Some {:value "hello"}
```

★ **THIS KILLS THE "RENDER A WITNESS AND MATCH THE TEXT" DIRECTION BELOW.** Any validation that pins
the rendered TEXT would go green today and rot the instant stone H lands — re-breaking 118 examples for
a reason that has nothing to do with whether they are correct.
`[[feedback_a_measurements_boundary_is_its_claims_boundary]]`.

★ **The durable requirement, in the builder's words: the expected value must be an INSTANCE OF the
`@ret` type the doc declares.** That is a TYPE-MEMBERSHIP question, and it survives every rendering
change — `(Some "hello")`, `#wat.core.Option/Some ["hello"]` and `#wat.core/Option.Some {:value
"hello"}` are three spellings of one membership claim, and only the membership is stable.

**The nuance the builder flagged:** a runnable `@example` gets this free — execute, and the actual value
IS an instance by construction. A `-norun` cannot execute, so the check must run on the expected TEXT:
read it as a value, and ask whether its type satisfies the declared `@ret`. No I/O, no nondeterminism,
no execution — the same reasoning that says `-norun` is about the CALL, not about the answer's shape.

## Directions worth weighing — none chosen

- **Shape, not value.** Check the expected text parses, and that its type agrees with the intrinsic's
  declared `@ret`. `(Some "hello")` fails that today; `#wat.core.Option/Some ["hello"]` passes. This is
  the builder's *"validate ret types in no-run are logical"*, and it needs no execution.
- ~~**Render a witness** and require the expected text to match that rendering.~~ **STRUCK by the
  amendment above** — it pins a spelling that arc 296 stone H is already ruled to change.
- **Do nothing, but say so.** If `-norun` expected values are to remain decorative, the directive should
  admit it — a `#=>` that means "illustrative, unchecked" is honest; one that looks like every other
  `#=>` is not.

⚠ **Whatever is chosen, it lands in ONE place.** `wat-doc` currently has TWO parsers (`parse`,
`parse_special_form`) with two copies of the line walk and two recognized-tag lists. Validation added
twice is the shape this session has removed nine times.

## Kin

- `docs/arc/2026/04/109-kill-std/DESIGN-STONE-rip-the-heresy-from-the-prose.md` — the sibling defect in
  the same crate: the `@arg`/`@ret` TYPE check is `starts_with(':')`, a shape test standing in for a
  parse. Same crate, same class, and being fixed now.
- `docs/arc/2026/04/109-kill-std/DESIGN-STONE-a-doc-directive-may-wrap.md` — the third: a wrapped
  directive is silently discarded.
- `[[feedback_a_green_test_can_prove_nothing]]`, `[[feedback_a_probe_that_never_invokes_the_thing]]`.

★ Three defects in one crate, all the same shape: **the doc validator validates the GRAMMAR of a claim
and never the CLAIM.**
