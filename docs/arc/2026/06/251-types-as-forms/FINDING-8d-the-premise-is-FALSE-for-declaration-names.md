# FINDING — 8d's PREMISE IS FALSE. Declarations do not accept symbol names.

**Found 2026-09-20 by the counterpart during 251.8d-ii, confirmed and widened by the orchestrator's
weigh.** ⛔ **This is the sixth brief error and it is the load-bearing one: it blocks 8d-iii, not
just the bootstrap.**

## THE PREMISE, as written into every 8d document

> *"a spelling change over a substrate that **already type-checks both**"* — `DESIGN-STONE-251.8` §8d
> *"8d is semantically inert"* — the orchestrator, repeatedly

⭐ **That is TRUE for call sites** (8c proved it: a slashed head now checks byte-identically to a
colon head). **It is FALSE for DECLARATION NAMES.**

## The mechanism — `src/types.rs:4684`, `parse_declared_name`

```rust
let raw = match form {
    WatAST::Keyword(k, _) => k.clone(),
    other => return Err(TypeError::new(…, MalformedDecl {
        reason: format!("name must be a keyword; got {}", other.variant_name()) })),
};
…
Ok((raw, Vec::new()))          // ← the RAW KEYWORD is the TypeEnv KEY
```

**Two distinct defects, and the second is worse:**

1. A `Symbol` declaration name is **refused outright**.
2. ⛔ Even if accepted, `wat.core/Option` would be a **DIFFERENT KEY** from `:wat::core::Option`.
   **Type identity is the raw keyword spelling.** That is not a parser rule — it is the registry's
   notion of sameness.

## MEASURED — which forms refuse a symbol name

| form | symbol name | files in corpus |
|---|---|---|
| `defn` | ✅ **accepts** | 1959 |
| `defenum` | ⛔ `MalformedDecl` | **301** |
| `typealias` | ⛔ `MalformedDecl` | **41** |
| `defclause` | ⛔ *"first arg must be a keyword"* | — |
| `defrecord` | ⛔ *"macro … program body eval failed"* | **623** |
| `defstruct` | ⛔ same | **119** |

⚠ `defrecord`/`defstruct` fail inside **macro expansion**, a different door from `parse_declared_name`
— so this is at least **two** independent gaps, not one.

## ⛔ THE BLAST — measured as a DELTA, on real codemod output

179-file spread of the **converted** corpus (8d-i's own output), each `--check`ed against its
unconverted original:

| | |
|---|---|
| originals that checked **clean** | **161 / 179** (18 already failed — not 8d's fault) |
| of those 161, still clean **after** conversion | **57** |
| ⛔ **REGRESSIONS** | **104 — 65% of previously-clean files** |

**Extrapolated to 2,140 files: ~1,240 regressions.**

⚠ **The absolute number (68% of converted files fail) is NOT the finding** — 18 of the sample failed
before conversion too. Only the delta is 8d's
(`[[feedback_a_number_assembled_from_two_measurements]]`).

⭐ **Demonstration, on a real converted file** (`wat-tests/counter-actor-proof-process.wat`):

```
original :  --check → clean
converted:  --check → MalformedDecl "malformed :wat::core::defenum declaration:
                                      name must be a keyword; got symbol"
```

## Why the census said 2145/2145 and this still happened

⛔ **8d-i's gate measured CONVERSION, not LOADING.** *"The codemod can rewrite every file"* and
*"every rewritten file still checks"* are different claims, and only the first was ever gated.
The brief's own acceptance row said *"every converted file still `--check`s no worse than before —
captured as a delta, per file"* — **that row was written into the parent 8d brief and never run.**
`[[feedback_a_pass_answers_only_the_question_the_instrument_asks]]`.

## ⭐ AND IT RESTORES THE SEAM'S ORIGINAL CHAIN — the orchestrator was wrong to call it inverted

`docs/SEAM.md` says, in capitals:

> ⛔ **THE REGISTRY UNBLOCKS THE CLOJURIFICATION, NOT THE REVERSE.** *If you remember it the other way
> round, that is the pivot talking.*

On 2026-09-20 the orchestrator wrote that the chain had **inverted** — that 8c closed #95 without the
registry, so 255 was merely *free* rather than *required*. ⛔ **That was wrong.** #95 was indeed
closed without 255, but **8d-iii is gated by declaration-name identity, which IS registry work**
(what a type's key *is*). The SEAM's instruction was right even though the mechanism it named (#95)
was not the one that binds. **The warning even predicted the failure mode — "that is the pivot
talking" — and it was ignored within a day of being re-read.**

## What this does NOT change

- **8d-i stands.** The codemod is total, and that is still true — it *converts* all 2,145.
- **The `wat/` conversion is proven** (64/64) and its output is on disk under `/tmp/8d-ii/tree`.
- **The recovery is proven**: `git checkout -- wat/ && cargo build --release`, exit 0, binary alive,
  codemod still functional. Verified independently by the orchestrator.

## The fork — the builder's

1. ⭐ **Make declarations spelling-agnostic FIRST** (a prerequisite stone, or arc 255). `defenum`,
   `typealias`, `defclause`, plus the `defrecord`/`defstruct` macro path, plus `wat-source-derive`'s
   `wat_enum_from!`. ⛔ **And decide what a TypeEnv key IS** — that is the registry question the
   builder already ruled on for `wat.type`, arriving from a second direction.
2. **Convert call heads but NOT declaration names** — leaves a name spelled one way where it is
   declared and another where it is called. Almost certainly unacceptable; recorded for completeness.
3. **Abandon/narrow 8d.** Recorded for completeness; contradicts the builder's standing direction.

⚠ **Option 1 makes `wat.type` and this the same arc**, which is an argument for doing 255 next rather
than after 8d — the opposite of what the orchestrator recommended two hours ago, on this evidence.
