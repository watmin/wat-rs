# PascalCase ⇄ kebab-case — how to write the conversions (spec + grounding)

> **Status:** design/how-to. Surfaced 2026-06-14 from arc 209 C.3 (defservice derives method +
> request-constructor fn names from PascalCase op keywords). Captured *forward* — the
> PascalCase→kebab direction is buildable today; the kebab→PascalCase direction names a missing
> substrate primitive (`string::to-uppercase`). Grounded against HEAD `1df0e5e6`.

## Why this exists

wat has two live naming registers, and they are not interchangeable:

- **Types / records / enum variants are PascalCase**: `GetRequest`, `GetResponse`, `Op::Increment`.
- **Functions / methods / locals are kebab-case** (clojure idiom): `read-all-string`, `remove-at`,
  `get-request`, `increment`.

`defservice` reads op keywords that are PascalCase (`:Get`, `:Increment`, eventually `:GetObject`)
and must emit BOTH registers from each:

| op keyword | record (PascalCase) | method / constructor (kebab) |
|---|---|---|
| `:Get` | `GetRequest` / `GetResponse` | `get` / `get-request` |
| `:Increment` | `IncrementRequest` / … | `increment` / `increment-request` |
| `:GetObject` | `GetObjectRequest` / … | **`get-object`** / **`get-object-request`** |

The record name is plain concatenation (the op keyword is *already* PascalCase). The fn name needs
**PascalCase → kebab**. C.3's counter has only single-word ops, where "lowercase the keyword"
accidentally equals kebab — but `:GetObject` lowercased is `getobject`, which is **wrong and
silently compiles**. That latent *Honest* failure is what this conversion eliminates.

## The bijection contract (the acronym trap)

A naive converter lies on acronyms. Consider `HTTPServer`:

- "boundary before every uppercase, downcase" → `h-t-t-p-server` (garbage).
- "treat a run of capitals as one acronym" (heck/Rust style) → `http-server` — readable, BUT
  `kebab→Pascal("http-server")` → `HttpServer` ≠ `HTTPServer`. **Round-trip broken.**

There is no heuristic that makes consecutive-capital acronyms round-trip. So we do not paper over
it with a lossy heuristic — we **discipline the namespace** so the conversion is a clean bijection
(extirpare: make the non-bijective input unrepresentable rather than guess at it):

> **Naming discipline (the contract):** an op keyword carries **exactly one uppercase letter per
> word**, no raw consecutive-capital acronyms. Write `GetUrl`, not `GetURL`; `HttpServer`, not
> `HTTPServer`. A "word" is an uppercase letter followed by zero or more lowercase letters / digits.

On that disciplined subset the two functions are **total and mutually inverse**:
`kebab(pascal(x)) = x` and `pascal(kebab(y)) = y`. Enforcement (reject a multi-capital-run op at
`defservice` expand time with a `macro-error`) is a follow-up rung — name it now so the contract is
not just a convention. See *Build plan* below.

### The canonical rules

**Word boundary** = an uppercase letter that is **not** at position 0.
- `Get` → one word `[Get]`
- `GetObject` → `[Get][Object]`
- `Increment` → `[Increment]`
- digits ride the current word: `GetV2` → `[Get][V2]` → `get-v2`; `Get2` → `[Get2]` → `get2`.

**PascalCase → kebab:** downcase every char; insert `-` before each boundary; join.
- `Get` → `get` · `GetObject` → `get-object` · `GetV2` → `get-v2`

**kebab → PascalCase:** split on `-`; upcase the first char of each segment; concat.
- `get` → `Get` · `get-object` → `GetObject` · `get-v2` → `GetV2`

## What HEAD gives you (grounded)

Available string primitives (`src/string_ops.rs`, `wat/` registrations):

- `:wat::core::string::length s` → char count.
- `:wat::core::string::subs s start end` → **char-indexed** substring (start-incl, end-excl). So
  `char-at(s, i)` = `(:wat::core::string::subs s i (:wat::core::i64::+ i 1))`.
- `:wat::core::string::to-lowercase s` → lowercased copy. (Minted in arc 209 C.3 — it did NOT
  exist before; it is the lowercase basis this conversion needs.)
- `:wat::core::string::split s sep` → `Vector<String>`.
- `:wat::core::string::concat …` → joined string.
- `:wat::core::=` / `not=` on strings.
- uppercase **detection** is free: a 1-char string `ch` is an uppercase letter iff
  `(:wat::core::not= ch (:wat::core::string::to-lowercase ch))` (non-letters lowercase to
  themselves → reported not-upper, which is correct).

**Missing primitive:** there is **no** `to-uppercase` / `char/to-upper` anywhere in `src/` or
`wat/` (only `to-lowercase`). The kebab→PascalCase direction *cannot be written* until one is
minted. This is the reach-stumble — the absent verb the reverse direction reaches for.

## How to write it

### PascalCase → kebab (buildable today)

Fold over the char indices; the leading char never gets a separator; every other uppercase opens a
new word.

```clojure
;; :wat::str::pascal->kebab — "GetObject" -> "get-object", "Get" -> "get"
;; Contract: input is disciplined PascalCase (one capital per word). Total on that subset.
(:wat::core::defn :wat::str::pascal->kebab [s <- :wat::core::String] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String  i <- :wat::core::i64] -> :wat::core::String
      (:wat::core::let
        [ch     (:wat::core::string::subs s i (:wat::core::i64::+ i 1))
         lower  (:wat::core::string::to-lowercase ch)
         upper? (:wat::core::not= ch lower)]
        (:wat::core::if (:wat::core::and upper? (:wat::core::i64::> i 0))
          (:wat::core::string::concat acc "-" lower)   ;; word boundary
          (:wat::core::string::concat acc lower))))
    ""
    (:wat::core::range 0 (:wat::core::string::length s))))
```

### kebab → PascalCase (BLOCKED on `string::to-uppercase`)

The shape, for when the primitive lands — `capitalize` is the only piece that needs the missing
verb:

```clojure
;; capitalize "object" -> "Object"  (REQUIRES :wat::core::string::to-uppercase — NOT YET MINTED)
(:wat::core::defn :wat::str::capitalize [w <- :wat::core::String] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::string::length w) 0)
    w
    (:wat::core::string::concat
      (:wat::core::string::to-uppercase (:wat::core::string::subs w 0 1))   ;; <-- missing
      (:wat::core::string::subs w 1 (:wat::core::string::length w)))))

;; :wat::str::kebab->pascal — "get-object" -> "GetObject"
(:wat::core::defn :wat::str::kebab->pascal [s <- :wat::core::String] -> :wat::core::String
  (:wat::core::string::join ""
    (:wat::core::map :wat::str::capitalize (:wat::core::string::split s "-"))))
```

## Build plan (forward — not built)

1. **`pascal->kebab` helper** — a wat string fn in a string-tooling home (e.g. `wat/string.wat` or
   alongside the keyword helpers). Buildable today; the C.3-follow-up stone threads it through
   `defservice`'s method + constructor name derivation (replacing the current bare lowercase), with
   a deliberately multi-word op (`:GetObject`) added to the gate probe to prove it.
2. **`:wat::core::string::to-uppercase`** — mint the missing primitive (mirror `to-lowercase` in
   `src/string_ops.rs`). Unblocks `kebab->pascal`.
3. **`kebab->pascal` helper** — once (2) lands.
4. **Discipline enforcement** — `defservice` rejects an op keyword with a consecutive-capital run at
   expand time (`macro-error`), so the bijection contract is structural, not a convention.

Steps 1 + 4 are the load-bearing pair for defservice correctness; 2 + 3 are the symmetric
completion for general substrate use.
