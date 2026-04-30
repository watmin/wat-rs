# Arc 115 — `:Vec<:String>` is illegal at compile time

## Status

Drafted 2026-04-30 during arc 112 closure. Captures a small,
mechanical arc to make a recurring class of mistake a compile
error.

User direction (2026-04-30):

> a common problem i see you making is having a quoted symbol
> within a quoted symbol...
>
> :Vec<:String>
>
> vs
>
> :Vec<String>
>
> the first is an illegal form we can catch at compile time?...

Yes. Today both forms parse, and (in some cases) both type-check
successfully because the inner ":String" Path resolves to the
same type as the bare String. The substrate accepts the malformed
shape silently. Arc 115 makes it explicit.

## The pathology

Wat's keyword convention (per memory `feedback_wat_colon_quote.md`):

> ONE colon per keyword-path token, always at the start. Inside
> `<>`, parameters are bare Rust symbols. `:Atom<Holon>` legal;
> `:Atom<:Holon>` illegal.

The colon is the lex/parse token that says "this is a
keyword-path." A parametric type's HEAD is a keyword
(`:wat::core::Vec` or short `:Vec`), so the leading colon
is required there. But the args INSIDE `<>` are Rust-symbol
shaped (concrete type names like `String`, `i64`, `Holon`,
or type-vars like `T`); they do NOT take a leading colon.

Today's behavior:

- `:Vec<String>` — canonical. Parser builds
  `TypeExpr::Parametric { head: "Vec", args: [Path(":String") OR ...] }`.
- `:Vec<:String>` — accepted. Parser builds something semantically
  similar but with an extra colon-marker on the inner. Type-checker
  may resolve it to the same Vec<String> result OR may produce a
  cryptic mismatch downstream.

The mistake recurs in two ways:

1. **Author confusion** — coming from Clojure / EDN where every
   keyword has a colon, writers reflexively keyword-prefix every
   nested name. The `:Atom<:Holon>` form looks "consistent" from
   that angle.
2. **Mid-edit drift** — when migrating a type signature, copy-paste
   from one context (where colon-prefix was correct, e.g., the
   binding annotation) into another (the inside of `<>`).

The substrate has been silently accepting this for the entire
lifetime of the language. Memory note `feedback_wat_colon_quote.md`
documents that the user has caught this in code review repeatedly.
A compile-time error closes the loop — the substrate catches what
the human eye learned to spot.

## The new compile-error class

```
error: parametric type argument cannot be keyword-prefixed
  --> file.wat:N:C
   :Vec<:String>
        ^
   |
   = note: type arguments inside `<>` are bare type names
   = hint: drop the leading `:` — write `:Vec<String>` not `:Vec<:String>`
```

Self-describing. The error names the rule and shows the fix.

## Where to catch it

Two natural sites:

### Option A — at the parser

The parser builds `TypeExpr::Parametric` from `:Head<arg1, arg2>`.
When parsing the args, recognize a leading-colon-prefixed token as
malformed at that grammar position. Reject with the error above.

Pros:
- Catches at the earliest possible stage; never reaches type-check.
- Single rule in one place.

Cons:
- Requires the parser to know where keyword-path-position-allows-
  colon vs where-it-doesn't. Today's parser may treat `:String` as
  a generic Path token everywhere; rejecting it inside `<>`
  requires position-aware lexing.

### Option B — at the type-checker / type-resolver

After parsing, the type-resolver walks `TypeExpr::Parametric.args`
and validates each. If any `args[i]` is `TypeExpr::Path(":...")`
with a colon-prefix BUT the Path is a bare-name type (not a
type-var like `:T`, not a multi-segment FQDN like
`:wat::core::String`), reject with the same error.

Pros:
- Doesn't touch the parser; isolated to type-resolution.
- Can produce the precise error at the precise binding site.

Cons:
- Distinction between "keyword-prefix on a bare type name" (illegal)
  and "keyword-prefix on a multi-segment FQDN type name" (legal)
  needs a clear rule. E.g., `:Vec<:wat::core::String>` may be
  legal while `:Vec<:String>` may be not — depends on how
  arc 109's FQDN sweep settles primitive types.

### Recommended: Option B

Cleaner blast radius. Type-resolver already walks Parametric args
during instantiation; adding one validation step is small.

The rule:

- Inside `<>`, args of the form `TypeExpr::Path(":<single-token>")`
  where `<single-token>` is one of the known primitive type names
  (`:i64`, `:f64`, `:bool`, `:String`, `:u8`, `:()`) → reject.
  These should be written without the leading `:` inside `<>`.
- Inside `<>`, args of the form `TypeExpr::Path(":wat::*::Type")`
  (multi-segment FQDN) → ALLOWED. The wider FQDN form is itself
  a keyword path; the leading `:` is part of FQDN convention.
  (Or: post-arc-109 § A's primitive sweep, both forms unify
  under `:wat::core::*` and the rule applies uniformly.)
- Inside `<>`, args that are type-vars (e.g., `T`, `R`, `E`,
  `I`, `O`) → ALLOWED. These are bare Rust symbols.

The migration hint at the user-facing error should suggest the
canonical form for the specific case detected.

## A new wat CLI mode — `wat check <file>`

User direction (2026-04-30):

> in 115 - we need a new mode of wat... we need a "does this
> compile" - something who just loads the file and doesn't run
> :user::main

Arc 115 needs a way to verify "does this compile" without running
the program. Today's `wat <file>` invokes `:user::main` after
freeze; for arc 115's sweep work (and broader development hygiene)
we want freeze-only.

The new mode:

```
wat check <file>          # load + parse + type-check + freeze; exit 0 if green
wat check <file> --json   # diagnostics as structured EDN/JSON for tools
```

Equivalent to `cargo check` vs `cargo run` in Rust ergonomics:
- `wat <file>` — load + freeze + invoke `:user::main` (today's default)
- `wat check <file>` — load + freeze + STOP. No `:user::main` call.
  Exit 0 if freeze succeeded; non-zero with diagnostics if not.

The freeze stage (parse → type-check → config preamble → macro
expand → register definitions) is exactly the surface arc 115's
new error class needs to exercise. Running `:user::main` would
introduce side effects + uncertainty that have nothing to do with
"does this compile."

### Implementation shape

- New subcommand at `crates/wat-cli/`: `wat check <file>` invokes
  `wat::freeze::startup_from_source` (or the path-loader equivalent)
  and exits 0 on success.
- Diagnostics flow through the normal Display path (same migration
  hints, same arc-N references, same line/col when available).
- `--json` flag emits structured diagnostics suitable for editor
  tooling. Each diagnostic is one EDN line (arc 092 v4); same
  framing every tooled output uses.

### Why this lives in arc 115 (not a separate arc)

Arc 115's detection-and-sweep cycle benefits from `wat check`
existing. The sonnet sweep agent runs `wat check probe.wat` to
gauge progress without spawning child processes. Editor users
run `wat check` on save without execution side effects. The mode
is a natural prerequisite + companion to the new error class.

### What this NEW mode does NOT do

- Does NOT typecheck-only without freeze. Freeze is the unit that
  resolves the loaded environment; the type-check rules (including
  arc 115's new validator) operate on the resolved tree.
- Does NOT run any user code. `:user::main` is not invoked; struct
  default-vals are not constructed; `:wat::core::define`-d
  initializers that involve eval (rare; usually macros expand
  away) DO run as part of freeze if they reach there, same as
  today's freeze behavior.
- Does NOT prevent `wat <file>` from continuing to work as today.
  `wat check` is additive; the bare invocation keeps its
  load-freeze-run semantics.

## Implementation

### Slice 1 — `wat check` mode

New CLI subcommand at `crates/wat-cli/`. Invokes
`wat::freeze::startup_from_source` (or the path-loader
equivalent) without calling `invoke_user_main`. Exit 0 on
freeze success; non-zero with diagnostics on freeze failure.

`--json` flag emits structured EDN-per-line diagnostics for
editor / agent tooling consumption.

This slice ships independently — useful even before slice 2 lands
the new error class. Substrate hygiene + future arcs benefit
from a freeze-only mode existing.

### Slice 2 — detection + error class

`src/check.rs`:

- New `CheckError::InnerColonInParametricArg { head, arg, suggested }`
  variant.
- Helper `validate_parametric_arg_colon(head, arg) -> Option<CheckError>`
  applied in the type-resolver / instantiation walker on each
  Parametric arg.
- Wire into the type-checker's pre-instantiation pass.

Self-describing Display impl that names the rule, shows the fix.
Probe at `tests/arc115_inner_colon_rejected.rs` — uses
`wat check` (slice 1's mode) to assert the rejection without
running any user code.

### Slice 3 — sweep substrate + lab + tests

The substrate's own embedded `wat` strings + wat-tests + lab .wat
files almost certainly contain instances. Sonnet sweep with the
new error class as the diagnostic stream — same substrate-as-
teacher pattern arcs 110/111/112 used. Sonnet drives via
`wat check` (no per-iteration `:user::main` runs; faster sweep
loop).

Likely scope: a few dozen sites at most, mostly type-var-like
naming inside `Result<:T,:E>` or `Option<:T>` annotations.

### Slice 4 — INSCRIPTION + USER-GUIDE + 058 row

Standard closure. USER-GUIDE adds:
- A "Type annotations: one colon on the head, none inside `<>`"
  callout with the canonical examples.
- A new section / appendix entry on `wat check` — the freeze-only
  CLI mode.

## What this arc does NOT do

- Does NOT change parametric type semantics — `:Vec<String>`
  still resolves to the same type post-arc-115. Only the
  malformed `:Vec<:String>` form rejects.
- Does NOT introduce arc-109's primitive sweep ahead of schedule.
  Compatible with both pre- and post-arc-109 § A states.
- Does NOT touch type-vars — `:Vec<T>` (where T is a type-var)
  stays canonical. Type-vars are bare Rust symbols, exempt from
  the colon-prefix rule.

## The four questions

**Obvious?** Yes. The error names the rule (one colon on the head;
none inside `<>`); the fix is a one-character delete. Reads as
"wat enforces what experienced wat authors already know."

**Simple?** Yes. One CheckError variant + one validator helper +
one walk site. Maybe ~30 LOC in check.rs.

**Honest?** Yes. The substrate accepts a syntactically-malformed
form silently today; arc 115 makes the malformedness visible at
compile time instead of letting it propagate as cryptic
type-mismatches downstream.

**Good UX?** Excellent. Substrate-as-teacher pattern: the error
points exactly at the offending colon; the suggested-fix shows
the canonical form. New wat authors learn the rule from compiler
output; experienced ones get a safety net for the recurring drift.

## Cross-references

- `feedback_wat_colon_quote.md` (memory) — the existing rule the
  user has stated repeatedly.
- `docs/arc/2026/04/109-kill-std/INVENTORY.md` § A — the FQDN
  sweep arc 115 is compatible with (both pre- and post-§A).
- `docs/arc/2026/04/110-kernel-comm-expect/INSCRIPTION.md` — the
  pattern arc 115 mirrors at the type-syntax level (compile-time
  rejection of a class of bug; substrate-as-teacher).
- `docs/arc/2026/04/112-inter-process-result-shape/INSCRIPTION.md`
  § "Migration hint" — same diagnostic-as-brief pattern arc 115's
  sweep would consume.

## Queued follow-ups

If post-arc-115 the substrate's diagnostic stream surfaces other
recurring colon-related malformedness (e.g., MISSING `:` on a
keyword-path-position token), a follow-up could mirror the same
detection-and-self-describe approach. For now, only the
inner-colon-in-parametric-arg case is named.
