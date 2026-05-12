# Arc 170 slice 3 Gap F-3 BRIEF — parent type registry inheritance to spawn-process child

**Sonnet.** Second of the four Phase 2a gap slices (after Gap F-1 lands). Substrate fix for Phase E V4 failure pattern 3 — hermetic child subprocess type registry missing parent's struct/enum declarations.

## Backstory

V4 failure pattern 3 (commit `f2de549` SCORE): roundtrip.wat declares `(:wat::core::enum :test::Event ...)` + `(:wat::core::struct :test::Wrapper<E> ...)` in prelude. Under V4's top-level-`do` shape, these land in the OUTER frozen world. When `(:wat::test::run-hermetic <body>)` spawns a child via `spawn-process`, the child's world is built from `extract_closure`'s prologue. **`extract_closure` only captures types that appear in the fn signature** — not arbitrary parent-world types the body might use.

Result: child subprocess lacks `test::Event` / `test::Wrapper<E>` in its type registry. `(:wat::edn::read s)` deserializes a tagged EDN form, type registry lookup fails, child exits 1.

## Goal — extract_closure includes parent's full type registry

The fix: `extract_closure` propagates the PARENT'S type registry to the child. Types are immutable declarations under wat's Zero-Mutex doctrine; sharing via `Arc<TypeEnv>` (or equivalent) preserves hermetic isolation because types are runtime-shared-but-not-mutable.

## Hermetic semantics preserved (verify)

The concern: "hermetic" means isolated. Does sharing types violate isolation?

**No** — for two reasons:

1. **Types are immutable declarations**. The parent and child see the same type definitions; neither can modify them. There's no observable state difference between "child has its own copy" and "child shares parent's Arc."
2. **wat doctrine treats types as global constants**. Per arc 109 § kill-std, every substrate-provided symbol is FQDN-keyword'd. Types declared in user code follow the same model — they're identified by FQDN-keyword, declared once per world, referenced everywhere.

The only thing that crosses the hermetic boundary is type DEFINITIONS (immutable), not type-instance values. Hermetic isolation is preserved for runtime state (memory, signals, exit, panic) — type registry sharing does not violate it.

## Required reading IN ORDER

1. **`docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-PHASE-E-V4-DEFTEST-REWRITE.md`** — V4 failure analysis (failure pattern 3 at lines ~105-116)
2. **`docs/arc/2026/05/170-program-entry-points/RETIREMENT-THEATER-INVENTORY.md`** — full priority queue + gap-closure prerequisite framing
3. **`src/closure.rs`** (or `src/spawn.rs` — locate `extract_closure`) — current implementation; what gets captured
4. **`src/types.rs`** — TypeEnv structure; how types are registered + looked up
5. **`docs/ZERO-MUTEX.md`** — three-tier sharing doctrine (Arc + ThreadOwnedCell + program-with-mailbox); types fit Tier 1 (immutable Arc)

## Implementation path

### Phase 1 — Locate + audit `extract_closure`

Grep for `extract_closure` / `ClosurePackage` / closure-prologue assembly. Identify what's currently captured: function definitions, captured-let bindings, etc. Verify types are NOT included today.

### Phase 2 — Write probes (failing baseline)

Create `tests/probe_spawn_process_parent_type.rs`:

```rust
#[test]
fn probe_spawn_process_inherits_parent_struct() {
    let src = r#"
        (:wat::core::struct :test::Point
          (x :wat::core::i64)
          (y :wat::core::i64))
        (:wat::core::define (:user::main -> :wat::core::nil)
          (:wat::core::let [p (:test::Point/new 3 4)]
            (:wat::core::nil)))
    "#;
    // Spawn a process whose body uses :test::Point declared at parent top-level.
    // Verify child runs without "type not registered" error.
    // ... full probe per substrate spawn-process test pattern
}

#[test]
fn probe_spawn_process_inherits_parent_enum() {
    // Same shape for enum + variant-construction in body
}

#[test]
fn probe_spawn_process_inherits_parametric_type() {
    // Same shape for :test::Wrapper<E> (parametric)
}
```

Confirm probes FAIL with V4 failure pattern 3's "type not registered in child" error (or equivalent).

### Phase 3 — Extend `extract_closure`

Modify `extract_closure` to include parent's type registry in the produced `ClosurePackage`. Likely an additional field on `ClosurePackage` (or whatever struct it returns) carrying `Arc<TypeEnv>` (or equivalent immutable type-registry handle).

Update spawn-process child startup to USE the inherited types when initializing the child's world — types are visible in child's `TypeEnv` before user code runs.

### Phase 4 — Verify

- 3 new probes pass
- All existing probes still pass
- Workspace at 2209 + N + N' / 0 failed (N from Gap F-1, N' from F-3)
- Hermetic test suite (existing fork-program-* + spawn-process-* tests) unchanged — confirms isolation semantics preserved

## Scope (what's IN)

- `extract_closure` (or equivalent) extended to capture parent's type registry
- `ClosurePackage` (or equivalent) carries the type-registry handle
- Child startup uses the inherited types
- 3 new probes (struct + enum + parametric)
- Workspace stays at 0 failed

## Scope (what's OUT)

- Gap F-1 / Gap F-2 / Gap G — separate slices
- Phase E V5 (deftest rewrite) — after Phase 2a closes
- Anything under `docs/arc/` (FM 11)
- `~/.claude/` memory system
- Changing TypeEnv's internal representation
- Adding new substrate type-system features

## Ship criteria (6 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `extract_closure` extended to include parent type registry | grep + read |
| B | `ClosurePackage` (or equivalent) carries type-registry handle | grep + read |
| C | Child startup uses inherited types | grep + read |
| D | 3 new probes pass (struct + enum + parametric) | cargo test |
| E | All existing probes still pass; workspace at 2209 + N + 3 / 0 failed | full test |
| F | Hermetic isolation semantics preserved (existing fork/spawn-process tests unchanged) | full test |

**6 rows.** All must PASS.

## Predicted runtime

**30-60 min sonnet.** Self-contained substrate edit (one struct field + one fn modification + child startup update). Probes use existing spawn-process test patterns.

**Hard cap:** 120 min (2×).

## Constraints (hard)

- DO NOT modify TypeEnv's internal representation
- DO NOT add new type-system features (parametric inference, variance, etc.)
- DO NOT touch `docs/arc/` (FM 11)
- DO NOT commit (orchestrator atomic-commits)
- DO NOT use deferral language in SCORE
- DO NOT operate outside `/home/watmin/work/holon/wat-rs/`
- DO NOT touch `~/.claude/` memory system
- DO NOT use --no-verify or skip hooks
- DO NOT extend to Gap F-1 / F-2 / G scope
- Hermetic test suite must stay 100% pass (isolation semantics not regressed)

## Honest delta categories (anticipated)

1. **Inclusion strategy** — whole-registry (simplest) vs filtered-by-body-references (precise but requires walking). Surface choice + rationale.
2. **TypeEnv shape compatibility** — does TypeEnv compose cleanly with `Arc<...>` sharing? If not, surface.
3. **Hermetic regression** — verify existing fork-program/spawn-process tests don't break. If they do, root-cause (likely a test that depended on the child NOT seeing parent types).
4. **Parametric / generic type handling** — does propagating `:test::Wrapper<E>` work cleanly, or are there generics-resolution edge cases?
5. **Anything unexpected** — particularly any closure-extraction edge case (closures capturing closures, etc.)

## Cross-references

- V4 SCORE (failure pattern 3): `SCORE-SLICE-3-PHASE-E-V4-DEFTEST-REWRITE.md`
- ZERO-MUTEX.md (Tier 1 immutable Arc sharing — applies to types)
- Gap F-1 (predecessor slice; struct/enum pregen at register time)
- Gap F-2 (next slice; resolver quote-awareness)
- Gap G (after F-2; Path E macro shape)
- Phase E V5 (unblocked after all 4 gaps)
