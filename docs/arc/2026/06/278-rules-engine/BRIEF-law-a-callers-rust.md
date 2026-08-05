# BRIEF — the last 24, and they live in RUST, not `.wat`

> **Read `BRIEF-law-a-callers.md` FIRST, whole.** Every mapping rule, the fallback-value rule, the
> negative controls and the STOP triggers are there and they all still apply. This file is only the
> DELTA: a different edit surface, and a different way to verify.

## State, measured 2026-08-05

HEAD `5e3d1c5d`, pushed. The fence is ARMED in the working tree (`wat/rete.wat`, uncommitted) and
`./target/release/wat` is **already built with it** — use that binary, never rebuild.

Floor with the fence armed: **4337 passed / 24 failed.** All nine `.wat` grid families are already
green; these 24 are the surfaces a `.wat` sweep could never reach.

| cluster | n | file |
|---|---|---|
| accumulator oracle + differentials + custom | 10 | `tests/rete/probe_arc278_8{a,b,custom}*.rs` |
| `where` oracle + native differential | 6 | `tests/rete/probe_arc278_6b_ii_{a,b}*.rs` |
| node-share census | 4 | `src/rete/kernel.rs` — **`#[cfg(test)]` module only** |
| `:then` user forms | 1 | `tests/rete/probe_arc278_then_user_forms.rs` |

Refused heads: `=` ×8 · `>` ×6 · `i64::-` ×4 · `i64::+` ×3 · `first` ×2 · `>=` ×2.

## ⛔ CORRECTION 2026-08-05 — THIS BRIEF'S TITLE OVER-GENERALISES. CHECK YOUR FILE FIRST.

A rider caught this by reading instead of obeying. **Not every one of the 24 is inline wat.** There
are two shapes, and which one you have decides where the edit lands:

| shape | how to spot it | where the head lives |
|---|---|---|
| **inline string** | the rule is a `"…"` literal in the `.rs` | edit the `.rs` |
| **external fixture** | the `.rs` calls `startup_from_file` on a `const …_PATH: &str = "….wat"` | **edit the `.wat`**, not the `.rs` |

`tests/rete/probe_arc278_6b_ii_a_where_oracle.rs` is the second kind — it loads three co-located
`.wat` fixtures. Its failures are fixed in the FIXTURE. So:

> **If your assigned `.rs` has no inline rule to change, you are not done and you have not hit a
> STOP — you are in the second shape. Follow `startup_from_file` to the fixture and fix it there.
> Say so in your report, as the rider who found this did.**

I wrote the title from `8a_accumulate_oracle.rs` (genuinely inline) and generalised to all six
without checking the others — the same "an adjacent implementation is not the subject" shape the
record already carries. Check your own file.

## ★ THE DELTA — where the wat lives inside Rust string literals

```rust
"(:wat::core::defrecord :w::Reading [location <- :wat::core::String  value <- :wat::core::i64])\n\
 (:wat::rete::where (:wat::core::= ?n 3))"
```

Consequences you must respect:

1. **Only heads inside a `where`, an accumulator fence, or a `:then` item move.** A `defrecord`
   field type (`<- :wat::core::i64`) is a TYPE ANNOTATION, not a call — leave it. Same for
   `:wat::core::defn`, `:wat::core::defrecord`, `:wat::core::fn` used as a declaration head.
2. **Line-continuation backslashes and `\n\` must survive your edit byte-for-byte.** Breaking one
   turns a compile error into a *parse* error inside a string, which is far harder to read.
3. **This is why the codemod could not help.** `wat/fix.wat` walks `.wat` form trees; a Rust string
   is opaque to it. The record already carries this: *"a `.wat` sweep is BLIND to inline wat in Rust
   test strings"* — recorded 2026-07-24, and it caught us again today.

## ⛔ `src/rete/kernel.rs` — the ONE authorised `src/` edit, and it is narrow

Normally `src/` is off limits. Here you may edit **only the inline wat strings inside its
`#[cfg(test)]` module**. You may not touch engine code, a signature, a struct, or anything outside
that module. If a fix seems to require it — STOP and report.

## How to verify WITHOUT cargo

**Do NOT run `cargo build` / `nextest` / `clippy`.** Three riders share one `target/` lock; the
orchestrator measures centrally, once.

To check a mapping, copy the rule into a scratch `.wat` under `wat-scripts/scratch-pad/` and run it:

```bash
./target/release/wat wat-scripts/scratch-pad/<your-probe>.wat
```

The fence names the exact head and axis. **Delete your scratch probe when done** — every `.wat`
under `wat-scripts/` is parsed and type-checked by `every_wat_scripts_file_loads`, so a leftover
becomes someone else's red.

Every target you emit must appear as a `rete_name:` in `src/rete/vocabulary.rs`. Verify each; do not
infer from a pattern.

## The fallback rule, restated because it is the one that bites

Choose `:undefined <value>` so the **ENTIRE fenced expression** answers NO on undefined input —
every branch of every `if`/`cond`, not just the comparison the op sits in. A rider hit exactly this:
`(if (> s 6) true (< s 3))` with `:undefined 0` makes the else-arm TRUE and **the rule fires**.

Two sites can need different fallbacks for the same op:

```clojure
(not= (mod ?a 3) 0)  ->  :undefined 0   ; asks "non-zero?" -> must BE 0 to answer NO
(=    (mod ?k 4) 0)  ->  :undefined 1   ; asks "zero?"     -> must NOT be 0 to answer NO
```

## Rules of engagement

- Work only in `/home/watmin/work/holon/wat-rs/`. `.claude/worktrees/` is harness state, illegal.
- **Do NOT commit, push, stash, or revert.**
- You are a rider, not the orchestrator. **Ending your turn ENDS you** — nothing wakes you. Verify in
  the FOREGROUND; your turn ends when the numbers are in your hands.

**REPORT:** every `file:line` changed with head before→after and the fallback you chose *with its
one-line reason*; every STOP; and confirmation that no line-continuation was damaged.
