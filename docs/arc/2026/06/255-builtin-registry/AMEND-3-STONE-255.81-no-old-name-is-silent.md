# AMEND 3 — STONE 255.81: no old name is silent

**Drawn 2026-10-02.** **Executor: grok via pulsare, working solo** (it runs the floor). Continues at `a15d4800f`
(grok's STOP-2). Commit locally on `main`; **do not push**.

## The two STOP-2 reds, traced by the orchestrator

1. **`probe_arc255_77_framing_floor_pin`: `[38 42 55 24]` → `[18 18 55 19]`.** The i64, f64 and bool branches each lost
   exactly their fixed cost (20, 24, 5); UUID survived. `wat/telemetry.wat:320-323` builds the expected types from
   **strings**: `(:wat::core::type-equal? t (:wat::core::keyword-node ":wat::core::i64"))`. No codemod sees inside a
   string, and with the door deleted, `type-equal?` compares a **retired** spelling and answers `false` without a word.
   The pin did its job: this is the silent-zero trap 255.77 named.
2. **`probe_arc278_6b_ii_b_where_native_differential::native_where_passes`: `UnknownField wat::core::PersistentVector`
   on record `wat::rete::Query`.** The world (`tests/rete/probe_arc278_6b_ii_b_where_native_differential.rs:46`, a
   `format!` template) still writes the constructor **head** `(:wat::core::PersistentVector :- [:wat::rete::Query] …)`.
   After the flip that head names nothing, and instead of "unknown" or "retired" the call is misrouted into a record
   keyword-argument path and fails with a misleading field error.

## The rulings these force

- **A retired name is never silent.** Any runtime door that parses a type from a keyword or a string
  (`type-equal?`, `keyword-node` → type, `metadata-of`/`render-doc` type arguments, the type door wherever eval reaches
  it) **refuses** a retired `:wat::core::<24>` with the retirement remedy, exactly as the checker does. A comparison
  against a retired name must not return `false`.
- **The flip breaks old-spelled constructor heads now, so they convert in this stone** (the brief's "heads are stone 5"
  assumed the old head still dispatched; the code says it does not). `(:wat::core::<24> :- […])` heads become
  `(wat.type/<24> :- […])` by the recorded codemod (rule J, extended as needed) and `wat-fix-rust`. A head that is not one
  of the 24's constructors stays for stone 5.
- **An unknown head says so.** Find why `(:wat::core::PersistentVector :- [T] …)` reached a record keyword-argument path
  and fix the dispatch, so an unknown or retired head raises naming the head (and the remedy), not a field.

## The work

1. **Telemetry:** build the four expected types as data (a quoted `wat.type/X` node, or the type form), not a string;
   the pinned floor must read `[38 42 55 24]` again. Then census **every** string in `.wat` and embedded wat that carries
   one of the 24's old names to a type door (`keyword-node`, `read-string`, `parse`-like verbs, `metadata-of`,
   `render-doc`), and convert each to data or the new spelling.
2. **The runtime refusal** above, with a test: `(:wat::core::type-equal? t (:wat::core::keyword-node ":wat::core::i64"))`
   raises naming `wat.type/i64`.
3. **The driver miss:** say why `wat-fix-rust` did not convert the rete `format!` template (the `{threshold}` and
   `{{…}}` placeholders, or the candidate test), fix the driver if it is the driver, and re-run it.
4. **The constructor heads** of the 24, converted (`.wat` and embedded), and the dispatch cure above.
5. **The six census flips** (`wat.core/<24>` symbol spellings, a metadata `:ret` vector, `:wat::core::char` passed to
   `metadata-of`/`render-doc`, a quoted `:wat::core::List`): each converted by a codemod rule, or shown to be a test whose
   subject is the old spelling.
6. **Then** the remaining printed-spelling reds, re-captured only under amendment 2's as-data audit, and the floor.

## STOPs

- **STOP-5:** the pinned framing floor does not return to `[38 42 55 24]` once the types are data. Quote it and STOP.
- **STOP-6:** making a retired name refuse at run time breaks a program that never named one (a false positive). Quote it
  and STOP.
- The earlier amendments' STOPs stand. A STOP means STOP.

## Doctrine

As before. Equality is data equality; a type is never named by a string to a comparison.
