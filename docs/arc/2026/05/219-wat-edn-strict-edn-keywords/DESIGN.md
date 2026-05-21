# Arc 219 — wat-edn strict-EDN keyword namespace compliance

**Opened:** 2026-05-21
**Branch:** `arc-170-gap-j-v5-deadlock-state`
**Blocks:** arc 218 Stone 218.5 (re-cast vigilia must audit post-strict substrate)
**Trigger:** orchestrator audit during arc 218 surfaced the `::` in wat-edn keyword bodies. Practitioner direction: *"open 219 and do it now - edn now demands it - we satisfy it - 218 is blocked until 219 is done."*

## The defect

`crates/wat-edn/src/vocab.rs:101-122` `is_symbol_continue` accepts both `b':'` and `b'#'` as symbol-body bytes. This is a wat-extension beyond strict EDN:

**EDN spec symbol body chars** (per `github.com/edn-format/edn`): alphanumeric + `. * + ! - _ ? $ % & = < >`. NO `:` (except as keyword prefix at position 0). NO `#` (reserved for tag/discard prefixes).

**wat-edn currently accepts:** spec chars + `:` + `#` + `/`. The `/` is required by the lexer for namespace boundary (parser-level enforces single-`/`). The `:` and `#` are wat-extensions.

**Concrete consequence:**
```
:wat::core::HashMap        ← wat-edn parses (extension); strict EDN rejects
:wat.core/HashMap          ← both wat-edn and strict EDN parse
```

Standard Clojure `edn/read` chokes on `::` in keyword bodies. wat-edn output containing `::` is not portable. The 2026-05-21b forward-correction locked FQDN tags (`#wat.core/Some` — dotted) but left keyword bodies extended.

## Mission

Make wat-edn output round-trippable through standard `clojure.edn/read`. Tighten the dialect to strict EDN on input AND output. The cost: callers must spell wat namespaces with `.` (or get translation at the boundary).

## The blocking chain (post-arc-219)

```
arc 219 (strict-EDN keyword bodies)
  → arc 218 Stone 218.5 (re-cast vigilia + INSCRIPTION + arc closure)
  → arc 217 (Clojure-IPC bridge — depends on strict-EDN; this is the natural FORCING FUNCTION)
  → arc 216 stones 216.8 (#wat.core/Some et al) / 216.9 (#wat.time/Duration) / 216.10 (closure)
  → arc 214 Slice 4
```

## Three scope options

### Option α — Minimal (write-side only)

- `crates/wat-edn/src/writer.rs` translates `::` → `.` on namespace emit
- `crates/wat-edn/src/vocab.rs` `is_symbol_continue` keeps `:` (extension); parser keeps accepting `::`
- Round-trip: `wat::core` → write → `wat.core` → read → store as `wat.core` (LOSSY for wat-rs `::` identity)
- Boundary cost: zero in wat-rs; wat-rs constructed `wat::core` becomes `wat.core` after one round-trip

**Failures:** the dialect is still wat-extended on input. `clojure.edn/read` still chokes on `:wat::core::HashMap` if it appears in wat-extended input. Doesn't satisfy "edn demands it."

### Option β — Moderate (substrate strict; boundary translation in wat-rs)

- `crates/wat-edn/src/vocab.rs` `is_symbol_continue` DROPS `:` and `#` (strict-EDN chars only)
- `crates/wat-edn/src/parser.rs` rejects `::` and `#` in symbol/keyword bodies (substrate-as-teacher fail-loud)
- `crates/wat-edn/src/writer.rs` translates `::` → `.` on namespace emit (input-tolerance for wat-rs callers)
- `crates/wat-edn/src/value.rs` `Keyword::ns` / `Symbol::ns` / `Tag::ns` constructors translate `::` → `.` on construction (so wat-rs callers using legacy `::` literals get auto-translated; storage canonical `.`)
- wat-rs internal storage unchanged (still uses `::` in SymbolTable keys, Rust string literals)
- Boundary: wat-rs constructs wat-edn `Keyword::ns("wat::core", "HashMap")`; constructor translates to `wat.core` form internally; wat-rs reads back via `Keyword.namespace()` returning `"wat.core"` (or new accessor returning the user's original form? — sonnet decides)

**Cost:** moderate sweep across wat-rs construction sites if any pass `::`-form strings directly. Constructor-level translation lets the existing call sites work; the storage becomes `.` form internally.

### Option γ — Maximal (substrate-wide convention shift)

- wat-rs itself adopts `.` as the canonical namespace separator
- `.wat` source can keep `::` as SUGAR; wat source parser translates to `.` at parse time
- Internal storage everywhere uses `.` — SymbolTable, Rust string literals for FQDN registration, error messages, displays
- wat-edn becomes pure strict-EDN; no translation layer needed
- The `::` lives ONLY in `.wat` source syntax (and Rust path syntax for the Rust module tree)

**Cost:** very large. Hundreds of FQDN registration sites in `src/runtime.rs`, `src/check.rs`, `src/macros.rs` etc.; symbol-table key reshape; display sweep. Multi-day arc.

## The four-questions on each option

| Option | Obvious? | Simple? | Honest? | Good UX? |
|---|---|---|---|---|
| **α (write-only)** | NO — reader still wat-extended; "strict" claim partial | YES — one writer change | NO — claims strict EDN but READER accepts non-strict input | NO — Clojure consumer's data → wat-edn → rejected because wat-edn allows AND emits both forms ambiguously |
| **β (substrate strict + boundary translation)** | YES — wat-edn is strict EDN; wat-rs has clear boundary translation rule | YES — substrate change is localized; constructor translation hides the boundary for most callers | YES — wat-edn IS strict EDN; wat-rs uses `::` internally as it always has; boundary is explicit + tested | YES — Clojure consumers get strict EDN; wat-rs callers writing `::` get auto-translated by constructors |
| **γ (substrate-wide flip)** | YES — `.` everywhere, no translation | YES architecturally but NO in execution (hundreds of sites) | YES — strongest single-convention discipline | YES (long-term); but the migration cost is the largest substrate change since arc 109 |

**Option β wins on Honest + Good UX with manageable execution cost.** α fails Honest (claims strict but isn't). γ wins on doctrine purity but the execution cost is multi-day and out of scope for arc 219's "do it now" charge.

**Decision LOCKED: Option β.**

## Stone decomposition

| # | Stone | Scope | Why |
|---|---|---|---|
| **219.1** | Substrate tighten — strict EDN at vocab + parser + writer + constructors | `vocab.rs::is_symbol_continue` drops `:` and `#` (strict chars only) + `parser.rs` symbol/keyword body lex still accepts via current code path (now stricter via vocab) + `writer.rs` writes namespace with `.` (no change needed if constructors store `.`) + `value.rs::Keyword::ns / Symbol::ns / Tag::ns` constructors translate `::` → `.` on input | Substrate becomes strict-EDN on input AND output; constructor auto-translation hides the boundary for most wat-rs callers |
| **219.2** | wat-edn test fixture sweep | All wat-edn-internal tests using `:wat::core::Foo` literals → `:wat.core/Foo`; or pass through the constructor translation (gets the same internal storage). Display tests asserting on `::` output → flip to `.` output. | Tests reflect post-219.1 substrate truth |
| **219.3** | wat-rs caller sweep at the boundary | Any wat-rs Rust-side code constructing wat-edn Keyword/Symbol/Tag passes the `::`-form string; constructor translates. Tests should pass without explicit `.` migration. Verify with grep + targeted re-runs. | Validates the boundary contract — wat-rs's existing `"wat::core"` literals work because constructors translate |
| **219.4** | INSCRIPTION + arc 218 unblock | INSCRIPTION-219.md inscribed; arc 218's Stone 218.5 unblocks (re-cast vigilia runs on post-strict substrate); doctrine cross-reference into DESIGN-216 (encoding doctrine forward-correction acknowledges the keyword strictness). | Closure paperwork; releases arc 218.5 from BLOCKED |

## Constructor translation semantics (Stone 219.1 detail)

Current `Keyword::ns(ns, name)` stores `ns` and `name` literally. Post-219.1:

```rust
impl Keyword {
    pub fn ns(ns: &str, name: &str) -> Self {
        let translated_ns = translate_wat_to_strict(ns);  // "wat::core" → "wat.core"
        // ... rest unchanged
    }
}
```

`translate_wat_to_strict`: replace `::` with `.`. One-pass; idempotent (`.` input stays `.`).

The TRANSLATION is one-way at construction time. Storage is canonical `.`. Display/render is `.`. wat-edn output is `.`. Round-trip is preserved.

**`Keyword::try_ns` + `Symbol::try_ns` + `Tag::try_ns`** apply the same translation. The `validate_first_char` rule runs against the TRANSLATED namespace string (so `wat::core::HashMap` → `wat.core.HashMap` → validate "wat" as first segment).

**`Keyword::from_parts_unchecked`** — design choice: does this also translate, or accept whatever bytes the caller passes? **It MUST NOT translate** — it's the unchecked path; caller's responsibility. wat-rs internal SymbolTable code uses `from_parts_unchecked` likely; those sites must pre-translate.

This is the boundary the BRIEF for Stone 219.3 enumerates: every `from_parts_unchecked` site that constructs a wat-edn keyword from a wat-rs `::` namespace must pre-translate.

## What this arc does NOT do

- **Touch wat-rs internal SymbolTable / FQDN registration** — wat-rs keeps `::` in its own keys; arc 219 is wat-edn-specific
- **Change `.wat` source syntax** — `:wat::core::HashMap` remains the source-level form
- **Touch Rust-side string literals for FQDN registration in `src/`** — they stay as `"wat::core::Foo"`
- **Address the wat-edn `<...>` type-arg list shape** — orthogonal; arc 218 handles those
- **Touch tagged-literal namespaces** — already dots per 2026-05-21b forward-correction

## Cross-references

- `crates/wat-edn/src/vocab.rs:101-122` `is_symbol_continue` — the load-bearing rule
- `crates/wat-edn/src/parser.rs:383-407` `splitn(3, '/')` — single-slash invariant already locked (218.3)
- `crates/wat-edn/src/value.rs::Keyword::ns / Symbol::ns / Tag::ns` — translation site (Stone 219.1)
- DESIGN-218 — arc 218 closure (Stone 218.5) blocked on arc 219
- DESIGN-216 § "Forward-correction 2026-05-21b" — FQDN tags doctrine; arc 219 extends the strictness to keyword bodies
- `feedback_fqdn_is_the_namespace` — doctrine; strict EDN is the canonical surface
- `feedback_inscription_immutable` — arc 218's 218.1-218.4 INSCRIPTIONs stay; arc 218 INSCRIPTION when 218.5 ships will cite arc 219 as the prerequisite that landed
- `project_wat_llm_first_design` — strict-EDN is the LLM-readable canonical form

## Status

Arc 219 opens 2026-05-21 (mid arc 218; before 218.5 closure). Stone 219.1 next concrete work.

**Practitioner's standing instruction:** *"open 219 and do it now - edn now demands it - we satisfy it - 218 is blocked until 219 is done."*

*The substrate had the answer. The audit surfaced the gap. EDN demands strict; we satisfy.*
