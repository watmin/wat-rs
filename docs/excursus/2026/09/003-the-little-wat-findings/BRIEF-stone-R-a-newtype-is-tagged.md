# BRIEF — STONE R: a newtype is tagged, `#ns/Name <inner>`, and reads back as itself

**Drawn 2026-09-25.** Builder: *"they look like records - records must always be tagged .... i don't
know how an untagged thing could ever be emitted..."* — ruling: a newtype is a record-shaped value and
is always tagged.

## Correcting stone Q — the orchestrator's error

Stone Q (`d492a6805`) fixed the F-030 panic by writing a newtype as its BARE inner value (`5`), on the
orchestrator's instruction, citing the typed reader's comment *"the wat-side wrapper is invisible at the
EDN layer"* (`src/edn/render.rs:~2496`, from `8afd78bf9`, arc 170, 2026-05-10). That comment had NEVER
met a writer — until stone Q, writing a newtype panicked. Following it put an untagged aggregate on the
wire, and broke round-trip identity:

```
(:wat::core::defrecord :q::R [v <- :wat::core::i64])   →  "#q/R {:v 5}"    reads back as :q::R anywhere
(:wat::core::newtype   :q::N :wat::core::i64)          →  "5"             reads back as a bare i64 unless the slot is typed
after a wire trip:  (= (:q::N 5) m)  →  TypeMismatch (one side Aggregate, one side i64)   — stone Q executor, measured
```
Arc 237 already treats a newtype as NOMINAL: *"nominal `Path` (struct / newtype) → identity check
(value's tag == name)"*.

## The work

1. **Writer** (`src/edn/render.rs:~4742`, the `Nature::Struct` arm stone Q changed): a newtype renders
   as `Tagged(tag_from_type_path(":{class}"), <inner value's EDN>)` — e.g. `#q/N 5`. Never bare.
2. **Recognising a newtype at the writer.** Stone Q used the structural check `names == ["0"]`.
   Prefer a positive marker that does not need the registry (the writer runs with `types: None` in unit
   tests) — e.g. carry "this is a newtype" on the value from where `eval_struct_new` builds it
   (`src/record/construct.rs:~103`, `Some(TypeDef::Newtype(_)) => names ["0"]`). If no clean marker is
   available, keep the structural check and say why. Report which.
3. **Untyped reader** (`tagged_to_value`, the registry dispatch at `~:3592`): a tag resolving to
   `TypeDef::Newtype(n)` decodes its body as `n.inner` and rebuilds the newtype value exactly as
   `eval_struct_new` builds it. Today a scalar-bodied user tag has no route.
4. **Typed reader** (`edn_to_typed_value_inner`'s `Newtype` arm, `~:2508`): accept `#q/N 5` and rebuild
   the NEWTYPE (not the bare inner). Correct its "invisible wrapper" comment — it is what misled stone Q.
   Whether it should ALSO still accept a bare inner value in a newtype-typed slot: report what reads
   that today, and keep it only if something real depends on it.

## Prove it

- `(:wat::edn::write (:q::N 5))` → `#q/N 5`.
- `:wat::edn::read` of `#q/N 5` → a value `=` to `(:q::N 5)` (untyped path).
- Over a process wire: `(= (:q::N 5) m)` → `true` (stone Q's wire probe measured the mismatch; flip it).
- A newtype over a non-scalar inner (e.g. a Vector or a record) round-trips.
- The F-030 probe still prints without panicking — now `#u/N 5`.
- **Mutation:** write the bare inner again → the round-trip and wire cases red.
- Stone Q's own probe (`tests/comms/probe_ex003_stone_q_printing_a_newtype_does_not_panic.*`) pinned the
  bare form — update it to the tagged form.

## STOP triggers

1. Something in the corpus reads a newtype's bare inner form and would break — report the site.
2. Any gate reddens that you did not add (other than the named stone-Q probe) — capture whole, name the
   arm. ⛔ Do not re-run first.

## Mechanics — ⛔ read

- **`cargo nextest run --release`, never `cargo test`** — focused runs too.
- Floor: `nohup scripts/floor.sh > <file> 2>&1 &`, then repeated foreground
  `until grep -qE '^ *Summary' <that file>; do sleep 30; done` blocks (each under the 600s cap). Do not
  end your turn while it runs. Never `pgrep -f 'cargo …'`. Read `Summary` lines.
- No `cargo fmt` / `rustfmt`. Stage explicit paths, never `git add -A`.
- Test lints: EDN string literals trip `no_inlined_edn` (use `.edn` goldens); `contains`/`ends_with` in
  an assert trips `no_loose_string_assert`; composing `Enum::Variant` with `::` outside
  `identifier.rs` trips `one_variant_separator`.
