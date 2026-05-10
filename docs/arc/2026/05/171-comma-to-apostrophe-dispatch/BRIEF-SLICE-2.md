# Arc 171 slice 2 — BRIEF

**Consumer sweep; sonnet.** Migrate all keyword-body commas
to apostrophes per the user's locked table. Slice 1 made the
lexer accept apostrophe; this slice sweeps the consumers.
Slice 3 retires comma acceptance + ships closure.

## Mission

Apply this transformation to every keyword-body comma site:

```
:<verb>,<digit>       → :<verb>'<digit>           (arity suffix)
:<verb>,<id>-<id>     → :<verb>'<id>'<id>         (type discriminator)
:<verb>,<id>-<id>-... → :<verb>'<id>'<id>'...     (multi-type discriminator)
```

Per user's locked table:

```
:wat::core::op            (base; unchanged)
:wat::core::op'2          (was :op,2)
:wat::core::op'i64'i64    (was :op,i64-i64)
:wat::core::op'i64'f64    (was :op,i64-f64)
:wat::core::op'f64'i64    (was :op,f64-i64)
:wat::core::op'f64'f64    (was :op,f64-f64)
```

**Comma → apostrophe.** **Dashes within the suffix portion →
apostrophe.** Dashes elsewhere (in the verb's own identifier,
e.g., `read-line`) STAY as dashes — only dashes in the
post-comma suffix get rewritten.

## Scope — 167 sites pre-grep (verify at sweep time)

```bash
grep -rEn ":[a-zA-Z][a-zA-Z_:\*\+\-/]*,[0-9a-zA-Z\-]+" \
  --include="*.wat" --include="*.rs" \
  . | wc -l
```

Distribution per pre-grep 2026-05-10:

| Bucket | Files | Approx sites |
|---|---|---|
| Substrate wat | `wat/core.wat` (most), `wat/holon/*`, `wat/edn.wat`, etc. | ~40-60 |
| Test wat | `wat-tests/*` (`service-template.wat`, etc.) | ~5-10 |
| Crate wat | `crates/wat-{telemetry,telemetry-sqlite,holon-lru}/wat/`, `wat-tests/` | ~20-30 |
| Example wat | `examples/{interrogate,with-loader,with-lru,console-demo}/wat/` | ~5-10 |
| Scripts | `wat-scripts/{ping-pong,ping-pong-fork,seed-fixture}.wat` | ~10 |
| Rust source | `src/{lexer,types,runtime,check}.rs` (diagnostic strings + matchers) | ~10 |
| Wire-encoding tests | `crates/wat-edn/tests/wire_encoding.rs` | ~3 |
| Archived | `docs/arc/2026/05/130-*/complected-2026-05-02/*.wat` | ~14 |

## What to NOT do

- **No lexer changes.** Slice 1 shipped the apostrophe
  acceptance; slice 3 retires comma. This slice is consumer-only.
- **No archived-arc edits.** Files under `docs/arc/*/complected-*/`
  are historical record per "what is inscribed is inscribed."
  Surface these as honest delta (skipped); don't migrate them.
- **No EDN-wire-encoding test changes** if those tests are
  literally about validating comma's role in the OLD wire format.
  Inspect `crates/wat-edn/tests/wire_encoding.rs` carefully — if
  it's testing slice 1f-W's encoding rules (commas inside `<>`),
  those commas are LOAD-BEARING; don't sweep them. Surface as
  honest delta.

## Substrate-grep citations

- Sweep pattern: `:[a-zA-Z][a-zA-Z_:\*\+\-/]*,[0-9a-zA-Z\-]+`
  (matches any `:<verb>,<suffix>` site)
- The user's locked convention table: see
  `feedback_apostrophe_dispatch_separator.md` memory
- Slice 1 SCORE: `SCORE-SLICE-1.md` (confirms lexer accepts
  apostrophe; transition mode active)
- Wire encoding sibling: `4278c4d` (slice 1f-W) — DIFFERENT
  rule (commas inside `<>` get underscore-swap on the wire);
  don't conflate

## Test approach during sweep

After each file (or batch of files):
```
cargo check --release 2>&1 | tail -3
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'
```

Target: workspace count stays at slice 1's baseline
(1334 passed / 854 failed) ±5. Drift means a sweep step
broke something (likely a test that asserts a specific
keyword spelling); surface for review.

## Ship criteria

| Row | What | Pass criterion |
|-----|------|----------------|
| A — All wat source files swept | grep for `:[a-z][\w:]*,[\w-]+` in `wat/`, `wat-tests/`, `crates/*/wat*/`, `examples/*/wat/`, `wat-scripts/` returns 0 hits | ✓ |
| B — Rust diagnostic strings swept | grep in `src/*.rs` returns only intentional matches (e.g., transition-mode tests in `lexer.rs` testing the legacy shape — surface those) | ✓ |
| C — Archived arc docs UNCHANGED | `git diff --stat docs/arc/2026/05/130-*` returns 0 | ✓ |
| D — Wire-encoding test commas preserved | `crates/wat-edn/tests/wire_encoding.rs` — verify any unchanged commas are testing the `<>` wire-encoding rule, not keyword-body content | ✓ |
| E — `cargo check --release` green | no compile errors after sweep | ✓ |
| F — Workspace at baseline ±5 | 1329-1339 passed; 849-859 failed | ✓ |
| G — Slice 1 tests still green | the 6 apostrophe tests + 1 comma-transition test all pass | ✓ |
| H — Zero new dependencies | Cargo.toml unchanged | ✓ |
| I — Honest deltas surfaced | per FM 5 (any non-trivial site that wasn't pure-mechanical) | ✓ |

## Honest delta categories

Surface, don't work around:

- **EDN wire-encoding test commas in `crates/wat-edn/tests/wire_encoding.rs`** — distinguish keyword-body commas (sweep) from `<>` wire-encoding commas (preserve). Likely the latter; surface findings.
- **Transition-mode test in `src/lexer.rs`** — `keyword_comma_suffix_transition` test explicitly verifies `:wat::core::op,2` still parses. PRESERVE this test (slice 3 will retire it); surface to confirm.
- **`Cargo.lock` not touched** — sweep doesn't trigger dep changes.
- **Archived arc docs** — `docs/arc/2026/05/130-*` contains 14
  comma sites in complected-2026-05-02 wat files. SKIP per
  "what is inscribed is inscribed"; surface as honest delta
  confirming.
- **Wat sources used in Rust tests as raw strings** — some
  `tests/wat_*.rs` Rust files contain wat source as raw string
  literals; those raw strings also need keyword-body comma
  sweep. Inspect at sweep time.
- **Doc comments in Rust source** — Rust doc comments may
  contain example wat snippets with `,` keyword-body commas
  (e.g., `/// Example: :wat::core::+,2`). Update these comments
  in lockstep so docs stay honest.

## Predicted runtime

60-120 min sonnet. Mechanical regex sweep with cargo-test
validation per phase. The unknowns are the Rust-source
diagnostic strings (which may need per-site judgment) +
archived-arc skip-confirmation.

**Hard cap:** 180 min.

## Reference

- DESIGN.md (this arc; the broader scope)
- SCORE-SLICE-1.md (slice 1 lock-in; lexer accepts `'`;
  transition mode preserved)
- `feedback_apostrophe_dispatch_separator.md` (the user's
  locked convention table)
