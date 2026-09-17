# DESIGN — `.wat.bad` is an unenforced claim, and 16 files carrying it do not fail

> ⛔⛔ **THIS DESIGN REPLACES A FIRST DRAFT THAT WAS MEASURED WITH THE WRONG DRIVER.** The first
> version is preserved as the lesson, in § *What the wrong instrument said*, because the correction
> is the most useful thing here.

## Why

Work-list **C18**: *"`assert!(!ok)` is unfalsifiable in this repo's negative-fixture idiom — every
`.wat.bad` ends `(:user::main [] -> nil nil)`, which is itself a startup failure, so the `!ok` half
cannot go red under the very mutation it exists to detect."*

**Measured with the driver the tests actually use, the rowed mechanism is nearly empty and a
different, larger defect is sitting beside it.**

## ⛔ WHAT THE WRONG INSTRUMENT SAID — the correction is the finding

The first draft of this strike drove every fixture through **`./target/release/wat <file>`** and
reported *"17 fixtures fail with `MainSignatureError`; 230 of 281 have no main; 200 tests are
unfalsifiable."* **All of that describes a path the test suite never takes.**

Builder, 2026-09-03:

> *"we only need a `:user::main` declared if and only if the wat binary is going to eval it … the
> rust test runners can construct a world and invoke some func at their will … this has been our
> pattern"*

Confirmed at `src/freeze.rs:942` — the main check is **conditional on a main being declared**:

```rust
// Conditional on `:user::main` being declared at all — `startup_bare()` (no main) passes cleanly.
if world.symbols().get(":user::main").is_some() {
    validate_user_main_signature(&world).map_err(StartupError::MainSignature)?;
    validate_user_main_not_useless(&world).map_err(StartupError::MainSignature)?;
}
```

The **binary** demands a main because it *evals* one. **`startup_from_file` does not.** And
**577 test files use `startup_from_file`/`startup_beside`; 8 shell out to the binary.**

So a missing main is not a failure mode for the tests, the 230 no-main fixtures are **falsifiable**,
and the first draft's headline was an artifact of the instrument. Same 16 files, two drivers,
opposite verdicts.

## What the RIGHT driver says

Every `.wat.bad`, through `startup_from_file`:

| outcome | count |
|---|---|
| failed for its own reason | **263** |
| `MainSignature` — the rowed mechanism | **2**, and both are `wat_arc170_slice_1e_user_main_nil_*`, whose subject **IS** the main signature |
| ⛔ **`Ok` — DID NOT FAIL AT ALL** | **16** |

**C18's rowed mechanism is 2 files and both are legitimate. The real defect is the 16.**

## The finding: an extension that claims something nothing checks

`.wat.bad` is a claim — builder's own definition: *"we use `.wat.bad` for tests that ensure files
fail to parse, correctly."* **Nothing enforces it.** No lint reads the corpus for that property;
`diagnostic_output_is_deterministic.rs` reads these files for byte-stability, not for failure.

The 16 divide into two honest kinds, and **neither is a bug in the test** — the tests are right and
the *filenames* lie:

1. **Premise retired, test correctly flipped.** `probe_arc237_8a_no_implicit_coercion.rs:113` is
   `arith_f64_i64_mixed_coerces_to_f64`, asserting **`is_ok()`**: *"f64 + i64 now coerces to f64
   (arc 300 C4 retired 237.8a's reject)"*. The fixture is a **positive** fixture wearing `.wat.bad`.
2. **Fails at EVAL, not at startup.** `probe_diagnostic_dynamic_keyword_invocation.rs:141` —
   *"non-keyword head (String) must error **at eval**; got Ok"* — starts the world up, then invokes.
   Exactly the builder's pattern. The file is a valid program; `.wat.bad` is the wrong extension.

## The contract decision, pinned

**Give `.wat.bad` an enforced meaning, and rename the files that do not hold it.**

- **`.wat.bad` means `startup_from_file` returns `Err`.** One gate walks the discovered corpus and
  fails any file that returns `Ok`, printing the path and what the file did instead.
- **The 16 are renamed to `.wat`** — with their test references updated — because each is either a
  positive fixture or a valid program that fails later. A file that starts up clean is not "bad".
- **The 2 `MainSignature` fixtures stay.** They fail at startup for their own declared reason; the
  main signature IS their subject.

This is the extirpare ladder's middle rung: a naming *convention* nothing checks becomes a *check
that fires at construction*. The top rung — an extension that cannot be applied to a passing file —
is not reachable in a filesystem, so this is the highest rung the material allows, and it is stated
rather than implied.

## Out of scope = REJECTED

- **Anything about `:user::main` in fixtures.** The first draft's whole premise. Tests construct a
  world and invoke at will; a fixture needs a main only if the binary will eval it.
- **Rewriting the 200 `assert!(is_err())` call sites.** They are falsifiable already under the real
  driver — that was the first draft's error, and the successor it proposed is withdrawn with it.
- **Changing what any of the 16 tests assert.** The tests are correct. Only the filenames are wrong.
