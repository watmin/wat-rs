# Arc 170 slice 3 Gap G BRIEF — Path E strict-isolation shape for `deftest-hermetic`

**Sonnet.** Fourth and final Phase 2a gap slice (after Gap F-1 + F-3 + F-2 all land). Mints the strict-isolation contract that user-facing `deftest-hermetic` provides: prelude lives INSIDE the closure; nothing modifies parent's frozen world; substrate-enforced via spawn-process boundary.

## Backstory

Phase E V4 (commit `f2de549` SCORE) attempted to collapse `deftest-hermetic` into a `deftest` alias (Path A only — prelude at outer top-level). User direction 2026-05-13 corrected this:

> *"Tests will continue to use 'strict isolation' when the user chooses hermetic — nothing in the global runtime is able to be modified when a hermetic test runs."*
> *"users must make a choice where their programs run."*

The two macros provide two distinct isolation contracts:

| Macro | Prelude lives in | Parent world | Contract |
|---|---|---|---|
| `deftest` | Outer top-level under `do` (Path A) | Has the prelude content | Convenient; less strict |
| `deftest-hermetic` | Inside the closure passed to `run-hermetic` (Path E) | UNTOUCHED | Strict isolation |

This Gap mints `deftest-hermetic`'s Path E shape + enforcement probes that prove strict isolation holds.

## Goal — `deftest-hermetic` macro body emits Path E shape

Target expansion (mirrors V4 V3 BRIEF's Delta 4 alternative shape):

```scheme
(:wat::core::defmacro
  (:wat::test::deftest-hermetic
    (name :AST<wat::core::nil>)
    (prelude :AST<wat::core::nil>)
    (body :AST<wat::core::nil>)
    -> :AST<wat::core::nil>)
  `(:wat::core::define (~name -> :wat::kernel::RunResult)
     (:wat::test::run-hermetic
       (:wat::core::do
         ~@prelude
         ~body))))
```

**Key property:** prelude is INSIDE the closure. Outer top-level only has the test fn. Child process runs prelude + body in its own world. Parent's frozen symbol table doesn't get the test's types / helpers.

Mirror `:wat::test::make-deftest-hermetic` factory to emit configured-deftest-hermetic variants that follow Path E.

## Enforcement: probes prove strict isolation

The substrate ENFORCES isolation by SHAPE — prelude in closure → naturally scoped to child. But probes verify the contract holds at observable behavior:

1. **Parent symbol table doesn't have test's prelude types**
   - Probe: declare a struct in deftest-hermetic prelude; verify `world.symbols().get(":test::Type")` returns None
   - Demonstrates: parent's frozen world is untouched
2. **Cross-test prelude isolation**
   - Probe: two `deftest-hermetic` calls in same file each declare DIFFERENT types with same FQDN; both work (each in its own child world)
   - Demonstrates: no collision because tests don't share world
3. **Body cannot reach into parent's runtime**
   - Probe: parent declares config X = 1; deftest-hermetic body checks config value (in child it's the inherited config but immutable copy); demonstrate body's view is sealed
   - Trickier; may be partial — surface scope honestly
4. **Test fn name visible in parent**
   - Probe: deftest-hermetic registers the test FN at parent's top-level (so test-runner can find it); verify parent has the fn name but NOT the prelude's content
   - Demonstrates: only the fn-entry-point crosses the parent/child boundary

## Required reading IN ORDER

1. **`docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-PHASE-E-V4-DEFTEST-REWRITE.md`** — V4 failure analysis + Path E discussion
2. **`docs/arc/2026/05/170-program-entry-points/RETIREMENT-THEATER-INVENTORY.md`** — Phase 2a priority queue
3. **`docs/arc/2026/05/170-program-entry-points/TIERS.md`** — runtime tier doctrine (each tier shares less)
4. **`wat/test.wat:305-345`** — current deftest + deftest-hermetic macro bodies (post-Gap F-1/F-3/F-2 state)
5. **`wat/test.wat:496-575`** — run-hermetic + run-hermetic-driver (Phase C; what deftest-hermetic uses)

## Implementation path

### Phase 1 — Rewrite `:wat::test::deftest-hermetic` macro body

Replace current body with the Path E shape (prelude inside the `(:wat::core::do ~@prelude ~body)` passed to run-hermetic).

### Phase 2 — Rewrite `:wat::test::make-deftest-hermetic` factory

Mirror — the factory's emitted inner macro uses Path E shape.

### Phase 3 — Documentation header update

`wat/test.wat:320+` deftest-hermetic header. Update to describe Path E shape + strict isolation contract explicitly:

> *"deftest-hermetic provides strict isolation: prelude content runs inside the test's child subprocess; parent's frozen symbol table does NOT receive the prelude. Use when test setup includes types/helpers that should NOT pollute the parent's world. Compare to deftest: deftest splices prelude at outer top-level (parent has them) — appropriate when prelude is shared file-level setup; deftest-hermetic when prelude is sandbox-internal."*

### Phase 4 — Enforcement probes

Create `tests/probe_deftest_hermetic_isolation.rs` with the 4 probes above (or sonnet's equivalent set). Each demonstrates a strict-isolation property.

### Phase 5 — Verify

- All probes pass
- All existing tests pass (deftest-hermetic users — those currently using sandbox-internal prelude content — now have their content correctly land in the child)
- Workspace at full pass

## Scope (what's IN)

- `:wat::test::deftest-hermetic` macro body Path E rewrite
- `:wat::test::make-deftest-hermetic` factory Path E rewrite
- Documentation header update
- 4+ enforcement probes proving strict isolation
- Workspace stays at 0 failed

## Scope (what's OUT)

- `:wat::test::deftest` rewrite — that's Phase E V5 (post-Phase-2a forward work)
- Gap F-1 / F-3 / F-2 — predecessors, all shipped before this
- Phase F retirement of run-sandboxed-* — after V5
- Anything under `docs/arc/` (FM 11)
- ~/.claude/ memory system
- Cross-test leakage DETECTION walker (would be separate arc; this slice provides shape-based enforcement only)

## Ship criteria (6 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `:wat::test::deftest-hermetic` body uses Path E shape (prelude inside `do` inside `run-hermetic`) | grep + read |
| B | `:wat::test::make-deftest-hermetic` factory follows Path E | grep + read |
| C | Documentation header explicitly names "strict isolation" contract | manual review |
| D | 4+ enforcement probes pass | cargo test |
| E | Workspace at baseline + new probes / 0 failed | full test |
| F | Existing deftest-hermetic users (service-template, ambient-stdio, roundtrip) work correctly under Path E | full test |

**6 rows.** All must PASS.

## Predicted runtime

**45-90 min sonnet.** Macro body change is small. Probes are the load-bearing work; require careful design to actually demonstrate strict isolation (not just "tests pass").

**Hard cap:** 180 min (2×).

## Constraints (hard)

- DO NOT modify `:wat::test::deftest` macro body (that's Phase E V5)
- DO NOT modify run-hermetic / run-hermetic-driver substrate
- DO NOT modify any test call site outside the new probe file
- DO NOT touch `docs/arc/` (FM 11)
- DO NOT commit (orchestrator atomic-commits)
- DO NOT use deferral language in SCORE
- DO NOT operate outside `/home/watmin/work/holon/wat-rs/`
- DO NOT touch `~/.claude/` memory system
- DO NOT use --no-verify or skip hooks
- If existing deftest-hermetic users break, STOP — Gap F-1/F-3/F-2 should have closed the substrate gaps; if a test fails it indicates the gap-closure was incomplete

## Honest delta categories (anticipated)

1. **Probe design** — what specifically demonstrates "parent's frozen world unchanged"? Surface the exact assertion shape
2. **Cross-test prelude collision probe** — does it work as designed? (Path E's promise)
3. **make-deftest-hermetic factory composition** — does the `~~default-prelude` double-unquote still work?
4. **Documentation header wording** — surface for orchestrator review
5. **Anything unexpected** — particularly if any existing deftest-hermetic user reveals an assumption that Path E breaks

## Cross-references

- V4 SCORE: `SCORE-SLICE-3-PHASE-E-V4-DEFTEST-REWRITE.md`
- TIERS.md: runtime tier doctrine (each tier shares less)
- Gap F-1 / F-3 / F-2 — substrate enablers (all closed before this)
- Phase E V5 — deftest Path A rewrite (post-Phase-2a forward work)
- Cross-test prelude leakage DETECTION (future arc; substrate enforcement walker)
