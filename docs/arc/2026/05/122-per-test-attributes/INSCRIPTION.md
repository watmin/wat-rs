# Arc 122 — INSCRIPTION

## Status

**Shipped + closed 2026-05-01.** Same evening as arc 121
(per-deftest `#[test] fn`). With arc 122 + arc 123
(time-limit) + arc 124 (hermetic + alias), wat deftests reach
100% cargo-test contract parity.

## What this arc closes

Arc 121 made each deftest its own `#[test] fn`. Cargo's filter
/ list / parallelism / failure isolation worked. What arc 121
did NOT cover: per-test attributes — `#[ignore]` and
`#[should_panic]`. The 5% remaining gap.

User direction immediately post-arc-121:

> if its a new arc, then its a new arc - go make it, then we
> close this out as dependent on the new one — 100% close out

Arc 122 closes the gap. After arc 122, every cargo-test
attribute Rust tests carry has a wat-side equivalent; the
proc macro emits the corresponding `#[test]`-level attribute
on the generated function.

## What shipped

**Single file change** — `crates/wat-macros/src/discover.rs`:

- Two new no-op verbs at wat-side (registered in
  `wat/test.wat`): `(:wat::test::ignore "<reason>")` and
  `(:wat::test::should-panic "<expected substring>")`.
- Scanner state in `discover.rs`: `pending_ignore: Option<String>`
  and `pending_should_panic: Option<String>`. When the scanner
  sees one of the two annotation forms, it records the string
  argument as pending. When it sees the next deftest form, it
  attaches the pending fields onto the `DeftestSite` and clears
  them.
- `DeftestSite` struct gained `ignore: Option<String>` and
  `should_panic: Option<String>` fields.
- Codegen in `lib.rs` emits the corresponding Rust attributes
  on the generated `#[test] fn`:
  - `Some(reason) → #[ignore = "<reason>"]`
  - `Some(expected) → #[should_panic(expected = "<expected>")]`
- ~50 LOC scanner addition + ~10 LOC codegen.

End-to-end verification —
`crates/wat-sqlite/wat-tests/arc-122-attributes.wat`:

```
test deftest_wat_tests_sqlite_arc_122_test_arc_122_plain ... ok
test deftest_wat_tests_sqlite_arc_122_test_arc_122_should_panic
  - should panic ... ok
test result: ok. 2 passed; 0 failed; 1 ignored
```

## What got surfaced

The annotation-attachment state-machine (sibling forms
preceding a deftest) became the model for arc 123's
`:time-limit` annotation — same scanner pattern, third
pending field. Arc 124 then extended discovery to alias forms;
the same annotations attached automatically because the alias
catch-all consumed the pending state identically to direct
deftests.

The `:should-panic` annotation became load-bearing for arc
126's slice 2 (6 deadlock-class tests pivoted from `:ignore`
to `:should-panic("channel-pair-deadlock")` once the structural
check shipped). Arc 129 then fixed the timer-wrapper bug
that was eating the panic substring.

The annotation-vs-attribute name distinction the proc macro
maintained (wat-side `:should-panic`, Rust-side `#[should_panic]`)
mirrors how every other arc 122-shipped attribute works. The
substrate doesn't enforce semantics; the codegen lifts the
string verbatim into the Rust attribute and lets cargo libtest
do the work.

## The four questions

**Obvious?** Yes. Per-test attributes are table-stakes for
anyone coming from Rust tests; every existing test framework
has them.

**Simple?** Yes. ~60 LOC of scanner + codegen. No runtime
changes; the codegen attribute does all the work.

**Honest?** Yes. The mechanism is the smallest one that
works: lift the wat string verbatim into the Rust attribute.
No re-validation, no envelope.

**Good UX?** Phenomenal. Authors who know Rust tests already
know what `:ignore` and `:should-panic` do — the names are the
same, the strings are the same. Zero learning curve.

## Cross-references

- `DESIGN.md` — pre-implementation design (status section
  records the closure history).
- `docs/arc/2026/05/121-deftests-as-cargo-tests/INSCRIPTION.md`
  — the parent arc.
- `docs/arc/2026/05/123-time-limit/INSCRIPTION.md` — the
  sibling arc that landed the same evening, reusing the
  state-machine pattern this arc established.
- `docs/arc/2026/05/124-hermetic-and-alias-deftest-discovery/INSCRIPTION.md`
  — extends discovery to four shapes; arc 122's annotations
  attach to all four.
- `docs/arc/2026/05/126-channel-pair-deadlock-prevention/INSCRIPTION.md`
  § "Slice 2" — uses `:should-panic("channel-pair-deadlock")`
  as the test-scaffold mechanism.
- `crates/wat-macros/src/discover.rs` — the file that changed.
- `crates/wat-sqlite/wat-tests/arc-122-attributes.wat` —
  end-to-end verification fixture.
