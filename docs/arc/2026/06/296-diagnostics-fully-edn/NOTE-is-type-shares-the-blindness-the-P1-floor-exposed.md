# NOTE — `is-type?` shares the blindness the P-1 floor exposed

**2026-09-08.** Stone Q shipped `:wat::runtime::is-type?` as *"one authority over three
mechanisms."* Stone P-1's floor proved the authority is over **two stores of at least four**, and
`is-type?` is built on the same pair. Recorded beside Q rather than edited into it.

## The evidence

`src/reflect/verbs.rs:1546` `eval_is_type`, its whole membership test:

```rust
let stripped = type_kw.strip_prefix(':').unwrap_or(&type_kw);
let known = types.contains(&type_kw) || crate::runtime::is_builtin_primitive(stripped);
```

`TypeEnv::contains` (`src/types.rs:601`) is `types ∪ builtin_names`. So `is-type?` asks:

```
TypeEnv.types · builtin_names · is_builtin_primitive       asked
UseDeclarations       (src/rust_deps/mod.rs:290)           NOT asked   :rust::* FFI imports
subtype_edges         (src/types.rs:542)                   NOT asked   derive markers
```

P-1's floor named the consequence in real code: `:rust::test::Greeting` (a `use!`d FFI type) and
`:t::Marker` / `:wat::spawn::Spawned` (derive markers) are all legitimate, usable annotations that
the two-store question calls unknown.

## ⛔ MEASURED 2026-09-08 — and the first version of this section was WRONG

An earlier revision of this NOTE said the probe *"was not run because a peer build held the
binary."* **That reason was false.** My check was `pgrep -f "cargo|rustc"`, which matched two
`~/.cargo/bin/wat --mcp` servers — the wat MCP, whose PATH contains `.cargo` — and my own command
line. No build was running. The pattern matched a population I never inspected, which is this
campaign's disease pointed at my own instrument.
`[[feedback_validate_a_search_pattern_before_trusting_its_count]]`

Run against `target/release/wat` at 15:41 (RELAND-1's build, newer than every source file):

```wat
(:wat::core::use! :rust::sqlite::Connection)
(:wat::runtime::is-type? :rust::sqlite::Connection)   ;; -> false   ⛔ use!'d in this very file
(:wat::runtime::is-type? :wat::spawn::Spawned)        ;; -> false   ⛔ a LIVE derive marker
(:wat::runtime::is-type? :wat::core::Option)          ;; -> true
```

★★★ **The substrate now holds TWO CONTRADICTORY ANSWERS to "is this a type?"** — the annotation
wall's four stores, and `is-type?`'s two. RELAND-1 widened one and not the other, so the disease is
not merely unfixed here; it has been *institutionalized* by the stone that diagnosed it.

## What follows

Whatever union RELAND-1 lands for the annotation wall, **`is-type?` should consume the same one** —
not a parallel copy. Two predicates answering "is this a type?" from different store sets is the
exact shape this campaign keeps finding. `[[feedback_a_name_checked_against_a_partial_set]]`

⚠ And it sharpens Q's own ruling. *"`type-of` is STRUCTURE; `is-type?` is MEMBERSHIP"* still holds —
but membership over WHICH set was never pinned, and "the union" turned out to be a moving number.
`HAERESIS EST ITERVM ROGARE`.

---

## ⬜ A SECOND door, found while verifying store 4

`derive` does not validate its MARKER. `src/types.rs:3410-3421` reads the parent as a bare keyword
and calls `env.register_subtype(&child, &parent, decl_span)` with no membership check. So:

```wat
(:wat::core::derive :t::A :t::Typo)     ;; :t::Typo is now a subtype_edges PARENT
(:user::f [m <- :t::Typo] -> …)         ;; ...and therefore a VALID annotation
```

**Store 4 is a store of unvalidated names.** The annotation wall must ask it — a marker is a real
bound — but one member of the union admits whatever anyone derived from. Not RELAND-1's defect and
not a reason to narrow the union; a door to close where it actually is, in `derive`.

⚠ Untested as written above — derived from reading the registration arm. Write the fixture before
acting on it. `[[feedback_a_failure_to_find_is_not_a_proof_of_absence]]`
