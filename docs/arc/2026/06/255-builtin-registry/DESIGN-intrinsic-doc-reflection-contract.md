# Arc 255 — The Intrinsic Doc + Reflection Contract (LOCKED 2026-06-21)

Builder co-design. This is the authoritative spec for how a wat **intrinsic** (a Rust handler
behind a `:wat::…` FQDN) is *documented, verified, reflected, and rendered to the wiki*. It
extends the registry (255.1b) into a Pry/RDoc-grade reflective substrate. Ethos: the macro makes
the doc a **forced, self-verifying contract** — "force LLM maintainability through the roof."

The governing principle, end-to-end:
> **code ⇄ docs** kept honest by *mutual compile-time checks + doctests*;
> **docs ⇄ wiki** kept identical by *generation*.
> The `watmin/wat-rs` wiki is a **build artifact of the registry**, not a maintained thing — it
> cannot be stale or lie.

---

## 1. The forced comment contract (`#[wat_intrinsic]`)

The attribute is **name-only**: `#[wat_intrinsic(":wat::core::Bytes::to-hex")]` (identity,
compiler-bound to the handler ident). Everything else lives in the `///` block, which the macro
PARSES and ENFORCES — a missing *required* element is a `compile_error!` (same forcing as the
arity guard; "forgettable" is annihilated by enforcement). Example:

```rust
/// Encode a `:wat::core::Bytes` into its lowercase-hex `:String`.
///
/// Markdown prose, GFM — flows straight to the wiki page body.
///
/// @added   1.0.0
/// @arg     bs — the bytes to encode
/// @ret     the lowercase hex string, two chars per byte, no separators
/// @example (:wat::core::Bytes::to-hex (:wat::core::Vector 255 0 16)) => "ff0010"
#[wat_intrinsic(":wat::core::Bytes::to-hex")]
pub(crate) fn bytes_to_hex(bs: &WatAST, env: &Environment, sym: &SymbolTable, span: &Span)
    -> Result<Value, EvalBreak> { … }
```

### Directive set (RDoc/YARD-inspired; prose is GitHub-Flavored Markdown)
| directive | req? | → metadata key | notes |
|---|---|---|---|
| (prose) | **required** | `:doc` | GFM, verbatim (Clojure-convention whole string). Wiki page body. |
| `@added <ver>` | **required** | `:added` | `compile_error!` if missing. `"1.0.0"` for all current intrinsics (honest genesis). The one declared-historical fact (un-sniffable). |
| `@arg <name> — <desc>` ×params | **required** | `:args` | name+count **mutual-checked vs the signature** (below). |
| `@ret — <desc>` | **required** | `:ret` | type derives from the scheme (255.2). |
| `@example <expr> => <expected>` | **required ≥1, repeatable** | `:examples` (list) | **doctested** when `pure ∧ deterministic` (below). |
| `@deprecated <ver> <use-instead>` | optional | `:deprecated` | SOFT deprecation (still works, warns) = Clojure's. Distinct from the retirement table (HARD cut). |
| `@see <fqdn>` | optional, repeatable | `:see` | **registry-checked** — no dangling refs. |
| `@yields …` | optional | `:yields` | for HOF intrinsics (a `fn` arg). Later. |
| `@category <name>` | optional | `:category` | wiki grouping. Later. |

Keep the **required** set high-value (prose/`@added`/`@arg`/`@ret`/`@example`); don't force the long tail.

---

## 2. The mutual checks — docs ⇄ code agreement, *measured*

The macro reads BOTH the signature and the directives and ENFORCES agreement. A doc that lies is
a compile error or a failing test. This is the heart of the contract.

- **`@arg` names + count ⇄ signature params** → `compile_error!` on mismatch. You cannot document
  a nonexistent arg, mis-order, or skip one. (available now — macro reads both.)
- **`@example` ⇄ behavior** → for `pure ∧ deterministic` intrinsics the macro **generates a
  doctest** (eval the expr, assert `== expected`). Change the code, the example-test goes red.
  Effectful/nondeterministic → required-but-illustrative (surfaced, not auto-run). (now)
- **`@see` ⇄ registry** → the target FQDN must be a registered intrinsic, else error → refs can't
  dangle (registry makes cross-refs checkable, like Rust intra-doc links). (now-ish; may be a test)
- **`@arg`/`@ret` types ⇄ the registered `TypeScheme`** → the documented types must equal the
  checker's scheme. (255.2, the type-sig layer.)

---

## 3. Derived fields (no directive — sniffed/computed, can't drift)

`metadata-of` carries these with NO declaration:
- `:arity` ← sniff the fixed-arg signature (count `&WatAST` params; DONE 255.1b-ii)
- `:call-seq` / `:arglists` ← derive from arg names (signature) + types (scheme). *RDoc hand-writes
  `:call-seq` for C methods because it can't derive; **wat derives it** → no drift.*
- `:pure` ← `!is_effectful_op(name)` · `:deterministic` ← `pure ∧ ∉ NONDETERMINISTIC` (e.g. `Uuid/v4`)
- `:file` / `:line` ← `Span::call_site()`
- `:defined-in` · `:layer` · `:kind` (enums — see §5)
- `:source` ← captured handler source (see §4)

---

## 4. `show-source` — the Pry lens (free)

The proc-macro already holds the handler's tokens; it captures the source verbatim
(`Span::source_text()`, fallback to token-restringify) → `:source`. `(:wat::core::show-source
<fqdn>)` returns it. **Uniform:** intrinsics show their Rust source; **user forms show their wat
source** (AST → `write-forms`) — one verb, both kinds (exactly like Pry's `show-source` on Ruby +
C methods). Costs nothing — we already have it.

**The reflection surface (Pry/IRB/RDoc-grade), all over the one registry:**
- `child-namespaces` / `names` = `ls` (the namespace tree → wiki nav)
- `metadata-of` = the structured card
- `doc` (= `clojure.core/doc`) = `ri` — prints `:doc` + `:arglists` + `:examples`
- `show-source` = Pry's `$`

---

## 5. Closed-domain values are ENUMS, not free keywords (typo-proof)

A bare `:rust` / `:intrinsic` as a *free keyword value* is string-shape-as-truth — a typo
(`:rsut`) compiles and lies. So closed-domain metadata VALUES are **enums**:
- `:kind` → `Kind { Macro | Fn | Intrinsic }`
- `:defined-in` → `DefinedIn { Wat | Rust }`
- `:layer` → `Layer { Substrate | Userland }`
- `:pure` / `:deterministic` → bool

In Rust the derivation uses the enum (compiler rejects a typo'd variant); surfaced to wat as a
**`defenum` value** (a wat consumer `match`es exhaustively; a new variant breaks every match
loudly). This resurrects the `Kind`/`DefinedIn`/`Layer` enums the 255.1b-i trim dropped *as
unread* — now `metadata-of` reads them, so they return **as enums**. (REVISION to the landed
255.1b-iii core, which currently emits keyword values — flip when hardening.)

Keys stay plain Clojure-style keywords (`:doc`/`:added`/`:arglists`); only closed-domain VALUES
become enums.

---

## 6. Clojure metadata vocabulary — mapped (faithful, ours-where-better)

- `:doc` ← `///` · `:added` ← `@added` · `:deprecated` ← `@deprecated` (soft) · `:arglists` ← derived
- `:private` → **skip Clojure's; use `:restricted-to`** (arc 198 / `#[restricted_to]`, composes) —
  richer (caller prefix-whitelist vs binary; `:private` ≡ `:restricted-to [<own-ns>]`)
- `:line`/`:file` ← span · `:macro`/`:kind` ← derived · `:ns`/`:name` ← FQDN
- `:dynamic`/`:const`/`:test` → N/A · `:tag` → the type-sig layer (255.2)
- a `(:wat::core::doc <name>)` accessor mirrors `clojure.core/doc` (later)

---

## 7. The wiki = a projection of the registry

A generator walks the reflection surface → GitHub-Flavored Markdown → `watmin/wat-rs` wiki:
- `child-namespaces`/`names` → nav tree + page hierarchy
- per intrinsic: prose (MD body) + a **Parameters** table (`@arg` + derived types) + **Returns**
  (`@ret` + type) + **Examples** (` ```clojure ` fenced — GitHub has no `wat` lexer, but the
  Clojure-ified surface highlights as `clojure`; free) + **Added/Deprecated** badges + **See also**
  links + a **Source** `<details>` (`:source`).
- regenerate on change → always in sync. **Nothing hand-written.** (Its own later strike.)

---

## 8. Lifecycle stages (deprecation ≠ retirement)
- `@deprecated` = **soft** — still callable; `metadata-of`/`doc` surface it; a lint warns callers. (Clojure's.)
- retirement table (`remedy/retirement.rs`) = **hard** — form GONE; resolve errors with the replacement.
- Order: deprecated → retired. Two distinct stages, two distinct mechanisms.

---

## 9. Sequencing (strikes; each pure∧derived where possible)
- ✅ **255.1b-iii core** (DONE, uncommitted): `metadata-of` over the registry, `:doc` sniffed,
  baseline (arity/kind/defined-in/layer/pure/deterministic) — proven on Bytes (2 probes green,
  floor 953/36/1). **Currently emits keyword values** → §5 flip pending.
- **255.1b-iv** — harden: flip closed values keyword→enum (§5); `@added` required + enforced;
  `@arg`/`@ret` required + the **signature mutual-check** (§2); `@example` required ≥1 +
  doctest-gen (purity-gated, §2). Re-prove on Bytes.
- **255.1b-v** — `show-source`/`:source` (§4); `@see` registry-check; `(doc …)` accessor.
- **per-home carve** — each home under the full contract; the carve sonnets WRITE the
  prose/`@added`/`@arg`/`@ret`/`@example` per intrinsic.
- **255.1b-RESOLVE** — close the catastrophic hole (resolver consults the registry).
- **255.2** — type-sig layer → `@arg`/`@ret` type mutual-check (§2); the wiki generator (§7).
- **255.1c** FnDef split · **255.3** consumer-collapse (rete/purity + is_pure_total delete) · **255.N** inscription.
