# BRIEF — Arc 283.1: harden rename-keyword-prefix (type-args + boundary guard)

You are a single-hop sonnet executor in `/home/watmin/work/holon/wat-rs`. **Do NOT spawn sub-agents.
Do NOT run `git`.** Build, run the named tests, report. The orchestrator weighs independently.

## The work (one paragraph)

`fix::rename-keyword-prefix` is start-anchored: it MISSES a symbol used as a type-argument
(`Vector<t::Old>`) and CORRUPTS prefix-siblings (`:t::OldExtra` → `:t::NewExtra`, no boundary check).
Replace its leaf logic with a boundary-aware whole-name rewrite that renames every VALID occurrence of
the colon-stripped prefix and nothing else.

## The contract — implement EXACTLY the DESIGN

Read **`docs/arc/2026/06/283-source-file-lift/DESIGN-283.1-rename-typearg.md` § "The fix"** and implement
its boundary rule verbatim: colon-strip the prefixes; a match at index `i` is valid iff present ∧
left-valid ∧ right-valid, where **left-valid** = `(i==1 && char-at(name,0)==":")` OR `char-at(name,i-1) ∈ {"<",",",space}`, and **right-valid** = at-end OR `char-at(name, i+len(old-bare))` ∉ `[a-zA-Z0-9_-]`. A
`rename-in-name` char-walk (recursive over the index, using `string::subs` + `string::length`) produces
the new name; if it differs from the old, emit `(off, length(name), new-name)`.

## Read in order (the rooms)

1. `docs/arc/2026/06/283-source-file-lift/DESIGN-283.1-rename-typearg.md` — THE SPEC (boundary rule + char-walk sketch).
2. `wat/fix.wat:544-575` — `rename-prefix-edits` (the leaf logic to REPLACE) — keep its structural-recurse
   branch (`:555-557`); rewrite the keyword-leaf branch (`:559-573`) to the boundary-aware whole-name rule.
3. `wat/fix.wat:163-200` — `fix-text-offset-of` / existing edit shape (`Tuple off old-len new-text`) — your
   emitted edit is `(off, length(name), new-name)` (replace the WHOLE keyword token, not just the prefix).
4. `wat/fix.wat:577-591` — `rename-keyword-prefix` (the public fn) — unchanged; it just calls your rewired edits.
5. `wat/core.wat` — confirm `string::subs`, `string::length`, `string::concat`, `string::contains?`,
   `HashSet`/`contains?` are available (they are). Use `subs name i (+ i 1)` for a single char.
6. Find the existing fix-tool deftests (grep `rename-keyword-prefix` in `wat-tests/`); add a deftest there
   for the type-arg + boundary cases. If none exists, add a small one in the most fitting `wat-tests/` file.
7. `tests/probe_arc283_1_rename_typearg.rs` — remove the `#[ignore = "arc 283.1 …"]`.

## Implementation notes

- `is-ident-char?` over a single-char String: a `HashSet` of the 64 identifier chars is verbose — instead
  use `(:wat::core::string::contains? "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-" c)`
  (c is the one-char string) — true = identifier-continuation = right-INVALID.
- The head case `:t::Old` → the colon-stripped `t::Old` sits at index 1 (after the leading `:`); left-valid
  via `(i==1 && char-at name 0 == ":")`. Confirm with the probe's `-> :t::New` + `:t::New/make` assertions.
- Emit ONE edit per changed keyword (whole-token replace), so overlapping/multiple in-name occurrences are
  all handled inside `rename-in-name` (no multi-edit-per-token bookkeeping).

## STOP triggers (halt + report, do not improvise)
1. If `:t::OldExtra` is still corrupted (→ `:t::NewExtra`) — the boundary guard is wrong; STOP, report.
2. If a head-anchored rename regresses (`:t::Old` / `:t::Old/make` no longer rename) — STOP, report.
3. If `:other::t::Old` (an unrelated symbol ending in the path) gets touched in your own testing — STOP;
   left-valid must exclude `::`-preceded matches.

## Blast radius
`wat/fix.wat` (rename-prefix-edits rewrite + helpers), a `wat-tests/` deftest, un-ignore the probe.
No Rust changes. No git. (Do NOT attempt the 283 SourceFile lift here — that's the NEXT step, re-run by
the orchestrator on top of this hardened tool.)

## Verify (run these, paste output verbatim)
```
cargo test --release -p wat --test probe_arc283_1_rename_typearg     # 1/1 GREEN (type-arg + boundary)
cargo test --release --test test 2>&1 | grep "test result"          # deftest binary: 261 passed / 1 failed (was 260, +1 new; 1 = run_string_entry_direct)
cargo test --release --test test_stdlib_load_order 2>&1 | grep result # deporder: 1 passed / 0 failed
cargo test --release -p wat --lib 2>&1 | grep "test result"          # lib: 929 passed / 36 failed (UNCHANGED)
```
Plus, if an arc-269 rename probe/test exists, run it (must stay GREEN — start-anchored renames still work).
Report: the rewritten `rename-prefix-edits` + helpers, the command outputs verbatim, any delta. Do not
claim green you did not see.
