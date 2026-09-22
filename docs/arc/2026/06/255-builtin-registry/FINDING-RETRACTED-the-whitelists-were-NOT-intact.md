# ⛔⛔ RETRACTED IN PART, 2026-09-22 — THE TITLE IS WRONG. THE CAPABILITY WALL WAS **OPEN**.

> ⛔ **This document said `✅ :restricted-to — INTACT` and `⛔ Do not re-litigate`. That instruction,
> had it been obeyed, would have left a LIVE CAPABILITY ESCAPE in the tree.** 255.11 ignored it
> correctly and found one. **The four probes below are all TRUE and all in a CALL position.**
>
> ⭐ **THEY PASSED BECAUSE OF A BACKSTOP, NOT BECAUSE OF THE WALL.** `normalize_symbol_refs`
> (step 7) rewrites symbol→keyword in **CODE** positions, so a symbol-spelled *call head* reached
> `walk_for_restricted_call` already canonicalized. ⛔ **A DATA position is never normalized**
> (`resolve/boundary.rs`), and arc 198's own ruling is that **a restriction governs MENTION, not
> head position.** The walker matched `WatAST::Keyword` alone. Reproduced by the orchestrator on
> two binaries built from one tree:
>
> ```
> PRE-CURE  (:wat::core::quote (:my::kernel::restricted-fn 7))  → rc=1  DefRestrictedCallerNotAllowed
> PRE-CURE  (:wat::core::quote (my.kernel/restricted-fn   7))  → rc=0  ⛔⛔ ESCAPED
> POST-CURE both spellings                                      → rc=1, same error
> POST-CURE a PERMITTED caller, symbol mention                  → rc=0  ✅ not over-restricted
> ```
>
> ⛔⛔ **AND THE "DIFFERENT MECHANISM, UNPROBED" LINE BELOW IS WRONG IN KIND.** The Rust-side
> `#[restricted_to(…)]` substrate fences drain into the **SAME `binding_metadata` map** read by the
> **SAME walker**. Orchestrator-measured, pre-cure, mentioned from `:user::` code:
> `wat.io.IOWriter/from-fd` **rc=0**, `wat.kernel/close` **rc=0** — raw-fd writes and resource close,
> escaped. Keyword spellings were rc=1. **Cured and verified by 255.11.**
>
> ⭐ **THE LESSON, AND IT IS THE ORCHESTRATOR'S:** *four green probes in one position are not a
> verdict on a wall.* A wall must be probed in **every position its own design ruling names** —
> here, MENTION, not merely head. **Never write "do not re-litigate" over a security wall.**

---

# FINDING — the capability whitelists are INTACT. The macro purity gate was not.

**Measured 2026-09-22 at `fd1991d01`**, prompted by the builder's question *"did we break our
whitelists?"*. Answer: **NO.** Evidence below. **A DIFFERENT wall was open, and is now closed.**

## ✅ `:restricted-to` — INTACT, four ways

Fixture: `tests/kernel/wat_arc198_def_restricted_bad_outside_namespace.wat` (caller outside the
whitelisted namespace) and its positive sibling.

| probe | result |
|---|---|
| entry as a **bare symbol** — `{:restricted-to [my.kernel]}` (the builder's ruling) | ⭐ **DENIED** |
| **call head** symbol-spelled — `(my.kernel/restricted-fn 7)` | ⭐ **DENIED**, `DefRestrictedCallerNotAllowed`, **byte-identical reason** |
| the whole fixture **codemod-converted** | ⭐ **DENIED** |
| ⛔ **POSITIVE CONTROL** — a permitted caller, converted | ⭐ **ALLOWED, rc=0** |

**The positive control is the load-bearing row:** the whitelist is not merely refusing everything.
It denies in every spelling and permits in every spelling.

⭐ **This was DELIBERATE, not luck.** `check.rs::restricted_to_entries` matches `Keyword` **and**
`Symbol` and makes anything else a hard error, with the reason recorded in its own doc:

> *"251.8d-i: the silent `filter_map` drop would empty every whitelist after the flip."*

**The whitelist-emptying hazard was found and fixed one arc ago.**

## ⛔ What WAS open — the F5 macro purity gate (arc 249), a DIFFERENT wall

Not capability; **macro purity**. An impure head inside a macro body is refused **at definition**.
It dispatched on the keyword spelling, so a symbol head fell to the `_ =>` recurse arm and the
allow-list never ran. Measured on **two binaries built from the same tree**:

```
PRE-CURE   (:wat::kernel::println …) in a macro body → rc=1  "default-deny F5 gate, arc 249"
PRE-CURE   (wat.kernel/println   …) in a macro body → rc=0  ⛔⛔ CLEAN — the macro was DEFINED
POST-CURE  both spellings                            → rc=1, BYTE-IDENTICAL refusal
```

⛔ **It was LIVE, not latent.** The probe is an ordinary keyword-spelled file with one symbol-spelled
call hand-written into a macro body. wat has accepted symbol heads since 8c, so this was writable
today. **8d-iii would have made the bypass the DEFAULT spelling.** Closed by 255.10.

## ⚠ NOT VERIFIED HERE — state plainly rather than imply coverage

- **The two mutation walls** (`freeze.rs:1894`, `runtime.rs:14442`). ⛔ **Code shape confirmed
  keyword-only** (`Some(WatAST::Keyword(head,_)) = items.first()`). 255.9 measured them
  **BYPASSED-but-BACKSTOPPED** — *the refusal survives, the DIAGNOSTIC does not*. ⚠ **My own probe
  did not reach the wall** (type-check error first), so **no independent measurement is claimed.**
  255.9 flagged widening a security wall's input as **the builder's call**.
- The Rust-side `#[restricted_to(…)]` proc-macro sites (`io.rs`, `rete/kernel/arm.rs`) — a different
  mechanism, unprobed.
- Field-level and ctor restrictions (`declare/register.rs:901/908`, `types/defstruct.rs:103/236`).
- **Runtime** enforcement as distinct from `--check`. ⛔ 255.10 disclosed that **154 of 156 clean
  converted files were never RUN**, and `scan_for_setter` is the proof that `--check` clean hides a
  behaviour change.

## ⭐ RECOMMENDED — a security-wall audit, BEFORE 8d-iii

The generalisable rule the F5 bug teaches:

> ⛔ **A DEFAULT-DENY GATE THAT DISPATCHES ON A SPELLING IS DEFAULT-ALLOW FOR EVERY OTHER SPELLING.**
> A wall whose unrecognised branch is `_ => recurse` or `_ => Ok(None)` **fails OPEN.**

**Enumerate every wall, classify its unrecognised branch (fail-open vs fail-closed), and probe each
in BOTH spellings with a positive control.** This is not a sweep of 665 read-sites — it is a bounded
list of *walls*, and it is the one audit that must precede the corpus flip, because the flip makes
the symbol spelling the default everywhere.
