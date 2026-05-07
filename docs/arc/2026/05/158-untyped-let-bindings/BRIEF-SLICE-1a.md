# Arc 158 — Substrate BRIEF (slice 1a)

**Drafted 2026-05-07.** Slice 1a of arc 158.

User direction: *"we do not support brackets yet - they are coming
- let's remove the typed bindings first"* + *"clojure is our
guiding light - we're just building a strongly typed clojure on
rust"* + *"we are doing the hard grunt work to enable what i have
planned."*

## Workspace state pre-spawn

- HEAD: `7805b76` (arc 158 DESIGN shipped)
- Working tree: clean (verify `git status -s` returns nothing)
- Pre-baseline (verified post arc 157 closure): **2029 passed /
  0 failed / 0 warnings**

## Goal

Drop the per-binding type annotation `:T` from `:wat::core::let`.
Each binding's type is inferred from its expression — same lesson
as arc 145 (typed-let backout) and arc 157 (def ships untyped),
applied to the inner-binding slot of `let`.

| Before | After |
|---|---|
| `(:wat::core::let (((name :T) expr) ...) body)` | `(:wat::core::let ((name expr) ...) body)` |

Each binding goes from `((name :T) expr)` (3 paren levels — outer
list, binding pair, type-annotated name) to `(name expr)` (2 paren
levels — outer list, binding pair). The OUTER bindings list `(...)`
stays. The body slot stays.

Migration: clean break per arc 154 / 155 precedent. Substrate
accepts new shape only; walker fires `LegacyTypedLetBinding`
CheckError on legacy `((name :T) expr)` shape. Atomic substrate
+ wat-rs sweep (1a + 1b) per recovery doc § 7.

**Out of scope for arc 158:** Clojure-style square-bracket binding
form `[name expr name expr]`. Per user direction *"we do not
support brackets yet - they are coming - let's remove the typed
bindings first."*

## Substrate edits

### `src/check.rs`

1. **`LegacyTypedLetBinding` CheckError variant.** Mirror arc
   154's `BareLegacyLetStar` shape exactly:
   - Variant: `LegacyTypedLetBinding { binding_name: String, span: SourceLocation }`
     (or whatever shape matches the existing CheckError convention
     post-arc-138 spans)
   - `Display`: names the legacy shape and points at the canonical
     fix (e.g. "let binding `((<name> :T) expr)` is legacy form
     post-arc-158; use `(<name> expr)` — type is inferred from
     the expression").
   - `diagnostic()` arm.

2. **Walker** `walk_for_legacy_typed_let_binding` — detect
   per source-level legacy binding. Mirror arc 154's
   `validate_legacy_let_star` shape:
   - Walks the AST looking for `:wat::core::let` heads
   - For each binding in the bindings list, checks shape
   - If binding is `((<keyword> <type-expr>) <expr>)` → emit
     `LegacyTypedLetBinding`
   - If binding is `(<keyword> <expr>)` → no-op (canonical)

3. **`infer_let` accepts new binding shape.** Currently the
   binding-extract path expects `((name :T) expr)`. Make it
   accept both:
   - New shape `(name expr)` is the canonical path
   - Legacy `((name :T) expr)` continues to parse (for the
     migration window — walker tells caller to migrate;
     inference still uses inferred type from expr, not the
     declared `:T`)
   - Note: the legacy `:T` on the binding is now IGNORED at
     inference (the walker tells caller to migrate; inference
     uses expr's inferred type, NOT the declared `:T`). This
     means a (legacy) `((x :wat::core::String) 1)` would
     compile cleanly with `x : :wat::core::i64` (and walker
     fires telling caller to migrate). Per the arc 145 lesson:
     don't use the declared annotation when inference suffices.

   Sonnet picks the cleanest implementation — the `infer_let`
   binding-extract logic at src/check.rs:5861 is the precedent.

### `src/runtime.rs`

The runtime already binds `name → value` at let-eval; the only
shape change is the SOURCE-LEVEL binding form. Runtime should
need no functional changes IF the AST eval path consumes
post-check binding values rather than re-parsing the source
shape.

Sonnet should verify by reading `eval_let` (likely near where
`eval_let_star` lived pre-arc-154) — if it consumes the typed
shape from AST, mirror the inference path's shape-flexibility.

If runtime needs the new-shape support, mirror the same
pattern as `infer_let`.

### `src/special_forms.rs`

The registry sketch for `:wat::core::let` may need updating to
reflect the new binding shape. Currently the sketch is
`["<bindings>", "<body>+"]`. The new shape doesn't change the
top-level slots; the change is INSIDE `<bindings>`. Sketch may
not need change — sonnet decides per existing convention.

### NEW `tests/wat_arc158_let_bindings.rs`

Harness shape per `tests/wat_arc154_kill_let_star.rs` /
`tests/wat_arc155_fn_rename.rs` / `tests/wat_arc157_def.rs`.

10 tests covering:

**Canonical (new) shape — 4 tests:**
1. Single binding: `(:wat::core::let ((x 2)) (:wat::core::i64::+,2 x 1))`
   → 3 (i64). Runtime evaluation succeeds.
2. Multiple bindings, sequential: `(:wat::core::let ((a 1) (b a))
   ...)` → both registered; b sees a's value.
3. Type inferred from expr: `(:wat::core::let ((floor 0.5))
   (:wat::core::f64::+,2 floor 1.0))` — floor's type is
   `:wat::core::f64` (inferred from literal); type-check passes.
4. Closure capture works: `(:wat::core::let ((x 2))
   (:wat::core::fn ((y :wat::core::i64) -> :wat::core::i64)
     (:wat::core::i64::+,2 x y)))` — let-local x captured by fn.

**Legacy (old) shape — 4 tests:**
5. Bare legacy binding: `(:wat::core::let (((x :wat::core::i64) 2))
   ...)` → fires `LegacyTypedLetBinding` walker per binding.
6. Multiple legacy bindings: each fires the walker.
7. Mixed (legacy + canonical): each legacy fires; canonical
   doesn't fire.
8. Walker diagnostic names the canonical fix.

**Behavior parity — 2 tests:**
9. Type inference is identical regardless of binding shape (legacy
   `((x :wat::core::i64) 2)` and canonical `(x 2)` both give
   `x : :wat::core::i64`).
10. Sequential semantics preserved (binding N's expr can reference
    binding N-1; mirrors arc 154's let-as-sequential discipline).

## Constraints

- **Substrate-only edits.** Likely 4 files: `src/check.rs`,
  `src/runtime.rs` (if needed), `src/special_forms.rs` (if
  sketch needs update), NEW `tests/wat_arc158_let_bindings.rs`.
  NO consumer wat edits. NO other crate.
- **DO NOT COMMIT.** Working tree stays modified for atomic
  commit with sweep 1b per recovery doc § 7
  atomic-commit-across-coordinated-sweeps.
- **The workspace WILL break post-substrate-change** — every
  existing legacy binding `((name :T) expr)` site fires
  `LegacyTypedLetBinding`. EXPECTED. Sweep 1b clears them.
- **STOP at unexpected red.** Distinguish:
  - **Expected:** `LegacyTypedLetBinding` on every legacy
    binding site across the workspace (~951 sites in wat-rs)
  - **Unexpected:** anything else (substrate panic, parse
    error, unrelated TypeMismatch)
- No grinding. No speculative scope expansion. No bracket form.
- Time-box: **60 min wall-clock** (2× predicted upper-bound
  30 min).

## Pre-flight crawl (mandatory)

1. `docs/arc/2026/05/158-untyped-let-bindings/DESIGN.md` —
   full read
2. `docs/arc/2026/05/154-kill-let-star/INSCRIPTION.md` — closest
   precedent for let-related substrate change with walker recipe
3. `docs/arc/2026/05/155-fn-rename/INSCRIPTION.md` — multi-piece
   substrate slice + Path B retirement pattern
4. `docs/arc/2026/05/157-core-def-form/INSCRIPTION.md` —
   most-recent let-adjacent slice (def consumes the same lesson)
5. `docs/SUBSTRATE-AS-TEACHER.md` — diagnostic-as-migration-brief
   pattern; CheckError variant + Display discipline
6. `feedback_substrate_already_typed.md` (memory) — paid-for
   lesson on type-annotation redundancy
7. `src/check.rs::infer_let` (line 5861) — the binding-extract
   path you're modifying
8. `src/check.rs` — `BareLegacyLetStar` variant + Display +
   `validate_legacy_let_star` walker (closest precedent shape)
9. `src/runtime.rs::eval_let` — runtime path; likely needs no
   change but verify
10. `src/special_forms.rs` — `:wat::core::let` registry entry
11. `tests/wat_arc154_kill_let_star.rs` — test harness shape

## Pre-flight verification

```bash
cd /home/watmin/work/holon/wat-rs
cargo test --release --workspace 2>&1 | grep -E "FAILED|^test result" | tail -5
```

Confirms 2029 / 0 / 0 baseline.

## Verification (after edits)

```bash
cargo test --release --test wat_arc158_let_bindings 2>&1 | tail -10
```

Expect: 8-10 of 10 new tests pass; some positive-case tests may
be blocked by stdlib pre-sweep state (mirrors arc 154 / 155
slice 1a pattern — stdlib still uses legacy shape; sweep 1b
clears).

```bash
cargo test --release --workspace 2>&1 | grep -E "test result|FAILED" | head -10
```

Expect: many `LegacyTypedLetBinding` errors firing on existing
sites; NO unexpected substrate red.

## Reporting (~250 words)

Per BRIEF: pre-flight crawl confirmation; edit summary per file
with LOC delta; verification (new test pass count + workspace
failure shape — `LegacyTypedLetBinding` firing as expected on
~951 legacy sites); path classification (Mode A / B / C);
honest deltas:

- Did `infer_let`'s existing binding-extract logic accept both
  shapes cleanly, or did it need restructuring?
- Did runtime's `eval_let` need changes, or was the
  shape-flexibility purely check-side?
- Any surprises in the walker pattern matching (the legacy
  shape is nested deeper than arc 154's `:wat::core::let*`
  keyword which was at the outer head; this walker matches a
  binding shape WITHIN the bindings list)?
- Did the `:T` ignored-vs-asserted decision surface any
  consumer-visible behavior change beyond the walker firing?

DO NOT write a SCORE doc — orchestrator scores after sweep 1b
ships and atomic commit lands.

## Time-box

60 minutes wall-clock (2× predicted upper-bound).
ScheduleWakeup will fire at 60 min if sonnet hasn't returned.

## Why this matters

User direction 2026-05-07: *"we made the core system work and
proved it does exactly what we want it to do. now we need to
make it ergonomic.. there are so many creature comforts
coming.. we just need to do the mass refactors step by step."*

Arc 158 is ergonomic-grunt-work — the substrate-correctness
work proved out in arcs 153-157 lets the user-facing surface
shed legacy ceremony. `let` is the most-used form in the
codebase (~1916 sites total); making it untyped is a high-
visibility consistency win that aligns it with `def` (arc 157)
and the broader Clojure-faithful direction.

After 1a, 1b sweeps wat-rs, 1c sweeps the lab, 2 closes.
