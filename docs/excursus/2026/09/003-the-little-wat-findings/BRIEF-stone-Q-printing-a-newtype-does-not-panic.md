# BRIEF — STONE Q: printing a newtype does not panic (the-little-wat F-030), and the writer never panics on a name

**Drawn 2026-09-25.** Found by the "errors that point into our source" census, which bucketed it as
"no :file at all" — because a Rust panic has no span. Under the builder's standing ruling (*"the least
amount of panics possible"*) it outranks every location fix.

## The defect, driven at `587f9bd78`

`the-little-wat/probes/ml/newtype-value.wat`:
```wat
(:wat::core::newtype :u::N :wat::core::i64)
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:u::N 5)))
```
```
thread 'main' panicked at crates/wat-edn/src/value.rs:330:33:
invalid keyword name "0": first character must be non-numeric
note: run with `RUST_BACKTRACE=1` …
[#wat.kernel/LociDiedError.Panic … :failure #wat.core/Option.None {}]
```

## Two layers

### 1. The case — a newtype writes as its inner value (the record already decided this)

The writer's `Value::Aggregate … Nature::Struct` arm (`src/edn/render.rs:~4697-4712`) renders a
newtype like a struct: a map keyed by field NAMES — and a newtype's one field is named `0` (its
accessor is `<Name>/0`). `:0` is not a legal EDN keyword, so `Keyword::new` panics.

⭐ **Do not invent a representation.** The typed READER already states the contract
(`src/edn/render.rs:~2496-2510`): *"newtypes coerce against their inner declared shape (the wat-side
wrapper is invisible at the EDN layer)"* — a `:u::N`-typed slot reads the INNER value's EDN. So the
writer renders a newtype value as **its inner value's EDN** (`5`), which round-trips through that slot.
Find how a newtype value is told apart from a one-field struct at runtime (registry via `types`, a
marker on the value, …) and report it.

### 2. The class — the writer never panics on a name

`wat_edn::Keyword::new` is DOCUMENTED to panic on an invalid name (`crates/wat-edn/src/value.rs:~324`);
`Keyword::try_new` exists (`:~265`, `:~352`). `src/` calls `Keyword::new` **58 times**; several take a
name from DATA (`name.clone()`, `n.clone()`, `k.clone()` — `render.rs:~4707, 4722, 4743, 4758, 4888`).
Any name that is not a legal EDN keyword is a reachable panic; F-030 is only the one someone hit.

- Census the 58: split them into **literal names** (`Keyword::new("value")` — cannot fail; leave) and
  **names from data** (can fail). Report the split.
- Every data-named call in the WRITER uses `try_new` and turns a failure into a refusal value
  (`WireEncodeError`, the writer's existing error path — `value_to_edn_in` already returns `Result`),
  naming the offending name and the type it came from. Never a panic, never a silent rename.
- Data-named calls OUTSIDE the writer: report them; fix any that are reachable from a wat program.

## Prove it

- The probe: prints `5`, exit 0, empty stderr.
- A newtype round-trips: write it, read it back through a `:u::N`-typed slot (e.g. `:wat::edn::validate`
  or a typed record field) → equal.
- A newtype crosses a process wire and arrives equal (model: stone M's wire probe).
- A struct/record whose field name is not a legal EDN keyword — find one the checker admits, or state
  that none can be declared — is refused as a value, not a panic.
- **Mutation:** restore the struct-arm treatment for newtypes → the probe panics again.

## STOP triggers

1. A newtype cannot be told apart from a one-field struct at the writer — report what IS available.
2. Something reads the old (never-successful) newtype map form — report it.
3. Any gate reddens that you did not add — capture whole, name the arm. ⛔ Do not re-run first.

## Mechanics — ⛔ read

- **`cargo nextest run --release`, never `cargo test`** — focused runs too (`-E 'test(/name/)'`).
- Floor: `nohup scripts/floor.sh > <file> 2>&1 &`, then repeated foreground
  `until grep -qE '^ *Summary' <that file>; do sleep 30; done` blocks (each under the tool's 600s
  cap). Do not end your turn while it runs. Never `pgrep -f 'cargo …'`. Read `Summary` lines.
- No `cargo fmt` / `rustfmt`. Stage explicit paths, never `git add -A`.
- Test lints: an EDN string literal trips `no_inlined_edn`; `contains`/`ends_with` in an assert trips
  `no_loose_string_assert`. A test that compares full error `Display` text can be pinning an accident
  (stone P found one) — compare the part that is the invariant.
