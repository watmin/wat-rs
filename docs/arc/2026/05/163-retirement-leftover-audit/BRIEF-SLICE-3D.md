# Arc 163 Slice 3d BRIEF — hard-retire `:wat::core::vec` + `:wat::core::list` runtime arms; sweep test fixtures

**Drafted 2026-05-07.** Slice 3d completes what slice 3a should have
been (corrected after my BRIEF errors + slice 3c substrate prep).

## Why slice 3d now

Slice 3c just shipped — `runtime.rs:16811` canonicalize step now
accepts `:wat::core::Vector` (the canonical) alongside the retired
`:wat::core::vec` / `:wat::core::list` keywords. **Substrate is
ready for consumers to use canonical Vector everywhere.**

Slice 3d's job: kill the retired runtime arms + sweep all test
fixtures to canonical.

## Bash IS verified working for sub-agents

Per probe `which cargo` returning `cargo 1.93.0`. Per memory
`feedback_verify_sonnet_tool_claims.md` — DO NOT falsely claim
Bash denied. If you hesitate, run `which cargo` once to confirm.

## Working directory

`/home/watmin/work/holon/wat-rs` on `main` branch at `040e2cc`.

## Workspace baseline

```bash
cd /home/watmin/work/holon/wat-rs
cargo test --release --workspace 2>&1 | grep -E "test result" | awk '{passed+=$4; failed+=$6} END {print "Pre passed:", passed, "Pre failed:", failed}'
```

Expect: 2041 / 0.

## Edit plan — substrate first, then sweep consumers

### Phase 1 — Substrate edits (4 sites in runtime.rs + 1 in lower.rs)

#### Site 1.1: `src/runtime.rs:3082` — DELETE `:wat::core::vec` runtime arm

Currently lines 3076-3083:
```rust
        // List construction
        // Arc 109 slice 1f — :wat::core::Vector is the canonical
        // constructor; :wat::core::vec stays during the migration
        // window (Pattern 2 poison surfaces a hint at type-check
        // time but runtime keeps working). Both paths produce the
        // same Value::List(Vec<Value>).
        ":wat::core::vec" => eval_list_ctor(args, env, sym),
        ":wat::core::Vector" => eval_list_ctor(args, env, sym),
```

Replace with:
```rust
        // List construction
        // Arc 163 slice 3d — `:wat::core::Vector` is canonical;
        // legacy `:wat::core::vec` and `:wat::core::list` runtime
        // arms retired. Type-checker Pattern 2 poison (check.rs:3840,
        // 3858) still surfaces friendly redirect for users typing
        // legacy keywords; runtime arm gone for defense-in-depth.
        ":wat::core::Vector" => eval_list_ctor(args, env, sym),
```

#### Site 1.2: `src/runtime.rs:3088` — DELETE `:wat::core::list` runtime arm

Currently lines 3084-3088:
```rust
        // Arc 109 slice 1g — :wat::core::list retires (was always
        // an alias for vec). Type checker emits the migration hint
        // (Pattern 2 poison); runtime keeps the alias arm working
        // through the migration window.
        ":wat::core::list" => eval_list_ctor(args, env, sym),
```

Delete entirely (5 lines).

#### Site 1.3: `src/runtime.rs:16811` — drop `vec | list` from canonicalize arm

Currently has `":wat::core::Vector" | ":wat::core::vec" | ":wat::core::list" =>` (slice 3c added Vector). Drop `| ":wat::core::vec" | ":wat::core::list"`. Update adjacent comment to drop the "transitional" phrasing.

#### Site 1.4: `src/runtime.rs:5600, 5609` — update `op:` / `head:` error fields

Inside `eval_list_ctor`. Both currently say `":wat::core::vec".into()`. Update to `":wat::core::Vector".into()`. The fn handles all three forms but the error messages should reference the canonical name.

#### Site 1.5: `src/runtime.rs:5585` — doc-comment

Currently: `/// `(:wat::core::vec :T x1 x2 ...)` —`
Update to: `/// `(:wat::core::Vector :T x1 x2 ...)` —`

#### Site 1.6: `src/lower.rs:254` — drop vec from Bundle input recognition

Currently: `if k == ":wat::core::vec" || k == ":wat::core::Vector" =>`
Update to: `if k == ":wat::core::Vector" =>`

Update adjacent comment (line 241 area: `// Expect exactly one argument: a (:wat::core::vec :T item ...) form.`) → `// Expect exactly one argument: a (:wat::core::Vector :T item ...) form.`

### Phase 1 verify

```bash
cd /home/watmin/work/holon/wat-rs
cargo build --release 2>&1 | tail -5
```

Build MUST stay clean. If it doesn't, STOP — you've broken substrate-internal recognition somewhere.

### Phase 2 — Test fixture sweep

Now sweep all test fixtures + doc-comment + macros uses from
`(:wat::core::vec ` (with trailing space) and `(:wat::core::list `
(with trailing space) → `(:wat::core::Vector ` (WITH trailing space —
critical to preserve, otherwise you'll concatenate Vector with the
next token like `(:wat::core::Vectorelement` which is wrong).

Approach: Edit per-file with `replace_all: true`:
- Pattern: `(:wat::core::vec ` → `(:wat::core::Vector `
- Pattern: `(:wat::core::list ` → `(:wat::core::Vector `

Files known to contain these (from prior sonnet's audit):
- `src/runtime.rs` (test fixtures in tests module)
- `src/check.rs` (test fixtures in tests module)
- `src/macros.rs` (quasiquote uses)
- `src/lower.rs` (test fixtures)
- `src/resolve.rs` (test)
- `src/string_ops.rs` (doc comments)
- `tests/wat_arc150_variadic_define.rs`
- Various crate `.rs` files

**Verify after each file edit** that the Edit tool reports successful replacement (not "string not found"). If the file has no `(:wat::core::vec ` matches, skip it.

### Phase 2 verify

```bash
cd /home/watmin/work/holon/wat-rs
cargo build --release 2>&1 | tail -5
cargo test --release --workspace --no-fail-fast 2>&1 | grep -E "test result" | awk '{passed+=$4; failed+=$6} END {print "Pass:", passed, "Fail:", failed}'
```

Expect: 2041 / 0.

If any test fails: investigate. Don't paper over — the failure may be revealing a substrate-internal site that still needs `vec` recognition (e.g., I missed a site in Phase 1's enumeration).

### Phase 3 — KEEP these (Bucket C/D — DO NOT TOUCH)

- `src/check.rs:3840` (`":wat::core::vec" => {...}` Pattern 2 poison + redirect to Vector) — user-facing diagnostic, KEEP
- `src/check.rs:3858` (`":wat::core::list" => {...}` Pattern 2 poison) — KEEP
- All `:wat::core::vec` / `:wat::core::list` mentions in retirement-context comments ("Arc 109 slice 1f retired", "(formerly...)", etc.) — KEEP verbatim
- `:wat::core::vec` / `:wat::core::list` mentions in test fixtures whose PURPOSE is verifying the retirement diagnostic fires (search for tests that assert `BareLegacyContainerHead` or similar) — KEEP

If unsure whether a site is Bucket A vs C/D — check the surrounding context. If the comment says "retired", "retirement", "(formerly", "Pattern 2 poison", "BareLegacy" — KEEP. If it's a test fixture using the keyword as an active constructor that the test EXPECTS to work — that's Bucket A, sweep.

## Constraints

- DO NOT commit. Working tree dirty for orchestrator review.
- "STOP at unexpected red" — don't paper over breakage.
- Test count must stay 2041 (or higher).
- Trailing spaces on the `(:wat::core::vec ` / `(:wat::core::list ` patterns are MANDATORY — without them you'll concatenate Vector with the next token (broken syntax).
- Time-box: 60 min wall-clock.

## Reporting (~200 words)

1. Phase 1 substrate edit summary (each of the 6 sites)
2. Phase 2 per-file replacement counts (file: N sites updated)
3. Test pass: pre vs post (must stay 2041 or higher)
4. Path: Mode A / B / C
5. Honest deltas:
   - Did Phase 1 break the build? Why?
   - Any test files you classified as Bucket D (retirement-diagnostic tests) and KEPT untouched
   - Any sites you weren't sure how to classify — list them for orchestrator decision
   - Did `cargo test` reveal substrate-internal sites that still need `vec` (Phase 1 missed)?

DO NOT commit. Orchestrator commits + scores after.
