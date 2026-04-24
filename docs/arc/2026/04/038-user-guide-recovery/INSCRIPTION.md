# Arc 038 — INSCRIPTION

**Closed:** 2026-04-24.
**Commits (in order):**
- `cdec632` — revert USER-GUIDE.md to 467a3d4
- `a85f664` — DESIGN + BACKLOG opened
- `89167de` — Slice 1: §1 Setup overhaul
- `d685694` — Slice 2: §6 Algebra forms
- `0c7296b` — Slice 3: wat-lru namespace promotion
- `fa99e71` — Slice 4: §13 Testing + §2 set-dims! straggler
- `7e5ba82` — Slice 5: §4 Macros subsection
- `cddc374` — Slice 6: §4 Containers subsection
- `a27af57` — Slice 8: Appendix forms table audit + §15 prose
- `<this commit>` — Slice 9: INSCRIPTION + cross-references

(Slice 7 — sigma defaults — was folded into Slice 1 to keep the
"Config setters (optional)" subsection cohesive in one place.)

## What happened

Commit `5b5fad8` (arc 028 slices 5+6 — *INSCRIPTION + full doc
sweep*, 2026-04-22) introduced content into `docs/USER-GUIDE.md`
that crashed input processing on read. Wholesale mechanical doc
sweeps had been a known poison vector since BOOK Chapter 32; this
arc made the lesson concrete at the doc layer.

The recovery had two phases:

1. **Restore safety.** `git checkout 467a3d4 -- docs/USER-GUIDE.md`
   rolled the file back to its prior known-good state — the
   2026-04-22 21:12 commit "USER-GUIDE — sync with arc 021 + 022
   shipped state." Working tree clean. File readable. ~2 days of
   shipped work missing.
2. **Sync forward via targeted edits.** Eight slices grouped by
   USER-GUIDE section folded arcs 023-037 + the wat-lru namespace
   promotion into the rolled-back content. Every slice = a small
   set of `Edit` calls; never a `sed`/`perl`/whole-file rewrite.

## What shipped per slice

### Slice 1 — §1 Setup overhaul

- arc 027: `loader:` option mention preserved (the rolled-back doc
  already had partial coverage); added context where it was missing.
- arc 028: every `(:wat::core::load! :wat::load::file-path "...")`
  rewritten to `(:wat::load-file! "...")`; same for eval forms
  rewritten to `(:wat::eval-file! ...)`.
- arc 037: retired `(:wat::config::set-dims! 10000)` from the
  minimal-entry-file example, the multi-file entry example, and the
  test file example. Minimal entry now shows just
  `(:wat::config::set-capacity-mode! :error)` plus `:user::main`.
- arc 031: dropped `1024 :error` from the `deftest` signature in
  §1's Tests subsection.
- New "Config setters (optional)" subsection covering all four
  optional setters: `set-capacity-mode!`, `set-dim-router!`,
  `set-presence-sigma!`, `set-coincident-sigma!`. arc 024's sigma
  setters were folded in here rather than the originally-planned
  Slice 7 to keep the subsection cohesive.

### Slice 2 — §6 Algebra forms

- Intro paragraph counts updated: 3→4 measurements, 10→11 idioms.
- New typealiases named: `:wat::holon::Holons` (arc 033),
  `:wat::holon::BundleResult` (arc 032).
- `:wat::holon::Bundle`'s return type uses the `:BundleResult`
  typealias.
- Measurements section adds `coincident?` (arc 023) and the
  `eval-coincident?` family (arc 026 — 4 forms covering bare /
  edn / digest / signed verification before eval+atomize+compare).
- Idioms section adds `:wat::holon::ReciprocalLog` (arc 034).
- Config accessors flagged: `dims` and `noise-floor` are compat
  shims under arc 037's multi-tier dim-router. The shims still
  ship; they're deprecation targets.

**Drift caught pre-commit:** my first draft cited
`(:wat::config::dim-router)` as a read-side accessor — verified via
grep that no such accessor exists. Removed the speculation before
push.

### Slice 3 — wat-lru namespace promotion (arc 036)

- `:user::wat::std::lru::*` → `:wat::lru::*` across §10 examples.
- Two appendix forms-table rows for `LocalCache` and `CacheService`
  also updated in the same slice (rather than waiting for Slice 8)
  to keep all wat-lru migration coherent in one commit.

### Slice 4 — §13 Testing + §2 set-dims! straggler

- arc 031: `deftest` signature drops `dims` + `mode` parameters; the
  sandbox inherits the outer file's `Config`. Updated three deftest
  signature examples in §13.
- arc 029: new `make-deftest` factory subsection — the idiomatic
  shape for test files with shared loads/helpers; default-prelude
  carries common setup; per-test calls are bare-name.
- Inner test::program bodies (in `run-ast` and `run-hermetic-ast`
  examples) drop `set-dims!`/`set-capacity-mode!` setters since the
  inner sandbox inherits.
- §2 set-dims! straggler (the second `wat/main.wat` example in the
  stdin-echo section) folded into this slice — same arc 037
  retirement we did in §1, would have orphaned otherwise.

### Slice 5 — §4 Macros subsection

The rolled-back §4 had no macros coverage at all. Added a new
subsection covering:
- `defmacro` skeleton with hygiene (compile-time AST→AST rewriting).
- Quasiquote `, ,@` for the common cases.
- Nested quasiquote `,,` deep-splice (arc 029) — the form
  `:wat::test::make-deftest` uses to build per-test sandboxes.
- `macroexpand` / `macroexpand-1` (arc 030) for expansion debugging.

Bigger than a fold-in (56 lines added); honest scope expansion
because §4 was missing macros entirely.

### Slice 6 — §4 Containers subsection

Five-verb polymorphic table for `get` / `assoc` / `conj` /
`contains?` / `length` over `HashMap<K,V>` / `HashSet<T>` /
`Vec<T>`:

- arc 025: unified four-verb surface with semantically-forced
  illegal cells (`assoc` on HashSet illegal — use `conj`; `conj`
  on HashMap illegal — use `assoc`).
- arc 035: `length` joined the polymorphic surface (Vec already had
  it; HashMap/HashSet added).

Includes the verb-by-container table, three usage examples (one per
container), and the principle: illegal cells reflect semantics, not
implementation laziness — type checker rejects them at startup.

### Slice 8 — Appendix forms table audit + §15 prose

Comprehensive audit of the appendix table.

**Removed:** `set-dims!`, `:wat::core::load!` family (arc 028 root
hoist), `:wat::core::eval-*!` family (arc 028).

**Added:** `set-dim-router!`, `set-presence-sigma!`,
`set-coincident-sigma!`, `:wat::config::global-seed` (was missing),
`:wat::load-file!` / `load-string!` / `digest-load-*` /
`signed-load-*`, `:wat::eval-*!` family, `ReciprocalLog`,
`coincident?`, eval-coincident family, `macroexpand` /
`macroexpand-1`, polymorphic `assoc` / `get` / `contains?` rows,
`make-deftest`.

**Updated:** `dims` / `noise-floor` flagged as compat shims;
`Bundle` return type uses `:BundleResult` typealias; `deftest`
signature drops dims+mode.

**§15 prose:** signed/digest load form references updated from
retired `:wat::core::*` paths to arc-028 root forms.

**Drift caught during the audit:** Slice 2's prose described
`eval-digest-coincident?` as "4-args" and `eval-signed-coincident?`
as "6-args"; actual arities are 8 and 12 (verified against
`src/check.rs`). Both prose and appendix now agree on 8/12.

## What this arc proved

**Targeted edits per arc work.** Eight slices grouped by USER-GUIDE
section, ~30-100 line changes each, every commit independently
verifiable. No `sed`/`perl`/whole-file rewrite at any point.
Commit-by-commit audit trail; cheap rollback if any single edit
re-introduced a poison.

**Wholesale doc sweeps stay retired as a pattern.** The corrupting
commit `5b5fad8` was a 21-file mechanical doc sweep. arc 038's
discipline — small targeted Edits per arc — is the replacement
shape. Future doc syncs land per-arc as their substrate work ships,
not as bulk sweeps weeks later.

**The book's discipline applied recursively.** Chapter 32 named
"perl/sed with pipes in pipe-dense markdown" as a poison class.
This arc named the higher-order class: **wholesale mechanical
sweeps over markdown-heavy targets**, regardless of which tool
runs them. Both classes share the structural failure mode — a
single mass-edit that corrupts the file at a scale where review
can't catch it before commit.

## Out of scope (future arcs)

- **Other docs in commit `5b5fad8`** — README.md (86 lines touched),
  CONVENTIONS.md (4), INVENTORY.md (52), `wat-tests/README.md` (31).
  Builder named only USER-GUIDE.md as poisoned. If a future read
  flags any of these as similarly broken, that gets its own arc
  (arc 039+) with the same discipline.
- **Compat-shim retirement** — `:wat::config::dims` and
  `:wat::config::noise-floor` are documented as shims pointing at a
  "future arc" that will retire them. That arc owns the lab-side
  caller migration first.
- **Appendix completeness audit** — Slice 8 added all the new forms
  that arcs 023-037 surfaced, but the table predates several
  arcs and may have other latent gaps unrelated to this recovery.
  A separate completeness pass when a caller surfaces a missing
  row.

## Files touched

- `docs/USER-GUIDE.md` — eight slices' worth of targeted edits.
- `docs/arc/2026/04/038-user-guide-recovery/{DESIGN,BACKLOG,INSCRIPTION}.md`
  — the arc record itself.
- `docs/README.md` — arc index extended (this slice).
- `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION-CHANGELOG.md`
  — cross-repo audit trail row (this slice).

## What this means for the user guide

The guide guides again. Every shipped surface from arcs 023-037 has
a documented home. Every retired form is gone from the prose. The
appendix table is honest. The setup section shows the minimum form
that actually works under arc 037's defaults.

Future arcs that ship user-facing surface land their USER-GUIDE
edit alongside the substrate commit — small, targeted, per-arc.
The guide stays current by walking forward one arc at a time, the
same way the substrate does.
