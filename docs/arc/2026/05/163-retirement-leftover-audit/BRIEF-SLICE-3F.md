# Arc 163 Slice 3f BRIEF — substrate-internal primitive paths to FQDN

**Drafted 2026-05-07.** Slice 3e shipped substrate-internal container
heads to FQDN (commit `25860be`). Slice 3f applies the same rule to
primitive paths.

## The rule

Per user direction 2026-05-07: **every wat-internal substrate-side
string identifier that names a wat type must be FQDN.** No exceptions.

Slice 3e flipped container heads (`Parametric.head`). Slice 3f flips
the parallel category: `TypeExpr::Path` strings for primitive types.

| Pre-3f (legacy) | Post-3f (canonical) |
|---|---|
| `Path(":i64")` | `Path(":wat::core::i64")` |
| `Path(":f64")` | `Path(":wat::core::f64")` |
| `Path(":bool")` | `Path(":wat::core::bool")` |
| `Path(":String")` | `Path(":wat::core::String")` |
| `Path(":u8")` | `Path(":wat::core::u8")` |

Pure-Rust identifiers (no wat semantics) stay Rust form.

## Scope

Audit grep returns ~155 substrate sites:
- `":i64"`: 45 sites
- `":f64"`: 26 sites
- `":bool"`: 28 sites
- `":String"`: 52 sites
- `":u8"`: 4 sites

## Working directory + state

`/home/watmin/work/holon/wat-rs` on `main` branch at `6cee7eb`. Test
baseline: 2041/0 (slice 3e + recovery doc shipped clean).

## Discipline

This is a substrate-wide migration. Per `docs/SUBSTRATE-AS-TEACHER.md`:

> The substrate's compiler IS the brief. The diagnostic stream encodes
> the migration path. Run cargo test; read errors; apply rule; iterate
> until green. The fail-count is the progress meter.

Don't enumerate categories upfront expecting completeness. The first
cargo test reveals one category; sweep drops the count by ~80-90%;
next test reveals the next category. Trust the loop.

Same waterfall pattern as slice 3e (848 → 0 in 7 iterations).

## Phase order

### Phase 1 — Substrate canonicalize step: flip primitive-path arms from DOWNGRADE to UPGRADE

`src/types.rs` around lines 1710-1721 (`parse_type_inner` primitive
canonicalize step). Currently:

```rust
let path = if canonicalize {
    match raw_path.as_str() {
        ":wat::core::i64" => ":i64".to_string(),       // DOWNGRADE
        ":wat::core::f64" => ":f64".to_string(),
        ":wat::core::bool" => ":bool".to_string(),
        ":wat::core::String" => ":String".to_string(),
        ":wat::core::u8" => ":u8".to_string(),
        _ => raw_path,
    }
} else {
    raw_path
};
```

Flip to UPGRADE (parallel to slice 3e's container head arms):

```rust
let path = if canonicalize {
    match raw_path.as_str() {
        ":i64" => ":wat::core::i64".to_string(),       // UPGRADE
        ":f64" => ":wat::core::f64".to_string(),
        ":bool" => ":wat::core::bool".to_string(),
        ":String" => ":wat::core::String".to_string(),
        ":u8" => ":wat::core::u8".to_string(),
        _ => raw_path,
    }
} else {
    raw_path
};
```

Add the same RETIREMENT WINDOW comment as the container-head arm
(slice 3h gates arc 163 closure). The upgrade arm is TEMPORARY
bridge scaffolding for fixtures using bare-form wat source.

### Phase 2 — Sweep substrate-internal `":i64"`/etc. writes to FQDN

~155 sites across `src/`, `crates/`. Per-file `replace_all: true`
substitutions:

```
":i64".into()     → ":wat::core::i64".into()
":f64".into()     → ":wat::core::f64".into()
":bool".into()    → ":wat::core::bool".into()
":String".into()  → ":wat::core::String".into()
":u8".into()      → ":wat::core::u8".into()
```

Quote-anchored + `.into()` suffix avoids touching unrelated text.

Comprehensive grep:
```bash
grep -rn '":i64"\|":f64"\|":bool"\|":String"\|":u8"' src/ crates/ --include="*.rs"
```

### Phase 3 — Sweep `Path(":i64")` etc. constructions

`TypeExpr::Path(":i64".into())` and similar. Find via:
```bash
grep -rEn 'Path\(":(i64|f64|bool|String|u8)"' src/ crates/ --include="*.rs"
```

Update each to FQDN form.

### Phase 4 — Sweep dispatch arm matching, error fields, type-name FQDN

Categories from slice 3e's experience:
- `value_tag == ":i64"` matching → `value_tag == ":wat::core::i64"` (if any)
- Error message `expected: ":i64"` fields → FQDN
- `Value::type_name()` primitive arms (currently bare per slice 3e
  revert): flip to FQDN (lines src/runtime.rs:457-463 + Tuple line)

Iteratively from cargo test failures.

### Phase 5 — Iterate from cargo test until green

```bash
cargo test --release --workspace --no-fail-fast 2>&1 | grep -E "^test result" | awk '{passed+=$4; failed+=$6} END {print "Pass:", passed, "Fail:", failed}'
```

Stop at 2041/0 or higher passing.

If failures plateau without clear category to sweep: STOP and
report residual diagnostic patterns.

## Constraints

- DO NOT commit. Working tree dirty for orchestrator review.
- DO NOT touch wat fixture sources (test `.wat` files using bare
  `:i64` syntax) — that's slice 3g scope.
- DO NOT remove the canonicalize=true upgrade arms — slice 3h gates
  arc closure on those.
- Pure-Rust identifiers (no wat semantics) stay Rust form.
- Time-box: 90 min wall-clock.

## Reporting (~250 words)

1. Phase summary (per-file site counts updated)
2. Failure-count waterfall: pre-sweep → after each phase → final
3. Path classification (Mode A: green at 2041/0; Mode B: stuck above
   zero with residuals named; Mode C: build broke or sweep regressed)
4. Honest deltas — sites you weren't sure how to classify; tests
   that hardcoded bare-form expected strings
5. Any additional categories surfaced from the diagnostic stream

DO NOT commit. Orchestrator commits + scores after.
