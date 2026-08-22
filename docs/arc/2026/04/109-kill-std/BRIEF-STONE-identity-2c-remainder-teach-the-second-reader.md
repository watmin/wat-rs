# BRIEF — identity 2c's remainder: teach `extend-type`'s other readers the form

Closes the last 3 of 22 ANNOTATION bindings. DESIGN context:
`DESIGN-STONE-the-angle-string-is-not-a-type-identity.md`; the classification is in
`TABLE-defservice-type-name-sites.md` (its line numbers are stale — locate by NAME).

## The three bindings, and why they are still angle-spelled

```
handle-bare-name        extend-type's TARGET argument      (position 1)
dialable-ty             extend-type's SATISFIED-SURFACE    (position 2)
typedcap-ty             extend-type's SATISFIED-SURFACE    (position 2)
```

Converting them emits `(Head :- [args])` into those slots, and a reader refuses it.

## ★ TASK 1 — establish WHICH readers. Do not take my list.

I grepped `":wat::core::extend-type"` and found **ten** sites that match the head:

```
src/types.rs:3642        src/check.rs:2638, :8372        src/freeze/env.rs:230, :455
src/runtime.rs:1267, :2521, :2547, :2822, :8181
```

I know two of them read the argument slots. I looked at five more through a **7-line window** and
saw nothing — **that is a window, not a census**, and `:2521`/`:2547` at least are head-name
membership lists rather than readers.

**Your first deliverable is the real list**: for each of the ten, does it read `items[1]` or
`items[2]`, and does it require a `Keyword`? State the command you used and what it cannot see.

⚠ **This stone exists because I did not do that.** Three times in two days I verified a form was
accepted *somewhere* and shipped "the slot accepts it" — `extend-type`'s surface arg (check-time
taught, runtime not), `(Head :- [args])` (expander taught, resolver not), and a `defservice`
annotation the macro **read back out of its own emission**. Each surfaced downstream, blaming an
innocent party. **A slot with two implementations is two slots.**

## The known two, and the exemplar to copy

```rust
// src/types.rs:3661 — position 2, ALREADY TAUGHT (the A-i flight). COPY THIS SHAPE.
Some(node @ WatAST::List(_, _)) => parse_type_node(node)?.base_fqdn().ok_or_else(…)?,

// src/types.rs:3644 — position 1, Keyword-only.  "expected keyword type name at position 1"
// src/runtime.rs:8226 — position 2 at the RUNTIME evaluator, Keyword-only.
//                       "extend-type second arg must be a keyword protocol name; got {}"
// src/runtime.rs:8212 — position 1 at the RUNTIME evaluator, Keyword-only.
```

Both doors already exist — `parse_type_node` then `TypeExpr::base_fqdn()`. **Write no third
base-extraction helper**; there are already sixteen hand-rolled `find('<')` sites in the tree and
adding a seventeenth is how the lattice's extraction stopped being singular.

## What "done" looks like

1. Each reader you found that requires a `Keyword` also accepts a `List`, via the two existing doors.
2. `handle-bare-name`, `dialable-ty` and `typedcap-ty` convert to
   `(:wat::core::keyword/to-type-form-colon (:wat::core::keyword-node (:wat::core::string::concat ":" <str>)))`
   — the pattern the other 19 already use; copy it, do not invent one.
3. ★ The **subtype edge still registers**. `extend-type` exists to register an edge; a reader that
   parses the form but drops the edge would pass a `--check` and break satisfaction silently. Prove
   the edge by a satisfaction check that would FAIL without it.
4. Both spellings work at every taught reader — the angle keyword must not stop working.
5. `wat/service.wat` has no `keyword/from-string` left on an ANNOTATION-role binding.

## Boundaries

- Do NOT run `scripts/floor.sh` or a full `cargo nextest` — the orchestrator measures centrally.
  ⚠ A scoped run is not the floor: on the previous stone `binary_id(wat::services)` was 128/128
  green while the floor was red by six.
- Do NOT commit, push, stash, revert or amend. Leave everything in the working tree.
- Do NOT touch DECL-NAME / RUNTIME-ARG / CTOR-ARG bindings or the OTHER sites.
- Write no new base-extraction helper.

## Your own checks

`cargo build --bin wat`, `target/debug/wat --check <file>` on files under `wat-scripts/scratch-pad/`,
and `cargo nextest run --release -E 'binary_id(wat::types)'` plus `-E 'binary_id(wat::services)'`.
Prefix long commands with `systemd-run --user --scope -q -p MemoryMax=16G -p MemorySwapMax=0 timeout 900`.
Diagnostics go to **stderr** — judge by exit code AND empty output, never grep alone.

Delete any scratch `.wat` that must fail; `tests/lint/wat_scripts_fixes_load.rs` type-checks
everything under `wat-scripts/`.

## STOP triggers — ship nothing further and report

- **STOP-1.** If a reader you found cannot accept a `List` without a second base-extraction path,
  STOP and report which. The two existing doors are meant to be sufficient; needing a third means
  the design is wrong, not that you should write one.
- **STOP-2.** If row 3 fails — the edge registers but satisfaction stops working, or vice versa —
  STOP and report. Parsing the form is not the point; registering the edge is.
- **STOP-3.** If your census in TASK 1 finds a reader outside `types.rs`/`runtime.rs`/`check.rs`,
  STOP and report before changing it. That would be a fourth pass nobody has accounted for.

## Your report

Your TASK 1 census — the command, the count, what it cannot see, and the verdict per site. The diff.
Every acceptance row with verbatim output, row 3 especially. What surprised you. Anything you
inspected and deliberately left alone, with the reason.
