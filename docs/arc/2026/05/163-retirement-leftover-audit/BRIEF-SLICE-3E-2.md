# Arc 163 Slice 3e CONTINUATION BRIEF — drive to 2041/0 via substrate-as-teacher iteration

**Drafted 2026-05-07.** Slice 3e first sweep landed substrate-internal `head: "X"` writes to FQDN form (commit at `334f61a`/`fa544a8`; sonnet `a106fae30415c38a5` shipped 4 files modified — src/check.rs, src/runtime.rs, src/types.rs, src/edn_shim.rs). Then orchestrator deleted vestigial typealiases for Option/Result/HashMap/HashSet/Vector at `src/types.rs` (pre-existing self-reference loop after head-FQDN landed).

Build is clean. Workspace test count: **848 failed / 1193 passed / 1 ignored** vs baseline 2041/0. The remaining gap surfaces the substrate's hidden second surface — every system that bridges between wat type expressions and Rust runtime values that previously used the legacy bare form (`"Vec"`, `"Option"`, etc.).

This continuation BRIEF closes the gap by pushing the FQDN-everywhere rule across the remaining categories, then iterating from the diagnostic stream until 2041/0.

## The discipline — substrate-as-teacher (READ FIRST)

Per `docs/SUBSTRATE-AS-TEACHER.md` + `docs/arc/2026/04/111-result-option-recv/REALIZATIONS.md`:

> The substrate's compiler IS the brief. The diagnostic stream encodes the migration path. Sonnet's brief collapses to: "run cargo test; read the errors; apply the rule; iterate until green."

> The error count is the progress meter. Watching cargo test fail count drop is watching the migration converge.

This BRIEF gives you the **rule** and the **categories**. The diagnostic stream gives you the **per-site fixes**.

## The rule

**Every wat-internal substrate-side string identifier that names a wat type or value must be FQDN.** No exceptions. Per user direction 2026-05-07:
- *"wat internals are fully qualified - no exceptions"*
- *"if there's a short form - its illegal"*
- *"if the internal code is mapping to a rust primitive then we use the rust form"*

| Wat-internal storage | Pre-3e (legacy) | Post-3e (canonical) |
|---|---|---|
| `Parametric.head` | `"Vec"`, `"Option"`, `"Result"`, `"HashMap"`, `"HashSet"` | `"wat::core::Vector"`, `"wat::core::Option"`, etc. |
| `Value::type_name()` | `"Vec"`, `"Option"`, `"Result"` (mixed with FQDN for some types) | `"wat::core::Vector"`, `"wat::core::Option"`, etc. |
| value_tag matching | RIGHT side `value_tag == "Vec"` | RIGHT side `value_tag == "wat::core::Vector"` |
| Dispatch arm guards | `if h == "Option"` | `if h == "wat::core::Option"` |
| Error message `expected` fields | `expected: "Vec"` | `expected: "wat::core::Vector"` |
| Test fixture assertions | `assert_eq!(head, "Vec")` | `assert_eq!(head, "wat::core::Vector")` |

Pure-Rust identifiers (Rust language `Vec::new()`, `Option::Some(...)` as Rust types) stay as Rust idents — they are NOT wat-internal naming.

## Working directory + state

`/home/watmin/work/holon/wat-rs` on `main` branch at `fa544a8`. Working tree is DIRTY — sonnet `a106fae30415c38a5`'s edits + orchestrator typealias deletions are present. DO NOT revert. DO NOT commit. Continue from this state.

## Categories surfaced (pre-flight enumeration)

Five known categories from orchestrator's diagnostic walk. Sweep these first; then iterate from cargo test for whatever else surfaces.

### Category A — `Value::type_name()` returns

`src/runtime.rs:455-510` (the `impl Value { pub fn type_name(&self) -> &'static str { match self { ... } } }` block).

Currently returns mixed bare + FQDN. Update bare arms to FQDN per the rule:
```
Value::bool(_)   => "wat::core::bool"
Value::i64(_)    => "wat::core::i64"
Value::u8(_)     => "wat::core::u8"
Value::f64(_)    => "wat::core::f64"
Value::String(_) => "wat::core::String"
Value::Vec(_)    => "wat::core::Vector"
Value::Unit      => "wat::core::nil"
Value::Option(_) => "wat::core::Option"
Value::Result(_) => "wat::core::Result"
Value::Tuple(_)  => "wat::core::tuple"   // or whatever tuple's FQDN is — verify by greping
```

Other arms (e.g., `wat::core::keyword`, `wat::holon::HolonAST`) already FQDN — leave alone.

### Category B — value_tag matching arm RIGHT sides

`src/runtime.rs:3661-3665` and the surrounding function. Currently:
```rust
"wat::core::Vector" => value_tag == "Vec",
"wat::core::HashMap" => value_tag == "rust::std::collections::HashMap",
...
```

Update RIGHT side strings to match the new FQDN value_tag form:
```rust
"wat::core::Vector" => value_tag == "wat::core::Vector",
"wat::core::Option" => value_tag == "wat::core::Option",
"wat::core::Result" => value_tag == "wat::core::Result",
```

HashMap/HashSet right sides are `"rust::std::collections::HashMap"` (Rust paths) — those mirror the actual Rust type. Per the rule: the value_tag for a Rust-stored container is the wat name OR the Rust path? Audit the symmetry. Likely Category A's rename `Value::wat__std__HashMap(_) => "rust::std::collections::HashMap"` should become `"wat::core::HashMap"` to match the new convention (wat-side type name, NOT rust-side path). Match arm RIGHT side updates accordingly.

### Category C — scattered dispatch arm guards (`if h == "Option"`)

`src/check.rs:5607,5625,5643` and any other site where a single-letter binding `h` is compared against bare strings. Search comprehensively:
```bash
grep -rEn 'if h == "(Option|Result|Vec|HashMap|HashSet)"' src/ --include="*.rs"
grep -rEn 'h == "(Option|Result|Vec|HashMap|HashSet)"' src/ --include="*.rs"
```

Update to FQDN form.

### Category D — error message `expected` fields

`src/runtime.rs:5657, 14567, 14714` + `src/rust_deps/marshal.rs:174,260,285`. Currently:
```rust
expected: "Vec",
expected: "Option",
expected: "Result",
```

For wat-internal error sites: update to FQDN. For `rust_deps/marshal.rs`: AUDIT — this module marshals between wat values and Rust deps. The `expected` field may name a RUST type (Rust form, no FQDN). Read each site's context to decide.

### Category E — test fixture `assert_eq!(head, "X")` assertions

`src/types.rs:2299, 2413, 2529, 2551, 2574, 2592` plus likely others. Test code that hardcodes the legacy head string:
```rust
assert_eq!(head, "HashMap");
assert_eq!(head, "Result");
assert_eq!(head, "Vec");
```

Update to FQDN:
```rust
assert_eq!(head, "wat::core::HashMap");
assert_eq!(head, "wat::core::Result");
assert_eq!(head, "wat::core::Vector");
```

Comprehensive audit:
```bash
grep -rEn 'assert_eq!\(head, "(Vec|Option|Result|HashMap|HashSet)"\)' src/ --include="*.rs"
```

## Iteration loop (the substrate is the teacher)

After each phase / category sweep:

```bash
cd /home/watmin/work/holon/wat-rs
cargo test --release --workspace --no-fail-fast 2>&1 | grep -E "^test result" | awk '{passed+=$4; failed+=$6} END {print "Passed:", passed, "Failed:", failed}'
```

Watch the failed count drop. When it stabilizes (one round produces same count as prior), pivot to reading specific failures:

```bash
# Get one specific failure with output
cargo test --release --lib <test::path::name> -- --nocapture 2>&1 | tail -20
```

Each panic / assertion failure names a site. Apply the FQDN rule. Iterate.

## Stopping condition

Workspace test count = 2041 / 0 (or HIGHER — slice 3e may make some currently-skipped tests viable; that's a positive delta). Build clean. Working tree dirty for orchestrator review.

## Constraints

- DO NOT revert any of the existing modifications to src/check.rs, src/runtime.rs, src/types.rs, src/edn_shim.rs. They are CORRECT (head FQDN sweep + alias deletions). The remaining failures are downstream consumers needing parallel updates.
- DO NOT commit. Working tree dirty for orchestrator review + commit.
- DO NOT touch `rust_deps/marshal.rs` `expected` fields without auditing — those may be RUST-side identifiers (e.g., naming Rust `Vec`/`Option`), in which case they stay as Rust form.
- Pure-Rust identifiers (no wat semantics) stay Rust form. Wat-internal storage strings go FQDN.
- The walker `BareLegacyContainerHead` etc. + Pattern 2 poison arms in `src/check.rs:3840,3858` STAY as user-source diagnostics (different system).
- Test files that hardcode legacy strings IN ASSERTIONS get updated; tests that LITERALLY VERIFY THE WALKER FIRES on legacy syntax KEEP the legacy syntax (their PURPOSE is verifying retirement diagnostic).

## Time-box

90 minutes wall-clock for the mechanical sweep across categories A-E + iteration. If you exceed 90 min OR the failure count plateaus above 0 with no clear additional category to sweep, STOP and report the residual.

## Reporting (~250 words)

1. Per-category sweep summary (sites updated per file)
2. Failure-count waterfall: pre-sweep → after A → after B → after C → after D → after E → final
3. Any additional categories surfaced from the diagnostic stream during iteration (name them; they're the substrate teaching us about more hidden surfaces)
4. Honest deltas — sites you weren't sure how to classify (rust-side vs wat-side ambiguity); test files you classified as "verifies retirement diagnostic" and KEPT legacy syntax
5. Final test count + path classification (Mode A: clean to 2041/0; Mode B: stuck above zero with diagnostic-driven residuals named; Mode C: build broke or sweep regressed)

DO NOT commit. Orchestrator commits + scores after.
