# Arc 184 — Bare `:wat::core::try` + `:wat::core::expect` as dispatch primitives

**Status:** stub opened 2026-05-12 per user direction.
**Tracking:** arc 109 v1 milestone blocker (per user direction; gate before 109 closes).

## Motivation

> *"should we have :wat::core::try and :wat::core::expect who
> operate the same on Option and Result? Option/try and
> Result/try both exist are called by the 'unqualified'
> :wat::core::try ? same for expect. this mirrors what we do
> with :wat::core::+"*

Today (post arc 191 / arc 109 slice 1j):
- `:wat::core::Option/try` + `:wat::core::Result/try` — canonical (Type/verb)
- `:wat::core::Option/expect` + `:wat::core::Result/expect` — canonical
- Bare `:wat::core::try` — RETIRED (poisoned `<retired-use-Result/try>`)
- Bare `:wat::core::option::expect` + `:wat::core::result::expect` — RETIRED
  (poisoned `<retired-use-Option/expect>`, etc.)

This arc UN-RETIRES the bare forms — but as DISPATCH primitives,
not as duplicate Type-verb aliases. Mirrors `:wat::core::+` per
arc 148:

- `:wat::core::+` (bare, dispatched across i64/f64/String/Vector/Holon)
  — user-canonical
- `:wat::core::i64::+'2` etc. — Type-explicit, accessible for the
  explicit-when-you-want-it case

Same pattern for try/expect:
- `:wat::core::try` (bare, dispatched across Option/Result) — user-canonical
- `:wat::core::Option/try` + `:wat::core::Result/try` — Type-explicit, stay
- `:wat::core::expect` (bare, dispatched) — user-canonical
- `:wat::core::Option/expect` + `:wat::core::Result/expect` — Type-explicit, stay

## Why this is NOT retirement-theater reversal

Arc 109 slice 1j retired bare `try`/`expect` because the bare names
collided with what was previously a single-type form (Option-only
or Result-only). The Type/verb shape (arc 191) disambiguated.

This arc un-retires the BARE NAMES with a NEW SEMANTIC: type-driven
dispatch via arc 146 mechanism. The dispatch infrastructure didn't
exist when arc 109 § 1j ran; now it does. The un-retirement is an
INTENTIONAL DESIGN PIVOT based on new substrate capability, not a
revisiting of the arc 109 decision.

## The Rust parallel (load-bearing precedent)

Rust's `?` operator:
- `Some(v)?` → `v`; `None?` propagates `:None` (Option)
- `Ok(v)?` → `v`; `Err(e)?` propagates `Err(into(e))` (Result)

ONE syntactic form, dispatched via the `Try` trait. No `Option::?`
or `Result::?` user-facing. The trait is internal substrate
plumbing. Wat's `:wat::core::try` mirrors this exactly — one bare
form, dispatch via arc 146 multimethod, per-Type implementations
remain accessible as `:wat::core::Option/try` / `:wat::core::Result/try`
(unlike Rust where the trait method is hidden).

## Sketch (placeholder; user fills the design)

TBD. Likely:

1. **Substrate**: un-retire the registrations in `src/special_forms.rs`
   for `:wat::core::try`, `:wat::core::option::expect`,
   `:wat::core::result::expect`. Replace poisoned `<retired-use-X>`
   placeholders with proper signatures.
2. **Dispatch wiring**: register Option/try and Result/try as
   dispatch handlers under the bare `:wat::core::try` form (arc 146
   mechanism). Same for expect.
3. **Migration hint walker**: the arc-109-slice-1j migration hint
   functions in `src/check.rs:1253-1295` retire (their job is done
   the moment bare forms work again).
4. **Doc sweep**: USER-GUIDE / SERVICE-PROGRAMS / WAT-CHEATSHEET etc.
   teach the bare forms as canonical. (This is the doc-rot purge
   tracked separately as the Phase G-arc-191-purge candidate.)
5. **Migration hint retirement**: the existing `result::expect` /
   `option::expect` / `try` migration hints in src/check.rs can retire
   once bare forms work — the redirect is no longer needed.

## Open questions for the design

- **Type-checker dispatch order**: arc 146's existing `length` /
  `empty?` etc. dispatch on argument type at check time. `try` /
  `expect` ALSO interact with the enclosing fn's return type
  (propagation target). Verify the dispatch mechanism composes
  with the return-type-aware check; surface any new check-pass
  arm needed.
- **Cross-type propagation**: `(:wat::core::try opt-val)` inside a
  fn returning `:Result<T, E>` — does it auto-promote `None` to
  `Err(some-default)`? Rust uses `From` trait; wat would need an
  equivalent. May be out-of-scope for v1; bare form might require
  the propagation type to match enclosing fn's return-type variant.
- **Migration cost**: substrate disk truth shows ~zero live call
  sites of either bare form (all in docs / wat-scripts as doc rot).
  Migration sweep is nearly free.

## Cross-references

- arc 191 (Option/Result Type/verb method forms — current canonical)
- arc 109 slice 1j (the bare-form retirement this arc un-retires)
- arc 146 (substrate dispatch mechanism — the enabler)
- arc 148 (arithmetic / comparison via dispatch — the precedent)
- arc 178 (primitive Type/fn shape — sibling concern; may fold)
- Rust `Try` trait + `?` operator — the design precedent
- (Pending) Phase G-arc-191-purge — drains the textual residue
  BEFORE this arc; un-retirement assumes clean ground
