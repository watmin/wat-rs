# Naming Conventions

Rules for adding new primitives to wat. Derived from the arc 005
stdlib naming audit. When these disagree with a primitive in the
[inventory](./arc/2026/04/005-stdlib-naming-audit/INVENTORY.md),
the audit fixes the primitive — not the convention.

---

## Privileged prefixes

`:wat::*` and `:rust::*` are reserved by the runtime. User code
cannot register under them; the evaluator refuses. These two
namespaces ship only via the privileged `register_stdlib_*` paths
or through `#[wat_dispatch]` for `:rust::*`.

Every other prefix is user territory.

## Namespaces

| Prefix | What lives here |
|---|---|
| `:wat::core::*` | Evaluator primitives — forms (`define`, `lambda`, `let*`, `if`, `match`), primitive types (`i64`, `bool`, `String`, ...), macros, eval-family, primitive-type operations (`i64::+`, `bool::and`), core collection constructors (`vec`, `list`, `cons`, `conj`). Cannot be written in wat. |
| `:wat::config::*` | Runtime-committed configuration values (noise floor, dimensions). Read-only after config pass. |
| `:wat::algebra::*` | VSA primitives — `Atom`, `Bundle`, `Unbundle`, `Amplify`, `Compose`, `cosine`, `presence?`, `Resonance`, `Thermometer`, `Blend`. |
| `:wat::kernel::*` | CSP primitives — `spawn`, `send`, `recv`, `select`, `drop`, `join`, `make-bounded-queue`, `HandlePool`, signal handlers. |
| `:wat::io::*` | Stdio primitives — `stdin`, `stdout`, `stderr`, `println`. |
| `:wat::std::*` | Stdlib built on primitives. Each entry should be expressible (in principle) in wat itself, even if shipped as Rust for performance. `LocalCache`, `stream::*`, `program::Console`, `program::Cache`, `list::*`, `math::*`. |
| `:rust::*` | Surfaced Rust types via `#[wat_dispatch]`. Paths mirror real Rust (`:rust::std::iter::Iterator`, `:rust::crossbeam_channel::Receiver`). |
| `:user::*`, `:app::*`, `:my::*`, etc. | User code. Any prefix not in the reserved list. |

## Name formats

- `snake-case` for functions: `make-bounded-queue`, `for-each`,
  `spawn-producer`.
- `PascalCase` for types: `Bundle`, `HashMap`, `Console`, `Stream`.
- `?` suffix for predicates: `presence?`, `empty?`, `capital-available?`.
- `!` suffix for side-effect forms where the caller should notice:
  `use!`. Most side-effect primitives (`send`, `recv`, `println`)
  don't carry `!` because their purpose is visible in the name;
  `!` is for forms that would otherwise read as pure.
- `::` segments the path; `-` segments words within a segment.
- Qualifiers AFTER the base name:
  `:wat::core::i64::+` (ops on `i64`), not `:wat::core::+::i64`.

## When to add a primitive

The stdlib is a blueprint, not a reference library. A primitive
earns its slot when a concrete caller demands it — not
speculatively.

Before adding a new form, two checks:

**1. Absence is signal.** If the feature seems missing, ask *why*
before patching. The gap often points at real substrate work —
a one-site patch that should have been a substrate pass. (See
arc 004's `reduce` — two half-passes that should have been one.)

**2. Verbose is honest.** Before adding an "ergonomic" form,
write out what it expands to and list what it ELIMINATES. For
each eliminated thing: ceremony or information? If information,
rejected. (See arc 004's pipeline composer — the eliminated
per-stage type annotations were information, not ceremony.)

Both lessons were captured as numbered procedures in arc 004's
INSCRIPTION. Both are memory entries
(`feedback_absence_is_signal`, `feedback_verbose_is_honest`)
because both recur across sessions.

## Sources of truth

- **Canonical primitive list**:
  [`arc/2026/04/005-stdlib-naming-audit/INVENTORY.md`](./arc/2026/04/005-stdlib-naming-audit/INVENTORY.md)
- **Language specification**: `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION.md`
- **As-shipped contracts**: `arc/*/INSCRIPTION.md` — each arc's
  shipped surface. If INSCRIPTION and DESIGN disagree, INSCRIPTION
  wins.
