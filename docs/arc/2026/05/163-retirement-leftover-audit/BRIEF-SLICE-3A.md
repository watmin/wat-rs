# Arc 163 — Slice 3a BRIEF (kill `:wat::core::list` runtime alias arm)

**Drafted 2026-05-07.** Slice 3a — surgical hard-retirement of
`:wat::core::list` runtime alias.

## Why slice 3a

User direction 2026-05-07: *"Hard retire — kill typealiases."*
Per slice 2's audit, `:wat::core::list` is currently soft-retired:
- Type checker emits Pattern 2 poison (`TypeMismatch` with redirect)
- Runtime keeps an alias dispatch arm at `src/runtime.rs:3088`
  (`":wat::core::list" => eval_list_ctor(args, env, sym)`)
- Runtime tests at `src/runtime.rs:20447+` exercise the alias
  behavior (~20 test sites)

Slice 3a deletes the runtime alias arm + retires the tests, making
`:wat::core::list` produce "unknown form" at runtime (defense-in-
depth on top of the type-checker poison).

## Working directory

`/home/watmin/work/holon/wat-rs` on `main` branch.

## Workspace state pre-spawn

- HEAD: `97dd8d9` (arc 163 slice 2 shipped)
- Working tree clean
- Workspace: 2041 passed / 0 failed
- Bash verified working for sub-agents

## Scope (precise)

### Delete

1. `src/runtime.rs:3084-3088` — comment block + dispatch arm:
   ```rust
   // Arc 109 slice 1g — :wat::core::list retires (was always
   ...comment...
   ":wat::core::list" => eval_list_ctor(args, env, sym),
   ```
   Delete both the comment AND the dispatch arm cleanly.

2. `src/runtime.rs:16811` — pattern match arm. Replace
   `":wat::core::vec" | ":wat::core::list" => {...}` with just
   `":wat::core::vec" => {...}`.

3. Migrate runtime tests at `src/runtime.rs:20447+` (~20 sites)
   that exercise `(:wat::core::list :i64 1 2 3)`:
   - **Option (a)** — rename to `(:wat::core::vec :i64 1 2 3)`
     (matches the current canonical low-level constructor)
   - **Option (b)** — rename to `(:wat::core::Vector :i64 1 2 3)`
     (matches the canonical user-facing typed constructor)
   
   Both produce the same internal `Parametric { head: "Vec" }`.
   Pick `:wat::core::vec` (option a) since the tests are testing
   runtime-level behavior; lower-level form is more appropriate.

   ALSO retire the dedicated test that explicitly tests the alias
   behavior (e.g., `list_constructor_is_alias_for_vec` if it
   exists) — its purpose dissolves when the alias dies. Search
   `src/runtime.rs::tests` for `list` test fn names; retire any
   testing the alias specifically.

4. `src/runtime.rs:5585` — doc-comment showing both forms:
   `/// `(:wat::core::vec :T x1 x2 ...)` / `(:wat::core::list :T x1 x2 ...)` —`
   Remove the `/ ` :wat::core::list` …` clause.

### Preserve (Bucket C/D)

- The type-checker Pattern 2 poison for source-level `:wat::core::list`
  in `src/check.rs` — KEEP. Users typing `:wat::core::list` should
  still get the friendly redirect-to-`vec` error.
- Any `BareLegacy*` variant + Display referencing list — KEEP per
  arc 113 precedent.
- Comments documenting the arc 109 slice 1g retirement — KEEP.

## Pre-flight crawl

1. Read `docs/arc/2026/05/163-retirement-leftover-audit/BRIEF-SLICE-3A.md`
2. Read `docs/arc/2026/05/163-retirement-leftover-audit/DESIGN.md` (slice plan)
3. Read `docs/arc/2026/05/162-lambda-internal-rename/BRIEF-SLICE-1.md` (Bucket framework)
4. Read `src/runtime.rs:3082-3092` — runtime alias arm
5. Read `src/runtime.rs:5580-5600` — doc comment
6. Read `src/runtime.rs:16805-16820` — match arm
7. Search `src/runtime.rs` for `(:wat::core::list ` to find all 20+ test sites

## Audit baseline

```bash
cd /home/watmin/work/holon/wat-rs
cargo test --release --workspace 2>&1 | grep -E "test result" | awk '{passed+=$4; failed+=$6} END {print "Pre passed:", passed, "Pre failed:", failed}'
echo "Total :wat::core::list sites: $(grep -rn ':wat::core::list' --include='*.rs' --include='*.wat' . 2>/dev/null | grep -v complected | wc -l)"
```

## Procedure

1. Delete the runtime alias arm (`runtime.rs:3088` + adjacent comment)
2. Update the dual-arm match at `runtime.rs:16811` (drop `| ":wat::core::list"`)
3. Update the doc-comment at `runtime.rs:5585`
4. Migrate test sites: `(:wat::core::list :T ...)` → `(:wat::core::vec :T ...)`
5. Retire the alias-specific test (e.g., `list_constructor_is_alias_for_vec`)
6. `cargo build --release` — must compile
7. `cargo test --release --workspace` — must stay 2041 passed / 0 failed

## Constraints

- DO NOT commit. Working tree dirty for orchestrator review.
- "STOP at unexpected red" — if any pre-existing test breaks, stop.
- Test count must stay 2041 (or higher).
- Type-checker Pattern 2 poison MUST still emit for source-level
  `:wat::core::list` — verify with a probe if needed.
- Time-box: 30 min wall-clock.

## Verification (after edits)

```bash
cd /home/watmin/work/holon/wat-rs
cargo build --release 2>&1 | tail -3
cargo test --release --workspace --no-fail-fast 2>&1 | grep -E "test result" | awk '{passed+=$4; failed+=$6} END {print "Pass:", passed, "Fail:", failed}'

# Probe — source-level :wat::core::list should still produce the
# Pattern 2 poison diagnostic:
cat > /tmp/probe-list.wat <<'EOF'
(:wat::core::define
  (:test::probe -> :wat::core::Vector<wat::core::i64>)
  (:wat::core::list :wat::core::i64 1 2 3))
EOF
cargo run --release --quiet --bin wat -- --check /tmp/probe-list.wat 2>&1 | head -5
```

The probe should show a TypeMismatch / poison diagnostic, NOT a
runtime "unknown form" error (because check fires before runtime).

## Reporting (~150 words)

1. Per-step summary: deletes applied, test migrations count
2. Test pass: pre vs post (must stay 2041 or higher)
3. Path: Mode A / B / C
4. Probe result: did `:wat::core::list` source produce the type-
   checker Pattern 2 poison as expected?
5. Honest deltas:
   - The retirement-specific test (e.g., `list_constructor_is_alias_for_vec`) — did it exist? Retired?
   - Any other unexpected test that exercised the alias behavior?

DO NOT commit. Orchestrator commits + scores after.

## Time-box

30 minutes wall-clock.
