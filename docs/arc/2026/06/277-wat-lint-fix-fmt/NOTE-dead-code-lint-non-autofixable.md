# NOTE — a "function not used" (dead-code) lint, NON-autofixable (2026-06-21)

Builder ask: implement, in wat's self-hosted linter, something like rust/cargo/clippy's
**`dead_code` ("function is never used")** — a name that is defined/registered but never
referenced anywhere in the corpus.

## The autofix axis
The linter carries autofix as a **per-rule flag** (`fix: Some(FixEdit)` = auto-applicable;
`None` = report-only — see `wat/lint.wat:49`, the concat→format fix is the first autofixable
rule). **This dead-code lint is explicitly NON-autofixable** (`fix: None`). You cannot blindly
auto-remove a possibly-dead function: it may be public API, a not-yet-wired hook, a
test-only helper, or reflectively/dynamically reached. Removal is a human-judgment call. So
the rule REPORTS ("`:my::ns/foo` is defined but never referenced") and stops there — never
edits.

## Why it becomes feasible (the arc-255 synergy)
This lint needs to answer "is this name referenced anywhere?" — which needs (a) the full set
of defined/registered names and (b) reference-tracking across the corpus. **Arc 255 (builtin
registry) delivers (a):** once builtins AND user forms are registered + reflectable in `sym`
(with `child-namespaces`/`names` enumeration + `metadata-of`), the linter can enumerate every
callable and cross-check it against the call-graph the rete/resolve passes already build. The
dead-code lint is a natural consumer of the 255 registry — it queries the registry for the
universe of names, the resolver/call-graph for the referenced set, and reports the difference.

## Scope marker
Not built yet — a queued lint rule for the 277 linter, unblocked by 255's registry +
reference enumeration. NON-autofixable (`fix: None`, report-only). Pairs with the registry's
reflection surface (255.2) and the resolver's reference tracking.

---

# NOTE — the self-retiring `expect-dead` annotation (builder, 2026-06-21)

**The ask (wat-side primary; Rust-side too "would be awesome"):** the plain dead-code lint above is
*report-only*. But a `#[allow(dead_code)]`-style annotation (the thing you put on a deliberately-dead
item to silence the report) **rots silently** — and arc-255 iv-b1 just planted a live instance: a dated
`#[allow(dead_code)]` on `IntrinsicEntry.examples` / `ExampleSubmission` with a comment-clause "remove
when iv-b2's seam reads these." That clause is enforced by *nobody*. Two ways it rots:
1. the item **becomes used** (iv-b2 reads `examples`) and we forget to remove the allow → the annotation
   now **lies** (says "dead" over live code);
2. the item **stays dead forever** (the promised reader never lands) → a "temporary" allow becomes
   permanent cruft, and the dead code lingers behind a silence.

So the builder wants a **self-retiring** annotation — an allow that **cannot go stale**, enforced
**bidirectionally**:
- **used-while-annotated → ERROR** — "`:my::ns/foo` is marked expected-dead but is referenced at
  `<site>`; remove the annotation, it's alive now." (Forces removal the moment the reader lands.)
- **still-dead-past-its-trigger → ERROR** — "`:my::ns/foo` has been expected-dead since `<trigger>` and
  is still unreferenced; wire its reader or delete it." (Forces a decision; no eternal silence.)

Both end-states force action: the only stable state is *resolved* (used → annotation gone; or truly
dead → deleted). The annotation is a promise the toolchain keeps for you.

## Enforceability (which half is clean, which needs a trigger)
- **used-while-annotated → ERROR** is **fully checkable** from the call-graph the dead-code lint already
  needs (255 registry = the name universe; resolver = the referenced set). For an `expect-dead` item,
  *referenced* flips from OK to a finding. This is the high-value half and it's the same machinery as the
  plain lint, inverted. **Build this first.**
- **still-dead-past-its-trigger → ERROR** needs a **machine-readable trigger** on the annotation (a
  version, a named arc/stone, or a date), else "past its trigger" is unknowable. Simplest first cut: the
  annotation carries a free-text removal-clause and the linter *lists* every outstanding `expect-dead`
  (so they can't hide) + errors on the used-half; the trigger-deadline is a later refinement.

## Rust-side (bonus — and Rust already ships half of it)
**Prior-art collision, noted honestly: Rust 1.81 stabilized `#[expect(lint)]`** (RFC 2383). `#[expect(dead_code)]`
is *exactly* the used-while-annotated→ERROR half: it allows the item being dead, but emits an
**unfulfilled-expectation** warning the moment the item *is* used (the lint it expected no longer fires).
So the Rust side of the builder's idea is largely **free**: swap our dated `#[allow(dead_code)]` →
`#[expect(dead_code)]` and the compiler *forces the removal* when iv-b2's seam reads `examples`. The
**deadline half** (still-dead-past-trigger) is NOT in Rust's `#[expect]` — that remains our addition
(a build-gate that scans for `#[expect(dead_code)]` older than a named trigger).

**Immediate, concrete action available now:** the iv-b1 dated allows in `src/intrinsic/mod.rs` are the
textbook case — change them to `#[expect(dead_code)]` (verify toolchain ≥ 1.81 first) and the
"remove when iv-b2 lands" clause becomes compiler-enforced instead of comment-enforced. (Surfaced to the
builder; not done unprompted — it's a real toolchain/edition check + a decision.)

## Scope marker
Wat-side: a queued lint rule (the self-retiring `expect-dead`), built on the same 255-registry +
call-graph machinery as the plain dead-code lint above — build the used-while-annotated half first.
Rust-side: adopt `#[expect(dead_code)]` for transient allows (near-free); the deadline-gate is ours to
add. The live test case is iv-b1's `IntrinsicEntry.examples` allow — retire it via this mechanism when
iv-b2 reads it.
