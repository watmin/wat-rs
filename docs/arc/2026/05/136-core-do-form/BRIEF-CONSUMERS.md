# Arc 136 — Consumer Sweep BRIEF (slice 1b)

**Drafted 2026-05-06.** Slice 1b of arc 136.

User direction (verbatim, post-slice-1a):
> *"1b is on deck"*

## Goal

Replace pure `let*`-with-unit-bindings chains with the new
`(:wat::core::do ...)` form across the entire codebase. Pure
ergonomic cleanup; no breaking change (both forms are valid; the
do form is just cleaner).

## The transform

**Pure-unit chain** (every binding is `((_ :wat::core::unit) ...)`):

```scheme
;; Before
(:wat::core::let*
  (((_ :wat::core::unit) FORM_1)
   ((_ :wat::core::unit) FORM_2)
   ((_ :wat::core::unit) FORM_3))
  BODY)

;; After
(:wat::core::do FORM_1 FORM_2 FORM_3 BODY)
```

**Mixed bindings** (any binding is NOT `((_ :wat::core::unit) ...)`):
LEAVE AS `let*`. The do form doesn't introduce names; mixed sites
need the binding semantics let* provides.

```scheme
;; STAYS as let*
(:wat::core::let*
  (((_ :wat::core::unit) FORM_1)   ; unit-discard
   ((x :wat::core::i64) (:compute)) ; real binding
   ((_ :wat::core::unit) FORM_2))
  BODY)
```

## Sweep scope

Estimated 100+ sites across:

1. **`wat/*.wat`** (substrate stdlib bundled with binary)
2. **`crates/*/wat/**/*.wat`** (per-crate substrates)
3. **`wat-tests/**/*.wat`** (workspace test wat)
4. **`crates/*/wat-tests/**/*.wat`** (per-crate test wat)
5. **`examples/**/*.wat`** (example programs)
6. **Embedded wat strings in `tests/*.rs` + `src/*.rs`** (Rust tests/lib tests with inline wat)

Detection helper (run pre-sweep to scope):

```bash
grep -rln '((_ :wat::core::unit)' wat/ wat-tests/ crates/*/wat/ crates/*/wat-tests/ examples/ tests/ src/
```

## Sweep strategy

Per site, the algorithm:

1. Open the file at the let* site
2. Examine all bindings in the let*
3. If EVERY binding is `((_ :wat::core::unit) <form>)`:
   - Transform let*'s shape into `(:wat::core::do <form>... <body>)`
   - Body becomes the final form in do
4. If ANY binding is NOT a unit-discard:
   - Leave the let* untouched
5. After each batch of files (e.g., per directory), run cargo test
   to verify nothing regressed

The do form has identical eval semantics for unit-typed non-finals,
so the transform is semantics-preserving by construction. The
transform CAN reveal pre-existing latent type bugs (sites where
someone was using `(_ :unit)` to silently coerce a non-unit value
to unit — those will surface as TypeMismatch under the new infer_do
which doesn't constrain non-finals' types). Surface those as honest
deltas; don't try to fix them in this sweep.

## Constraints

- **DO COMMIT + PUSH** when workspace = 0-failed.
- **No substrate edits** (`src/*.rs`). If you discover a substrate-
  internal bug during 1b, STOP and report (Mode B).
- **No `holon-lab-trading/` edits** (separate workspace).
- **Mixed bindings stay let*** — phase-2 judgment; not in 1b's
  scope to refactor those into mixed do/let* combinations.
- **STOP at first unexpected red** — anything OTHER than "this
  let* is now an obvious do form" or "this site has mixed bindings
  and stays let*."
- **No grinding** — if a single site requires >3 reads/edits to
  resolve cleanly, surface as Mode D honest delta.

## Pre-flight crawl (mandatory)

1. **`docs/arc/2026/05/136-core-do-form/DESIGN.md`** — read TOP
   SECTION (post-back-out realization); historical sections below
   are reference but NOT load-bearing
2. **`docs/arc/2026/05/136-core-do-form/BRIEF-SUBSTRATE.md`** —
   what slice 1a shipped (the do form's exact semantics)
3. **`tests/wat_arc136_do_form.rs`** — canonical do form examples
   from slice 1a
4. **A few existing let*-with-unit-bindings examples** — open
   `wat/test.wat` and skim a few sites to internalize the shape

## Pre-flight verification (test BEFORE editing)

```bash
cargo test --release --workspace 2>&1 | grep -cE "FAILED"
```

Must be 0 (workspace currently clean post-arc-136-slice-1a at HEAD
= `ff45f38`).

## Verification (during + after sweep)

After each major batch (stdlib done, per-crate done, tests done,
embedded done), run:

```bash
cargo test --release --workspace 2>&1 | grep -E "test result:|FAILED" | tail
```

Expect: 0 failed continuously. Any new failure = STOP and surface.

Final verification:

```bash
grep -rln '((_ :wat::core::unit)' wat/ wat-tests/ crates/*/wat/ crates/*/wat-tests/ examples/ tests/ src/ | wc -l
```

Should be substantially reduced from pre-sweep baseline. (Won't
be 0 — mixed-bindings sites stay.) Sample the remaining sites to
confirm they are all genuinely mixed.

## Reporting (~250 words)

1. **Pre-flight crawl confirmation:** DESIGN, BRIEF-SUBSTRATE,
   tests/wat_arc136_do_form.rs, sample existing let* sites all
   read.

2. **Sweep summary:** transform count per directory bucket
   (`wat/`, `crates/*/wat/`, `wat-tests/`, `crates/*/wat-tests/`,
   `examples/`, `tests/`/`src/` embedded). Total file count + total
   call-site count migrated. Mixed-bindings sites left untouched
   (count + sample paths if interesting).

3. **Latent-bug surfaces:** any sites where the transform revealed
   a pre-existing `(_ :unit)` silently coercing a non-unit value
   to unit — those will surface as TypeMismatch under the new
   infer_do. Flag for follow-up if any.

4. **Verification:**
   - Workspace stayed 0-failed throughout
   - Final `cargo test --workspace` shows 0 failed
   - Remaining `((_ :wat::core::unit)` count vs pre-sweep baseline

5. **Path:** Mode A clean (sweep complete; workspace 0-failed) /
   Mode B substrate-internal-bug / Mode C unexpected-shape (mixed
   site that's actually transformable in some way) / Mode D
   per-site grinding.

6. **Honest deltas:** any patterns of `(_ :unit)` usage that
   suggested deeper design questions; any test sites that read
   strangely after the transform; any class of files where the
   transform was particularly heavy/light.

7. **Commit + push** when Mode A. Use a commit message following
   project pattern. INSCRIPTION + 058 row are slice 2 closure
   (out of scope here).

## Time-box

90 minutes wall-clock (predicted upper-bound 45-60 min; mechanical
sweep at ~30-60 sites/hour; 1.5× cap allows for batched cargo
verification).

## Why this brief

Slice 1a shipped the do form. Slice 1b retires the let*-with-unit-
bindings crutch by mechanically transforming pure-unit chains.
After 1b, every site advertising the do pattern uses the do form;
the crutch's mental tax (binding ceremony for sequencing) lifts.

Mode A clean = the codebase loses ~hundreds of lines of let*
binding ceremony; the do form ships across all consumers; arc 136
slice 2 (closure paperwork) ready to spawn next.
