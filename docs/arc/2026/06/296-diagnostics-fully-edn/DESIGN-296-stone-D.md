# Stone D — the read side: `#[derive(Edn)]`, register the emitted vocabulary, close the round-trip

**Arc 296, the spine we skipped.** Stones A/B made wat's diagnostics EDN on the
**emit** side and I wrongly called R1 done. The builder caught it: wat emits a
whole vocabulary of tags (`#wat.core/Span`, `#wat.core/Pos`, `#wat.core.Option`,
the 11 error families) it **cannot read back**. "Diagnostics *fully* edn" means
the round-trip. Stone D closes it, and only then does **R1 *NE SIBI OBSOLESCAT*
→ PROBATVM EST**.

## The visible symptom (the closing condition)

`edn::read "#wat.core/Span {…}"` → `unknown tag … no matching struct or enum`.
And, downstream, a child process's death carries its error **as a string**:

```clojure
#wat.kernel/ProcessPanics [ #wat.kernel.ProcessDiedError/RuntimeError [ "…child error EDN AS A STRING…" ] ]
```

`ProcessDiedError`'s variants carry `message: :wat::core::String` (types.rs) — a
string-that-is-EDN in a vec, R1's exact catch, at the process seam. It stays
stringly **because** the parent can't parse the child's error EDN back (the read
gap). **Closing condition for stone D: `#wat.kernel/ProcessPanics` comes back as
nested EDN — zero strings-that-are-EDN.**

## What the crawl + probe proved

- The reader (`reconstruct_record`, edn_shim.rs:2456) is **generic and already works**: it reads any *registered* type's `#ns/Name {…}` into a wat record `Value`, walking the field schema, `rewrap_option_field` for `Option<T>`.
- A registered record **round-trips** (`edn::write`→`edn::read` = `true`, proven).
- **The entire gap is registration.** Probe: hand-registering `:wat::core::Pos` (a `Holder::Record` builtin, like the existing `:wat::kernel::Location`) made `edn::read "#wat.core/Pos {…}"` reconstruct it. GREEN.
- Registration mechanisms exist: `TypeEnv::with_builtins` / `register_stdlib_types` (hand), and a link-time **`inventory`** registry already drained in `freeze/env.rs:200` (used for `RestrictionEntry`). A derive can `inventory::submit!` its schema.

## The pinned decisions

- **D1 — `#[derive(Edn)]` replaces `#[derive(ToEdn)]`.** ONE derive, both faces: it emits the write impl (today's ToEdn) AND `inventory::submit!`s the type's schema (name + fields) so the read path finds it. The trait/derive name says it: a wat type **is** Edn (round-trips), by construction.
- **D2 — the wall: no write-only.** Deriving Edn gives the round-trip; there is no write-only derive. Emitting a tag you can't read back has no form (a genuinely-unreconstructable type opts out by hand-impl, the rare exception — never the default). *FACTVM NON PACTVM* on the round-trip.
- **D3 — light them ablaze; they self-identify.** Change `ProcessDiedError` (and any error-chain field that holds serialized EDN) from `String` → a nested round-trippable error value. The compile cascade screams at every construction site (`process_died_error_runtime(message: String)`, …) — the substrate identifies them; we do not audit. At the process boundary the parent `edn::read`s the child's error string (now possible) into the typed value.

## Rooms / scope

1. `crates/wat-to-edn-derive/` — the derive gains the register-half: emit `inventory::submit!(EdnSchema { tag, fields: [(name, type_path)] })` alongside the existing write impl. Rename the derive/trait `ToEdn` → `Edn` (the write method stays; the type is now round-trip).
2. A link-time `EdnSchema` inventory type + a drain in `freeze/env.rs` (mirror `RestrictionEntry`) that `register_builtin`s each schema into the `TypeEnv`.
3. `crates/wat-reader/src/span.rs` — `Span`/`Pos` flip `#[derive(ToEdn)]` → `#[derive(Edn)]` → they register + read. (The probe's hand-registered `:wat::core::Pos` in types.rs is then removed — the derive does it.)
4. The 11 error families flip to `#[derive(Edn)]` → they register + read.
5. `ProcessDiedError` (types.rs:960 + `runtime.rs` builders `process_died_error_*`) — variant fields `String` → nested error value; construction sites re-nest via `edn::read`; the compile cascade drives the sweep.
6. `Option`/`Result` already read (tagged_to_value hand-cases) — leave; or fold into the schema path later (out of scope here).

## STOP triggers (rejection criteria)

- **STOP-1:** if `inventory::submit!` from the proc-macro crate cannot be drained into the `TypeEnv` (link-time visibility / ordering), STOP — report; fall back to hand-registration in `register_stdlib_types` is NOT allowed silently (it re-opens the write/read disconnect).
- **STOP-2:** if a field type in a derived schema has no `TypeExpr` mapping (an exotic Rust type with no wat type), STOP — surface it; that type isn't round-trippable and needs a decision.
- **STOP-3:** if re-nesting `ProcessDiedError`'s field requires reading an error the registry still can't (a family not yet flipped to Edn), STOP — that's the ordering; flip the family first.
- **STOP-4:** a genuinely-unreconstructable type (opaque handle) must NOT be forced to derive Edn — it stays hand-`ToEdn` or `#wat-edn.opaque`. Do not break the opaque refusal path.

## Expectations (scorecard)

| # | what | command | expected |
|---|---|---|---|
| 1 | wat reads its own tags | `edn::read "#wat.core/Span {…}"`, `#wat.check/CheckErrors {…}` | reconstruct to records, no "unknown tag" |
| 2 | Span/Pos/errors round-trip | `edn::write` → `edn::read` = equal | `true` |
| 3 | **THE closing condition** | force a child ProcessPanics; read the envelope | nested EDN — NO strings-that-are-EDN in the vec |
| 4 | the wall holds | a type that derives Edn is registered | grep the inventory drain — every Edn type present |
| 5 | full suite green | `cargo nextest run` (capture ONCE to a file; grep the file) | 0 new failures (save the 7 wat_dispatch flakes) |
| 6 | no write-only remains | grep `derive(ToEdn)` | empty (all → `derive(Edn)`), except sanctioned opaque opt-outs |

## On landing

**R1 *NE SIBI OBSOLESCAT* → PROBATVM EST — for real.** Errors are EDN through the
process seam, not just within one process. wat reads its own vocabulary. ednq
becomes `read-flat` + `pprintln` for free. And the "wat is EDN" thesis is whole:
write AND read, structural on both sides.

## Reference

- The green probe: `:wat::core::Pos` hand-registered (types.rs) → `edn::read` reconstructs it.
- The round-trip proof: a defrecord round-trips `true`.
- `:wat::kernel::Location` (types.rs:1010) — the existing file/line/col record, the exact shape Span/Pos take.
- `RestrictionEntry` (freeze/env.rs:200) — the proven `inventory` link-time registration pattern the derive mirrors.
