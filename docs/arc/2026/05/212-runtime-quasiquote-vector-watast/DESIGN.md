# Arc 212 — runtime quasiquote substitution inside Vector<WatAST> constructor

**Status:** OPEN 2026-05-18 — opened to fix `t6_spawn_process_factory_with_capture_round_trips` whose failure was made readably diagnosable by arc 211's panic-as-EDN tooling. Arc 211 closure depends on this arc's resolution per the **tooling-proven-by-use** discipline (see INTERSTITIAL § 2026-05-18 (post-arc-211e)).

**Priority:** BLOCKING arc 211 INSCRIPTION + the broader arc 170 closure cascade.

## Origin

Arc 170 slice 6 (the substrate redesign retiring closure-extract) inscribed t6 as a known "downstream stone" in its SCORE:

> *"T6 substrate-discovery gap — `wat_arc170_program_contracts::t6_spawn_process_factory_with_capture_round_trips` originally tested closure-capture-across-fork. New substrate retires closure-extract; substrate-equivalent is runtime AST template construction via `:wat::core::quasiquote` + `:wat::core::unquote`. T6's migration to this shape FAILS — runtime quasiquote inside `(:wat::core::Vector :wat::WatAST ...)` constructor does not substitute unquoted symbols. Surfaced as downstream stone; T6's failure preserved with documenting comment."*

Arc 211c (panic_any! audit) confirmed t6 as a known consistent failure. Arc 211d's revert + Category D fixes addressed the dup-removal regression but NOT this substrate-discovery-gap. Arc 211e dedup work didn't touch the macro substrate.

**Today (2026-05-18, post-211e):** running `arc170_program_contracts` binary with `--no-fail-fast` shows 23/24 tests pass; the ONE failure is t6 with exactly the structured EDN diagnostic that arc 211b shipped:

```
#wat.kernel/ProcessPanics [#wat.kernel.ProcessDiedError/RuntimeError 
  ["<entry>:11:61: unknown function: :wat::core::unquote"]]
```

The substrate is now telling us — readably, via the panic-EDN format — exactly what's broken. This arc fixes it.

## Scope

**In scope:**
- Extend substrate's quasiquote/unquote substitution to work inside `(:wat::core::Vector :wat::WatAST ...)` constructor calls
- Ship the fix
- Verify t6 passes in isolation + under `--no-fail-fast` run
- SCORE inscribes the actual mechanism (where in macros.rs / runtime.rs the substitution path needed extension)

**Out of scope:**
- Broader quasiquote behavior on other container shapes (decide per honest diagnostic data; sibling arc if surfaces)
- Closure-extract retirement work (slice 6 territory; already shipped)
- t6 test rewrite (the test exercises the intended pattern; substrate should support it)

## Closure conditions

1. Substrate change ships
2. `t6_spawn_process_factory_with_capture_round_trips` passes in isolation
3. `arc170_program_contracts` binary passes 24/24 with `--no-fail-fast`
4. SCORE doc inscribes mechanism + delta
5. Arc 211 closure becomes unblocked (one of two pre-conditions; arc 213 is the other)

## Cross-references

- Arc 170 SCORE-SLICE-6 (the original substrate-discovery-gap inscription)
- Arc 211 SCORE-211C-AUDIT (confirmed t6 as consistent failure)
- Arc 211 DESIGN § "Tooling-proven-by-use closure condition" (the blocking relationship)
- Arc 211 INSCRIPTION (pending; awaits this arc)
- INTERSTITIAL § 2026-05-18 (post-arc-211e) "Tooling proven by use — closure-discipline extension"
- `tests/wat_arc170_program_contracts.rs:483` (t6 source)
- t6 panic output: `<entry>:11:61: unknown function: :wat::core::unquote`

## Tooling-proven-by-use principle

This arc serves dual purpose:
1. **Fix t6** (substrate correctness)
2. **PROVE arc 211's tooling enabled this fix** (substrate-tooling-validation)

The arc 211 panic-as-EDN format made t6's failure honestly diagnosable. Pre-arc-211, the same failure surfaced as `Box<dyn Any>` placeholder — a kind of "we know SOMETHING is broken but cannot say what." Arc 212's existence — its ability to scope precisely from the visible diagnostic — IS the proof.

When arc 212 closes, the SCORE will reference the EDN diagnostic as the entry point. That reference is the validation evidence arc 211 needs.

---

## Scope EXPANDED 2026-05-18 (post-slice-α) — failure-engineering: eliminate the CLASS

**User direction:** *"why is there more than exactly once fix?... i think 212 is blocked on /everyone/ being correct - always"*

The spot-fix in slice α (walk_quasiquote's missing Vector arm) addressed t6 but not the CLASS. Audit revealed 9+ analogous walkers with the same pattern (recurse on List, skip Vector). Per failure-engineering doctrine (`scratch/FAILURE-ENGINEERING.md` component 3: *"eliminate the CLASS, not the symptom"*), the honest fix is at the substrate layer — not N per-walker patches.

### Locked expanded scope

| Slice | What | Status |
|---|---|---|
| **212-α** | walk_quasiquote Vector arm (t6's specific fix) | SHIPPED `135607b` — preserved as historical record of the spot-fix that revealed the broader scope |
| **212-β** | Mint `WatAST::children()` primitive in src/ast.rs | SHIPPED in this commit — substrate owns "what are the children of an AST node?"; walkers can't get it wrong |
| **212-γ** | Comprehensive audit: every fn in src/ + crates/ that pattern-matches on WatAST classified as Walker (recurses → must migrate) or Leaf-decomposition (single shape → leave) | PENDING — sonnet spawn |
| **212-δ** | Migration sweep: every walker uses `node.children()` for generic recursion; walker-specific special-case logic preserved | PENDING — sonnet spawn (may bundle with γ) |
| **212-ε** | INSCRIPTION + closure | PENDING — gated on γ+δ + tests green |

### Why the expanded scope is honest

The narrow scope (slice α) fixed t6 + claimed arc 211 tooling validation. Both true. BUT it left the CLASS of "walker silently skips Vector" still alive in 9+ other walkers. Arc 211 closure was about to ship with this class still present in the substrate.

The discipline: **arc 212 closure requires that EVERY walker is correct, ALWAYS.** Not "this one walker is fixed." Not "we'll audit the others later." The class itself is eliminated — `children()` makes "miss Vector arm" structurally impossible for walkers that route through it.

When future AST variants land (e.g., new compound shape), `WatAST::children()` updates ONCE; every walker that uses it benefits automatically; no per-walker audit needed.

### Per-failure-engineering doctrine

This expansion is **failure engineering at the walker layer**:

| FE component | Application |
|---|---|
| 1. Failure is data | t6's failure read literally → walker-divergence pattern surfaced |
| 2. Stop immediately | Did not ship 9 per-walker patches; halted to find the class-level fix |
| 3. Eliminate the CLASS | `children()` primitive owns recursion; "miss Vector arm" structurally impossible for users |

The substrate ships `children()`; the walker layer is rebased on it; the bug class is gone forever.

### Closure conditions (EXPANDED)

1. `WatAST::children()` method shipped (slice β) — DONE in this commit
2. Comprehensive audit catalog produced (slice γ) — pending
3. Every walker migrated to `children()` OR explicitly documented as intentional single-shape-handler (slice δ) — pending
4. Workspace tests still green post-migration (no regressions) — pending verification
5. t6 still passes (slice α preserved by virtue of children() handling Vector at the substrate layer) — verified post-migration
6. SCORE-212-AUDIT.md (or equivalent) inscribes findings + validates arc 211 tooling-proven-by-use chain
7. Arc 211 closure becomes unblocked (one of two pre-conditions; arc 213 is the other)

### Tooling-proven-by-use cascade extended

Arc 212's expanded scope also validates arc 211 at a NEW layer:
- Original validation (slice α): the panic-EDN diagnostic for t6 enabled the fix
- Extended validation (slices β-δ): the user's failure-engineering discipline + the substrate's audit pattern enabled CATCHING the class-level flaw (not just t6's instance)

Both validations land in SCORE-212-AUDIT.md when slice ε ships. Arc 211 closes when slice ε ships AND arc 213 closes.
