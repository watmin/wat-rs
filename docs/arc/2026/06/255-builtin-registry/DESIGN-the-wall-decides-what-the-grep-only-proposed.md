# DESIGN — the wall decides what the grep only proposed

**Ruled 2026-09-10 at `47011872f`** (floor 5318/5318, clippy 0). Option **C** of the four-questions
in the main chat: *text proposes · the type env decides · the lint pins.*

## Why the two earlier options are dead

**A — a wider text predicate, alone.** Measured precision: the separator-as-data predicate proposes
**78** candidates for **16** real decomposers (21%); the composition predicate proposes **82** for
**5** (6%). And the count over four passes ran **2 → 8 → 19 → 21**, each pass with a wider
predicate, each finding sites the previous one *could not propose*. A predicate that has never yet
closed this set is not evidence the fifth one will. **FAILS Honest.**

**B — land the flip and read the reds.** Bounded by test coverage (`peragrare`: silence is not
proof), and — measured before this session — flipping the composer alone takes `wat/core.wat` down
at load, so its reds would be one load error repeated thousands of times. Not separable from the
landing. **FAILS Honest.** This is *why* the seam's ONE-landing ruling stands.

★ Neither reached the UX tiebreaker. The first three questions settled it.

## The instrument, and the control that shaped it

The decisive question is the one all 21 sites already ask internally: **does the split parent
resolve to a `TypeDef::Enum`?** — `variant-parent-of`'s own predicate, the door this campaign
minted. Text can only ever nominate; the type env decides.

The wall that pins it, once the routing is done:

```
hit  = the separator as DATA      "::" | b"::"
     | the separator COMPOSED     }:: | ::{  inside a string literal
     | the general ACCESSOR       identifier::path( | identifier::leaf( | .path() | .leaf()
gate = the file talks about variants at all   TypeDef::Enum | EnumVariant | /\bvariant/i
```

⚠ **THE GATE WAS WIDENED BY A KNOWN-ANSWER CONTROL, NOT BY TASTE.** The obvious narrow gate — *the
file mentions `TypeDef::Enum`* — was run against the 21 hand-read sites and **caught 19**. It
missed `src/match_arm.rs:134` (`is_namespaced_variant`, a file that never names the type) and
`crates/wat-doc/src/print.rs:128` (`axis_edn`). The wide gate catches **21/21**.

```
narrow gate   11 files   122 screams   19/21   ⛔ DISQUALIFIED BY THE CONTROL
wide gate     40 files   191 screams   21/21   ✓
                          76 COMPOSE · 60 DATA · 55 ACCESSOR
```

★ **The tighter predicate costs 69 rune lines less and two known sites more.** Choosing it to save
the tax is precisely the error that produced this whole correction. The tax is the price of the
guarantee; it is paid once.

## The split — all the judgement lands in the first half

The stepping-stone test (`COMPACTION-AMNESIA-RECOVERY.md` § 5) answers YES:

- **③a-ii — ROUTE THE 21.** No lint. Every confirmed variant site goes through
  `compose_variant`/`decompose_variant`. Behaviour-preserving *by construction* while the separator
  is still `::`, exactly as ③a proved on cluster 1.
- **③a-iii — THE WALL.** The lint above, plus a rune on every remaining scream. **After ③a-ii,
  every scream is non-variant by construction**, so no rider ever has to decide *route or rune* —
  the decision that has been wrong three times already is made once, by me, in ③a-ii.

## ③a-ii's rows — the 21, and why each is one

`decompose_variant(n)` is **exactly** `Some((path(n), leaf(n)))` when `n` contains `"::"`, and
`None` otherwise — `path` returns `""` iff there is no `"::"`, which is precisely when the new door
returns `None`. Every row below is therefore a rewrite with no observable change.

```
PATTERN G — guard + path + leaf                     -> let (p, v) = decompose_variant(x)?;
  1  src/record/construct.rs:261,264,265               try_eval_enum_map_ctor
  2  src/check.rs:13689,13692                          literal_enum_variant_ctor
  3  src/check.rs:13896,13899                          the second checker decomposer
  4  src/closure_extract.rs:1589,1590                  dependency recording (if-let form)

PATTERN R — rsplit_once("::")                       -> decompose_variant(x)
  5  src/rete/expr_ir.rs:818        6  src/rete/validate.rs:1227
  7  src/rete/validate.rs:1483      8  src/rete/purity.rs:804
  9  src/check.rs:6881             10  src/check.rs:7007
 11  src/check.rs:7179             12  src/check.rs:7436        13  src/check.rs:7682

PATTERN P — the separator AS a predicate            -> decompose_variant(x).is_some()
 14  src/match_arm.rs:134           is_namespaced_variant
 15  src/rete/expr_ir.rs:604        the Pat::Variant discriminator

PATTERN M — MIXED: composes via the door, decomposes via the general accessor
 16  src/rete/expr_ir.rs:1321       leaf(name) -> decompose_variant(name).map_or(name, |(_,v)| v)

PATTERN C — hand-rolled COMPOSITION the door missed -> compose_variant(parent, variant)
 17  src/runtime.rs:3196   format!("{}::Op::{}",    protocol_fqdn, variant)
 18  src/runtime.rs:3198   format!("{}::Reply::{}", protocol_fqdn, variant)
 19  src/runtime.rs:8638   format!(":{}::{}", fv.enum_class, fv.variant)   Value::ForeignVariant
 20  src/check.rs:4837     format!("{enum_path}::{variant_leaf}")
 21  crates/wat-doc/src/print.rs:128  format!("{wat_type_path}::{variant}")   axis_edn
```

★ **Row 16 is why the pair had to be named.** It ALREADY uses the composition door and the flip
still breaks it, because its other half went to the general accessor. A site can be half-migrated
and look finished.

⚠ **Rows 17/18's parent is `{protocol_fqdn}::Op`, and that `::` STAYS.** The enum's own type path
is `S::Op`; the variant is `Bump`. Post-flip the correct spelling is `S::Op.Bump`. The same
distinction retires `format!("{}::Op", surface.name)` (`types.rs:3224`, `check.rs:17098`) from this
list entirely — those compose a TYPE PATH, not a variant.

## Out of scope, affirmatively

**The display strings.** Roughly eleven `format!`s render `Type::Variant` into a human-facing error
message (`record/construct.rs:314,331,352`, `check.rs:13961,13978`, `edn/render.rs:3090,3101,3110,3960`,
`holon/outcome.rs:350,367`). They are neither composition nor decomposition — they are the surface
spelling shown to a person, and after the flip they should read `Type.Variant`. ③a-iii's rune
categories are what will enumerate them: `grep 'rune:lint(one-variant-separator, display)'`. Not
tracked as a deferral — it is a row of ③a-iii, which is drawn before ③a-ii commits.

## Acceptance

```
floor  5318/5318 via scripts/floor.sh          clippy --release --all-targets -D warnings = 0
the 21 rows each read through the door; identifier::path/leaf UNTOUCHED as general accessors
no behaviour change: no test expectation moves — a moved expectation is a FINDING, not an update
```
