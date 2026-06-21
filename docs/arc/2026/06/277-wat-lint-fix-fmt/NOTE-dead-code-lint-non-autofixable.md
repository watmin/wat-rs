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

So the builder wants a **self-retiring** annotation — one whose single job is to **raise the moment you
use the thing**:
- **used-while-annotated → ERROR** — "`:my::ns/foo` is marked expected-dead but is referenced at
  `<site>`; remove the annotation, it's alive now." Forces removal the instant the reader lands.

That one direction IS the feature. Builder (2026-06-21): *"the conditional trigger feels completely
unnecessary — i just wanted something to raise if i use the thing."* **No deadline / "still-dead-past-N"
machinery** — that was an over-elaboration, explicitly rejected. (Eternal-cruft dead code is already
caught by the plain report-only dead-code lint above; this annotation's one job is raise-on-use.)

## Enforceability (wat-side)
**used-while-annotated → ERROR** is **fully checkable** from the call-graph the dead-code lint already
needs (255 registry = the name universe; resolver = the referenced set): for an `expect-dead` item,
*referenced* flips from OK to a finding. Same machinery as the plain lint, inverted. That is the whole rule.

## Rust-side — Rust already ships exactly this
**Prior-art collision, noted honestly: Rust 1.81 stabilized `#[expect(lint)]`** (RFC 2383).
`#[expect(dead_code)]` IS raise-on-use: silent while the item is genuinely dead, but emits an
**unfulfilled-expectation** warning (an ERROR under `-D warnings`) the moment the item is referenced.
Since raise-on-use is the whole feature, **Rust's `#[expect]` is the complete Rust side** — no
deadline-gate, nothing to add.

**DONE (2026-06-21):** the iv-b1 transient allows in `src/intrinsic/mod.rs` (`IntrinsicEntry.{args,
examples,deprecated,see}` + `ExampleSubmission`) were switched `#[allow(dead_code)]` → `#[expect(dead_code)]`
(toolchain 1.93.0). They are silent now (genuinely dead) and the compiler will say "remove me" the
instant iv-b2's `verify-examples` seam reads them. Gotcha learned: do NOT let a `#[cfg(test)]` test read
an `#[expect(dead_code)]` field — that counts as a use and trips the expectation under `cargo test`; the
real runtime reader (iv-b2) is what should retire it, so the premature in-src confirmation test was removed.

## Scope marker
Wat-side: a queued lint rule (`expect-dead`, raise-on-use), built on the same 255-registry + call-graph
machinery as the plain dead-code lint above. Rust-side: `#[expect(dead_code)]` — adopted for transient
allows, complete. The live instance (iv-b1's carry fields) self-retires when iv-b2 reads it.
