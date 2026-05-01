# Arc 109 Slice 1i — Result variants FQDN (`Ok` → `:wat::core::Ok`, `Err` → `:wat::core::Err`)

**Status: shipped 2026-05-01.** Walker (commit `35e44dc`) +
four-tier sweep across commits `5eff224` → `d04ecfc` →
`6ac06c2` → `c7ab499`. 39 files swept; **~337 rename sites
total** (280 patterns + 57 constructors — lower than the ~500
estimate because most substrate stdlib paths use match patterns,
not constructors, and constructor-heavy code lives in tests).
Zero MANUAL flags. cargo test --release --workspace 1476/0.

**Two substrate-gap fixes during sweep** (mirrored slice 1h's
gap pattern):

1. **`src/check.rs`** — MatchShape detection at the
   `pat_items.first()` keyword arm. Slice 1h's `// slice 1i will
   add` comment marker became literal: `:wat::core::Ok` /
   `:wat::core::Err` keyword recognition added to the FQDN
   keyword path. Without this, every `((:wat::core::Ok ...))`
   pattern defaulted to Option shape and cascaded TypeMismatch
   errors in `wat/std/stream.wat`.

2. **`src/runtime.rs`** — `try_match_pattern` list-pattern
   dispatcher. Extended bare-Symbol `Ok`/`Err` arms to also
   accept `:wat::core::Ok` / `:wat::core::Err` keywords. Without
   this, FQDN match patterns fell through to the user-enum
   keyword arm and never matched the Result value, causing
   `PatternMatchFailed` at runtime.

**Architectural milestone shipped**: § C is now structurally
complete. Post-1h+1i, the substrate has **zero bare-symbol-at-
callable-head exceptions**. The "callable heads must be FQDN
keywords" rule is universal. Mechanical extension worked exactly
as planned — slice 1h proved both substrate paths (Symbol +
Keyword); slice 1i was just two more applications of the
Symbol-headed-with-payload mechanism.

**Originally drafted as a compaction-amnesia anchor mid-slice;
preserved here as the durable record. Pattern 2 mechanism (verb
retirement via synthetic TypeMismatch) is now thoroughly
proven across both verb retirements (1f/1g) and AST-grammar
exceptions (1h/1i). Reusable for any future similar work.**

## What this slice does

The Result<T,E> variant constructors retire as bare-grammar
exceptions; canonical FQDN forms take over. Same Pattern 2
mechanism slice 1h applied to Option (Some/:None), now applied
to Result (Ok/Err).

| Today | After slice 1i |
|---|---|
| `Ok` (bare Symbol at callable head) | `:wat::core::Ok` (FQDN keyword) |
| `Err` (bare Symbol at callable head) | `:wat::core::Err` (FQDN keyword) |

Both bare forms appear at TWO call sites each:
- **Constructor position** — `(Ok x)` / `(Err e)` produce values.
- **Match pattern position** — `((Ok v) body)` / `((Err _) body)`
  destructure values.

## Why this is mechanical

Slice 1h proved both substrate paths needed for variant FQDN:
- Symbol-headed-with-payload (Some)
- Keyword-without-payload (:None)

Ok and Err are BOTH Symbol-headed-with-payload — same shape as
Some. Slice 1i is two more applications of the Symbol-with-payload
mechanism. **Zero new substrate paths, zero new mechanism.**

The runtime FQDN dispatch arms for `:wat::core::Ok` and
`:wat::core::Err` already shipped in slice 1h's commit
`016b3a3` (added for dispatch-table consistency). Slice 1i just
adds:

1. **Pattern 2 poisons** at `infer_variant_constructor` for
   bare `Ok` and `Err` (synthetic TypeMismatch + redirect).
2. **Migration hint helpers** (`arc_109_ok_variant_migration_hint`,
   `arc_109_err_variant_migration_hint`).

That's it. Walker mechanism is unchanged from slice 1h.

## Closes the bare-symbols-at-callable-head exception

Post-slice-1h+1i, the substrate has **zero bare-symbol exceptions**
at callable head positions. The "callable heads must be FQDN
keywords" rule (WAT-CHEATSHEET / scratch 009) becomes universal,
no carve-outs.

## What this slice does NOT do

- Already shipped in slice 1h:
  - `:wat::core::Ok` / `:wat::core::Err` keyword recognition at
    runtime constructor + match-pattern dispatchers
  - list-pattern head dispatcher accepts FQDN keyword forms for
    Ok/Err (continuity with Some)
- The **render_value FQDN flip** is task #189 (deferred, not in
  this slice).

## What to ship

### Substrate (Rust)

1. **Poison bare `Ok`** in `src/check.rs::infer_variant_constructor`:
   add synthetic TypeMismatch when the Symbol-headed `Ok` arm
   matches; continue dispatching so the program type-checks.
   Mirrors slice 1h's `Some` poison.

2. **Poison bare `Err`** identically — synthetic TypeMismatch +
   redirect to `:wat::core::Err`.

3. **Migration hint helpers** in `src/check.rs::collect_hints`:
   - `arc_109_ok_variant_migration_hint` — fires on callee == "Ok"
   - `arc_109_err_variant_migration_hint` — fires on callee == "Err"

That's all. The `:wat::core::Ok` / `:wat::core::Err` FQDN
recognition arms are already in place from slice 1h's
substrate-gap fixes (`pattern_coverage`, `check_subpattern`,
`is_match_canonical` all updated).

### Verification

Probe coverage:
- `(Ok 5)` → fires (bare Ok poison)
- `(:wat::core::Ok 5)` → silent (FQDN canonical)
- `(Err :reason)` → fires (bare Err poison)
- `(:wat::core::Err :reason)` → silent
- Match arms with bare patterns → fire
- Match arms with FQDN patterns → silent

## Sweep order

Same four tiers as slice 1h.

1. **Substrate stdlib** — `wat/`, `crates/*/wat/`.
2. **Lib + early integration tests** — `<test>:N:M` source.
3. **`wat-tests/`** + **`crates/*/wat-tests/`**.
4. **`examples/`** + **`crates/*/examples/`**.

## Estimated scope

Ok/Err sites: every Result-returning function call site, every
match arm. Probably similar to slice 1h's Some count (~250 each).
Total ~500-800 sites.

## What does NOT change

- **Internal Rust string literals** (`"Ok"` / `"Err"` /
  `WatAST::Symbol(ident, _) if ident.as_str() == "Ok"`) — canonical-
  form internal recognizers. Don't touch.
- **Some / :None** — slice 1h territory; already shipped.
- **The walker logic** — already covers the AST shapes.
- **TYPE annotations `:Result<T,E>`** — slice 1e shipped that.
- **Variants in user-defined enums** — not the substrate Ok/Err.

## Closure (slice 1i step N)

When sweep is structurally complete:

1. Update `INVENTORY.md` § C — strike `Ok` and `Err` rows; mark
   ✓ shipped slice 1i.
2. Update `J-PIPELINE.md` — slice 1i done.
3. Update `SLICE-1I.md` — flip from anchor to durable
   shipped-record.
4. Add 058 changelog row noting **§ C is now structurally
   complete** AND **the substrate's bare-symbol-at-callable-head
   exception is universally closed**.
5. WAT-CHEATSHEET update if § 0 / § 3 mentioned the exception.

## Cross-references

- `docs/arc/2026/04/109-kill-std/SLICE-1H.md` — the precedent
  slice; same Pattern 2 mechanism applied to Symbol +
  Keyword shapes. Slice 1i is mechanical extension.
- `docs/arc/2026/04/109-kill-std/INVENTORY.md` § C — the four
  rows; slice 1i strikes the last two.
- `src/check.rs::collect_hints` — where the two new hint helpers
  land.
