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

## The four questions

Run them on the deftest at the bottom of the file:

- **Obvious?** When this fails, will I know which named layer broke? If no → Level 1.
- **Simple?** Is the body 3-7 lines? If yes → likely good. If 10+ → Level 1.
- **Honest?** Do the named helpers do EXACTLY what their names promise? If a helper is named `put-one` but actually does spawn + put + tear-down, the name is lying.
- **Good UX?** Can a fresh reader trace top-down with no jumping? Does each layer add ONE new thing? If no → Level 2.

## When scanning wat files

Every `.wat` test file under `wat-tests/` or `crates/*/wat-tests/` is a candidate. Look for:

1. **deftest body line count** — `wc` the let* body of each `:wat::test::deftest` (or `:deftest` alias). > 10 bindings → Level 1.
2. **per-helper deftests** — for each `:wat::core::define` defined as a helper at file scope OR in a `make-deftest` prelude, search for a sibling `:wat::test::deftest` that exercises just that helper. Missing → Level 2.
3. **dependency direction** — does each helper reference only helpers defined ABOVE it? Use `grep -n` for forward references.
4. **file count** — does the test scenario span multiple files? If so, was it the user's intent (separate concerns) or accident (proof_004 stepping stones)? Multi-file stepping stones → Level 2.

For Rust integration test files (`tests/wat_*.rs`) that embed wat source as strings, the same rules apply to the embedded scenarios: short string + named helpers in surrounding wat-test files, NOT a 200-line embedded let*.

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
