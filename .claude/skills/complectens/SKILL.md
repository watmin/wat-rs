---
name: complectens
description: Weave together. The datamancer complectēns the tests — does each layer compose only from layers above it? Does each layer carry its own proof? Or did this test attempt to one-shot a hard problem?
argument-hint: [file-path]
---

# Complectēns

> *complectēns* — Latin: embracing, encompassing, comprehending. Present active participle of *complectī*: *com-* (together) + *plectere* (to weave, plait). Past participle *complexus* → English "complex" — components woven into a comprehensible whole.

> The test should read like a story. Each layer adds one new chapter. The named helpers are the characters. The composition is the plot. The deftest at the bottom is what happens.

The third spell of the wat-rs grimoire alongside *perspicere* (see through deeply-nested types) and *vocare* (call the test to its caller). Where *perspicere* checks whether types name what they mean, and *vocare* checks whether tests verify what callers see, *complectēns* checks **whether the test was woven or thrown together**.

A test that one-shots a hard problem cannot answer YES to the four questions. A test that composes from named, individually-proven layers can. The compiler tells you the test runs. The complectēns tells you what the test PROVES — and whether, when it fails, you'll know which thread broke.

## The four questions

Every artifact in wat-rs — every type signature, every function shape, every test file — passes through these four gates before it ships. The complectēns applies them to test-file structure specifically. Run them on the deftest at the bottom of the file:

- **Obvious?** When this test fails, will I immediately know which named layer broke? If the failure narrows to a specific function name in the trace → YES. If "expected X, actual Y" with no further structure → NO (Level 1 lie).
- **Simple?** Is the deftest body 3-7 lines, composing only the topmost named helpers? If yes → YES. If 10+ sequential bindings → NO (Level 1 lie). The inherent complexity of the scenario is preserved in the helpers; the deftest body itself stays small.
- **Honest?** Do the named helpers do EXACTLY what their names promise? If `:test::put-one` actually does spawn-then-put-then-tear-down, the name is lying about its scope (Level 1). The function name's contract IS the discipline; verify it matches the body.
- **Good UX?** Can a fresh reader trace top-down with no jumping forward? Does each layer add ONE new thing over the layer above? If a helper at line 50 references a helper defined at line 200 → NO (Level 1: late dependency).

The four questions MUST run in order. Obvious + Simple + Honest must all hold before Good UX matters. A test that hides its diagnostic surface is broken regardless of how readable it might be.

## What the complectēns sees

### Monolithic deftests — the load-bearing violation

A deftest body with > ~10 let* bindings is making a structural claim it cannot support. When such a test fails, the panic message gives you NO narrowing surface. "expected X, actual Y" — the bug could be in any of 30 anonymous bindings.

Empirical bound: a deftest body with more than ~10 sequential bindings is a Level 1 lie. It claims to test a scenario but cannot diagnose which unit of work failed.

```scheme
;; ❌ Level 1 lie — monolithic. When it fails, what broke?
(:deftest :test-cache-round-trip
  (:wat::core::let*
    (((spawn ...) (...))
     ((pool ...) (...))
     ((driver ...) (...))
     ((handle ...) (...))
     ((reply-pair ...) (...))
     ((reply-tx ...) (...))
     ((reply-rx ...) (...))
     ((ack-pair ...) (...))
     ((ack-tx ...) (...))
     ((ack-rx ...) (...))
     ((_put ...) (...))
     ((results ...) (...))
     ((_join ...) (...)))
    (:wat::test::assert-eq ...)))
```

Anonymous sequential bindings are the lie. Every binding is a name that pretends something exists when actually nothing named does. The let* has erased the diagnostic surface.

### Helpers without their own deftests — a Level 2 mumble

Each named helper above the final test should have its own deftest proving it in isolation. Without per-helper deftests, the discipline degrades to "named factoring" without the proof tree. Failure trace can't bisect the layers.

```scheme
;; ❌ Level 2 mumble — helpers exist but no deftests for them.
(define :test::send-one-put ...)        ; helper, but never tested alone
(define :test::send-one-get ...)        ; helper, but never tested alone
(define :test::put-then-get ...)        ; composes; never tested alone

(deftest :final-test                    ; only this is proven
  (test::put-then-get ...))
```

When `final-test` fails, you cannot tell whether `put`, `get`, or the composition broke. The proof tree is missing.

### Multi-file stepping stones — a Level 2 mumble

Splitting incremental complexity across many files (proof_004's historical step-A.wat, step-B.wat, ..., step-E.wat) violates the "one file" rule. The reader file-hops to follow the dependency graph. The reader of a stepping-stone family wants ONE document, top-down, where each layer's existence is visible at a glance.

The user's framing (verbatim): *"i dislike that proof 004 does this across many files... it should be one files with incremental complexity.... its a linear - top down - dependency graph."*

Layered helpers + per-layer deftests + the final scenario all in one file. File-hopping defeats the diagnosability the discipline is supposed to deliver.

### Late dependency — a Level 1 lie

Test files MUST read top-down. A helper at line 200 referencing a helper at line 400 is the file lying about its dependency direction. The reader can no longer trust top-down comprehension; they must scan forward AND backward.

```scheme
;; ❌ Level 1 lie — line N defines a helper that calls one defined at line N+M.

(define :test::layer-1-helper       ; line 50
  (:test::layer-2-helper ...))      ; references something defined LATER

(define :test::layer-2-helper       ; line 200
  ...)
```

The dependency graph must run upward only. Earlier defines compose into later defines, never the reverse.

### Anonymous accidental complexity — Level 2 mumble

A scenario has irreducible inherent complexity (must spawn, must pop, must send, must recv, must drop, must join). Anonymous bindings preserve that complexity but extract the diagnostic surface — every binding is "_finish", "_join", "_put", "spawn", "pair". When something fails, NONE of these names tell you what step in the scenario actually broke.

The fix: extract the composed sequence into a NAMED helper, with a NAMED deftest. The complexity moves into the helper's body where it's bounded and inspectable; the deftest body becomes 3-7 lines of named composition. The four questions all answer YES.

## Severity levels

**Level 1 — Lies.** Tests that pretend to test a scenario but cannot localize failure. Monolithic deftests with > ~10 bindings. Late-dependency violations (forward references). Final tests with no per-layer deftests proving the helpers. Always report.

**Level 2 — Mumbles.** Helpers without their own deftests. Stepping stones split across files. Anonymous accidental complexity in shorter deftests (5-10 bindings) that could be split into 2 layers. Comments that explain WHAT a binding does instead of just naming the helper that does it. Report them.

**Level 3 — Taste.** Could the helper's body shrink one more let* binding? Should `:test::lru-spawn-then-put` be `:test::lru-put-once`? Stylistic preferences where reasonable people would choose differently. NOT findings. Note them if useful but do not count.

The complectēns converges when Level 1 and Level 2 are zero. Level 3 will always exist; the complectēns does not chase taste.

## How to cast

The spell runs in two phases:

### Phase 1 — mechanical survey

Find candidates programmatically. The eventual home for this is a sibling file in this directory: `complectens.wat`, runnable as `cat <target> | ./target/release/wat .claude/skills/complectens/complectens.wat`. The wat substrate has the primitives needed (`:wat::io::IOReader/read-line` for stdin, `:wat::core::string::contains?` / `starts-with?` / `length` / `split` / `concat`, recursive defines for line-counting + paren-balance) — what's missing is the implementation. Queued as a future arc; until it lands, **any scan tool is acceptable** — shell, awk, python — as long as the survey is reproducible and reports concrete `(file, line, deftest-name, body-line-count)` tuples.

The mechanical pass DOES NOT judge; it only finds candidates.

What to count:

1. **deftest body line count** — paren-balanced extraction from each `(:wat::test::deftest ...)` / `(:wat::test::deftest-hermetic ...)` / `(:<alias> ...)` from a `make-deftest` factory. The line count of the body is the proxy for binding count. Empirical thresholds: > 30 lines = suspect; > 50 lines = likely Level 1; > 100 lines = definite Level 1. Sort findings by body line-count descending.
2. **let\* binding-count per body** — sharper than line count. Count the entries in each top-level `(:wat::core::let* ((...) ...) body)` form within a deftest body. > 10 entries → Level 1.
3. **forward references** — for each `:wat::core::define` of a helper, grep for references to helpers / aliases NOT yet defined above it in the same file. Any forward reference → Level 1.
4. **file count for stepping-stone families** — `find` for groups of `step-*.wat` / `proof_*.wat` files in the same directory. Multi-file stepping stones → Level 2 candidate (the discipline says ONE file).
5. **helpers without deftests** — for each `:wat::core::define` in a `make-deftest` prelude or at file top-level, search for a sibling `(:deftest ...)` referencing it. Missing → Level 2 candidate.

Output of phase 1 is a structured findings list: `(file, line, deftest-name OR helper-name, body-line-count, severity-candidate)`.

For Rust integration test files (`tests/wat_*.rs`) that embed wat source as strings, the same rules apply to the embedded scenarios: short string + named helpers in surrounding wat-test files, NOT a 200-line embedded let*.

### Phase 2 — judgment

Read each candidate. Apply the four questions. Distinguish:

- A 30-line deftest body might be inherently complex (e.g., a long `match` expression on a complex enum) and NOT a Level 1 lie. The line count is a candidate flag, not a verdict.
- A helper without a sibling deftest might be a single-use detail that doesn't need its own proof (e.g., a thin wrapper used in exactly one place). Level 3 taste.
- A forward reference might be a macro auto-recursion (e.g., `make-deftest` referencing the alias it just registered). Not a real violation.

Phase 2 is where the LLM (or a careful human) reads the candidate and decides Level 1 / Level 2 / Level 3. Phase 1's mechanical scan over-flags by design — it's the funnel, not the verdict.

### Why two phases

The mechanical pass is reproducible: any agent re-running it finds the same candidates. The judgment pass is the load-bearing intelligence: applying the four questions to actual code requires reading the bindings' semantics, not just counting them.

Future work: when `wat-lint` matures, phase 1 becomes a single command that emits structured EDN/JSON findings. Phase 2 then iterates over those findings, fetching context and rendering verdicts. The spell sits atop both.

## Edge cases

Surfaced by the HologramCacheService calibration sweep (2026-05-03) — file at `docs/arc/2026/05/130-cache-services-pair-by-index/CALIBRATION-HOLOGRAM-SCORE.md`.

### Mixed-outcome files — the two-prelude pattern

A test file can mix deftests that PASS cleanly with deftests that `:should-panic`. The naïve approach — ONE `make-deftest` factory whose prelude includes ALL helpers — makes EVERY deftest in the file fire whatever check the prelude triggers. Step1 + step2 (intended to pass cleanly) regress to `:should-panic` outcomes; step3-6 (intended to `:should-panic`) keep firing.

The fix: TWO `make-deftest` factories in the same file. One per outcome class.

```scheme
;; Factory 1 — Layer 0 helpers; pure lifecycle; no channel-pair patterns; clean pass.
(:wat::test::make-deftest :deftest-hermetic
  (
   (:wat::core::define :test::layer-0a ...)   ;; spawn + drop + join
   (:wat::core::define :test::layer-0b ...)   ;; receive a value end-to-end
  ))

;; Factory 2 — Layer 1+ helpers; service-aware; includes channel-pair patterns
;; that trigger arc 126 at freeze; all deftests using this prelude :should-panic.
(:wat::test::make-deftest :deftest-service
  (
   (:wat::core::define :test::layer-1-put ...)        ;; helper-verb call site
   (:wat::core::define :test::layer-1-get ...)
   (:wat::core::define :test::layer-2-put-then-get ...)
  ))

;; Clean-pass deftests use factory 1.
(:deftest-hermetic :test::test-layer-0a (:test::layer-0a))

;; Should-panic deftests use factory 2.
(:wat::test::should-panic "channel-pair-deadlock")
(:deftest-service :test::test-layer-1-put (:test::layer-1-put))
```

The two-prelude split is a one-file solution to the mixed-outcome constraint. Same discipline; just two namespaces of helpers within the same file.

### Cross-function tracing — DO NOT factor `make-bounded-channel` into a helper

Arc 126's `channel-pair-deadlock` walker traces Sender / Receiver arguments back through `(:wat::core::first|second pair)` chains to a `make-bounded-channel` anchor. **The trace stops at function-call boundaries.** Factoring `(make-bounded-channel ...)` + `(first pair)` + `(second pair)` into a helper that returns the (Sender, Receiver) tuple SILENCES the check — the same code shape that fires arc 126 inline does NOT fire when wrapped.

```scheme
;; ❌ Tempting: clean abstraction. But arc 126 stops tracing here;
;; if a deadlock pattern existed inline, it's hidden now.
(:wat::core::define
  (:test::make-ack-channel -> :(wat::lru::PutAckTx,wat::lru::PutAckRx))
  (:wat::core::let*
    (((pair :wat::lru::PutAckChannel) (:wat::kernel::make-bounded-channel :wat::core::unit 1)))
    (:wat::core::Tuple (:wat::core::first pair) (:wat::core::second pair))))

;; ✓ Correct: keep the make-bounded-channel + first/second sequence INLINE in the
;; helper that uses both halves. Arc 126's trace can follow the chain.
(:wat::core::define
  (:test::send-put-with-ack (req-tx :ReqTx) (k :K) (v :V) -> :wat::core::unit)
  (:wat::core::let*
    (((ack-pair :wat::lru::PutAckChannel) (:wat::kernel::make-bounded-channel :wat::core::unit 1))
     ((ack-tx :wat::lru::PutAckTx) (:wat::core::first ack-pair))
     ((ack-rx :wat::lru::PutAckRx) (:wat::core::second ack-pair))
     ((_put :wat::core::unit) (:wat::lru::put req-tx ack-tx ack-rx ...)))
    ()))
```

When extracting helpers, leave `make-bounded-channel` allocations IN the helper that calls the helper-verb. Don't abstract the channel-allocation into its own function — abstract the WHOLE workload (allocate + call + drop) instead.

### `HandlePool::finish` requires pop-before-finish

A `make-deftest` lifecycle helper that demonstrates spawn-and-shutdown without doing any actual work must STILL pop a handle from the pool before calling `HandlePool::finish`. The substrate's runtime check raises "orphaned handles" if the pool is finished with un-popped slots. The lifecycle helper:

```scheme
(:wat::core::define
  (:test::lifecycle-spawn-and-shutdown -> :wat::core::unit)
  (:wat::core::let*
    (((driver ...)
      (:wat::core::let*
        (((spawn ...) (:wat::lru::spawn 1 ...))
         ((pool ...) (:wat::core::first spawn))
         ((d ...) (:wat::core::second spawn))
         ((req-tx ...) (:wat::kernel::HandlePool::pop pool))   ;; ← required
         ((_finish :unit) (:wat::kernel::HandlePool::finish pool)))
        d))
     ((_join ...) (:wat::kernel::Thread/join-result driver)))
    ()))
```

The pop-then-drop pattern is the standard form even when no work happens — the substrate's pool-accounting requires every slot to be visited.

### Non-unit Thread output requires recv-before-join

For services with `Thread<I, O>` output where O is NON-UNIT (the driver loop calls `(:wat::kernel::send out final-state)` before returning), the lifecycle helper MUST `recv` from `Thread/output` BEFORE calling `Thread/join-result`. Dropping the receiver before the send completes panics the driver with "out disconnected".

```scheme
(:wat::core::define
  (:test::svc-spawn-and-shutdown -> :wat::core::unit)
  (:wat::core::let*
    (((driver-and-final ...)
      (:wat::core::let*
        (((spawn ...) (:svc::Service 1))
         ((pool ...) (:wat::core::first spawn))
         ((d :wat::kernel::Thread<wat::core::unit,svc::State>) (:wat::core::second spawn))
         ((final-rx :wat::kernel::Receiver<svc::State>) (:wat::kernel::Thread/output d))
         ((req-tx ...) (:wat::kernel::HandlePool::pop pool))
         ((_finish :unit) (:wat::kernel::HandlePool::finish pool))
         ((_final-state :svc::State)
          (:wat::core::Option/expect -> :svc::State
            (:wat::core::Result/expect -> :wat::core::Option<svc::State>
              (:wat::kernel::recv final-rx)
              "spawn-and-shutdown: thread died before sending final-state")
            "spawn-and-shutdown: thread output closed without sending")))
        d))
     ((_join ...) (:wat::kernel::Thread/join-result driver-and-final)))
    ()))
```

Thread<unit, unit> services (LRU, HCS) DON'T have this issue — the driver doesn't call `send` because there's nothing to send. Only services where the spawned function's output type is non-unit need the drain.

### Arc 126 fires on CALL SITES, not just `make-bounded-channel` definitions

The previous edge case warns against factoring `make-bounded-channel` into a helper. The full picture: arc 126's check fires on any function-call site that passes both halves of a channel pair as arguments — INCLUDING calls to user-defined helpers whose signatures take both halves.

```scheme
;; ❌ Arc 126 fires HERE — at the call to send-ack-wait — because both
;; ack-tx and ack-rx are passed in the same call.
(:wat::core::define
  (:test::send-ack-wait
    (req-tx :svc::ReqTx)
    (ack-tx :svc::AckReplyTx)
    (ack-rx :svc::AckReplyRx)
    -> :wat::core::unit)
  ...)

;; In a deftest body or another helper:
(:test::send-ack-wait req-tx ack-tx ack-rx)   ;; ← arc 126 fires here
```

The corollary: do NOT factor a CALL SEQUENCE that uses both halves of a channel pair into a helper that accepts both halves as parameters. Keep `(:wat::kernel::send req-tx ...)` and `(:wat::kernel::recv ack-rx)` as SEPARATE inline calls in the scenario body. Arc 126 only fires when one CALL passes both halves; separate sequential calls each pass one half.

### Embedded literals — visual line count vs OUTER LOGICAL BINDINGS

Some deftest bodies contain LITERALS that are part of the test's data, not part of its composition logic. These literals are inherently irreducible:

- `(:wat::test::run-hermetic-ast (:wat::test::program ...))` — the embedded program AST runs in a forked subprocess; it can't reference the outer prelude's helpers, so it must be self-contained.
- `(:wat::lru::HologramCacheService::MetricsCadence/new gate (:wat::core::lambda ...))` — a cadence's tick lambda; the lambda is data passed to the factory, not composition.
- `(:wat::core::lambda ((tx :Sender) ...) ...)` as a dispatcher / translator / reporter argument — the lambda is the test fixture, not the test logic.

The mechanical phase's `>30 lines = suspect` heuristic over-flags any deftest containing such literals. Phase-2 judgment counts the **OUTER LOGICAL BINDINGS** of the deftest's let*, NOT the total visual line count. A test whose outer let* has 5 bindings is well-shaped, even if visual line count is 80+.

When extracting helpers FOR an embedded-literal test, target the OUTER scaffolding:
- Helpers that SETUP the literal (build it from primitive parts; e.g., a `make-null-cadence` helper).
- Helpers that PROCESS the result (extract stdout/stderr, assert on contents).
- Helpers that COMPOSE multiple invocations (call the scenario, drain the channel, assert).

Helpers that try to share logic INSIDE an embedded program's body, or inside an embedded lambda, are not possible without arc-094-style AST quasiquote builders — out of scope for the discipline.

The simpler rule: when one of the deftest's let* bindings has an RHS that EVALUATES to data (an AST, a lambda, a closure, a struct literal), that RHS is the test's fixture and is exempt from the line-count metric. The OUTER let*'s binding count remains the proxy for composition complexity.

## Reference

- `docs/arc/2026/05/130-cache-services-pair-by-index/REALIZATIONS.md` — the canonical doc that named the discipline.
- `docs/arc/2026/05/130-cache-services-pair-by-index/complected-2026-05-02/` — the failed sweep preserved as the calibration set: substrate + test files frozen at the moment the discipline broke, plus a README explaining what each violation looks like.
- `crates/wat-lru/wat-tests/lru/CacheService.wat` — worked demonstration: layered helpers + per-layer deftests + a 6-line final test body.
- `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat` — six-step layered test using `make-deftest` prelude with helper functions.

## Sibling spells

- `/perspicere` — see through deeply-nested types; suggests typealiases that name the noun the depth was hiding.
- `/vocare` — call the test to its caller; verify that the test exercises the caller's interface, not the implementation.
- `/complectens` — this spell. Weave the test from named, proven layers.

## The principle

The test file is a top-down dependency graph in ONE file. Each function does ONE thing. Each layer composes from layers above. Each layer carries its own deftest. The final deftest body is short BECAUSE the layers exist. The failure trace IS the dependency graph. When the test fails, the broken layer's name is the diagnostic.

A test that one-shots a hard problem is a discipline violation. The complectēns finds where the weave fell apart.
