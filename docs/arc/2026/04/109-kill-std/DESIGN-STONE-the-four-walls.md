# DESIGN — arc 109 step ③: THE FOUR WALLS. Every heretic screams, and three of four are handed their own fix.

> Builder: *"we are building walls to annihilate heresy — do your measurements — identify how to make
> the compiler make every heretic scream as they are set ablaze."*

Step ① (`f454c4650`, `df90b9904`) made the bracket ACCEPTED everywhere. This stone designs the walls
that make everything else ILLEGAL. Sequenced after ②'s codemod for the writable classes — see
*Ordering*, which is the load-bearing section and is NOT what you would guess.

## The mechanism already exists — do not build one

`Remedy { form: String, kind: RemedyKind, note: Option<String> }` (`src/remedy/mod.rs:65`), where
`form` is *"the candidate form offered as a replacement."* `CheckErrorKind::MalformedForm` already
carries `remedies: Vec<Remedy>`, and remedies serialize into the error's EDN. **`RemedyKind::Retirement`
already means "a retired form with an explicit replacement"** — the angle form is exactly that. No new
variant, no new machinery: the worklist is already machine-readable.

## The four walls

### WALL 1 — the angle form. `Head<…>` anywhere. ~3,236 sites. ★ REMEDY WRITABLE

**Fires at:** `types.rs:4279`, the `Some(lt_index)` arm of the type-keyword parser. It already splits
`base = &stripped[..lt_index]` and `params_part = &stripped[lt_index..]`, so at the point of refusal it
holds both halves.

**Says:** `:wat::core::HashMap<wat::core::String,wat::core::i64> is retired; parametrics take a type
vector` · **remedy** `(:wat::core::HashMap [:wat::core::String :wat::core::i64])`

⚠ Split the params with `split_type_list_top_level` (`types.rs:4858`), **never** a flat `split(',')` —
a nested `State<K,V>` tears otherwise. That splitter dies at the END of this campaign, not at its start.

**Also delete here:** the lexer's `angle_depth` machinery (`crates/wat-reader/src/lexer.rs:792`, `:942`).
Until it goes, `Head<…>` still lexes as one keyword token and the wall is a check rather than a no-form.

### WALL 2 — a bare parametric head. 238 sites. ⛔ REMEDY **NOT** WRITABLE

**Fires at:** wherever a container head resolves with zero type args
(`BARE_CONTAINER_HEADS`, `check.rs:1008`).

**Says:** `:wat::core::PersistentMap requires a type vector` · **remedy: NONE.**

★★ **This is the one wall the compiler cannot fix for you, and it is the whole reason Ordering below
is not obvious.** A bare head has no args to recover — the information was never written down. The
checker cannot infer K,V at an *annotation* site; that would need whole-program inference it does not
do. All 238 need a human or a rider reading each site's usage.

Measured population (type position only, after `<-`/`->`): `PersistentMap` **136** bare vs 4
parametric — a 34:1 inversion; `PersistentVector` **102** bare vs 210. Against `Vector` 1, `HashMap` 3,
`HashSet` 0. See `NOTE-the-persistent-collections-never-had-the-parametric-requirement.md`.

### WALL 3 — a bracket-less constructor call. 977 sites. ★ REMEDY WRITABLE

**Fires at:** the `infer_*_constructor` fns — `infer_persistentvector_constructor`,
`infer_tuple_constructor`, `infer_persistentmap_constructor` and their three wired siblings.

**Says:** `:wat::core::PersistentVector requires a type vector; inferred as [:wat::core::i64]` ·
**remedy** `(:wat::core::PersistentVector [:wat::core::i64] 1 2 3)`

★ These fns compute the type FROM THE ELEMENTS before they could refuse — `t_ty` for the vectors, one
per position for `Tuple`, K and V for the maps. The fix is in hand at the moment of refusal.

⚠ **Two ways this wall lies, both to be checked before trusting it** (already recorded in the
type-vector stone): a **fresh unresolved type variable has no spelling**, and an empty
`(PersistentVector)` infers nothing at all — those sites emit no usable remedy and must be counted,
not silently skipped. And an inferred CONCRETE type where a SUPERTYPE was intended **ships green**,
because the checker then accepts its own guess. That is the only place in this migration where a wrong
answer passes.

### WALL 4 — `Fn(…)->T`. 141 sites. ★ REMEDY WRITABLE — but NOT this stone

**Fires at:** `parse_fn_body` (`types.rs:4773`), which already holds args and ret.
**Says:** remedy `[:wat::core::f64 :-> :wat::core::i64]`.

Its own stone: `DESIGN-STONE-fn-types-are-brackets.md`, sequenced after this one. Both dialects flow
through the same splitter; migrating them concurrently means a floor that cannot say which wall caused
which red.

## ⛔ ORDERING — and it is the opposite of the obvious one

The obvious order is "raise all the walls, read the screams." **That is wrong here, and Wall 2 is why.**

Walls 1, 3 and 4 emit ~4,354 errors that each carry their own fix. Wall 2 emits 238 that carry
nothing. Raise them together and 238 sites needing genuine judgment are buried inside 4,354 that need
none — and the only way to tell them apart afterwards is to re-read every one.

```
2a  FIRST, before any wall:  fix the 238 bare heads BY HAND (rider-assisted, per-site usage read).
                             They are judgment work and they must not compete for attention.
                             ⚠ rete.wat:30,:37,:1830,:2383 are four of them — the engine's own
                             Token bindings. `Value,Value` type-checks and matches what the code
                             does; whether a truer type exists is UNMEASURED. Look before writing.
2b  THEN raise WALL 1        ~3,236 screams, each with its fix. Codemod applies remedies. Waterfall.
2c  THEN raise WALL 3          977 screams, each with its fix — minus the no-spelling cases, which
                             must be COUNTED and named, never skipped.
2d  THEN delete the lexer's angle machinery — the check becomes a no-form.
③   THEN the Fn stone, then the casing pass.
```

★ The rule underneath: **do the unwritable work first, alone, where it cannot hide.** A worklist where
every entry carries its own answer is a machine's job; a worklist where 5% need judgment and look
identical to the 95% that do not is how the 5% get rubber-stamped.

## What the walls do NOT touch

- **EDN literal forms.** `[1 2 3]` · `{"a" 1}` · `#{1 2 3}` keep inference — measured working, and
  ruled: *"the only place we support inference is for edn forms."* A literal and a constructor call
  are two different acts, not two spellings.
- **`<-` `->` `:->` `<` `<=` `>` `>=`** — 9,912 sites that must survive untouched. The discriminator is
  the lexer's own: inside a keyword, `<` after `::` is the operator; `<` after alphanumeric/`_`/`'`
  opens type params.
- **Keywords / the `wat.type/` flip.** The second hard problem, and explicitly not this one.

## The four questions

- **Obvious?** YES — each wall names one illegal form and hands back the legal one.
- **Simple?** YES, and it SUBTRACTS: the lexer's angle machinery, and (with Wall 4) the depth splitter.
- **Honest?** YES — and the honesty is in Wall 2 carrying **no** remedy rather than a guessed one, and
  in Wall 3's two named lying modes being counted rather than skipped.
- **Good UX?** YES — the compiler does not merely name the sites, it writes the fix for 95% of them.
