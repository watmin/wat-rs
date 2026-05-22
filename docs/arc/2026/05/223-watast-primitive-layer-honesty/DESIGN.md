# DESIGN — Arc 223 — WatAST primitive-layer honesty (CharLit + NilLit + TagLit)

**Opened:** 2026-05-22 (placeholder; ratified post-arc-221 Phase A)
**Branch:** `arc-170-gap-j-v5-deadlock-state`
**Depends on:** none structurally — arc 223 is wat-rs only; doesn't need arc 221 or 222 to ship first. Can run in PARALLEL with arc 221 Phase B / arc 222.
**Recommended ordering:** ships AFTER arc 221 Phase A completes (so arc 220 Slice 5 closure path is clear) and BEFORE arc 222 (so arc 222's literal-as-surface doctrine is implementable on a clean WatAST primitive layer).

## Mission

Complete WatAST's primitive-literal coverage so the wat-source surface matches EDN-spec primitive completeness AND clojure-compatibility (wat IS clojure-on-rust per 2026-05-22 locked identity). Mint:

- `WatAST::CharLit(char)` — replaces `\c` call-form desugaring
- `WatAST::NilLit` — replaces `Keyword(":wat::core::nil")` convention-based encoding
- `WatAST::TagLit(String, Arc<WatAST>)` — first-class clojure-compatible tagged-literal source syntax (`#uuid "..."`, `#inst "..."`, `#wat.core/Some 42`, etc.) — closes the clojure-compat gap surfaced 2026-05-22

Same doctrine as arc 221 applied to the wat-rs layer above holon-rs.

## Triggering observation

User question 2026-05-22 (post-Stone-221.1 ship, mid Stone-221.2 sonnet flight):

> *"why don't we have CharLit ?... we have all the EDN lits?..."*

The substrate inventory confirmed the asymmetry. WatAST has:
- `IntLit(i64)`, `FloatLit(f64)`, `BoolLit(bool)`, `StringLit(String)`, `Keyword(String)`, `Symbol(Identifier)` — six primitive literal variants
- **No `CharLit`** — Char goes through call-form `(:wat::core::Char/of "c")` per Stone 220.2's scope-minimization (parser desugars `\c` token to that List form at parse time)
- **No `NilLit`** — `nil` is encoded as `Keyword(":wat::core::nil")` per arc 109 + arc 153 canonical FQDN form (convention-based discriminator inside Keyword)

Two convention-based encodings inside existing variants. Same dishonesty class as the HolonAST Symbol/Keyword/Nil collapse that arc 221 just settled.

## Doctrine: same as arc 221, applied one layer up

Arc 221 inscribed (per DESIGN-221 + INTERSTITIAL 2026-05-22): **untagged EDN primitives deserve distinct substrate variants; no convention-based encoding inside other variants.**

Arc 221 applied the doctrine at the HolonAST layer (Char + Keyword + Nil + Tag leaves). Arc 223 applies the SAME doctrine at the WatAST layer above. The wat-rs language has its own substrate layer; same honesty bar.

**The wat-reveals-holon dynamic operates layer-wise too** — wat-rs surface matures (arc 222 will surface literal-as-data ergonomics), which exposes WatAST's primitive-layer compromises (Stone 220.2's call-form for Char, arc 153's convention-based Nil). Arc 223 lifts the compromises.

## Current WatAST primitive coverage

| EDN form | Wat source | Current AST | Honest AST |
|---|---|---|---|
| nil | `:wat::core::nil` keyword OR `()` empty literal | `Keyword(":wat::core::nil")` | **NEW: `NilLit`** |
| true / false | `true` / `false` | `BoolLit(bool)` | unchanged |
| integers | `42`, `-1` | `IntLit(i64)` | unchanged |
| floats | `3.14` | `FloatLit(f64)` | unchanged |
| strings | `"hello"` | `StringLit(String)` | unchanged |
| keywords | `:foo` | `Keyword(String)` (with leading colon) | unchanged structurally; potential canonical-bytes seed distinction (Stone 221.5 mirror) |
| symbols | `foo` | `Symbol(Identifier)` | unchanged |
| **chars** | `\c` | parser desugars to `List([Keyword(":wat::core::Char/of"), StringLit("c")])` | **NEW: `CharLit(char)`** |
| **tagged literals** | `#uuid "..."`, `#inst "..."`, `#wat.core/Some 42` | **lexer REJECTS — only `#{` (set literal) is recognized at form-start; bare `#tag` fails to parse** | **NEW: `TagLit(String, Arc<WatAST>)` — closes clojure-compat violation surfaced 2026-05-22** |

### Clojure-compatibility note (arc 219 disambiguation)

Arc 219 dropped `:` and `#` from `is_symbol_continue` — that restriction is about characters allowed INSIDE a symbol body (preventing `foo#bar` from parsing as one symbol). It does NOT preclude `#` as a form-start dispatch character.

Clojure source legally has multiple `#`-prefixed forms:
- `#{1 2 3}` — set literal (wat ALREADY supports this via Token::LHashBrace at lexer.rs:271)
- `#uuid "..."` — tagged literal (wat REJECTS today; arc 223 Stone 223.3 closes)
- `#inst "..."` — tagged literal (wat REJECTS today; arc 223 Stone 223.3 closes)
- `#wat.core/Some 42` — namespaced tagged literal (wat REJECTS today)
- `#_form` — discard reader macro (wat REJECTS today; OUT OF SCOPE — not a value-producing form)
- `#"regex"` — regex literal (wat doesn't need; we use `:wat::core::regex::*` verbs)
- `#(...)` — anonymous fn shorthand (wat has `defn` + `fn`; OUT OF SCOPE — sugar, not honesty)

Stone 223.3 closes the tagged-literal gap (the load-bearing one for clojure-compat + literal-as-surface). The other `#_` / `#"..."` / `#(...)` shorthand forms are deferred — not honesty issues, just convenience sugar.

## Stones

### Stone 223.1 — `WatAST::CharLit(char)` mint + parser direct dispatch

`src/ast.rs`:
- Add `CharLit(char, Span)` variant alongside `BoolLit` / `IntLit` / etc.
- Doc comment cites arc 220 Stone 220.2 (Char Value variant) + arc 221 Stone 221.1 (HolonAST::Char leaf)

`src/parser.rs:296`:
- Replace `Token::Char(c) => WatAST::List([Keyword(":wat::core::Char/of"), StringLit(c.to_string()), span])` with `Token::Char(c) => WatAST::CharLit(c, span)`
- Direct dispatch; no more desugaring at parse time
- The `(:wat::core::Char/of "x")` verb STAYS in `src/string_ops.rs:444` for runtime construction from String

Consumer cascade (substrate-as-teacher; exhaustive-match compiler):
- `src/check.rs` — infer arm for CharLit → `:wat::core::Char` type
- `src/runtime.rs` — eval arm for CharLit → `Value::wat__core__Char(c)`
- `src/closure_extract.rs:1499` — produce `WatAST::CharLit(c)` instead of call form
- `src/runtime.rs:14779` (Stone 221.2 just-shipped) — `holon_to_watast(Char(c))` produces `WatAST::CharLit(c)` instead of call form
- `src/types.rs:2580` (type-name renderer) — `WatAST::CharLit(_, _) => "char literal"`
- `src/parser.rs:571` (label renderer) — `WatAST::CharLit(_, _) => "character literal"`
- `src/macros.rs` — macro-walker / splice handling for CharLit
- `src/lower.rs` — lowering pass for CharLit
- `src/load.rs` / `src/config.rs` — if they walk WatAST trees
- Any other exhaustive match across the ~1316 WatAST::* sites

Tests in `tests/wat_arc223_charlit.rs`:
- `\a` parses to `CharLit('a')` (NOT the old List call-form)
- `\a` evaluates to `Value::wat__core__Char('a')`
- `(:wat::core::Char/of "a")` STILL works (verb-form preserved for String→Char construction)
- HolonAST round-trip: `(holon::Atom \a)` produces HolonAST::Char (via Stone 221.1 + 221.2)
- closure_extract round-trip uses CharLit form

### Stone 223.2 — `WatAST::NilLit` mint + Keyword-to-NilLit migration

`src/ast.rs`:
- Add `NilLit(Span)` variant (no payload — Nil is singleton)
- Doc comment cites arc 153 (`nil` is the singleton type / value) + arc 221 Stone 221.3 (HolonAST::Nil leaf, when shipped)

Consumer migration:
- `src/check.rs` — all sites that match `WatAST::Keyword(":wat::core::nil", _)` for nil-as-value (NOT nil-as-type) → `WatAST::NilLit(_)`
- `src/parser.rs` — empty `()` literal already produces some form (probably `WatAST::List(vec![], span)` or similar) — decide: does it produce NilLit too?
- `src/runtime.rs` — eval arms; many sites probably produce `Value::Unit` from various paths
- `src/closure_extract.rs` — produce NilLit for Value::Unit
- `src/runtime.rs::holon_to_watast` — `HolonAST::Symbol("nil")` → `WatAST::NilLit` (or after Stone 221.3 ships: `HolonAST::Nil` → `WatAST::NilLit`)

Per arc 109 / arc 153: `:wat::core::nil` is the canonical FQDN TYPE form; the VALUE is what changes. Stone 223.2 distinguishes them at the AST level — `WatAST::NilLit` for the VALUE; `WatAST::Keyword(":wat::core::nil", _)` stays valid as the TYPE annotation in a keyword position. The renderer + check decides which is which by context.

Tests in `tests/wat_arc223_nillit.rs`:
- `:wat::core::nil` (as value) parses + evaluates
- `()` empty-tuple literal — confirm what it produces today, preserve or migrate to NilLit
- Round-trip: HolonAST::Nil (when Stone 221.3 ships) ↔ WatAST::NilLit ↔ Value::Unit

### Stone 223.3 — `WatAST::TagLit(String, Arc<WatAST>)` mint + lexer/parser dispatch + data-reader registry

This stone closes the clojure-compat violation. Wat is clojure-on-rust per the 2026-05-22 locked identity; clojure source accepts `#uuid "..."` + `#inst "..."` + `#namespace/Type value` directly. Wat must too.

`src/lexer.rs:~271`:
- Add `#<tag-name>` recognition AFTER the existing `#{` check
- Tag name follows EDN-spec keyword-body rules (already validated: arc 219's `is_symbol_continue` minus colon)
- Emit `Token::TaggedLit(name: String)`
- The body form follows the tag (handled at parser level)

`src/parser.rs`:
- Add `Token::TaggedLit(name) => self.parse_tagged_lit(name, span)` branch
- `parse_tagged_lit` reads one form (the body) and constructs `WatAST::TagLit(name, Arc::new(body), span)`

`src/ast.rs`:
- Add `TagLit(String, Arc<WatAST>, Span)` variant
- Doc comment cites arc 219 (FQDN tag namespacing) + arc 216.7 (encoding doctrine) + the clojure-compat closure

`src/runtime.rs` — data-reader dispatch (Stone 223.3's load-bearing eval logic):
- `eval(WatAST::TagLit(name, body, span)` dispatches by tag name:
  - `"uuid"` → eval body as String → call `:wat::core::Uuid::from-str` equivalent
  - `"inst"` → eval body as String → call Instant constructor
  - `"wat.core/Some"` / `"wat.core/None"` / `"wat.core/Ok"` / `"wat.core/Err"` → construct Option/Result Value variants
  - `"wat.time/Duration"` → mint Duration constructor (per arc 216 Stone 216.9 plan)
  - Unknown tags → diagnostic error (or fall through to a user-extensible data-reader registry — DESIGN-decide)

Consumer cascade (substrate-as-teacher):
- `src/check.rs` — infer arm for TagLit → type depends on tag (uuid → Uuid; inst → Instant; etc.)
- `src/closure_extract.rs` — extract arm for TagLit values
- `src/runtime.rs::watast_to_holon` — `WatAST::TagLit(name, body)` → `HolonAST::Bind(HolonAST::Tag(name), watast_to_holon(body))` (uses Stone 221.3's HolonAST::Tag when shipped; falls back to `Symbol("#name")` if 221.3 not shipped — temporary scaffolding)
- `src/runtime.rs::holon_to_watast` — `HolonAST::Bind(Tag(name), body)` → `WatAST::TagLit(name, holon_to_watast(body))`
- `src/types.rs` (renderers + display) — TagLit description string
- `src/parser.rs` (label renderer) — "tagged literal"
- Any other exhaustive match across the ~1316 WatAST::* sites

Tests in `tests/wat_arc223_taglit.rs`:
- `#uuid "550e8400-e29b-41d4-a716-446655440000"` parses + evaluates to Value::wat__core__Uuid
- `#inst "2026-05-22T..."` parses + evaluates to Instant equivalent
- `#wat.core/Some 42` parses + evaluates to Value::Option(Some(42))
- Unknown tag fails with diagnostic (or routes to data-reader registry per DESIGN decision)
- Round-trip: TagLit → HolonAST → eval → Value

### Stone 223.4 — consumer ripple + cleanup

Sweep `WatAST::*` consumer sites for any patterns the compiler didn't catch (rare with exhaustive match but possible in `_ =>` fallthrough arms):
- Audit `_ => ...` catchall arms in WatAST match expressions for cases where Char/Nil/Tag were silently absorbed
- Update docstrings + examples in `wat/` source that reference the old call-form encoding
- Update USER-GUIDE entries (if any) that reference the call-form
- Add wat-source examples using tagged literals to USER-GUIDE (the clojure-compat now-honest surface)

### Stone 223.5 — INSCRIPTION + cross-references

- `INSCRIPTION.md` — arc 223 closure narrative
- CLIFFNOTES Currently update
- Cross-references to arc 220 (Char minting + List minting), arc 221 (HolonAST primitive-layer leaves), arc 222 (literal-as-surface doctrine consumer), arc 153 (nil canonical form)
- 058 changelog row (the lab's substrate change log)

## What this arc does NOT do

- Add new WatAST variants for collections (Vector / Set / Map literals are first-class via existing parser dispatch + already-existing variants — outside primitive-layer)
- Touch HolonAST (arc 221's job; Stone 223.3's Tag-bridge uses `HolonAST::Tag` from arc 221 Stone 221.3 when shipped, else scaffolds via `Symbol("#name")`)
- Touch wat-edn substrate (untouched; wat-edn already supports tagged-literal wire form)
- Add `#_` discard reader macro, `#"regex"` regex literal, `#(...)` anonymous fn — convenience sugar, not honesty issues; deferred (no current task)
- Modify the `(:wat::core::Char/of "c")` verb — kept for runtime String→Char construction; only the parser dispatch and AST representation change
- Add user-extensible data-readers (clojure has `*data-readers*` for custom tag dispatch) — Stone 223.3 supports the built-in EDN-spec tags + wat-coined tags; user-extensible registry can be a follow-up arc if/when needed

## Calibration (provisional)

| Stone | Predicted | Notes |
|---|---|---|
| 223.1 (CharLit) | 90-180 min | New variant + parser dispatch + ~10-30 consumer cascade sites (substrate-as-teacher) + tests; bigger than Stone 221.1 due to wat-rs's larger consumer surface (1316 WatAST sites vs holon-rs's ~50) |
| 223.2 (NilLit) | 90-180 min | Same shape as 223.1; NilLit has fewer payload-handling sites but more consumer match arms (nil appears EVERYWHERE) |
| 223.3 (TagLit) | 120-240 min | Largest of the three: lexer recognition of `#<name>` + parser dispatch + AST variant + data-reader dispatch for built-in tags (uuid/inst/wat.core/Some etc.) + cross-AST bridge with HolonAST::Tag (Stone 221.3) + consumer cascade. Load-bearing for clojure-compat. |
| 223.4 (ripple) | 60-120 min | Audit fallthrough arms; docstring + example updates; USER-GUIDE additions for tagged-literal source syntax |
| 223.5 (INSCRIPTION) | 30-45 min | paperwork |

**Total estimate:** 6.5-13 hours across 5 stones. Multi-session work due to the consumer-sweep nature.

Per `feedback_substrate_as_teacher`: the cargo test fail-count IS the progress meter. Sonnet iterates through cascades naturally; each round of `cargo test → read → fix → re-run` knocks a category down.

## Unblocks

- Arc 222 ratification of literal-as-surface doctrine — clean WatAST primitive layer makes `{:a true}` direct-to-HolonAST construction straightforward
- USER-GUIDE consistency — explanatory docs no longer need to caveat Char's call-form asymmetry
- Future LLM workflows that generate wat source — uniform primitive-literal handling

## Cross-references

- arc 220 Stone 220.2 (Char Value variant + `\c` lexer token + parser desugaring) — Stone 220.2 chose call-form for scope-minimization; arc 223 lifts the compromise
- arc 220 Stone 220.4 (List minting) — established the "wat-source literal directly to substrate Value" pattern for `'(1 2 3)`
- arc 221 (HolonAST primitive-layer honesty) — same doctrine at holon-rs layer; arc 223 mirrors at wat-rs layer
- arc 222 (EDN ↔ holon direct path) — consumer of arc 223's clean WatAST primitive layer
- arc 109 + arc 153 — canonical FQDN form for nil; arc 223 distinguishes nil-as-value from nil-as-type
- arc 219 (wat-edn strict-EDN keyword namespace) — restricted `#` from is_symbol_continue; explains why TagLit is NOT in arc 223 scope
- `project_wat_reveals_holon` memory — the dynamic operates layer-wise; this is the wat-rs layer version
- `project_3x2_conversion_topology` memory — arc 223's clean WatAST primitive layer is prerequisite for the 3×2 topology's literal-as-surface doctrine

## Open questions

1. **Empty tuple `()` literal** — what's its current AST form? If `WatAST::List(vec![])`, decide: keep it OR migrate to NilLit for unit-value cases. Investigate before Stone 223.2.
2. **Nil-as-type vs Nil-as-value at the AST level** — currently both are `Keyword(":wat::core::nil")` distinguished by context. Stone 223.2 introduces NilLit for the VALUE. Does the TYPE annotation also get a distinct form, or stay as Keyword in type-annotation position?
3. **Migration vs additive** — Stone 223.1 + 223.2 + 223.3 each have an additive variant + a migration ripple. Could split each stone into ADD (mint variant + parser dispatch only; old paths still work) and MIGRATE (sweep consumers to new variant + retire old paths). Decide before Stone 223.1 BRIEF.
4. **TagLit data-reader dispatch boundaries** — Stone 223.3 supports built-in EDN-spec tags (uuid, inst) + wat-coined tags (wat.core/Some, wat.core/None, etc.). Unknown tags: (a) error with diagnostic, OR (b) route to a user-extensible registry (clojure's `*data-readers*` pattern). DESIGN-decide; recommendation: (a) for first ship + (b) as follow-up arc if/when needed.
5. **TagLit-without-body validation** — clojure requires `#tag <form>` (tag MUST be followed by a value). What happens if `#uuid` appears at end of file or before a comment? Error per Clojure precedent. Lexer or parser surfaces?
6. **HolonAST::Tag dependency for Stone 223.3** — Stone 221.3 mints `HolonAST::Tag` as part of Phase B. If 221.3 hasn't shipped when Stone 223.3 runs, Stone 223.3's watast↔holon bridge scaffolds via `Symbol("#name")` then migrates. Or Stone 223.3 waits for 221.3. Decide.
