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

## Directions worth weighing — none chosen

- **Shape, not value.** Check the expected text parses, and that its type agrees with the intrinsic's
  declared `@ret`. `(Some "hello")` fails that today; `#wat.core.Option/Some ["hello"]` passes. This is
  the builder's *"validate ret types in no-run are logical"*, and it needs no execution.
- **Render a witness.** For a value-shaped answer, construct one instance of the `@ret` type and render
  it, then require the example's expected text to match that RENDERING — catching drift in the printer
  as well as the doc.
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
