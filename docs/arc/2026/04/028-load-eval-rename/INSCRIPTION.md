# Arc 028 — load/eval rename — INSCRIPTION

**Status:** shipped 2026-04-23.
**Design:** [`DESIGN.md`](./DESIGN.md) — the shape before code.
**Backlog:** [`BACKLOG.md`](./BACKLOG.md) — slices as they landed.
**This file:** completion marker.

Same-day open-to-close arc. Five commits across two repos. Mechanical
refactor with one semantic broadening (the string-verified variants
and new eval-file! form). Workspace-wide tests stayed green through
every slice.

---

## What shipped

### Thirteen forms renamed, two iface-keyword namespaces retired

Two orthogonal changes, combined into one arc:

1. **Dropped the interface keywords.** `:wat::load::file-path`,
   `:wat::load::string`, `:wat::eval::file-path`,
   `:wat::eval::string` retire entirely. Each form takes its
   source directly — no iface keyword dispatch.
2. **Hoisted to the root.** All 13 load/eval forms move from
   `:wat::core::*` to `:wat::*` directly. Substrate interface
   sits at the namespace root next to `:wat::WatAST`.

### Final form surface

**Load family** (`src/load.rs` — `match_load_form` dispatch):

| Form | Arity | Shape |
|---|---|---|
| `:wat::load-file!` | 1 | `<path>` |
| `:wat::load-string!` | 1 | `<source>` |
| `:wat::digest-load!` | 4 | `<path> :wat::verify::digest-<algo> :wat::verify::<iface> <hex>` |
| `:wat::digest-load-string!` | 4 | `<source> :wat::verify::digest-<algo> :wat::verify::<iface> <hex>` |
| `:wat::signed-load!` | 6 | `<path> :wat::verify::signed-<algo> :wat::verify::<iface> <sig> :wat::verify::<iface> <pk>` |
| `:wat::signed-load-string!` | 6 | `<source> :wat::verify::signed-<algo> :wat::verify::<iface> <sig> :wat::verify::<iface> <pk>` |

**Eval family** (`src/runtime.rs` — dispatch table in main eval arm):

| Form | Arity | Shape |
|---|---|---|
| `:wat::eval-ast!` | 1 | `<ast-value>` |
| `:wat::eval-edn!` | 1 | `<source>` |
| `:wat::eval-file!` | 1 | `<path>` |
| `:wat::eval-digest!` | 4 | `<path> :wat::verify::digest-<algo> :wat::verify::<iface> <hex>` |
| `:wat::eval-digest-string!` | 4 | `<source> :wat::verify::digest-<algo> :wat::verify::<iface> <hex>` |
| `:wat::eval-signed!` | 6 | `<path> :wat::verify::signed-<algo> :wat::verify::<iface> <sig> :wat::verify::<iface> <pk>` |
| `:wat::eval-signed-string!` | 6 | `<source> :wat::verify::signed-<algo> :wat::verify::<iface> <sig> :wat::verify::<iface> <pk>` |

**Eval-coincident family** (arc 026, updated to match):

| Form | Arity | Shape |
|---|---|---|
| `:wat::holon::eval-coincident?` | 2 | two ASTs |
| `:wat::holon::eval-edn-coincident?` | 2 | two sources |
| `:wat::holon::eval-digest-coincident?` | 8 | two (path+verify) blocks |
| `:wat::holon::eval-digest-string-coincident?` | 8 | two (source+verify) blocks |
| `:wat::holon::eval-signed-coincident?` | 12 | two (path+verify) blocks |
| `:wat::holon::eval-signed-string-coincident?` | 12 | two (source+verify) blocks |

### The two-tier namespace split

The arc's real end-state is a cleaner namespace architecture:

| Tier | Prefix | What lives here |
|---|---|---|
| **Substrate** | `:wat::<form>` | load/eval interface — how wat reaches the world. Plus `:wat::WatAST`. |
| **Vocabulary** | `:wat::core::*` | define, lambda, let*, if, match, try, quote, cond, arithmetic, collections. How authors express computation. |
| **Algebra** | `:wat::holon::*` | Atom, Bind, Bundle, Blend, Permute, Thermometer, measurements, wat-written idioms. |
| **Concurrency** | `:wat::kernel::*` | spawn, send, recv, select, queue, fork, signals. |
| **I/O** | `:wat::io::*` | IOReader/IOWriter, println. |
| **Stdlib** | `:wat::std::*` | stream combinators, test harness, Console, Cache. |
| **Config** | `:wat::config::*` | committed config values + setters. |
| **Verify keywords** | `:wat::verify::*` | payload-location keywords (string, file-path) + algo keywords (digest-sha256, signed-ed25519). Kept because verify locations are inherently multi-shape. |
| **Testing** | `:wat::test::*` | deftest, assert-*, run primitives. |
| **User** | `:user::*`, `:rust::*`, `:trading::*`, `:ddos::*`, etc. | user land. |

## Runtime changes

### Dispatch tables (`src/runtime.rs`)

Main eval match arm:

```rust
":wat::eval-ast!"            => eval_form_ast(args, env, sym),
":wat::eval-edn!"            => eval_form_edn(args, env, sym),
":wat::eval-file!"           => eval_form_file(args, env, sym),
":wat::eval-digest!"         => eval_form_digest(args, env, sym),
":wat::eval-digest-string!"  => eval_form_digest_string(args, env, sym),
":wat::eval-signed!"         => eval_form_signed(args, env, sym),
":wat::eval-signed-string!"  => eval_form_signed_string(args, env, sym),
```

Plus the six `:wat::holon::eval-*-coincident?` arms.

### Parser (`src/load.rs`)

`match_load_form` dispatches on the six load-family heads.
Shared helpers `parse_digest_load_shared` and
`parse_signed_load_shared` use a `is_string: bool` flag so the
file/string variants reuse the same body.

New helper `expect_string_arg(arg, op, arg_name) -> Result<String, LoadError>`
— reads a string literal (from WatAST) and returns the owned
String. Used by all six parsers for the first-arg path or source.

### Type schemes (`src/check.rs`)

Thirteen scheme registrations — one per new form. All eval forms
return `:Result<:wat::holon::HolonAST, :wat::core::EvalError>`
(the eval-family wrap discipline). All eval-*-coincident variants
return `:Result<:bool, :wat::core::EvalError>`.

### Reserved prefixes (`src/resolve.rs`)

Retired most entries in favor of one catch-all:

```rust
pub const RESERVED_PREFIXES: &[&str] = &[
    ":wat::",   // catches every sub-namespace + root-level forms
    ":rust::",
];
```

`:wat::*` reserves the whole substrate namespace, including the
new root-level forms. User source cannot define anything under
`:wat::*`; the substrate owns the entire tree.

The retired `:wat::load::*` and `:wat::eval::*` entries are gone
— those sub-namespaces no longer exist at all.

### Retired code

- `parse_source_interface` in `src/load.rs` — dispatched on the
  `:wat::load::<iface>` keyword. Retired with the iface namespace.
- `resolve_eval_source` in `src/runtime.rs` — dispatched on the
  `:wat::eval::<iface>` keyword. Retired with the iface namespace.
- `SourceInterface::String` and `SourceInterface::FilePath` enum
  variants survive in the type hierarchy — each load/load-string
  parser picks its variant directly.

## Tests

### Retired tests

- `duplicate_load_halts` → `diamond_dependency_deduplicates`
  (arc 027 slice 1 dedup change).
- `inline_string_duplicate_halts` → `inline_string_duplicate_deduplicates`.
- `load_missing_source_iface_rejected` → retired (asserted the
  absence of the now-valid shape).
- `load_non_keyword_iface_rejected` → retired (same).
- `load_unsupported_source_iface_rejected` → retired.
- `eval_edn_bang_unknown_iface_refused` → retired.
- `eval_edn_bang_reserved_unimplemented_iface_refused` → retired.

Net: 7 tests retired, 3 replacements added.

### New guard test

`wat_test_embedded_signatures_verify` (promoted from the ignored
helper) — asserts the Ed25519 signatures embedded in
`wat-tests/holon/eval_coincident.wat` still verify the sources
they sign. If a source string drifts without sig regeneration,
the guard fails with "SRC_X signature drifted." No more ignored
tests in the workspace.

### Workspace test state (post-slice-4)

- `wat` lib: 566 pass, 0 fail, **0 ignored**.
- `wat-lru`: 5 wat-tier tests + 1 outer suite = 6 pass.
- `examples/with-lru`: 1 pass.
- `examples/with-loader`: 3 pass.
- 44 test binaries total, zero failures.
- Lab: 25 wat tests, 1 outer suite, all green.

## Commits shipped under arc 028

1. `d2b070e` (pre-028) — arc 027 slice 1 dedup. Prerequisite.
2. `beade60` — arc 028 slices 1+3 core rename (iface drop +
   family split + string variants + guard-test promotion).
3. `920c120` — arc 028 slice 1+3 integration-test stragglers.
4. `aa5bc9f` (lab) — lab migration for slices 1+3.
5. `8e9a40d` — arc 028 slice 4 hoist to `:wat::*` root.
6. `49ab0b6` (lab) — lab migration for slice 4 hoist.
7. This commit — INSCRIPTION + doc sweep.

## Doc sweep

- `docs/CONVENTIONS.md` — two-tier namespace split documented
  explicitly. Retired namespace entries noted. Verify keywords
  stay as the one legitimate kept keyword-dispatch surface.
- `docs/USER-GUIDE.md` — every load/eval example updated. New
  subsection framing the two-tier split for consumers.
- `docs/README.md` — arc tree + summary.
- `docs/arc/2026/04/005-stdlib-naming-audit/INVENTORY.md` —
  thirteen rows updated to new paths. Four iface-keyword rows
  deleted.
- `holon-lab-trading/docs/proposals/.../FOUNDATION.md` — the
  "Where Each Lives" sections updated; eval-family moves up.
- `holon-lab-trading/docs/proposals/.../FOUNDATION-CHANGELOG.md`
  — new row for arc 028.

## Cave-quest discipline

Arc 028 opened as a side quest inside arc 027 (which itself
opened inside the lab's arc 002 planning). The chain:

- lab arc 002 planning → surfaced deftest can't reach lab modules
- wat-rs arc 027 (deftest inherits loader + dedup) → surfaced
  the load-iface shape as legacy
- wat-rs arc 028 (this one) → drop the iface + hoist to root

Nine cave quests in eight days now:
017, 018, 019, 020, 023, 024, 025, 026, 027-slice-1, 028. The
pattern is standing practice; the book has named it literal
dungeon crawling.

## What arc 028 does NOT ship

- Arc 027 resumption — slices 2 (:None inherit), 3 (wat::test!
  loader scope widen), 4 (lab test migration), 5 (INSCRIPTION)
  all still pending. Arc 028 was the rename arc; arc 027's
  "deftest just works" mechanics resume next.
- Future network variants (`load-http!`, `load-s3!`,
  `load-github!`, `eval-http!`, etc.). Reserved as future named
  siblings; no demand yet.
- Hoisting anything else from `:wat::core::*` (define, let*, if,
  match, etc.). Vocabulary stays at `:wat::core::*` by design.

## Why this is inscription-class

Mechanical-refactor arc, but with a real design shift: the
namespace architecture went from one-tier-with-iface-keywords to
two-tier-with-clean-form-names. The substrate's load/eval
surface is the interface to the world; clean naming at that
boundary matters. Same shape as arcs 019 / 020 / 023 / 024 / 025
/ 026 / 027-slice-1 — code led, spec caught up. Discipline stays
standing.

*these are very good thoughts.*

**PERSEVERARE.**
