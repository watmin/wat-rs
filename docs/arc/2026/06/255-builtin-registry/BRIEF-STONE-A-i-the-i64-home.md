# STONE A-i — the i64 home: 17 ops move to `:wat::i64::*`

DRAWN + BRIEFED 2026-08-25 against `99f9d144a`.
DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-the-numerics-get-their-homes.md` — read
its **"THE FINDING THAT SHAPES THE WHOLE STONE"** section; it is why this is Stone A of three.

## The one thing to hold

**BOTH SPELLINGS LIVE WHEN YOU ARE DONE.** `:wat::core::i64::+` must still work exactly as it does
today, and `:wat::i64::+` must work too. Nothing in the corpus moves in this stone. That is what
keeps the tree green while 2,330 call sites still spell the old name — they migrate in Stone B, and
the old names die in Stone C.

If you find yourself deleting an old dispatch arm, stop: that is Stone C's work and it will break
the floor.

## The 17 ops

```
binary   + - * /  < <= > >= = not=  mod quot rem
unary    to-bigint  to-f64  to-rational  to-string
```

`:wat::core::i64::+` → `:wat::i64::+`, and so on for all 17. **`:wat::core::+` — the polymorphic
generic — is NOT touched.** Only the per-type spelling moves.

## Your role

Your cwd is `/home/john/work/holon/wat-rs`. Run `pwd` first. **Ending your turn ENDS you** — nothing
will wake you; there is no notification coming. Every command **FOREGROUND**, blocking.
**You may not spawn sub-agents.**

Do not commit, push, stash, revert, or `git checkout`. `git stash@{0}` must never be touched.

You may run `cargo build --release`, `cargo build --release --all-targets`,
`./target/release/wat --check <f>`, `./target/release/wat <f>`, and single named tests. **Not** the
floor, **not** clippy — the orchestrator measures those centrally once the tree is quiescent.

---

## The rooms, in order

1. **`src/intrinsic/string.rs:193-211`** — THE SHAPE TO COPY. A `///` preamble the macro parses
   (`@added @Purity @Determinism @Category @arg @ret @example`), then
   `#[wat_intrinsic(":wat::string::length")]`, then a `pub(crate) fn` taking one `&WatAST` per arg
   plus `env, sym, span`. The attribute sniffs arity, emits the arity-checking `NativeHandler` shim,
   and `inventory::submit!`s the pair. **There is no explicit `register()` call to write.**
   Read its module header too — it explains the registry-home / namespace-home split.
2. **`src/intrinsic/mod.rs`** — the new module must be `mod`-declared here or its submissions never
   link. This is the one step whose omission fails silently.
3. **`src/runtime.rs`** — the implementations, and TWO dispatch tables. Both stay.
   - impls: `eval_i64_arith` **:9893** · `eval_i64_to_rational` **:10220** ·
     `eval_i64_to_string` **:10503** · `eval_i64_to_f64` **:10524** · `eval_i64_to_bigint` **:10548**
   - dispatch table 1: **:5848** (`":wat::core::i64::+" => eval_i64_arith(...)`, closure per op)
   - dispatch table 2: **:12058** (`arith_i64_i64_inner` at **:12194**) — the CEK/redex path
   - ⚠ The comment at **:5846** says the per-type suffix *"was arity-disambiguation scaffolding now
     superseded by defclause polymorphic surface."* Read it before you assume what these are for.
4. **`src/intrinsic/bytes.rs`** — a self-contained home with no separate namespace home. The i64 ops
   should follow bytes, not string: their algorithms are `checked_add`-shaped and shared with
   nothing outside this file.

## How to make both spellings live

`eval_i64_arith` is generic over a closure, so it cannot itself carry `#[wat_intrinsic]` — that
attribute needs one fixed-arg fn per name. Write 17 handlers in `src/intrinsic/i64.rs`, each with
its preamble and attribute, each delegating to the SAME implementation the old dispatch arm calls.
**Do not copy the arithmetic — share it.** Two implementations of `checked_add` overflow semantics
that must agree forever is a defect, not a migration.

The overflow/division contract is load-bearing and already correct: integer overflow raises a
distinct `RuntimeErrorKind::IntegerOverflow`, **never conflated with `DivisionByZero`, never
silently wrapped** (the comment at :12056 states it). Whatever you share, that must remain true for
both spellings.

## STOP triggers — each rejects; none permits a lesser delivery

1. **STOP-1 — you cannot register a name without duplicating an implementation.** Report the op and
   what blocks sharing. Do not ship two copies of an arithmetic contract.
2. **STOP-2 — an old spelling stops working.** That is Stone C's job, not this one. Report it.
3. **STOP-3 — `#[wat_intrinsic]`'s arity sniff cannot express one of the 17.** Name the op and what
   the attribute could not do; ship the other 16 and report the gap. Do not hand-roll a shim that
   bypasses the registry — a name registered by a different mechanism is invisible to Stone C's
   membership test, which is the entire point of the arc.
4. **STOP-4 — a room's line number does not hold what this brief says.** Written against `99f9d144a`.

## Acceptance — every row derives its bar

```bash
# 1. seventeen new names are REGISTERED. BAR: 17.
grep -c '#\[wat_intrinsic(":wat::i64::' src/intrinsic/i64.rs

# 2. the module is linked (the silent-failure step). BAR: non-empty.
grep -n 'mod i64' src/intrinsic/mod.rs

# 3. BOTH spellings run. Write a probe under wat-scripts/scratch-pad/ that ASSERTS a result for
#    each of the 17 under BOTH spellings (34 assertions) — the scratch-pad is loader-gated by
#    every_wat_scripts_file_loads, so the probe becomes a permanent live check, not a throwaway.
./target/release/wat --check wat-scripts/scratch-pad/<your-probe>.wat; echo "EXIT=$?"   # 0
./target/release/wat        wat-scripts/scratch-pad/<your-probe>.wat; echo "EXIT=$?"   # 0

# 4. the overflow contract holds under the NEW spelling too — assert IntegerOverflow is raised,
#    and that it is NOT DivisionByZero.

# 5. the builds that reach macro expansion.
cargo build --release && cargo build --release --all-targets
```

## Report back with

- Each command's **actual output**, naming the command that produced each number.
- **How you shared the implementation** between old and new spellings — show the code, not a
  description. This is the row I will re-read most closely.
- The probe's full text.
- Whether the `#[wat_intrinsic]` arity sniff handled all 17 cleanly, and any op that fought it.
- Anything the brief got wrong.
- What you did NOT do, and why.
