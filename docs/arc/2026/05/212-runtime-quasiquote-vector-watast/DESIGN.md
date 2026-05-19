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

---

## ζ-newtype-wall scope (drafted 2026-05-18 post-arc-213-α-spawn)

### The wall's mechanism

Mint a `Children` newtype in `src/ast.rs` whose inner `Vec<WatAST>` is **module-private**. Public surface: iteration accessors only. The three compound `WatAST` variants flip their tuple-inner type:

```rust
// In src/ast.rs (the only module that can access Children's inner Vec)
pub struct Children {
    items: Vec<WatAST>,  // PRIVATE — only ast.rs can index/iter directly
}

impl Children {
    pub fn new(items: Vec<WatAST>) -> Self { Self { items } }
    pub fn iter(&self) -> std::slice::Iter<'_, WatAST> { self.items.iter() }
    pub fn as_slice(&self) -> &[WatAST] { &self.items }
    pub fn len(&self) -> usize { self.items.len() }
    pub fn is_empty(&self) -> bool { self.items.is_empty() }
    pub fn first(&self) -> Option<&WatAST> { self.items.first() }
    pub fn get(&self, idx: usize) -> Option<&WatAST> { self.items.get(idx) }
    pub fn into_vec(self) -> Vec<WatAST> { self.items }  // consumption-only escape hatch
}

pub enum WatAST {
    // ... leaves unchanged ...
    List(Children, Span),
    Vector(Children, Span),
    StructPattern(Children, Span),
}

impl WatAST {
    pub fn children(&self) -> &[Children] {
        // returns &[WatAST] via Children::as_slice on the inner;
        // updated to dispatch through the newtype
    }
}
```

### Why this IS the L2 wall

Today: pattern-match destructure `WatAST::List(items, span)` binds `items: &Vec<WatAST>`. The walker can call `items.iter()`, `&items[0]`, anything `Vec` offers. The wrong shape (recurse on List, forget Vector) is one match-arm away from happening.

Post-ζ: pattern-match destructure `WatAST::List(children, span)` binds `children: &Children`. The walker can call `children.iter()` — same iteration surface, but the type signals "this is one variant's children; you are responsible for handling the other variants too." `Vec` surface is gone; direct indexing into a `Vec<WatAST>` outside `ast.rs` is **compile-time impossible**.

This does NOT prevent "walker matches on List only and ignores Vector/StructPattern" by itself — that bug class needs L3 (η-visitor) to eliminate structurally. L2 raises the friction: the walker's natural shape becomes "ask the parent node for `children()` → iterate over all compound shapes uniformly." The "match on List specifically" shape becomes visibly suspicious.

### Reference precedent: arc 213 α Pidfd

Arc 213 stone α (BRIEF + EXPECTATIONS at commit `e8c2243`) mints the canonical `Pidfd` type with the **same shape of structural enforcement at the kernel-interface layer**:

| Aspect | arc 213 α `Pidfd` | arc 212 ζ `Children` |
|---|---|---|
| Wrapped resource | `OwnedFd` (kernel-managed pidfd) | `Vec<WatAST>` (AST node children) |
| Inner field | `fd: OwnedFd` — `Pidfd`-private | `items: Vec<WatAST>` — `Children`-private |
| Public construction | NONE outside `spawn_lifelined` (typestate-equivalent for non-stale handle) | `Children::new(Vec<WatAST>)` (callable from anywhere; the wall is on ITERATION, not construction) |
| Public surface | `poll_exit`, `wait_status`, `try_wait`, `send_signal`, `pid()` | `iter`, `as_slice`, `len`, `is_empty`, `first`, `get`, `into_vec` |
| Wrong shape made impossible | `kill(pid)` by recovered PID (PID-reuse race) | Direct `Vec` indexing / methods outside the surface |
| Doctrine source | `feedback_substrate_owns_not_callers_match` | Same |

Both stones apply substrate-imposed-not-followed at their respective interface layers. Arc 213 α ships FIRST (per tractability tiebreaker — α's worked example informs ζ's design). When ζ ships, the DESIGN cites α as the precedent for the pattern.

### Construction-vs-iteration asymmetry

The wall lives on **iteration surface**, not construction. Anyone can call `Children::new(vec)` — the parser does this, the runtime quasiquote builder does this, macro expansion does this. That's correct: construction sites are bounded + well-known, and the inner `Vec` is OWNED by the newtype the moment construction completes; no caller retains a `&mut Vec<WatAST>`.

The wall stops **consumers** from reaching past the surface. After construction, `Children`'s inner `Vec` is inaccessible outside `ast.rs`. Walkers, type-checkers, resolvers, runtime evaluators all interact via the public surface — `iter` for traversal, `len/is_empty/first/get` for query, `as_slice` for slice-API compatibility, `into_vec` when ownership is required (parser may need this for splice operations).

### Migration scope (evidence-grounded 2026-05-18; refined post-extended-audit)

#### Pattern-match destructure sites (LHS — the WALL exposure point)

```
List sites:    331
Vector sites:   72
StructPattern:  32
TOTAL:         435 destructure sites
```

These bind `items: &Vec<WatAST>` (or `Vec<WatAST>` by value). Under ζ they bind `children: &Children` (or `Children` by value). Migration: rename binder + call Children surface methods instead of Vec methods. Most are mechanical.

#### Per-file pattern-match concentration

```
106  src/runtime.rs
 78  src/check.rs
 59  src/macros.rs
 42  src/closure_extract.rs
 17  src/types.rs
  5  src/load.rs
  4  src/parser.rs
  4  src/dispatch.rs
  3  src/resolve.rs
  3  src/config.rs
  3  src/ast.rs        ← internal; survives wall
  2  src/lower.rs
  2  src/form_match.rs
  1  src/test_runner.rs
  1  src/hash.rs
```

#### Construction sites (RHS — sites that BUILD compound variants)

Direct construction `WatAST::List(items_expr, span_expr)` appears at ~20+ sites in `closure_extract.rs` alone (closure→AST emission), plus parser/form_match/check sites. Conservative estimate: ~50-100 construction sites total.

Construction site migration: each `WatAST::List(items_expr, span_expr)` becomes either:
- `WatAST::List(Children::new(items_expr), span_expr)` (explicit), OR
- `WatAST::list_with_span(items_expr, span_expr)` (NEW convenience constructor variant — currently only `WatAST::list(items)` with `Span::unknown()` exists; ζ-1 adds the span-bearing variant)

**Existing convenience constructors are barely used:** `WatAST::list(...)` 2 call sites; `WatAST::vector(...)` 0; `WatAST::struct_pattern(...)` minimal. Construction is dominated by direct tuple-variant syntax. ζ-1 must extend convenience constructors AND update direct-construction sites.

#### Non-mechanical patterns audit (post-extended-grep, NEW evidence)

| Pattern | Count | Treatment |
|---|---|---|
| `items.iter()` | 46 | Mechanical — `children.iter()` (covered by Children surface) |
| `items.len()` / `is_empty()` / `first()` / `get(idx)` | 287 | Mechanical — covered by Children surface |
| `&items[..]` slice patterns | 192 | Mechanical — `children.as_slice()` covers most; some require `&children.as_slice()[..]` |
| `items[idx]` direct indexing | 228 | **HALF are LOCAL Vec builders (false positives — naming overload).** True destructure-side indexing migrates to `children.get(idx).expect(...)` or equivalent (need to verify no `unwrap()`-eliding-bounds-check sites) |
| `items.into_iter()` | 12 | **Children needs `IntoIterator<Item=WatAST>` impl** (my original DESIGN omitted this — known defect, now fixed) — OR sites use `children.into_vec().into_iter()` |
| `items.clone()` | 5 | **Children needs `#[derive(Clone)]`** (my original DESIGN omitted — known defect, now fixed) |
| `items.as_slice()` | 3 | Mechanical — covered |
| `items.iter_mut()` | 3 | **CLASSIFY** — likely LOCAL Vec builder false positives. If real destructure-mut, those sites violate the immutable-pattern-match contract today; need per-site classification. **NO `iter_mut` on Children** (defeats wall) |
| `items.push/extend/insert/remove/swap/sort` | 73 | **VAST MAJORITY are LOCAL Vec builders** (`do_items.push(...)` → `WatAST::List(do_items, ...)`). Sample-verified in `check.rs:7504-7540` and `closure_extract.rs:2092-2130` |

#### `items` is an overloaded identifier

Grep evidence shows `items` is used for two distinct things:
1. **Destructured compound-variant inner** (LHS of pattern-match) — under the wall; migrates to `children` binding + Children surface
2. **Locally-owned Vec being built up** (e.g., `let mut items = Vec::new(); items.push(...); WatAST::List(items, span)`) — NOT under the wall; the local Vec stays Vec; only the final construction wraps via Children::new

Mutation patterns (73 push/extend/etc. sites) are overwhelmingly category 2. They survive ζ migration unchanged except for the final construction call.

### Children surface (refined per audit — corrects original DESIGN omissions)

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Children {
    items: Vec<WatAST>,  // module-private to ast.rs
}

impl Children {
    pub fn new(items: Vec<WatAST>) -> Self { Self { items } }
    pub fn iter(&self) -> std::slice::Iter<'_, WatAST> { self.items.iter() }
    pub fn as_slice(&self) -> &[WatAST] { &self.items }
    pub fn len(&self) -> usize { self.items.len() }
    pub fn is_empty(&self) -> bool { self.items.is_empty() }
    pub fn first(&self) -> Option<&WatAST> { self.items.first() }
    pub fn get(&self, idx: usize) -> Option<&WatAST> { self.items.get(idx) }
    pub fn into_vec(self) -> Vec<WatAST> { self.items }  // escape hatch (minimize usage)
}

impl IntoIterator for Children {
    type Item = WatAST;
    type IntoIter = std::vec::IntoIter<WatAST>;
    fn into_iter(self) -> Self::IntoIter { self.items.into_iter() }
}

impl<'a> IntoIterator for &'a Children {
    type Item = &'a WatAST;
    type IntoIter = std::slice::Iter<'a, WatAST>;
    fn into_iter(self) -> Self::IntoIter { self.items.iter() }
}
```

**NO `iter_mut()`, `push()`, `extend()`, etc.** — the wall stays up. Mutation patterns operate on local Vec<WatAST>; Children is constructed once when the local Vec is finalized.

The `into_vec()` escape hatch exists for the rare case where a transformation genuinely needs owned `Vec<WatAST>` (e.g., consuming destructure into a different shape). ζ-1 should grep usage of `into_vec()` post-migration; if it exceeds ~10 sites, those sites are candidate refactors per `feedback_attack_foundation_cracks` (the wall is leaking; investigate).

### Sub-stone decomposition (per `feedback_iterative_complexity`; refined runtime predictions)

| Sub-stone | What | Proof gate | Predicted runtime |
|---|---|---|---|
| ζ-1-mint | Mint `Children` in `src/ast.rs` with Clone + Debug + PartialEq derives + IntoIterator impls (owned + borrowed); flip `WatAST::List/Vector/StructPattern` variant inner type to `(Children, Span)`; update `children()` accessor; extend `WatAST::list/vector/struct_pattern` constructors to wrap; ADD `WatAST::list_with_span(Vec<WatAST>, Span)` (and vector/struct_pattern variants) for sites that pass an existing span | `cargo build --release` surfaces ~435+ errors across consumer files; ast.rs compiles clean | 15-30 min (substrate-internal only) |
| ζ-2-closure-extract | Migrate `src/closure_extract.rs` FIRST (despite smaller pattern-match count) — heaviest CONSTRUCTION-site concentration (~20+ direct construction sites; closure→AST emission) | `cargo build --release` clean for closure_extract.rs scope; affected tests green | 60-90 min |
| ζ-3-runtime | Migrate `src/runtime.rs` (106 List + Vector/StructPattern siblings; biggest pattern-match consumer) | Same shape; cargo build + tests | 60-90 min |
| ζ-4-check | Migrate `src/check.rs` (78 List + siblings) | Same | 45-60 min |
| ζ-5-macros | Migrate `src/macros.rs` (59 List + siblings; includes 2 `items.into_iter()` sites) | Same; verify IntoIterator path | 45-60 min |
| ζ-6-tail | Migrate remaining files (types/load/parser/dispatch/resolve/config/lower/form_match/test_runner/hash + any others surfaced by cargo). Includes 5 remaining `into_iter()` sites + the 3 `iter_mut()` sites needing classification | `cargo build --release --workspace` clean; full workspace tests green | 30-60 min |
| ζ-7-verify | Workspace cargo test; t6 still passes; baseline preserved; grep usage of `Children::into_vec()` (if >10 sites, investigate per `feedback_attack_foundation_cracks`) | Workspace failure delta ≤ 0 vs pre-ζ baseline | 10-20 min orchestrator |

**Total predicted (refined):** 5-8 hours across 6 sonnet spawns (ζ-1 through ζ-6) + orchestrator verification (ζ-7). Per-stone trust gate: orchestrator commits ζ-N after independent verification before spawning ζ-(N+1). Order change vs original DESIGN: closure_extract.rs migrates BEFORE runtime.rs because construction-site density is the higher-friction concern (sample evidence: closure_extract has ~20+ direct `WatAST::List(items_expr, span)` construction sites concentrated in one file).

### Stone discipline per ζ-N spawn

Each sub-stone follows the same contract as δ-stones:
- ONE concern: ONE file's migration (ζ-2 through ζ-6) or the substrate mint (ζ-1)
- ONE proof gate: `cargo build --release` clean for the migrated file's compile errors
- STOP triggers VERBATIM: "If anything outside this file's migration surfaces, retreat — do not investigate, do not theorize, do not open another file"
- NO mention of workspace failure count (per the δ-process-scope lesson)
- Construction sites + pattern-match sites in the assigned file both migrate; nothing else

ζ-7 (workspace verify) is orchestrator-side; no sonnet spawn.

### Why this isn't FM-6 preemptive doc-update

ζ scope detail drafts BEFORE ζ-1 ships, but:
- ζ shape is DERIVED from the existing children() primitive (β shipped) + arc 213 α's worked precedent (BRIEF on disk)
- The migration plan is DERIVED from grep evidence (435 sites; per-file breakdown above)
- No code changes yet; no commitment to ship anything per this section until α SCORE returns
- This section UPDATES the DESIGN with verified scope; per FM 6 the anti-pattern is "speculative DESIGN updates before work is proven"; here the work being designed-for is the L4 endpoint already named in the L0-L4 ladder

Per `feedback_docs_when_confused`: scope is now obvious; the design has shape; future-me reads this and knows what ζ ships.

### Cross-references

- Arc 213 α BRIEF (`docs/arc/2026/05/213-libc-fork-mismanagement/BRIEF-213-ALPHA-MINT-PIDFD-PRIMITIVE.md`) — the worked precedent
- `feedback_substrate_owns_not_callers_match` — the doctrine ζ extends to the AST layer
- `feedback_tractability_tiebreaker` — why arc 213 α ships first (its Pidfd is the worked precedent ζ references)
- recovery doc § FM 15 — substrate-as-teacher cascade is the iteration mechanism for ζ-2 through ζ-6
- `feedback_iterative_complexity` — sub-stone decomposition (one file at a time)
- INTERSTITIAL § 2026-05-18 (post-Linux-doctrine) "Tractability tiebreaker" — the sequencing discipline
- INTERSTITIAL § 2026-05-18 "L4 endgame realized" — the doctrine commitment ζ implements at L2
