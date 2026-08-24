# PascalCase ⇄ kebab-case — how to write the conversions (spec + grounding)

> **Status:** design/how-to. Surfaced 2026-06-14 from arc 209 C.3 (defservice derives method +
> request-constructor fn names from PascalCase op keywords). Updated arc 265 (namespace-scoped
> acronym registry ships the bijection escape hatch; `to-uppercase` already minted as part of arc
> 209). Grounded against HEAD `bf24ba17`.

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

There is no heuristic that makes consecutive-capital acronyms round-trip. Two approaches:

1. **Discipline the namespace** (plain `pascal->kebab` / `kebab->pascal`): an op keyword carries
   **exactly one uppercase letter per word**, no raw consecutive-capital acronyms. Write `GetUrl`,
   not `GetURL`; `HttpServer`, not `HTTPServer`. On that disciplined subset the two plain functions
   are **total and mutually inverse**: `kebab(pascal(x)) = x` and `pascal(kebab(y)) = y`.
   Enforcement (reject a multi-capital-run op at `defservice` expand time with a `macro-error`) is
   a follow-up rung.

2. **Declare the acronyms (arc 265 — the escape hatch):** for namespaces that work with external
   naming systems (AWS, gRPC, etc.) where `WebACL`, `GetHTTPSEndpoint` are the canonical names, the
   namespace-scoped registry is the escape hatch. Declare once; round-trip total for registered
   acronyms even on external names.

   ```wat
   (:wat::core::string::declare-acronyms :my::aws ["ACL" "HTTP" "URL" "ARN"])
   ;; Now: (pascal->kebab-in :my::aws "CreateWebACL") → "create-web-acl"
   ;; And: (kebab->pascal-in :my::aws "create-web-acl") → "CreateWebACL"  ← casing restored
   ```

   The registry is **namespace-scoped** (`:my::aws` owns its acronyms; `:other::ns` is
   unaffected). No entry for a namespace → the plain converter's behavior applies.
   The `defservice` macro consults the registry via `pascal->kebab-in` at **expand time** (the
   service's own fqdn is the namespace key), so the round-trip is structural — not a convention
   that could silently drift.

> **The bijection invariant:** on the disciplined subset OR when acronyms are declared for the
> namespace, the forward + reverse are **total and mutually inverse**. The registry is the memory of
> the casing the projection drops.

### The canonical rules

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

## What HEAD gives you (grounded, arc 265)

Shipped string primitives (`src/string_ops.rs`, `wat/` registrations):

- `:wat::core::string::length s` → char count.
- `:wat::core::string::subs s start end` → **char-indexed** substring (start-incl, end-excl).
- `:wat::core::string::to-lowercase s` → lowercased copy.
- `:wat::core::string::to-uppercase s` → uppercased copy (minted arc 209; needed by `kebab->pascal`).
- `:wat::core::string::split s sep` → `(Vector :- [String])`.
- `:wat::core::string::concat …` → joined string.
- `:wat::core::string::pascal->kebab s` → PascalCase → kebab (plain; disciplined subset).
- `:wat::core::string::pascal->kebab-in ns s` → namespace-scoped PascalCase → kebab.
  Registered acronyms are ONE segment. No entry for `ns` → plain behavior.
  On `is_pure_total` (defservice macro calls it at expand time).
- `:wat::core::string::kebab->pascal-in ns s` → namespace-scoped kebab → PascalCase.
  Registered acronyms restored to canonical casing. No entry for `ns` → plain behavior.
  NOT on `is_pure_total` (no macro needs the reverse direction).
- `:wat::core::string::declare-acronyms ns [...]` → populate the namespace-scoped acronym registry.
  Must appear BEFORE the `defservice` (or other consumer) that reads it. Returns unit. Handled as
  a declaration form (like `defprotocol` / `defclause`): pre-registered at freeze step 4 into the
  macro-expansion SymbolTable, and again at step 6.96 into the runtime SymbolTable.
- `:wat::core::string::kebab->pascal s` → plain kebab → PascalCase (via `wat/string.wat`).

## Build plan (remaining rungs)

1. **Discipline enforcement** — `defservice` rejects an op keyword with a consecutive-capital run at
   expand time (`macro-error`), so the bijection contract is structural, not a convention. The
   registry-based path (declare-acronyms) is the endorsed alternative when external naming systems
   impose acronym conventions.
