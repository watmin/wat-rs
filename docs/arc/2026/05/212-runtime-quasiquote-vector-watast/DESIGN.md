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

Both validations land in INSCRIPTION when arc closes. Arc 211 closes when arc 212 INSCRIPTION ships AND arc 213 closes.

---

## Scope EXPANDED 2026-05-18 (post-L4-conversation) — substrate-imposed-not-followed at TWO layers

**User direction:** *"i think 'uh.. you can use list because reasons' is absolute bullshit - there's always only one way / the need to be list specific must be justified strongly - extremely high bar to breach for being special"* + *"can we make mistakes for calling anything but children a panic?... how strict can we get here?..."*

The convention-level discipline (L1: "everyone migrates to children() by hand") is the weakest layer. The substrate's existing discipline (`feedback_substrate_owns_not_callers_match`, `feedback_refuse_easy_solutions`) demands STRUCTURAL impossibility, not author discipline.

### Strictness ladder

| Level | Mechanism | Bug class eliminated |
|---|---|---|
| L0 | Spot-fix t6 (slice α shipped); mint children() (slice β shipped) | t6's specific bug |
| L1 | Convention: every walker migrates to children() | Bug class via author discipline |
| **L2** | Newtype wall: inner `Vec<WatAST>` private; only `children()` accessor public | Direct iteration STRUCTURALLY IMPOSSIBLE |
| **L3** | Visitor primitive: `walk_ast<F>` + `Action::{Descend,Skip,Stop}`; walker bodies refactor to closure shape | Writing recursion outside the visitor STRUCTURALLY IMPOSSIBLE |
| **L4** | L2 + L3 composed | Wrong becomes structurally impossible at BOTH access AND iteration layers |

L4 IS the idealized endpoint. L2 alone leaves walker authors free to write custom recursion; L3 alone leaves consumers free to bypass via pattern-match. L4 closes both doors.

### Single-shape-walker classification REJECTED

Sonnet's initial audit (uncommitted dirty tree, partially shipped) classified two walkers as "Single-shape-walker — intentionally List-only":
- `validate_comm_positions` — broke `arc112_slice2b_schemes_wire_through_typechecker` under naive children() migration
- `collect_process_calls` — would break stdlib patterns in `wat/test.wat`, `wat/kernel/hermetic.wat`, `wat/kernel/sandbox.wat`

This framing is REJECTED. The breakages are the substrate teaching that the WALKER RULES are incomplete:
- `validate_comm_positions` lacks position-awareness — needs to recognize bound-name-later-matched as a fourth permitted slot
- `collect_process_calls` lacks scope-boundary tracking — needs to RESET local-scope tracking at nested let boundaries

Both walkers can (and MUST) be made correct under children(). Neither earns exemption. Sharpening targets are stones δ-comm-positions and δ-process-scope.

### Locked stone chain (L0 → L4 trajectory)

| Stone | Layer | What | Status |
|---|---|---|---|
| α | L0 | walk_quasiquote Vector arm (t6 fix) | SHIPPED `135607b` |
| β | L0 | `WatAST::children()` primitive in src/ast.rs | SHIPPED `bc31342` |
| γ-1 | L0 | Audit catalog (read-only); per-site Walker/Leaf/sharpening-target classification | PENDING — sonnet |
| δ-bulk | L1 | 12 walker migrations already shipped on dirty tree (uncommitted at this DESIGN write); to be committed atomically with γ-1 BRIEF | DIRTY-TREE-RETAINED |
| δ-bare-primitives | L1 | Migrate `walk_for_bare_primitives` (line 2705) | PENDING — sonnet |
| δ-comm-positions | L1 | Sharpen `validate_comm_positions` with position-awareness (4th permitted slot: bound-name-later-matched) | PENDING — sonnet |
| δ-process-scope | L1 | Sharpen `collect_process_calls` with scope-boundary tracking (RESET at nested let) | PENDING — sonnet |
| ζ-newtype-wall | L2 | Private `Children` newtype; pattern-match + construction sites migrate across codebase. **ALSO scope-binds:** `walk_for_bare_legacy_console` (src/check.rs:3170; no standalone stone — no wat-test gate exists because legacy `:wat::console::*` paths are fully retired; the L2 wall will refuse this walker's direct `items` iteration at compile time, forcing children() migration). NOT deferral per recovery doc § FM 11 — the closure mechanism is structurally inevitable when L2 ships. | PENDING — sonnet (long sweep, substrate-as-teacher cascade) |
| η-visitor | L3 | Mint `walk_ast<F>` + `Action::{Descend,Skip,Stop}`; walker bodies refactor | PENDING — sonnet |
| θ | — | SCORE + INSCRIPTION | PENDING — orchestrator |

### Stone discipline (sonnet briefing contract)

Each δ/ζ/η stone is briefed under the failure-engineering operational mode + the slow-is-smooth-smooth-is-fast cadence. Per stone:

- ONE concern (one walker, one named test, one rule)
- ONE wat-test name as the proof gate
- STOP triggers VERBATIM: "if anything outside this concern surfaces, retreat — do not investigate, do not theorize, do not open the file"
- NO mention of "workspace failure count" (that framing invited arc 213 scope-creep when first attempted)
- Wat-test green pre/post is the success signal; nothing else

### Closure conditions (REVISED for L4 endpoint)

1. ✅ `WatAST::children()` shipped (β)
2. ✅ walk_quasiquote Vector arm shipped (α)
3. Audit catalog produced (γ-1)
4. 12 sonnet-shipped migrations atomically committed (δ-bulk)
5. Remaining BRIEF walker (`walk_for_bare_primitives`) migrated (δ-bare-primitives)
6. Both sharpening targets shipped with scope/position awareness (δ-comm-positions, δ-process-scope)
7. **L2 newtype wall in place — inner `Vec<WatAST>` is private outside ast.rs (ζ-newtype-wall)**
8. **L3 visitor primitive in place — `walk_ast<F>` is the only recursion site; walker bodies are closures (η-visitor)**
9. Workspace tests green post-each-stone (per-stone proof; no aggregate "workspace count" framing)
10. t6 still passes
11. INSCRIPTION ships (θ) telling the full L0 → L4 story
12. Arc 211 closure unblocks (one of two pre-conditions; arc 213 is the other)

### Per-failure-engineering doctrine (extended for L4)

| FE component | Application at L4 |
|---|---|
| 1. Failure is data | Sonnet's empirical breakages on `validate_comm_positions` + `collect_process_calls` are walker-rule-incompleteness data, NOT exemption justification |
| 2. Stop immediately | Halted sonnet mid-spawn for scope-creep into arc 213; reframed Single-shape-walker classification as sharpening targets |
| 3. Eliminate the CLASS | L4 closes both "miss Vector arm" (via L2 wall) AND "custom recursion shape" (via L3 visitor); bug class structurally extinct forever |

The substrate ships the wall AND the visitor; walker layer is rebased on both; the bug class cannot return because the wrong shape cannot be expressed.
