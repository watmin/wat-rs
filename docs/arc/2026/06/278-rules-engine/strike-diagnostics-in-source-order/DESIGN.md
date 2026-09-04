# DESIGN — check errors arrive in hash order; they should arrive in source order

## Why

**C20's remaining two files.** The first (`probe_arc278_rete_defn_recurse_mutual`) was cured
`bd83e6ea1`; these two were quarantined with captured evidence because their root was driven to be
different. It is.

**Re-driven at HEAD `645f219c4`, 16 runs each — both still vary:**

```
probe_arc170_c2_mixed_macro_swap.wat.bad    5 / 11
probe_arc170_w2a_kwargs_check_mint_swap     8 / 8
```

## What varies, exactly

Four errors, the same four, identical spans and messages. **Only the order moves**, and only one
error moves:

| | 1st | 2nd | 3rd | 4th |
|---|---|---|---|---|
| variant A | line 51 | line 51 | line 48 | **line 40** |
| variant B | **line 40** | line 51 | line 51 | line 48 |

Line 40 is in a different function from lines 48–51 (`:user::main`). **The errors are collected
per function, and the function iteration order varies.**

## The root

`SymbolTable.functions` is a **`HashMap<String, Arc<Function>>`** (`value/symbol_table.rs:34`), and
`check.rs:649` / `:738` walk it via `functions_iter()` (`symbol_table.rs:282` → `self.functions.iter()`).
`HashMap` iteration order is randomised per process, so the per-function error blocks emerge in a
different order every run.

**Same family as C20's first file** — a `HashMap` whose iteration order reaches user-visible output —
**and it needs a different cure.**

## The contract decision, pinned

**Sort the errors by span at the one site where they are returned. Do not touch the container.**

Every check error funnels through `check.rs:744-747`:

```rust
if errors.is_empty() { Ok(()) } else { Err(CheckErrors(errors)) }
```

`CheckError { span: Span, kind: CheckErrorKind }` — the sort key is already on the struct.

**Why not `BTreeMap` for `functions`, which is what cured C20's first file:** that would put
`O(log n)` on a **hot symbol-lookup path** to fix a diagnostic — a runtime cost paid on every call,
for an instrument's benefit. C10's standing ruling forbids exactly that trade.

**And a deterministic hash order would still be arbitrary.** Sorting by span makes the output
*meaningfully* ordered — a reader gets errors in the order they appear in their file — which is
strictly better than any stable-but-meaningless order. **This is the cure being better than the
defect's absence**, not merely a de-randomisation.

⚠ **The sort must be TOTAL, or it re-randomises.** Two errors can share a span (line 51 col 41 and
col 49 differ, but a same-span pair is possible). A sort keyed only on `(line, col)` leaves ties in
input order — which is hash order. **The key must break every tie**, and the strike must prove it
does rather than assume no ties exist.

## Out of scope = REJECTED

- **Changing `SymbolTable.functions`' container.** Hot path; see above.
- **Reordering anything but check errors.** Macro, parse and resolve diagnostics have their own
  paths; whether they share this defect is a separate measurement.
- **Removing the quarantine rows before the gate proves them fixed.** The determinism gate's
  `QUARANTINE_LEN` moves only on evidence.
