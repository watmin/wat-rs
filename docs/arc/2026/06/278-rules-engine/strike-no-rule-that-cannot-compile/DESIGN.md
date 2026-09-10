# DESIGN — no rule that cannot compile

## Why

`every_wat_scripts_file_loads` walks all 477 `.wat` under `wat-scripts/` and runs **validate**. The
four-axis fence/`:then` gate — `pure ∧ det ∧ total ∧ rete-primitive`, `wat/rete/compile.wat:463` —
lives at rete **compile**, which nothing in the tree ever reaches. Measured at `d7aa7c9ae`
(`tests/lint/rete-compile-census.sh`, re-runnable): **136 files declare rete rules; 125 compile;
11 cannot.** Full finding: `../the-fence-says-what-the-clause-cannot/FINDING-loading-is-not-compiling.md`.

⭐ The builder's ruling, and it is the whole frame: *"i never wish any wat user to ever learn
invalid behavior, pathological statements or any other unwanted patterns ever again."*

## What it delivers

**The gate is the deliverable.** The deletions and repairs are what make it green — deleting eleven
files without it is cutting the stem, and the class regrows the next time someone writes a fence.

1. A lint gate: every corpus file that declares rete rules must **compile** them.
2. Nine files deleted.
3. Three user-fn fences repaired to rete primitives, in two recorded migrations.
4. One false citation in `src/runtime.rs` re-grounded.

## ⛔ THE ONE CONTRACT DECISION

> **The gate ships with ZERO declared exemption categories. There is no `red-by-design` rune for
> "this rule is meant not to compile."**

After the nine deletions the corpus has **no** legitimate exception — every refusal those files
demonstrated is already asserted, with `assert_eq!` on the exact message, by
`tests/rete/probe_fence_names_the_head.rs` (mutation-proven, four fixtures) and
`wat_scripts_grid_axes_live.rs`.

The sibling gate states the discipline on its own category set: *"a CLOSED set. A second member is
a deliberate act needing its own discriminating question, written where a reader will meet it."*
This goes one rung further — **zero members until one is earned.**

⚠ **The rejected alternative, named so it is not re-litigated:** shipping a `red-by-design`
category "for future negatives." An exemption with no members is an invitation to launder the next
failure instead of fixing it, and this strike exists because eleven files were laundered by
silence. If a genuine negative ever needs to live here, minting the first category is a deliberate
act that must carry its own argument — which is exactly the bar the sibling gate sets.

## The algorithm — and it needs no binary, no `main`, no temp file

For each file declaring rules: read the source, append a synthesized zero-arg
`(:wat::core::defn :census::run [] …)` that calls `compile-all` over `collect-rules` for each
declared namespace, `startup_from_source` the combined text, look the symbol up, `apply_function`.

⭐ This is strictly better than `rete-compile-census.sh`'s method and the difference matters:
the script drives the CLI binary, so it must first neutralise the file's own `:user::main` to stop
a codemod writing files. `startup_from_source` **never evals `main`**, so the side-effect hazard
does not exist and the `sed` disappears. It is also the driver the sibling gate
`wat_scripts_fixes_load.rs` already uses — same question, one step further.

## Files

| file | change |
|---|---|
| `tests/lint/` (new) | the gate |
| `wat-scripts/fixes/to-faithful-clojure-net.wat` | 2 fences: `:fix::has-ns?`, `:fix::type-shaped?` |
| `wat-scripts/fixes/to-faithful-clojure-rete.wat` | 1 fence: `:fix::head-keyword-str?` |
| `src/runtime.rs:5371` | the citation |
| 9 paths (BRIEF, enumerated) | deleted |

## The repair, and why it needs no parked decision

`:wat::rete::core::String/contains?` already exists with `pure: true, deterministic: true,
total: true` — all four bars. `g4-namespaced` reached for `(:fix::has-ns? ?name)`, a user-fn
wrapper around the **partial** `:wat::core::string::contains?`, when the legal primitive was in the
table the whole time. Inlining is legal **today**.

⚠ Whether wat should *also* admit a bare user-fn call in a fence — the form Clara allows and
`expr_is_provably_boolean` refuses — stays parked in
`../the-fence-says-what-the-clause-cannot/DESIGN-widen-the-clause-then-refuse-the-fence.md`. The
repair does not depend on it and must not pre-empt it.

## Out of scope = REJECTED

- **Making `:fix::has-ns?` itself total.** It would change a general function's contract for a
  rete-only reason. Inline at the fence; leave the function alone.
- **The parked expressivity design.** These two codemods are *evidence for* it.
- **`.wat.bad` renames.** That extension is defined by a LOAD verdict and all eleven files load;
  a rename reds `every_wat_bad_fixture_actually_fails` by construction. Deletion is the ruling.
- **Any corpus file outside the eleven enumerated in the BRIEF.**
