# Arc 109 Slice 1h — Option variants FQDN (`Some` → `:wat::core::Some`, `:None` → `:wat::core::None`)

**Compaction-amnesia anchor.** Read this first if you're picking up
slice 1h mid-flight.

## What this slice does

The Option<T> variant constructors retire as bare-grammar
exceptions; the canonical FQDN forms take over.

| Today | After slice 1h |
|---|---|
| `Some` (bare Symbol at callable head) | `:wat::core::Some` (FQDN keyword) |
| `:None` (bare keyword at value position) | `:wat::core::None` (FQDN keyword) |

Both bare forms appear at TWO call sites each:
- **Constructor position** — `(Some 5)` / `:None` produce values.
- **Match pattern position** — `((Some v) body)` / `(:None body)`
  destructure values.

Both paths need substrate recognition of the FQDN form; both
paths need Pattern 2 poisons on the bare form.

## What this slice does NOT do

- **Result variants (`Ok` / `Err`)** — slice 1i territory.
  Different shape (Some + None mixes Symbol and Keyword paths;
  Ok + Err are both Symbol-headed). Splitting means 1h proves
  both paths separately; 1i is mechanical extension.

## Why split

§ C bundled all four variants at once would test three different
substrate paths simultaneously:

- Symbol-headed-with-payload (Some, Ok, Err)
- Keyword-without-payload (`:None`)

Splitting Option-first means slice 1h exercises **both** the
Symbol-with-payload path AND the Keyword-without-payload path,
landing the substrate work for both before Result piles on. Slice
1i then is just "two more applications of the Symbol-with-payload
mechanism."

If 1h reveals a substrate gap in (say) match-pattern dispatch, it
surfaces in Option-only test surface, not across all
Option+Result code at once. Localizes failure mode.

## The protocol

Pattern 2 (verb retirement) extended across two AST shapes:

- **Symbol-headed** (`(Some v)` and pattern `(Some v)`) —
  synthetic TypeMismatch in the variant-constructor dispatcher
  AND in the match-pattern detection. Both push redirect to
  `:wat::core::Some`.
- **Keyword-headed** (`:None` and pattern `:None`) — synthetic
  TypeMismatch in the keyword-dispatch path. Push redirect to
  `:wat::core::None`.

Plus migration-hint helpers in `collect_hints` for both bare
forms.

This is the **first slice to apply Pattern 2 to AST-grammar
exceptions** rather than to substrate-registered verbs. Bare
`Some` and `:None` are explicitly carved out of wat's general
"callable head must be FQDN keyword" rule (see
WAT-CHEATSHEET § 0 / scratch 009 / the user's "the rule is
types/aliases/functions MUST be symbol quoted" framing). Slice
1h closes that exception for the Option variants; slice 1i
closes it for Result. Post-1h+1i, **the substrate has zero
bare-symbol exceptions** at callable head positions.

## What to ship

### Substrate (Rust)

1. **Recognize FQDN forms** at every dispatch site that today
   matches bare `Some` / `:None`:
   - `src/runtime.rs:2171` (eval_call constructor dispatch) —
     add `WatAST::Keyword(s, _) if s == ":wat::core::Some"`
     arm next to the bare Symbol arm. Add `:wat::core::None`
     keyword recognition wherever `:None` is recognized.
   - `src/runtime.rs:6770` (match-arm pattern dispatch) — same
     additions.
   - `src/check.rs:1821` (`infer_variant_constructor` for Some)
     — accept either bare-Symbol or FQDN-Keyword as the head.
   - `src/check.rs:2058` (let-binding head detection that looks
     for "Some") — accept both forms.
   - `src/check.rs:2267` (MatchShape from variant head) — same.
   - `src/check.rs:2541` (variant-tag rendering for diagnostics)
     — emit FQDN form preferentially in Display output.
   - `src/check.rs:2817` (other variant detection sites) — same.

2. **Poison bare forms** with synthetic TypeMismatch:
   - When `WatAST::Symbol("Some")` is the head at a known
     constructor or pattern site, push synthetic
     `CheckError::TypeMismatch { callee: "Some", expected:
     ":wat::core::Some", got: "Some", ... }` then continue
     dispatching (don't halt; sweep call-by-call).
   - When `WatAST::Keyword(":None")` is recognized as a None
     value or pattern, push synthetic
     `CheckError::TypeMismatch { callee: ":None", expected:
     ":wat::core::None", got: ":None", ... }` then continue.

3. **Migration hint helpers** in `src/check.rs::collect_hints`:
   - `arc_109_some_variant_migration_hint` — fires when callee
     == "Some"; redirect to `:wat::core::Some`.
   - `arc_109_none_variant_migration_hint` — fires when callee
     == ":None"; redirect to `:wat::core::None`.

### Verification

Probe coverage:
- `(Some 5)` → fires (Pattern 2 Some poison)
- `(:wat::core::Some 5)` → silent (FQDN canonical)
- `:None` → fires (Pattern 2 None poison)
- `:wat::core::None` → silent (FQDN canonical)
- `(:wat::core::match opt -> :wat::core::unit ((Some v) ...) (:None ...))`
  → both pattern arms fire poisons
- `(:wat::core::match opt -> :wat::core::unit ((:wat::core::Some v) ...) (:wat::core::None ...))`
  → silent (both FQDN)
- `:my::pkg::Some` → silent (user namespace; head doesn't match
  bare "Some")

Plus Ok/Err sites continue working unchanged (slice 1i territory).

## Sweep order

Same four tiers. Substrate stdlib first.

1. **Substrate stdlib** — `wat/`, `crates/*/wat/`. Substrate
   boots clean before user code sees the new errors.
2. **Lib + early integration tests** — embedded wat strings.
3. **`wat-tests/`** + **`crates/*/wat-tests/`**.
4. **`examples/`** + **`crates/*/examples/`**.

Verification gate after each tier: cargo test --release
--workspace shows zero TypeMismatch errors mentioning bare
"Some" or ":None" before next tier.

## Estimated scope

- Bare `Some` constructors / patterns: many. Every Option-
  returning function call site, every match arm.
- Bare `:None` values / patterns: many. Default values, empty
  cases.

Probably ~600-1500 sites combined across substrate stdlib +
tests + examples. Sonnet's diagnostic-driven sweep keeps pace.

## What does NOT change

- **`Ok` / `Err` (Result variants)** — slice 1i. The walker
  doesn't flag them in slice 1h.
- **Internal Rust string literals** like `WatAST::Symbol(ident,
  _) if ident.as_str() == "Some"` — these are the canonical-form
  internal recognizers. The substrate dispatches against bare
  "Some" string match; that stays. New arms get ADDED for FQDN
  recognition.
- **TYPE annotations** `:Option<T>` / `:wat::core::Option<T>` —
  slice 1e shipped that.
- **The walker logic** (BareLegacyContainerHead etc.) — already
  shipped slices 1c-1g. Slice 1h adds new arms but doesn't
  reshape the mechanism.

## Closure (slice 1h step N)

When sweep is structurally complete:

1. Update `INVENTORY.md` § C — strike `Some` and `:None` rows
   (mark ✓ shipped slice 1h); leave `Ok` / `Err` rows pending
   slice 1i.
2. Update `J-PIPELINE.md` — slice 1h done.
3. Update `SLICE-1H.md` — flip from anchor to durable
   shipped-record.
4. Add 058 changelog row.
5. Note `WAT-CHEATSHEET § 0` (or wherever the bare-symbols
   rule lives) — partial closure of the exception (Option
   variants done; Result still exempt until 1i ships).

## Cross-references

- `docs/SUBSTRATE-AS-TEACHER.md` § "Three migration patterns" —
  Pattern 2.
- `docs/arc/2026/04/109-kill-std/INVENTORY.md` § C — the four
  rows; slice 1h strikes the first two.
- `docs/arc/2026/04/109-kill-std/SLICE-1F.md` /
  `SLICE-1G.md` — Pattern 2 precedent slices.
- `src/check.rs::collect_hints` — where the two new hint
  helpers land.
- `src/runtime.rs::eval_call` + the match-arm dispatcher — the
  two runtime paths that need FQDN keyword recognition.
