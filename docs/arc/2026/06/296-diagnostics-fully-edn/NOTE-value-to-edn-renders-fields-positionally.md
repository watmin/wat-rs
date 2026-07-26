# NOTE — one value, two EDN faces: `value_to_edn` renders record fields POSITIONALLY (`:field-0`) where the hand-written encoders render them NAMED

> **Deferred design decision (builder, 2026-07-25, arc 278 → owned by 296).** Surfaced in the vacuous-gate
> wall's own output (`91bbb8cd`): a mutated gate's failure printed the SAME `Failure` value twice in one
> terminal, once named and once positional. Builder: *"this was very confusing to see… is this the rust Debug
> => EDN thing doing it? the stderr looked reasonable for the actual test output."* **Answer: no — it is not
> the `Debug`/`Display` → `to_wire_edn` path.** It is `value_to_edn`'s field-name resolution falling back to
> positional keys. Recorded per the arc-109 `NOTE-*.md` convention.

## The wart

The same `:wat::kernel::Failure` value reaches the terminal through two different encoders that disagree
about whether records have field names.

**NAMED — `src/panic_hook.rs:196-205`** (`payload_to_edn`). A bespoke encoder that **hand-writes** its keys as
literal keywords:
```rust
OwnedValue::Map(vec![
    (OwnedValue::Keyword(Keyword::new("message")),  message_val),
    (OwnedValue::Keyword(Keyword::new("location")), location_val),
    (OwnedValue::Keyword(Keyword::new("actual")),   actual_val),
    …
])
```
→ `#wat.kernel/AssertionFailure {:thread … :message "assert-eq failed" :location #wat.kernel/Location
{:file … :line 33 :col 5} :actual "1" :expected "4242" :frames […]}` — readable, navigable.

**POSITIONAL — `src/freeze.rs:757`** (`DeftestOutcome::expect_passed` → `value_to_edn_string(&failure)`),
whose key generation is `src/edn_shim.rs:2273` (and the same fallback at `:2284`, `:2317`, `:3250`, `:3260`):
```rust
_ => (0..sv.fields.len()).map(|i| format!("field-{}", i)).collect(),
// and: .unwrap_or_else(|| format!("field-{}", i))
```
→ `#wat.kernel/Failure {:field-0 #wat.core/Fault {:field-0 "assert-eq failed" :field-1 #wat.kernel/Location
{:field-0 … :field-1 33 :field-2 5} :field-2 []} :field-1 [#wat.kernel/Frame {:field-0 … :field-2
":wat::test::assert-eq"}] :field-2 #wat.core.Option/Some ["1"] …}`

Same data. One face is EDN a human or a rule can navigate by key; the other forces the consumer to know the
declaration order of every record it meets. `:field-0` is a **positional index wearing a keyword's costume** —
the same anti-pattern this arc keeps killing, one layer down from
`NOTE-anon-fn-identity-structured-not-stringy.md`'s `<fn@…>` string.

## The symptom that surfaced it (twice, independently)

1. **The vacuous-gate wall's panic output** (this note's origin) — both faces printed adjacent in one terminal,
   which is what made it visible at all. The stderr envelope reads fine; the panic payload directly beneath it
   is the positional blob.
2. **Cache Stone 1** (`a86f521c`) — the rider hit the same seam independently on a *generic* `defrecord`:
   `value_to_edn` rendered `:wat::cache::Entry<K,V>` positionally, and `(:wat::core::show …)` rendered it
   `wat::cache::Entry{#0: :a, #1: 1}` — **while the kwargs constructor and the `Entry/key` accessor resolved
   the field names correctly at the same time.** So the names exist and are reachable; these two rendering
   paths do not reach them.

That second data point is the load-bearing one: this is not a `Failure`-specific quirk, and it is not that the
names are unavailable in principle.

## What was NOT investigated (deliberately — the builder deferred the chase)

**Why the `_ =>` fallback fires for these values.** Whether the struct-kind genuinely carries no resolvable
field names at that point in the pipeline, or whether the name lookup simply is not wired into this path, is
**ungrounded**. Do not carry the guess forward — read `edn_shim.rs:2260-2330` and the `sv` (struct value)
name source before drawing the fix.

## The fix (deferred)

Make record fields render by **name** through the generic `value_to_edn` path, so there is ONE EDN face per
value. Weigh at draw time:
- Resolve field names from the registered type (the accessors already do it) rather than falling back to
  `field-{i}`; keep positional only for genuinely anonymous/positional aggregates (tuples), where an index IS
  the honest key.
- If a name is genuinely unresolvable, that is a **failure to surface**, not a silent degradation to indices —
  the no-hidden-failures law applies to the encoder too.
- With the generic path correct, the bespoke hand-written encoders (`payload_to_edn` and kin) become
  candidates for deletion — they exist partly *because* the generic path renders badly. That is the real
  prize: one encoder, one face.

## Why this belongs to 296

The user experience 296 owes is *everything in the error delivery path is EDN all the things — backtraces are
a vec of EDN*. A backtrace whose frames are `{:field-0 … :field-1 …}` is EDN in shape and unusable in
practice. This is the same class as the arc's other findings (structured data that is technically present but
not addressable), and it is squarely in the error path.

## Status

**DEFERRED**, builder-ruled 2026-07-25 (*"i don't know if i care to chase this now"*). Grounded:
`src/panic_hook.rs:196-205` (named), `src/freeze.rs:757` → `src/edn_shim.rs:2273`/`:2284`/`:2317`/`:3250`/`:3260`
(positional). Observed live in the `91bbb8cd` gate output and independently in `a86f521c`. **Ruled out:** the
`Debug`/`Display` → `to_wire_edn` path (arc 296 Stone B) is NOT the cause.
