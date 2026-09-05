# BRIEF — `Span` equality becomes honest

Design: `[[DESIGN-STONE-span-equality-becomes-honest]]` (same dir). **Read it first — the census is
already run and its result is in there.** You are not discovering the blast radius; you are applying
a change whose blast radius is two named tests. Anchor: `/home/john/work/holon/wat-rs`; `pwd` first.

## THE CHANGE

```
crates/wat-reader/src/span.rs
   Span::eq            compare file/line/col/end       (today: `fn eq(&self, _) -> bool { true }`)
   Pos                 gains PartialEq + Eq            (it has NEITHER — positions have never been
                                                        comparable, which is why the honest eq does
                                                        not compile without it)
   Hash                UNCHANGED — stays a no-op. See the design; the Rust contract is satisfied.
   the module doc      its "# Equality and hashing" section must be rewritten. It currently claims
                       "it never compares Span values for equality" — FALSE at three sites.

crates/wat-reader/src/ast.rs
   WatAST              drop `PartialEq` from the derive; add a manual impl over the 14 variants:
                       compare structure, SKIP the span. This is where the real requirement lives
                       ("synthetic == parsed regardless of position") and the impl should say so.

tests/diagnostics/probe_arc243_stone3_typeerror_pattern_a.rs   (~line 80-107)
tests/diagnostics/probe_arc243_stone6_checkerror_pattern_a.rs  (~line 111)
   Bind ONE span and use it on both sides. Today each tuple calls `rust_caller_span!()` TWICE,
   at different lines, so the "expected" span was never the stored one.
```

## THE TWO TESTS — what they must become

They currently prove `err.span()` does not panic. After the change they must prove what their own
doc comment claims: *"Universal single-path access — works for EVERY TypeError regardless of which
kind variant."* That means the span the error was CONSTRUCTED with is the span `.span()` returns.

⛔ **Do not "fix" them by comparing `Debug` strings or by asserting only the line number.** The
whole stone is that span equality means something; a test that routes around it defeats the stone.

## STOP TRIGGERS

**STOP-1 — more than two tests fail.** The census measured exactly two (`5170 run, 5168 passed, 2
failed`). If your floor-adjacent runs surface a third, STOP and report it — that means the census
was wrong and I want to know before it is papered over.

**STOP-2 — `Hash` stays a no-op.** If making eq honest seems to require changing `Hash`, STOP.
`a == b ⟹ hash(a) == hash(b)` still holds (unequal values may collide); `WatAST`'s hash must stay
position-independent for `canonical_edn_wat`.

**STOP-3 — `WatAST` equality must stay position-blind.** A parsed AST and a synthetic one with the
same structure must still compare equal. If the manual impl cannot preserve that, STOP — that is
the requirement the old blindness existed for and it is not negotiable.

**STOP-4 — a red is a red.** Do NOT re-run. Capture the whole block, name the arm, report.

## What you run, and what you do not

FOREGROUND: `cargo build --release`, `cargo test --release -p wat-reader`,
`cargo nextest run --release -E 'test(probe_arc243)'`, and
`cargo nextest run --release --test lint` before you yield. **No full floor** — mine, centrally,
and it includes a doctest stage. No commit/push/stash/revert.

## Report

The two tests' before/after · the rewritten `span.rs` module-doc section · confirmation that
`WatAST` synthetic-vs-parsed equality still holds (name the test that proves it) · anything that
surprised you.
