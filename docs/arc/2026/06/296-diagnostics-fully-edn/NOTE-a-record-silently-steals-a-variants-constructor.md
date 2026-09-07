# NOTE — a record silently steals an enum variant's constructor

**Found 2026-09-06 while writing stone H-2's disconfirming probe.** Not H-2's to close; recorded so
whoever lands the source-side dot knows it is the second half of the same defect.

## Measured

```clojure
(:wat::core::defrecord :usr::Shape::Circle [r <- :wat::core::i64])
(:wat::core::defenum   :usr::Shape :wat::enum::Pure :Circle [r <- :wat::core::i64])

(:usr::Shape::Circle 2)   =>  #usr.Shape/Circle {:r 2}      ← a RECORD
```

With the **enum alone**, the identical call yields `#usr.Shape/Circle [2]` — the variant. With the
record alone it yields `#usr.Shape/Circle {:r 1}`. Declared together, the record wins **in either
declaration order**, with `EXIT=0` and no diagnostic. The enum's `:Circle` constructor becomes
unreachable and nothing says so.

The registration gate (`src/resolve/registration.rs`) has a `Duplicate` verdict for exactly this
shape. It does not fire here: the two names are registered through different paths, so neither ever
sees the other.

## Why it is NOT stone H-2

H-2 flips the **wire tag** — `#ns.Type/Variant` becomes `#ns/Type.Variant`, which separates the two
*renderings*. It does not change either constructor's **source name**: both are still spelled
`:usr::Shape::Circle`, and the theft happens at registration, before anything renders.

An H-2 acceptance row asserting "the record no longer steals the constructor" would be a row H-2
cannot satisfy — the failure this arc's own discipline exists to catch.

## What DOES close it

The dot reaching the **source** spelling, which is arc 251's flip under the builder's ruling
(2026-09-06): a variant reference is `usr/Shape.Circle` — dot in the NAME half — while the record is
`usr.Shape/Circle`, dot in the NAMESPACE half. Different names, so nothing to steal. `has_dotted_name`
(H-1) already guarantees a record can never forge the variant spelling.

Until then the collision is reachable by any program that declares both, and it is silent.

## The smaller, cheaper cure if the source flip is far off

Make the two registration paths share the door they already both pass — the `Duplicate` verdict
exists and is unreachable here only because the enum's variant constructors are minted somewhere the
gate does not see. That is a wall, not a rename, and it would turn a silent theft into a located
error without waiting on 251. Sized: not measured. Named, not claimed.
