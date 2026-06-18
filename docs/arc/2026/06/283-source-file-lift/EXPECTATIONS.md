# EXPECTATIONS — Arc 283 (weigh on the orchestrator's own build)

A behavior-preserving lift + rename. The proof is: the new home works, the old name is GONE, and
nothing else moved.

| what | command | expected |
|---|---|---|
| lift gate | `cargo test --release -p wat --test probe_arc283_source_file_lift` | 1 passed / 0 failed (`File/path` → `"t.wat"`) |
| zero survivors | `grep -rn ":wat::deporder::SourceFile" wat/ tests/ src/ \| wc -l` | **0** (rename total) |
| deporder gate | `cargo test --release --test test_stdlib_load_order 2>&1 \| grep result` | 1 passed / 0 failed (source.wat precedes deporder) |
| lib floor | `cargo test --release -p wat --lib -- --test-threads=1 2>&1 \| grep "test result"` | 929 passed / 36 failed (UNCHANGED) |
| deftest floor | `cargo test --release --test test 2>&1 \| grep "test result"` | 260 passed / 1 failed (UNCHANGED) |
| nursery floor | `cargo test --release -p wat --test nursery -- --test-threads=1 2>&1 \| grep "test result"` | 893 passed / 4 failed (UNCHANGED) |
| lint probes | `…probe_arc277_lint_if_ladder` + `…probe_arc277_1b_ladder_autofix` | GREEN (fixtures renamed) |

Runtime prediction: 15–25 min (mechanical; the dogfood does the heavy lifting).

## Trap-doors named

- **Dogfood over-reach** — `rename-keyword-prefix` is comment-faithful by design, but READ the diff: only
  `:wat::deporder::SourceFile`→`:wat::source::File` may change. A prefix rename catches accessors
  (`SourceFile/path`→`File/path`) because they share the prefix — that's correct, confirm it happened.
- **A survivor** — the grep MUST be 0. Likely miss spots: a comment mentioning the old name (the dogfood
  WILL rewrite comments faithfully, so even comments flip — good), or a `.rs` fixture not in the manual-3
  (re-grep `tests/` to be sure only those 3 had it).
- **Load order** — source.wat must be registered BEFORE deporder.wat in stdlib.rs, or deporder fails to
  resolve `:wat::source::File` → the deporder gate (which itself uses File) and lib tests break.
- **The driver leaks** — `wat/_rename_sourcefile.wat` must be DELETED (it's a one-shot; if committed it'd
  be a stray + deporder/lint would try to load… no, it's not registered, but it'd be a dirty artifact).
- **Behavior change** — any moved floor count means the lift wasn't pure. The rename is a pure symbol
  swap; the def move is pure relocation; neither changes semantics.

## Definition of done

Lift gate green; zero survivors; all floors byte-identical; `:wat::source::File` defined in
`wat/source.wat` (loaded before deporder); the 3 Rust fixtures + `.wat` corpus renamed; the dogfood
driver deleted. The `.wat` renames came from `rename-keyword-prefix` (the toolchain renamed its own
symbol — the dogfood), not hand-edits.
