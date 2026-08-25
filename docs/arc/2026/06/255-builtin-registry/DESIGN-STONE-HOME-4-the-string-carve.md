# DESIGN — HOME #4: the string carve, and the runner that has been hiding a red

> The last link of `CHAIN-rendering-before-the-string-home.md`. A→E are on disk (`23efc6056`).
> Home #4 was moved to LAST so it would register **final names with final signatures, once
> instead of twice** — and stone E just made those names final.

## ⛔ FIRST — THE IGNORE IS STALE, AND IT IS NOT DEFERRING AN UNBUILT FEATURE

`tests/reflection/probe_arc255_ivb2b_verify_examples.rs:32`:

```rust
#[ignore = "RED-at-HEAD: arc-255 metadata-of reflection (builtin-registry) not yet built;
            unlock when we circle back to arc 255"]
```

**Its unlock condition is this stone.** And its stated reason is false — both halves exist:

```
:wat::doctest::verify-examples   wat/doctest.wat:38        implemented
:wat::runtime::metadata-of        src/runtime.rs:5588       implemented
```

Run with `--run-ignored`, measured this session:

```
#wat.kernel/AssertionFailure {:message "verify-examples: expr eval failed"
 :location #wat.core/Span {:file "wat/doctest.wat" :line 64 :col 35}}
```

**The runner WORKS. An example is BROKEN.** The comment says "the verb doesn't exist → eval errors
here"; the verb exists, evaluates, and reports a real failure. This is a blocker note whose premise
expired — the second one found today (`wat/lint.wat:8`'s auto-fix STOP-1 is the other, and arc 281
made it false the same way). A stale blocker is self-protecting: its whole job is to stop people
looking.

★ **176 `@example` directives currently assert NOTHING.** That is the standing cost, and this
morning's realization named a concrete instance before anyone connected it to this ignore:
`src/intrinsic/reflect.rs:610-612` carries three `@example` lines asserting a call returns `true` —
**the call raises.**

### And the runner does not name the failing example

The diagnostic points at `doctest.wat:64:35` — inside the runner. It does not say which intrinsic,
which example, or what it evaluated. A doctest runner that fails anonymously makes its own red
expensive to act on, and that is a defect in the runner, not in the examples.

## WHY THE ORDER IS UNLOCK-THEN-CARVE

The carve registers 19 verbs, each with a `///` preamble the macro parses — `@arg`, `@ret`,
`@example`, `@see`. **Carving first adds ~19 more unverified claims to a population of 176 that
already asserts nothing.** Unlocking first means every example the carve writes is checked the
moment it lands.

This is the same argument that put home #4 last in the chain, applied one level down: do not create
the thing whose verification you have deferred.

## THE CARVE — 19 verbs, and the file they leave is a junk drawer

`src/check.rs` registers **19** `:wat::string::*` verbs (measured — this is the surface, not a grep):

```
concat  contains?  declare-acronyms  ends-with?  interpolate  join
kebab->pascal-in   length  pascal->kebab  pascal->kebab-in  split
starts-with?  subs  to-bool  to-f64  to-i64  to-lowercase  to-uppercase  trim
```

They live in `src/string_ops.rs` — **1254 lines, and NOT in the registry** (`#[wat_intrinsic]`
count: **0**). The registry holds 146 registered intrinsics across 19 modules; string is absent from
all of them.

⚠ **`string_ops.rs` is four domains in one file** — a `partire` finding, and it is why this stone
must name its own boundary:

```
:wat::string::*        15   ← this stone
:wat::core::Uuid/*     11   ← NOT string
:wat::core::char/*      2   ← NOT string
:wat::core::regex::*    1   ← NOT string
```

**Uuid, char, and regex get named as their own homes, not silently absorbed.** A "string carve" that
quietly relocates UUID generation is a carve that lied about its subject. They stay where they are
until someone draws them.

## THE SHAPE — what a registered intrinsic carries

Per `src/intrinsic/bytes.rs` (arc 255's first home), each handler carries a full preamble the
`#[wat_intrinsic]` macro parses; the macro sniffs arity, emits the arity-checking shim, and
`inventory::submit!`s the (fqdn → shim) pair. No explicit `register()` call:

```rust
/// Markdown prose, GFM — flows straight to the wiki page body.
///
/// @added 1.0.0   @Purity Pure   @Determinism Deterministic   @Category Transform
/// @arg     s :wat::core::String  …
/// @ret     :wat::core::String  …
/// @example (:wat::string::trim "  x  ") #=> "x"
/// @see     :wat::string::to-lowercase
#[wat_intrinsic(":wat::string::trim")]
```

## ACCEPTANCE

1. ★ **`verify_examples_reports_no_failures` is UN-IGNORED and GREEN.** Not "un-ignored"; green. The
   failing example is found, and either the example or the intrinsic is fixed — whichever is wrong.
2. **The runner names its failure.** Break one example deliberately and confirm the diagnostic says
   WHICH intrinsic and WHICH example. If it cannot, that is this stone's work too — a red nobody can
   locate is a red nobody will fix.
3. **All 19 string verbs are registered** — `metadata-of` answers for each, and the count of
   `#[wat_intrinsic]` in the string home is 19.
4. **Every new example RUNS**, because row 1 landed first. A carve whose examples are unchecked has
   not done the thing this home exists for.
5. **`string_ops.rs` retains Uuid/char/regex and nothing else** — the boundary is affirmative.
6. **`reflect.rs:610-612`'s three false examples are resolved** — they assert `true` on a call that
   raises. Whichever way they resolve, they stop lying.
7. Floor green accounted BY NAME; clippy 0.

## OUT OF SCOPE — affirmatively cut

- **Uuid / char / regex homes.** Named above, not carved here. Each is its own stone and this one
  says so rather than leaving them to drift.
- **The other 157 examples' CONTENT.** Row 1 makes them run; it does not promise every one is
  well-written. A red that surfaces there is a finding for a follow-up, not scope creep into this.
- **`probe_arc255_reflection_parity`'s own `#[ignore]`** — a sibling with the same stale reason
  (`"not yet built; unlock when we circle back"`). Measure it the same way; if it is also hiding a
  red rather than a gap, that is a second finding and it is cheap to get.
