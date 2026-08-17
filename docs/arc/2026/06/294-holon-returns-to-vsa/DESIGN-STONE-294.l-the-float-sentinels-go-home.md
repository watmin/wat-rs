# DESIGN STONE — 294.l · the float sentinels go home (and stop violating 278 A.0)

**Builder's ruling, 2026-08-16.** Asked whether NaN/±Inf should become an enum
(`:wat::core::f64::{PosInf,NegInf,NaN}`), then: *"we could use `-Inf` and `+Inf` as legal words?"* —
and on the outcome: *"floats are decided."*

**Destination:**

```
#wat.core.f64/NaN    []
#wat.core.f64/+Inf   []
#wat.core.f64/-Inf   []
```

Four sites: `crates/wat-edn/src/writer.rs:248-254` (three emits) and
`crates/wat-edn/src/parser.rs:361-365` (the intercept).

## Why NOT an enum — the ruling already exists, twice over

The builder's own ruling of **2026-08-05**, quoted in `src/rete/vocabulary.rs:1141`:

> *"Builder's ruling: **±Inf and NaN are undefined — mint the fallback rows.**"*

…with the mechanism spelled out in the same comment:

> *"the i64 family fails by RAISING (`IntegerOverflow`/`DivisionByZero`), while
> `:wat::core::f64::{+,-,*,/}` is raw IEEE 754 with no overflow guard and **never raises** — a domain
> failure surfaces as an `Ok` holding NaN or ±Inf instead… **Core itself is untouched** — core f64
> keeps returning raw IEEE values and keeps its `total: false` classification; totality is bought
> here, at the rete row, by carrying a fallback."*

So the f64 domain hole is **already faced twice, at two layers, deliberately**:

| layer | totality | mechanism |
|---|---|---|
| `:wat::core::f64::{+,-,*,/}` | `total: false` | raw IEEE — NaN/±Inf ARE f64 values a program can hold |
| `:wat::rete::core::f64::{+,-,*,/}` | `total: true` | `OpClass::Fallback` — the caller's `:undefined` value replaces them |
| `:wat::rete::core::f64::{>,<,>=,<=}` | `total: true` | bool output; `eval_f64_compare` is NaN-correct, no fallback needed |

**An enum would be a THIRD mechanism for one hole.** Worse, it would have to change the core return
type — `f64` or `f64|NonFinite` — which breaks `total: false`, breaks IEEE semantics, and touches
every f64 consumer. NaN and ±Inf are **values of `f64`**. EDN simply has no literal for them, so the
wire needs a sentinel. That is an *encoding* concern, not a *type* concern.

★ Three implementations of one concept is precisely the pattern that cost this arc four separate
strikes (`holon_to_watast` vs `from_holon_item`; `watast_to_holon` vs `to_holon_inner`; capability
encode-by-`type_path` vs decode-by-`name`; and 294.k's `tag_from_type_path` vs `struct_tag_for`).

## ★ But the wire SHAPE is an enum's shape — and today's is illegal

**MEASURED live, 2026-08-16.** Reading `#wat.core.f64/-Inf nil`:

```
unsupported substrate tag #wat.core.f64/-Inf has a bare-nil body
  — retired (arc 278 A.0); unit variants are `#tag []`
```

**Today's `#wat-edn.float/nan nil` violates arc 278 A.0.** It survives only because
`parser.rs:361` intercepts the `wat-edn.float` namespace *before* the substrate's tag dispatch ever
sees it. A grandfathered special case hiding a shape the rest of the corpus is forbidden to write.

Reading it again with the mandated body, `#wat.core.f64/-Inf []`:

```
unknown tag #wat.core.f64/-Inf (body shape: vector); no matching struct or enum in the type registry
```

— i.e. `#tag []` routes to the **enum/struct registry**. That is why the enum instinct kept feeling
right: **on the wire, a unit variant is exactly what NaN is.** The stone adopts the shape without
adopting the type.

## ★ `-Inf` and `+Inf` ARE legal tag names — verified, not assumed

`crates/wat-edn/src/vocab.rs:202`, `validate_first_char`: a leading `-`/`+`/`.` is permitted so long
as the **second** character is not a digit. `-Inf` and `+Inf` both pass.

And the parser agrees in practice — **both probe runs above read the tag name without complaint**;
every failure was downstream and about the *body*, never the name.

## Why not a real registered enum

`crates/wat-edn` has **no type registry**. It parses standalone for the Clojure interop tests
(`crates/wat-edn/interop-tests/`) and must yield `Value::Float(NAN)` on its own. It cannot defer to a
registry it does not have. The sentinel stays wat-edn's, handled by wat-edn, invisible above — which
is coherent, and is what happens today.

## Why `#wat.core.f64/…` and not `#wat.edn.float/…`

Both satisfy the `#wat.*` rule. `wat.core.f64` is chosen because it names **the type whose values
these are**. The layering objection that rules out a generic `#wat.core/nan` — the EDN library
claiming the substrate's namespace — does not apply to naming the one type whose values it otherwise
cannot write down.

## The four questions — flat

**Obvious? YES.** `#wat.core.f64/NaN []` reads as *an `f64` value named NaN*. `#wat-edn.float/nan nil`
reads as a Cargo crate name and a retired body shape.

**Simple? YES.** Four string constants. No new mechanism, no registry, no type change.

**Honest? YES**, and it closes a live inconsistency: the current form emits a body shape the substrate
forbids, and only a special case hides it.

**Good UX? YES.** If the intercept ever fails, the form now hits the registry path and errors as
*"unknown tag"* — a comprehensible complaint — instead of *"retired bare-nil body"* about a shape we
deliberately emit.

## The gate

| # | assertion |
|---|---|
| 1 | `grep -rn 'wat-edn\.float' src/ crates/` → **0** |
| 2 | writer emits `#wat.core.f64/NaN []`, `#wat.core.f64/+Inf []`, `#wat.core.f64/-Inf []` |
| 3 | parser reads all three back to `f64::NAN` / `INFINITY` / `NEG_INFINITY` |
| 4 | **round-trip**: `write(parse(write(x))) == write(x)` for all three, and for a finite float |
| 5 | ⛔ **the Clojure interop tests pass** — `crates/wat-edn/interop-tests/`. This is a wire-format change visible to external readers; they are the gate that says so |
| 6 | `crates/wat-edn/tests/spec_strict.rs` + `comprehensive.rs` green |
| 7 | floor GREEN via `scripts/floor.sh` — the **Summary line** |
| 8 | `cargo clippy --release --all-targets` → **0** |
| 9 | `#[ignore]` count **13**, unmoved |

Row 5 is load-bearing: this is the one family whose wire form a **non-wat** reader consumes.

## Out of scope

Core and rete f64 semantics are **untouched**. `:wat::core::f64::{+,-,*,/}` keep raw IEEE and
`total: false`; the rete fallback rows keep their `:undefined` shape. This stone changes **only how a
non-finite f64 is spelled in EDN text.**
