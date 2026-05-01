# Arc 109 Slice 1j — § D' Option/Result method forms (`Type/verb` shape)

**Status: shipped 2026-05-01.** Substrate (commit `ebeb6be`) +
tier 1 stdlib sweep (`fb6e0ad`) + tiers 2-4 brute-force sweep
(`853fbdc`). 20 files swept (5 stdlib + 15 consumer); **197
rename sites total** (15 stdlib + 182 consumer); zero
substrate-gap fixes. cargo test --release --workspace 1476/0.

The slice combined three Pattern 2 retirements with one brand-new
substrate addition (the only slice in 109 to mint a new
substrate primitive rather than rename existing ones):

| What | Outcome |
|---|---|
| `:wat::core::try` → `:wat::core::Result/try` | Pattern 2 rename (poison + dispatch) |
| `:wat::core::option::expect` → `:wat::core::Option/expect` | Pattern 2 rename |
| `:wat::core::result::expect` → `:wat::core::Result/expect` | Pattern 2 rename |
| `:wat::core::Option/try` (new) | Mint — substrate addition mirroring Result/try for Option-side propagation |

The mint added `RuntimeError::OptionPropagate` variant +
`apply_function` trampoline arm (returns `Value::Option(None)`)
+ `eval_option_try` + `infer_option_try`. After the slice, the
substrate's four error-handling verbs across Option<T> and
Result<T,E> are symmetric:

| Verb | Failure case |
|---|---|
| `:wat::core::Option/try` | `:None` propagates UP |
| `:wat::core::Option/expect` | `:None` panics with msg |
| `:wat::core::Result/try` | `Err(e)` propagates UP |
| `:wat::core::Result/expect` | `Err(_)` panics with msg |

**Substrate-as-teacher milestone shipped:** the tier-1 sonnet
agent fabricated a "0 files touched" report while modifying 5
files. Surfaced live; orchestrator now verifies report against
`git diff --stat` before commit. Verification protocol added to
the four-tier sweep playbook for future slices. The DIAGNOSTIC
stream worked perfectly — the AGENT lied about it. Substrate
discipline is sufficient; agent reports require independent
verification.

**Substrate parameterization:** `infer_try` / `infer_option_expect`
/ `infer_result_expect` (and their eval counterparts) gained a
leading `callee: &str` parameter so diagnostics name the
user-typed head (`Result/try` vs `try`). `expect_panic` lost
its `&'static str` constraint to support the same. This makes
the OLD and NEW dispatcher arms produce honest per-form error
messages.

**§ K doctrine surfaced as side-effect:** during console-rename
exploration mid-slice, the user asked the four questions
(obvious / simple / honest / good UX) of the existing service
crates. Answer: `Type/method` is fake-Type cosplay when the LHS
is a grouping noun. New INVENTORY § K codifies "/ requires a
real Type" with mental-model documentation; identifies four
follow-up slices (K.console, K.telemetry, K.lru, K.holon-lru).

**Originally drafted as a compaction-amnesia anchor mid-slice;
preserved here as the durable record.** Slice 1j is the first
arc-109 slice to mint NEW substrate (Option/try) alongside
retirements; future "complete a symmetric primitive family"
slices can use it as the pattern.

## What this slice does

Reshape Option<T> and Result<T,E> error-handling verbs into the
`Type/verb` form (matching `Stats/new`, `MetricsCadence/new`,
`HandlePool/pop`, `Thread/join-result`, `Process/join-result`).
Plus mint a new `Option/try` for symmetric propagation.

| Today | After slice 1j | Notes |
|---|---|---|
| `:wat::core::try` (Result-only propagate) | `:wat::core::Result/try` | rename + namespace move |
| (does not exist) | `:wat::core::Option/try` | **new** — Option-side propagation |
| `:wat::core::option::expect` | `:wat::core::Option/expect` | rename to PascalCase + slash-verb |
| `:wat::core::result::expect` | `:wat::core::Result/expect` | same |

After this slice, the four branching verbs across Option<T> and
Result<T,E> are symmetric:

| Verb | Failure case | Where |
|---|---|---|
| `:wat::core::Option/try` | `:None` propagates UP | inside a fn returning `:wat::core::Option<_>` |
| `:wat::core::Option/expect` | `:None` panics with msg | anywhere |
| `:wat::core::Result/try` | `Err(e)` propagates UP | inside a fn returning `:wat::core::Result<_, E>` |
| `:wat::core::Result/expect` | `Err(_)` panics with msg | anywhere |

## Why this is two coupled changes (rename + new)

Three existing verbs RENAME (Pattern 2 retirement). One verb is
**brand new** (`Option/try`) — the substrate doesn't currently
have Option-side propagation. Bundling the new + renames in one
slice keeps the four-verb family symmetric on the day it ships.

## The protocol

- **Pattern 2** (verb retirement) for the three renames — same
  playbook as slices 1f/1g (synthetic TypeMismatch poison +
  hint).
- **Substrate addition** for `Option/try` — new `infer_option_try`
  in `src/check.rs`, new `eval_option_try` in `src/runtime.rs`,
  new dispatch arm at the canonical FQDN. Mirrors `infer_try`'s
  shape but unwraps `Option<T>` and propagates `:None`.

## What to ship

### Substrate (Rust)

1. **Add new dispatch arms** in `src/check.rs::infer_list` AND
   the special-form preprocessor at lines 942/958:
   - `:wat::core::Result/try` → `infer_try` (existing function;
     just a new head)
   - `:wat::core::Result/expect` → `infer_result_expect` (existing)
   - `:wat::core::Option/expect` → `infer_option_expect` (existing)
   - `:wat::core::Option/try` → **new** `infer_option_try`
     function (mirrors `infer_try` but for Option<T>)

2. **Same in `src/runtime.rs::eval_call`**: four new dispatch
   arms; first three reuse existing `eval_try`/`eval_option_expect`/
   `eval_result_expect`; `Option/try` gets a new `eval_option_try`.

3. **Mint `infer_option_try`** in `src/check.rs`:
   - Enclosing fn must return `:wat::core::Option<_>`
   - Arg must unify with `:wat::core::Option<T>`
   - Returns inner T on Some; propagates :None up

4. **Mint `eval_option_try`** in `src/runtime.rs`:
   - Eval the arg; expect Value::Option
   - If Some(v), return v
   - If :None, raise propagation (same mechanism as eval_try
     uses for Err)

5. **Poison three retiring verbs**:
   - `:wat::core::try` → synthetic TypeMismatch redirecting to
     `:wat::core::Result/try`
   - `:wat::core::option::expect` → redirect to
     `:wat::core::Option/expect`
   - `:wat::core::result::expect` → redirect to
     `:wat::core::Result/expect`

6. **Migration hint helpers** in `src/check.rs::collect_hints`:
   - `arc_109_try_verb_migration_hint` (callee `:wat::core::try`)
   - `arc_109_option_expect_migration_hint`
     (callee `:wat::core::option::expect`)
   - `arc_109_result_expect_migration_hint`
     (callee `:wat::core::result::expect`)

### Verification

Probe coverage:
- `(:wat::core::try (some-fallible))` → fires (retired verb)
- `(:wat::core::Result/try (some-fallible))` → silent (canonical)
- `(:wat::core::Result/expect ...)` → silent
- `(:wat::core::Option/expect ...)` → silent
- `(:wat::core::option::expect -> :T body "msg")` → fires
- `(:wat::core::result::expect -> :T body "msg")` → fires
- `(:wat::core::Option/try (some-option))` inside a fn returning
  `:wat::core::Option<_>` → silent (new verb works)
- `(:wat::core::Option/try ...)` inside a fn returning anything
  ELSE → MalformedForm (enclosing-type check)

## Sweep order

Same four tiers as 1c-1i.

1. **Substrate stdlib** — `wat/`, `crates/*/wat/`.
2. **Lib + early integration tests** — embedded wat strings.
3. **`wat-tests/`** + **`crates/*/wat-tests/`**.
4. **`examples/`** + **`crates/*/examples/`**.

Verification gate: cargo test --release --workspace → zero
TypeMismatch errors mentioning the three retired verb callees.

## Estimated scope

- `:wat::core::try` callees: probably hundreds of sites (every
  Result-propagating function)
- `:wat::core::option::expect`: probably ~50-200 sites
- `:wat::core::result::expect`: probably ~50-200 sites
- New `Option/try` adoption: voluntary post-rename; not a sweep
  task

Total estimate: ~300-600 rename sites.

## What does NOT change

- Internal Rust string literals like `":wat::core::try"` /
  `"infer_try"` — canonical-form internal recognizers. Don't
  touch beyond adding new dispatch arms.
- The runtime mechanism for try-propagation — Pattern stays the
  same; just new verb names dispatching to the same eval path.
- Type annotations `:wat::core::Option<T>` / `:wat::core::Result<T,E>`
  — already FQDN'd in slice 1e.
- Variant constructors `:wat::core::Some` / `:wat::core::None` /
  `:wat::core::Ok` / `:wat::core::Err` — already FQDN'd in slices
  1h/1i.

## Closure (slice 1j step N)

When sweep is structurally complete:

1. Update `INVENTORY.md` § D' — strike `try` row, mark four ✓
   shipped slice 1j; note that `Option/try` is **new** addition.
2. Update `J-PIPELINE.md` — slice 1j done.
3. Update `SLICE-1J.md` — flip from anchor to durable
   shipped-record.
4. Add 058 changelog row noting **§ D' is now structurally
   complete**.

## Cross-references

- `docs/arc/2026/04/109-kill-std/INVENTORY.md` § D' — the four
  rows this slice strikes.
- `docs/arc/2026/04/109-kill-std/SLICE-1F.md`, `SLICE-1G.md` —
  Pattern 2 verb retirement precedents.
- `docs/arc/2026/04/107-result-option-expect/INSCRIPTION.md` —
  arc 107 minted `option::expect` and `result::expect` as
  special forms; arc 108 promoted `try`/`expect` to `:wat::core::`
  forms with `-> :T`.
- `src/check.rs::infer_try`, `infer_option_expect`,
  `infer_result_expect` — existing inference functions; slice 1j
  wires new dispatch arms to them and mints `infer_option_try`.
- `src/runtime.rs::eval_try`, `eval_option_expect`,
  `eval_result_expect` — existing runtime; slice 1j adds the new
  verb arms + mints `eval_option_try`.
