# Arc 031 — sandbox inherits outer config — INSCRIPTION

**Shipped:** 2026-04-23. Three slices (originally four; slices 2 +
3 merged at implementation because the four test macros were
mechanically coupled).

**Commits:**
- `ea9a857` — docs opened (DESIGN + BACKLOG)
- `f3fa9ea` — slice 1 (substrate: run-sandboxed-ast + fork inherit Config)
- `ef12e98` — slice 2 (drop mode+dims from all four :wat::test::* macros)
- `704c7d4` — lab migration (test_scaffold.wat + time.wat factory call)
- (this commit) — slice 3 (INSCRIPTION + doc sweep)

---

## What shipped

### Slice 1 — substrate (f3fa9ea)

Config inheritance at the sandbox freeze boundary.

New collector: `collect_entry_file_with_inherit(forms, &Config)`
seeds every field from the inherited baseline; duplicate-in-forms
still errors, but a single setter overriding an inherited value no
longer trips the duplicate gate. A new private `set_*: bool` tracker
per field distinguishes "seen in forms" from "value already set"
so inheritance doesn't count as a prior set.

New startup entry: `startup_from_forms_with_inherit(forms, base,
loader, &Config)` routes through the new collector and shares all
post-config stages (loads, macros, types, defines, resolve, check,
freeze) with `startup_from_forms` via a new private
`startup_from_forms_post_config` helper.

Call-site wiring:
- `eval_kernel_run_sandboxed_ast` reads `sym.encoding_ctx().config`
  and routes through the inheriting startup when available. Falls
  back to non-inheriting when no encoding context is attached
  (bare-SymbolTable test harnesses that haven't gone through freeze).
- `eval_kernel_fork_with_forms` snapshots `sym.encoding_ctx().config`
  into a local `Option<Config>` BEFORE fork. `child_branch` carries
  it across COW and uses the inheriting startup in the forked child.
  `unwrap_or` fallback to non-inheriting when parent had no encoding
  context.

Surface additions re-exported from `lib.rs`:
- `wat::config::collect_entry_file_with_inherit`
- `wat::freeze::startup_from_forms_with_inherit`

### Slice 2 — test macros drop mode+dims (ef12e98)

All four `:wat::test::*` macros lose two parameters each:

| Before | After |
|---|---|
| `(deftest name mode dims prelude body)` | `(deftest name prelude body)` |
| `(deftest-hermetic name mode dims prelude body)` | `(deftest-hermetic name prelude body)` |
| `(make-deftest name mode dims default-prelude)` | `(make-deftest name default-prelude)` |
| `(make-deftest-hermetic name mode dims default-prelude)` | `(make-deftest-hermetic name default-prelude)` |

Templates stop emitting `(set-capacity-mode! ,mode)` +
`(set-dims! ,dims)` — the sandbox inherits them through slice 1's
path. Factory templates drop `,,mode` and `,,dims` nested-unquotes;
only `,,default-prelude` remains in the factory-generated inner
template.

Callsite sweep:
- 15 files, 64 ` :error 1024` drops across `wat-tests/`,
  `crates/wat-lru/wat-tests/`, `examples/with-loader/`, `tests/`.
- Driven by a Python script (literal substring replace; no regex,
  no alternation, no shell-escaping risk — the exact failure mode
  the Chapter 32 crash warning names).
- Pre-run grep confirmed `:error 1024` appeared only in
  deftest/make-deftest contexts across the workspace, zero false
  positives.

Rust-level updates:
- `tests/wat_make_deftest.rs` — assertion on registered body shape
  updated from 6-item (deftest + 5 args) to 4-item (deftest + name
  + prelude + body). Doc comment updated.
- `tests/wat_test_cli.rs` — inline failing-deftest fixture updated
  to the 3-arg shape.

### Lab migration (704c7d4)

- `holon-lab-trading/wat-tests/test_scaffold.wat` — direct deftest
  drops ` :error 1024`.
- `holon-lab-trading/wat-tests/vocab/shared/time.wat` — make-deftest
  preamble drops ` :error 1024`; doc comment rewritten to reflect
  inheritance.

### Slice 3 — INSCRIPTION + doc sweep (this commit)

This file. Plus docs updated where examples referenced the old
5-arg deftest shape or the 4-arg make-deftest shape.

---

## Tests

**Unit (slice 1):** 7 new config tests in `src/config.rs`:
- `inherit_empty_forms_takes_every_parent_field`
- `inherit_with_no_setters_but_body_still_inherits`
- `inherit_setter_overrides_single_field`
- `inherit_both_setters_override_everything_explicit`
- `inherit_duplicate_setter_in_forms_still_errors`
- `inherit_preserves_derived_fields_when_not_overridden`
- `inherit_dims_override_recomputes_nothing_automatically` — the
  load-bearing corollary: overriding dims in the child does NOT
  recompute the noise_floor default. Inheritance is a baseline per
  field, not a recompute-on-cascade rule.

**Integration (slice 1):** 4 new tests in
`tests/wat_sandbox_inherits_config.rs`:
- `sandbox_no_setters_inherits_outer_dims` — outer 4096, inner no
  setters, inner reads `(:wat::config::dims)` → 4096.
- `sandbox_with_dims_setter_still_inherits_capacity_mode` — outer
  :error + 1024, inner dims=2048 only, inner still runs under
  inherited :error capacity mode.
- `sandbox_with_both_setters_still_uses_explicit_values` — back-
  compat: pre-031 shape still works.
- `hermetic_sandbox_inherits_outer_dims_through_fork` — fork path
  verified: child COW-inherits parent's Config.

**Regression:** all 581 lib + every pre-031 integration suite
passes unchanged (up from 574 — the 7 new config unit tests
account for the delta).

**Lab:** 25 wat-tests green under the migrated stack.

---

## Sub-fog resolutions

**1a — where does the caller's Config live at the dispatch site?**
Resolved: `sym.encoding_ctx().config`. Every freeze-path-built
SymbolTable has an encoding context attached (it carries the
vector manager + scalar encoder + registry + Config). Non-freeze
SymbolTables (bare `SymbolTable::new()` in test harnesses) return
`None` from `.encoding_ctx()`; the sandbox call sites fall back
to the non-inheriting startup path in that case.

**1b — fork inheritance mechanism.** Resolved: explicit parameter.
Parent reads `sym.encoding_ctx().map(|ctx| ctx.config)` BEFORE
`libc::fork()`; the resulting `Option<Config>` is a local
variable that lives on the parent's stack. `child_branch`
receives it as a parameter; the Copy trait on `Config` means the
child sees its own owned copy after COW. No reaching through the
inherited world pointer — the inherit is an honest explicit
argument.

---

## What did NOT change

- **Top-level entry files.** Main binaries, test binaries, and
  the wat-vm CLI still require `(:wat::config::set-capacity-mode!)`
  + `(:wat::config::set-dims!)` in their entry source. The
  required-field check in `collect_entry_file` is unchanged;
  inheritance only activates when callers explicitly use the
  `_with_inherit` variant.
- **Harness / Rust API.** `Harness::from_source` and its family
  still construct FrozenWorlds from source that must carry their
  own setters. Arc 031 is a wat-level improvement; Rust callers
  who want config inheritance would construct a parent FrozenWorld
  first and route sandbox calls through it.
- **Non-setter forms in sandbox input.** Load!, defmacro, struct
  declarations, etc. still process identically — only the Config
  field starts from a different baseline.

---

## Reserved-prefix discipline

No new primitives. No new `:wat::*` paths. The four test macros
keep their canonical names; their signatures shrunk.
`run-sandboxed-ast`, `run-sandboxed-hermetic-ast`, and
`fork-with-forms` keep identical wat-level arities; only the
internal startup route changed.

Arc 031's relationship to arc 027 is explicit: both are
**scope inheritance** moves applied to different environment
fields. Arc 027: source loader inherits (`scope :None` →
caller's loader). Arc 031: Config inherits (inner setters absent
→ caller's values). Together they make a sandbox a proper
child-of-caller scope rather than a fresh reset.

---

## What comes next

The 058 batch's entry-file discipline section may want a sub-
mention that sandbox freezes inherit by construction, but
sandbox forms are not themselves "entry files" — they occupy a
different position in the pipeline than the test binary's own
preamble. The CONVENTIONS doc sweep covers the user-facing form
of this distinction.

Arc 031 closes the make-deftest ergonomics arc that started with
arc 029 (nested quasiquote, which made the factory possible) and
arc 030 (macroexpand, which diagnosed arc 029's bug and caught
the nested-quote preservation issue). With arc 031, a test file's
shape reaches its honest minimum:

```scheme
(:wat::config::set-capacity-mode! :error)
(:wat::config::set-dims! 1024)

(:wat::test::make-deftest :deftest
  ((:wat::load-file! "...")))

(:deftest :my-test body)
(:deftest :another body)
(:deftest :third body)
```

One preamble. One factory call. N tests. No per-test config
ceremony. No per-test setter repetition. The honest shape Path B
described.
