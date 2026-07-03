# NOTE — the test-cleanup campaign, revealed by rete's work

**Provenance.** This test-hygiene cleanup was not planned; it fell out of the rete work, one thread pulling
the next:

1. Arc 300 built its wat→EDN conversion as a **real rete consumer** (`PORTA PORTAM APERIT`). That consumer
   exposed a fixpoint truth-maintenance flaw the single-pass parity benchmarks never touched (`ALIVS ARGVIT`
   / 278 R18) → we pivoted to 278 to fix negation in the kernel.
2. Building the native-stratification differential probe, the builder caught a **dead `:user::main`** and an
   **inlined-wat world** in the probe — and asked how pervasive each was. The disk answered: dead mains were
   cargo-culted across ~96 fixtures (swept, `8372404`), and inline wat is in ~half the test suite.
3. The `no-inlined-wat` gate turned out to enforce only **world-building** (`startup_from_source`), leaving
   the **driver / query** inline wat (`let run = format!("(:wat…")`) entirely uncaught — and it is **blind to
   the faithful-Clojure surface** 300 is about to ship (`(wat.core/defn …)` has no `(:` prefix; verified the
   `(:` needle misses it, and 8 test files already carry faithful forms from the arc-251 dual-surface work).

**What we are building.** A **reader-based inline-wat gate**: detect wat by feeding string literals to wat's
own reader (`parse_one`), not by a surface-specific regex — so the gate is surface-agnostic and follows the
one reader through 300's convert-then-retire with zero maintenance. It dogfoods 300's thesis (`VNVS LECTOR NE
DIVIDANTVR`) in the enforcement itself: the gate that keeps wat in `.wat` files is written in "wat is EDN, one
reader." Then the classified sweep (extract-to-`.wat` OR earn a `// rune:lint(no-inlined-wat) — <reason>`)
drives the flagged flood to zero.

**Scope discipline.** This is 278 test-hygiene, revealed by rete, tracked here so the provenance is not lost.
It is orthogonal to the rete engine itself (negation now behaves, `bdbf3021`) — a cleanup the rete work
surfaced, done in the same arc rather than deferred.
