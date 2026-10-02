# AMEND 2 — STONE 255.83: the committed tree is the floor's tree, and the wat-side identities

**Drawn 2026-10-02.** **Executor: grok via pulsare, working solo** (it runs the floor). Continues at `e20099ad5`. Commit
locally on `main`; **do not push**.

## 1. The orchestrator's floor on the committed tree is red

`.floor/2026-10-02T21-27-53Z` at `e20099ad5`: **6387 passed, 1 failed**. Not re-run. The block, verbatim:

```
thread 'macros::tests::whole_body_templates_in_the_corpus_are_pure' panicked at src/macros/tests.rs:1800:5:
assertion `left == right` failed
  left: ["tests/macros/probe_arc249_macro_engine_impure.wat.bad:1: quasiquote template purity check failed at definition of :my::impure-cu: …",
         "tests/resolve/probe_arc255_83_qq_whole_kw.wat.bad:1: quasiquote template purity check failed at definition of :user::m: keyword head `:wat::kernel::println` refused …",
         "tests/resolve/probe_arc255_83_qq_whole_sym.wat.bad:1: … same …"]
 right: ["tests/macros/probe_arc249_macro_engine_impure.wat.bad:1: …"]
```

It passed on your `.floor/2026-10-02T21-14-00Z`. The two fixtures were on disk then (written 14:12) but committed at
14:26, after that floor: the scan enumerates tracked files, so your green was a tree whose tracked set differed from the
commit. **A floor proves the tree it ran on, tracked set included:** commit (or `git add`) everything before the floor
you report.

**Cure:** the two fixtures are deliberate negative proofs, exactly like the arc-249 one the test already expects. Add them
to the expected list (or make the test derive its expected set from a marker the fixtures carry, if that is the
existing idiom); do not weaken the scan.

## 2. The wat-side identities (the precondition for 5c)

Your § 6 left several `ast-name` comparisons on text, on the ground that routing them through
`:wat::core::canonical-identity` "would also make a dotted keyword such as `:wat.core/if` match". **That is the point:**
one identity in either spelling is one head (the brief's whole subject). Markers that are not identities (`"->"`, `"<-"`,
`"&"`, `":-"`, `:from`, `:locus`, `:max-buffer-bytes`, `"T"`) correctly stay. These are **identities** and will miss a
symbol-spelled head once 5c converts the stdlib and corpus. Route each through `:wat::core::canonical-identity`, each
with a keyword/symbol differential pair as a driven test:

- `wat/Record.wat:153, 249` — `:wat::core::unquote-splicing`
- `wat/core.wat:643` — `:wat::core::agg-positional`
- `wat/rete/oracle/stratify.wat:233` — `:wat::rete::exists`
- `wat/lint.wat:143` — `:wat::core::=`
- `wat/fix.wat:81` (`:wat::core::if`), `:971`/`:972` (`:wat::enum::Pure`/`Impure`), `:1384` (`:wat::core::first`),
  `:1391` (`:wat::core::drop`)

`wat/fix.wat:1064, 1120` compare against a codemod's own `old` operand: say whether that operand can arrive in either
spelling, and route it if so.

## 3. An instrument blind spot (record it)

The ledger's literal walker cannot see `== Some(":wat::…")`; you rewrote new code into a shape it sees. Name in the SCORE
which comparison shapes the walker misses (at least `== Some(literal)`), so the gap is on the record for the ledger's
owner. Do not lower its `> 300` sanity floor.

## Gates

`git status` clean before the floor; then the floor (in the foreground, nothing else running), clippy, `census.sh --diff`.
STOPs as before. A STOP means STOP. Append to the SCORE, commit, **do not push**.
