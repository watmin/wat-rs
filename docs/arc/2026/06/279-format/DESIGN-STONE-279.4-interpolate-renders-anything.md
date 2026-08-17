# DESIGN STONE — 279.4 · `interpolate` renders anything, and the partial renderer ceases to exist

**Builder, 2026-08-17:** *"we need to fix interpolate.... that's next... good find...."*

**Depends on 279.3** — this stone's whole content is *"call `render_str_total` instead"*, and 279.3
is what mints it. Strike order is fixed: 279.3 green, then this.

---

## ★ THE DEFECT IS NOT "interpolate is partial". IT IS THAT THE CHECKER ALREADY PROMISED OTHERWISE.

`src/check.rs:13930-13935`, `infer_string_interpolate`'s own doc comment, on disk today:

> *"The remaining args are (keyword, value) pairs: keyword slots are validated structurally; **value
> slots accept ANY str-renderable type (do NOT reject non-String values — the intrinsic renders them
> unquoted at runtime)**. Returns `String`."*

The checker admits any value **and names the runtime's behaviour as its justification.** The runtime
does not have that behaviour:

```rust
// string_ops.rs — render_unquoted, the pre-279.2 partial renderer
Value::String | i64 | f64 | bool | u8  => Ok(…)
other                                  => Err(TypeMismatch { expected: "String | i64 | f64 | bool | u8" })
```

So `(interpolate "{r}" :r <a record>)` **type-checks and then raises**. The permissive rule is not
wrong; the reason written next to it is false. Arc 284 wrote that comment in good faith — at the
time "str-renderable" *was* five scalars, and the sentence overstated it. **279.2 made `str` total.
The world the comment describes can now exist, and this stone makes it exist.**

Stated precisely, because the stronger version would be an overclaim: wat's checker does not promise
that a checked program cannot raise — `Option/expect` raises by design. The defect here is narrower
and still real: **the stated justification for this verb's permissive typing rule is a runtime
capability that does not exist.**

## The second half — the twins disagree, and the crippled one is the one macros must use

| | renders via | total? |
|---|---|---|
| `format` (macro, expand-time) | emits `` `(:wat::core::str ~val-ast) `` — **`wat/core.wat:1737`** | **YES**, since 279.2 |
| `string::interpolate` (runtime twin) | `render_unquoted` — `string_ops.rs:612` | **NO** — 5 arms |

Same template grammar, same `{{`/`}}` escape, same kwargs shape — deliberately, and the code says so
(`string_ops.rs`: *"mirrors the format macro's state machine"*). Two different answers for the same
value.

And the asymmetry runs the wrong way. `interpolate` exists **because `format` is refused by the
purity gate inside defmacro bodies** — it is the expand-time-legal twin. So the substrate's own
macros are forced onto the partial one. **156 call sites** in the corpus, overwhelmingly inside
`wat/core.wat` and `wat/bracket.wat` macro bodies.

*(I told the builder that widening `interpolate` "changes what the `format` macro does with a
non-scalar." That was WRONG — `format` already goes through `str`. Corrected here so the record does
not carry it; the reason to make this its own stone is that it is its own stone, not that it is
risky.)*

## The shape — and it is a deletion, not a widening

```rust
// string_ops.rs:612, inside eval_string_interpolate
- let rendered = render_unquoted(eval(val_arg, env, sym)?.value_owned(), OP, val_arg.span())?;
+ let rendered = render_str_total(&eval(val_arg, env, sym)?.value_owned(), sym.types().map(|a| a.as_ref()));
```

`render_unquoted` has **exactly one caller** (measured: `grep -rn 'render_unquoted' --include=*.rs .`
→ the definition plus `string_ops.rs:612`). After this swap it has **zero**, and it is deleted along
with the `op` / `span` parameters that existed only to build an error that can no longer occur.

★ **That is why this is the root and not the stem.** The stone is not "make `interpolate` total" — a
fix, which leaves a second renderer on disk waiting to be reached for again. It is **the partial
renderer ceases to exist.** After 279.3 + 279.4 the substrate has exactly ONE value-rendering
function, `render_str_total`, with three callers: `str`, `join`, `interpolate`. A future verb that
wants to render a value has one door and no partial one to find.

## The four questions — flat, on all four options

- **A — call `render_str_total`; delete `render_unquoted`.**
  Obvious **YES** (one renderer, one name; `format` and `interpolate` visibly agree) ·
  Simple **YES** (one call-site swap, one deletion; net negative lines, no new concept) ·
  Honest **YES** (makes a comment that is already on disk *true*, and removes the second
  implementation of a rule whose second arm is invisible at the call site) ·
  UX **YES** (the verb does what the checker already told the caller it would). **ALL FOUR.**
- **B — widen `render_unquoted` in place, keep it.**
  Obvious **NO** (two renderers doing the same thing, distinguishable by nothing a reader can see) ·
  Simple **NO** (keeps `op`/`span` error machinery that can never fire) ·
  Honest **NO** (this is precisely the drift shape 279.3 minted the one door to prevent — and it
  would be minting it and then declining to use it) · UX moot.
- **C — tighten the CHECKER instead: reject non-scalar kwarg values.**
  Obvious YES · Simple YES · **Honest NO** — it widens the twin gap rather than closing it (`format`
  renders anything, `interpolate` would refuse), and it walks back a capability arc 284 granted
  deliberately, on the grounds that the runtime is weak. That is repairing the wrong end. ·
  UX **NO** (the macro-legal twin becomes strictly weaker than the one macros cannot use).
- **D — do nothing.** **Honest NO** — the checker's stated justification stays false.

**→ A.**

## The gate

| # | assertion |
|---|---|
| 0 | ★ **NON-VACUITY, FIRST:** `(interpolate "{r}" :r <a record>)` **raises today** — captured verbatim BEFORE the change. If it already renders, this stone is already done and something else is going on: STOP |
| 1 | after: that same call **renders the record**, named fields (`{:x 1}`, not `{:field-0 1}`) |
| 2 | ★ `(interpolate "hello {name}" :name "world")` → `hello world` — **BARE.** The non-vacuity control on the String arm and the proof that nothing started re-quoting |
| 3 | `grep -rn 'render_unquoted' --include=*.rs .` → **0** — the function is GONE, not merely unused |
| 4 | all **156** existing `interpolate` call sites green (the floor + the wat-scripts loader gate cover this) |
| 5 | `check.rs:13934`'s comment is now TRUE as written — read it and confirm; reword only if 279.3 changed the renderer's name |
| 6 | `render_str_total` now has **three** callers: `str`, `join`, `interpolate` |
| 7 | floor GREEN via `scripts/floor.sh` — the **Summary line** |
| 8 | `cargo clippy --release --all-targets` → **0** |
| 9 | `#[ignore]` count **13**, unmoved |

Row 0 is load-bearing in the way row 2 was for 279.3: without it, rows 1 and 2 could both pass on a
stone that changed nothing. Row 3 is what separates this from a patch.

## Out of scope — affirmative cuts

- **`str` vs `show` semantics for interpolate.** Not an open question: `render_unquoted` already
  renders a top-level `String` **bare**, and `render_str_total`'s first arm is identical. `str`
  semantics is what interpolate has always had; this stone preserves it exactly and only widens the
  tail. A template must substitute `world`, never `"world"`.
- **Teaching the checker to track partiality in general.** That is task #64 (*"every core primitive's
  domain hole becomes a faced outcome"*), a language-scale question. This stone closes one verb whose
  checker comment already claimed the answer.
- **The `format`/`interpolate` duplication itself** — two state machines parsing one grammar, one in
  wat and one in Rust, kept in sync by a comment. Real, and **not** this stone: it is a question about
  the purity gate (why `format` is refused in a defmacro body at all), not about rendering. Recorded
  here so it is not rediscovered as new.
- **The 156 call sites.** They all pass strings today and are unaffected. No migration.

## What this stone does NOT get to claim

It does not make wat's type checker sound, and it must not be written up as if it did. It closes
**one** verb where the checker's own comment named a runtime behaviour that was not there. That is
the whole claim.
