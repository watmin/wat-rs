# REFUTE — a metadata map is data, not a hypervector. Use `:wat::core::Value`.

**Builder, on seeing the edit:** *"how the fuck did it learn about holon-ast?..... didn't we nuke
this in all locations but vsa/hdc things?"*

## THE EDIT

```diff
- -> (:wat::core::Option :- [(:wat::core::HashMap :- [:wat::core::keyword :wat::core::Value])])
+ -> (:wat::core::Option :- [(:wat::core::HashMap :- [:wat::core::keyword :wat::holon::HolonAST])])
```

## ⛔ REVERT IT. `:wat::core::Value` IS CORRECT AND IT CHECKS

`metadata-of`'s own `@ret` declares it:

```
@ret (:wat::core::Option :- [(:wat::core::HashMap :- [:wat::core::keyword :wat::core::Value])])
```

**Measured: the declared spelling `--check`s clean.** The change was not forced by the checker.
⚠ **And so does `HolonAST`** — both type-check, which is why nothing stopped it.

## ★★★ WHY THIS MATTERS MORE THAN A TYPO — the wall exists for exactly this

`tests/lint/holon_is_vsa_only.rs`, from its own header:

> *"`holon::HolonAST` is a hypervector composition — bundling, binding, cosine similarity, the whole
> VSA/HDC algebra. `wat::WatAST` is the wat SYNTAX TREE … `src/special_forms.rs` **violated it
> anyway, for months**, storing a special form's `(:head <slot> <slot>)` SIGNATURE (syntax, not a
> hypervector) as a `HolonAST::Bundle`. `src/reflect/verbs.rs` then **copied the shape deliberately,
> citing consistency with the first misuse as its own justification.** A convention stated in prose
> did not merely fail to stop that: **it got cited as the REASON to propagate it.**"*
> (`RULING-holon-is-for-vsa-only-and-a-wall-will-say-so.md`, arc 294)

**A metadata map is keys and values — data. It is not a hypervector.** Annotating it `HolonAST` is
the identical misuse, one surface over.

## ⛔ AND THE WALL CANNOT SEE THIS SURFACE

```
tests/lint/holon_is_vsa_only.rs:   SCOPE   src/  and  crates/*/src/
```

**Rust only. The entire `.wat` corpus is outside it.** Which is why the corpus already carries:

```
HashMap :- [:wat::core::keyword :wat::holon::HolonAST]     9 instances
HashMap :- [:wat::core::keyword :wat::core::String]       34 instances
```

★ **Nine of these already exist**, so this edit is not an invention — it is the tenth, matching a
precedent that is itself the leak. That is the propagation mechanism the ruling named, happening in
the one place the wall was never pointed at.

## WHAT TO DO IN THIS STONE

- **Revert the three annotations to `:wat::core::Value`.** Proven to check.
- **Do NOT chase the other nine.** They are a real finding and they are not this stone's; they need
  the wall's scope widened, which is its own work.
- **Report it** in the SCORE as a named finding so it is not lost.

## STOP TRIGGERS

- **STOP-1 — do not "fix" this by widening the wall here.** Extending `holon_is_vsa_only` to `.wat`
  is a separate stone and would go red on nine pre-existing sites the moment it is armed.
- **STOP-2 — do not add a tenth.** If any annotation in this stone needs a HashMap value type, it is
  `:wat::core::Value`.

---

## ★★★ AND THE SUBSTRATE ALREADY KILLED THIS EXACT CLAIM — ONCE

**Builder:** *"wat-ast is the ast solution for wat .. holon-ast isn't wat's ast."*

That is not a preference. It is a correction a prior stone already landed, in `metadata-of`'s own
header (`src/runtime.rs:7321`, arc 255 Stone P6-c-W4):

> *"★ Doc correction: **the prior header claimed a uniform `(:Option :- [(HashMap :- [Keyword
> HolonAST])])` — FALSE on two counts**, checked against the body below. First, **the map's VALUES
> are never `HolonAST`**: the intrinsic-baseline branch inserts plain scalar/enum `Value`s … while
> the user-metadata branch wraps a CAPABILITY-only map's values as **`Value::wat__WatAST` — WatAST,
> not HolonAST** (**arc 201/251/294.f retired the HolonAST carrier on this whole reflection
> surface** — the same finding W3 made for `lookup-define`/`body-of`). … `:wat::core::Value` … is
> what actually fits."*

So the edit does not merely pick a wrong type. **It restores, verbatim, the annotation a prior stone
proved false and deleted** — on the surface where the HolonAST carrier was retired outright.

## ⛔ AND THE HEADER SAYS WHY IT SURVIVED

> *"`metadata-of` carries **NO checker `TypeScheme`** (absent from `register_builtins` …), so **this
> claim was never verified by anything**; it is corrected here, not newly enforced."*

A false type claim lived in the doc because nothing checked it; it was corrected in prose; and the
nine `.wat` annotations carrying the same false shape were never swept — in the one surface
`holon_is_vsa_only` does not scan. **This edit is the tenth, and the prose correction is what it
walked past.**

★ That is the whole argument for the wall over the convention, made twice now in the same arc:
`RULING-holon-is-for-vsa-only-and-a-wall-will-say-so` said a convention in prose *"got cited as the
REASON to propagate it."* Here a convention in prose was simply not read.
