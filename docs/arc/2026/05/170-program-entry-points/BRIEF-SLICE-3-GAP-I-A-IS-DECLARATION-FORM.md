# Arc 170 slice 3 Gap I-A BRIEF — mint `is_declaration_form` + unify prelude lift

**Sonnet.** Substrate fix that closes the drift Gap H surfaced. Gap H's `is_prelude_form` covered 3 of 8 declaration forms (define/struct/enum); 5 remain uncovered (def/defmacro/define-dispatch/newtype/typealias) and currently trigger `DefNotTopLevel` / `DefineInExpressionPosition` / `EvalForbidsMutationForm` errors at fn-body do-prefix despite all being top-level-only by the same constraint.

This Gap mints `is_declaration_form` as the source-of-truth predicate for the 8 declaration forms, routes the prelude-lift through it, retires `is_prelude_form`.

User direction 2026-05-13 (after four-questions + gaze convergence):
- *"should is_mutation_form be the thing who is consulted not is_prelude_form"* — yes, but narrower
- Loads + config setters explicitly out-of-scope (honest scope-bounding, NOT deferral)
- Name `is_declaration_form` settled by /gaze ward (`is_top_level_form` lies; `is_startup_form` / `is_binding_form` / `is_definer_form` / `is_def_form` mumble; `is_declaration_form` speaks)

## Backstory in one sentence

Gap H closed the lift drift partially (3 of 8 declaration forms); Gap I-A closes the rest by routing through a properly-scoped subset of `is_mutation_form`'s union.

## Goal — `is_declaration_form` is the lift's source-of-truth

Today: `is_prelude_form` (closure_extract.rs:1762) enumerates 3 keywords inline. If a user writes `(:wat::core::def :x 42)` at a fn body's do-prefix, `is_prelude_form` returns false; the form stays in the body; the check-validator catches it with `DefNotTopLevel`. Same for defmacro/define-dispatch/newtype/typealias — they hit other discipline mechanisms.

Target: mint `pub fn is_declaration_form(head: &str) -> bool` in `src/freeze.rs` adjacent to `is_mutation_form` covering the 8 declaration forms. Retire `is_prelude_form`; have `closure_extract.rs::split_body_prelude` call `is_declaration_form` directly. The lift now covers all 8 forms; the 5 newly-covered work at fn-body do-prefix.

## Why this scope (and NOT broader)

The architectural insight (four questions verdict 2026-05-13):
- `is_mutation_form` is a UNION over THREE semantic categories: declarations (bind names), loads (bring in external content), config setters (mutate runtime state).
- Routing the LIFT through the union would assert all three categories are equivalent. They're not.
- Mint a narrower predicate for ONE category (declarations). Each category gets its own discipline if it ever needs one.

Affirmative scope-bounding for the other two categories: loads (`load-file!` / `digest-load!` / `signed-load!`) + config setters (`config::set-*`) at fn-body do-prefix remain unlifted. If a real caller surfaces a use case, a separate arc opens. NOT deferral — out-of-scope by architectural intent.

## Required reading IN ORDER

1. **`docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-GAP-H-PRELUDE-LIFT-TO-PROLOGUE.md`** (commit `36030c3`) — the precedent slice; pattern for the lift integration
2. **`docs/arc/2026/05/170-program-entry-points/INTERSTITIAL-REALIZATIONS.md`** § "Gap I and the list of special things question" — four-questions verdict + gaze convergence + the architectural reasoning
3. **`src/freeze.rs`** lines 1225-1270 — `refuse_mutation_forms` + `is_mutation_form` (the source-of-truth union; the new predicate sits adjacent)
4. **`src/closure_extract.rs`** lines 1762-1851 — `is_prelude_form` + `split_body_prelude` (the retirement target + the lift integration site)
5. **`src/runtime.rs`** lines 2369-2387 — `try_parse_fn_shape_def` (documents that `defn` macro expands to `def` BEFORE the closure-extract path runs; `defn` is dead code for the lift, included via `def`)

## Implementation path

### Phase 1 — Mint `is_declaration_form` (5-10 min)

In `src/freeze.rs` adjacent to `is_mutation_form` (after line 1269):

```rust
/// Returns true for forms that DECLARE a name into the module's
/// type or value registry. Subset of `is_mutation_form` excluding
/// loads (`load-file!` family) and config setters (`config::set-*`),
/// which mutate state but do not introduce bindings.
///
/// Callers:
/// - `closure_extract::split_body_prelude` — lifts these forms from
///   fn-body do-prefix into closure prologue.
/// - `check::validate_def_position_with_wrapper` (Gap I-B) — walks
///   these forms for top-level position discipline at check time.
pub fn is_declaration_form(head: &str) -> bool {
    matches!(
        head,
        ":wat::core::def"
            | ":wat::core::define"
            | ":wat::core::defmacro"
            | ":wat::core::define-dispatch"
            | ":wat::core::struct"
            | ":wat::core::enum"
            | ":wat::core::newtype"
            | ":wat::core::typealias"
    )
}
```

### Phase 2 — Route the lift through it (5-10 min)

In `src/closure_extract.rs`:
- Retire `is_prelude_form` (lines 1762-1775). Replace inline keyword match with a call to `crate::freeze::is_declaration_form`.
- `split_body_prelude` at line 1820: `take_while(|child| is_prelude_form(child))` becomes a closure that extracts the head keyword and checks `is_declaration_form`. The shape:

```rust
let prefix_len = do_children
    .iter()
    .take_while(|child| {
        if let WatAST::List(items, _) = child {
            if let Some(WatAST::Keyword(k, _)) = items.first() {
                return crate::freeze::is_declaration_form(k);
            }
        }
        false
    })
    .count();
```

Or factor the head-keyword extraction into a small helper `head_keyword(&WatAST) -> Option<&str>` if the readability wins.

### Phase 3 — Probes for the 5 newly-covered forms (15-25 min)

Create `tests/probe_declaration_form_lift.rs` covering:

```rust
#[test]
fn probe_def_in_fn_body_do_prefix_lifts_to_prologue() {
    // (:wat::core::def :x 42) at fn body do-prefix.
    // Pre-Gap-I-A: child fails with DefNotTopLevel.
    // Post: child succeeds; :x registers in child's runtime_def_values.
}

#[test]
fn probe_defmacro_in_fn_body_do_prefix_lifts_to_prologue() {
    // (:wat::core::defmacro ...) at fn body do-prefix.
    // Pre: EvalForbidsMutationForm at freeze-time inside the closure.
    // Post: defmacro registers at child's startup; macro expansion in child body works.
}

#[test]
fn probe_define_dispatch_in_fn_body_do_prefix_lifts_to_prologue() {
    // (:wat::core::define-dispatch ...) at fn body do-prefix.
    // Pre: some position-discipline error.
    // Post: dispatch registers in child's DispatchRegistry.
}

#[test]
fn probe_newtype_in_fn_body_do_prefix_lifts_to_prologue() {
    // (:wat::core::newtype ...) at fn body do-prefix.
    // Pre: similar.
    // Post: newtype registers in child's TypeEnv.
}

#[test]
fn probe_typealias_in_fn_body_do_prefix_lifts_to_prologue() {
    // (:wat::core::typealias ...) at fn body do-prefix.
    // Pre: similar.
    // Post: typealias registers; downstream type refs resolve.
}

#[test]
fn probe_mixed_declaration_prelude_all_lift() {
    // All 8 declaration form kinds at fn body do-prefix; all lift in source order.
}
```

Probes confirm the failure baseline (each probe could be split into a "pre" failing assertion + "post" passing assertion, or just shipped as the post-fix passing case — your call; the load-bearing concern is verifying the lift works for each form kind).

### Phase 4 — Verify

```bash
# New Gap I-A probes
cargo test --release --test probe_declaration_form_lift

# Gap H regression (5 prior prelude probes must still pass)
cargo test --release --test probe_closure_body_prelude_lift

# All prior substrate probes
cargo test --release --test probe_do_splice_def --test probe_let_splice_def \
    --test probe_do_splice_define --test probe_let_splice_define \
    --test probe_do_splice_struct --test probe_do_splice_enum \
    --test probe_let_splice_struct --test probe_let_splice_enum \
    --test probe_spawn_process_parent_type \
    --test probe_resolver_quote_awareness \
    --test probe_deftest_hermetic_isolation

# Workspace
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result"
# Expected: 2232 + N / 0 failed (N = new probes; expect ≥ 6)
```

## Scope (what's IN)

- `pub fn is_declaration_form` minted in `src/freeze.rs` adjacent to `is_mutation_form`
- `is_prelude_form` in `src/closure_extract.rs` RETIRED (deleted; the inline keyword match becomes a call to `is_declaration_form`)
- `split_body_prelude` consumes `is_declaration_form` via head-keyword extraction
- 6+ probes in `tests/probe_declaration_form_lift.rs` covering def + defmacro + define-dispatch + newtype + typealias + mixed
- Workspace at 0 failed
- `defn` is NOT in `is_declaration_form` — it's a macro expanding to `def` before `extract_closure` runs; `is_declaration_form` covers the post-expansion shape only

## Scope (what's OUT)

- **Gap I-B** — `validate_def_position_with_wrapper` extension via `is_declaration_form`. Separate slice. The position validator currently catches only `:wat::core::def`; extending it to all 8 forms surfaces previously-runtime-only diagnostics earlier and may cascade through tests. Gap I-B is the stepping-stone slice after I-A.
- Loads (`load-file!` / `digest-load!` / `signed-load!`) at fn-body do-prefix — explicitly out-of-scope (honest scope-bounding; loads are a different semantic category; if a real use case surfaces, a separate arc examines)
- Config setters (`config::set-*`) at fn-body do-prefix — same architectural treatment as loads
- Error variant renames (e.g., `DefNotTopLevel` → `DeclarationNotTopLevel`) — not in scope; Gap I-A doesn't touch error variants; Gap I-B may surface the question as an honest delta
- Anything under `docs/arc/` (FM 11 — orchestrator owns paperwork)
- ~/.claude/ memory system
- Changes to `is_mutation_form` itself — it stays as the union; `is_declaration_form` is the new narrower predicate adjacent

## Ship criteria (6 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `is_declaration_form` minted as `pub` in `src/freeze.rs` adjacent to `is_mutation_form`; covers the 8 declaration forms | grep + read |
| B | `is_prelude_form` in `src/closure_extract.rs` retired; `split_body_prelude` consumes `is_declaration_form` via head-keyword extraction | grep + read |
| C | 6+ probes in `tests/probe_declaration_form_lift.rs` pass: def / defmacro / define-dispatch / newtype / typealias / mixed | cargo test |
| D | All 5 Gap H probes (`probe_closure_body_prelude_lift`) still pass — regression confirms is_declaration_form covers define+struct+enum identically | cargo test |
| E | All 11 prior substrate probes still pass | cargo test |
| F | `cargo check --release` green; workspace at 2232 + N / 0 failed (N ≥ 6) | full test run |

**6 rows. All must PASS.**

## Predicted runtime

**30-60 min sonnet.** Mechanical predicate mint + caller-route + probes. Pattern-mirrors Gap H but smaller surface (no new helper logic; just route through new predicate; no companion-bug-class risks).

**Hard cap:** 120 min (2×).

## Constraints (hard)

- DO NOT touch `is_mutation_form` itself — it stays as the source-of-truth union
- DO NOT touch `refuse_mutation_forms` — its scope is the union; Gap I-A doesn't affect it
- DO NOT extend `validate_def_position_with_wrapper` — that's Gap I-B's job
- DO NOT add loads / config setters to `is_declaration_form` — out-of-scope by architectural intent
- DO NOT modify error variants (`DefNotTopLevel` etc.) — Gap I-A is purely additive on the lift side
- DO NOT modify deftest-hermetic macro shape — separate slice
- DO NOT touch `docs/arc/` (FM 11)
- DO NOT commit (orchestrator atomic-commits)
- DO NOT use deferral language in SCORE
- DO NOT operate outside `/home/watmin/work/holon/wat-rs/`
- DO NOT touch `~/.claude/` memory system
- DO NOT use --no-verify or skip hooks
- If a Gap H regression surfaces (any of the 5 prior probes fails), STOP and report — the routing through `is_declaration_form` must be transparent for the original 3 forms

## Honest delta categories (anticipated)

1. **Head-keyword extraction helper** — should the lift use an inline closure or a factored `head_keyword(&WatAST) -> Option<&str>` helper? Pattern recurs across the substrate; surface the choice + rationale.
2. **`is_prelude_form` retirement strategy** — delete entirely vs keep as a thin wrapper that calls `is_declaration_form`? The doctrine answer: delete (one source-of-truth; no aliases). Surface the choice + confirm.
3. **Probe shape — failing-baseline vs passing-post-fix** — for the 5 newly-covered forms, ship probes as just-the-positive-case (post-fix passes) OR as before/after pair? Recommendation: positive-case-only (cleaner; pre-fix baseline is implicit via Gap H not covering these forms).
4. **`defn` documentation** — should the BRIEF's claim ("defn expands to def before extract_closure") get a comment in `is_declaration_form`'s docstring noting why `defn` is intentionally absent? Surface the choice.
5. **Anything unexpected** — particularly around the head-keyword extraction edge cases (List with non-Keyword head; nested forms; etc.)

## Cross-references

- `36030c3` Gap H (the precedent slice; pattern for the lift integration; companion bug fix discipline)
- `021884a` Gap G (the blockage that surfaced Gap H; resolved at Gap H closure)
- `fe06bb1` Gap F-3 (extract_closure extension precedent — type-registry sweep adjacent to where I-A's predicate routing lands)
- `f9c8aef` Gap F-1 (struct/enum pregen pattern — parallel concern at parent scope)
- After Gap I-A ships: Gap I-B becomes the next slice (position-validator extension); after both close, Phase 2a is complete and deftest-hermetic Path E macro shape ships
