# BRIEF — sequential macro registration during expansion (the defservice/deftest cluster root)

## The root — PROVEN, do not re-litigate

`expand_all` (`src/macros/expand.rs:39-58`) is **expand-then-hoist**:

```rust
let expanded = expand_form(form, registry, 0, env, sym)?;   // expands the form FULLY
...
out.extend(hoist_top_level_form(expanded, registry)?);      // and only THEN registers its defmacros
```

and `expand_form` takes `registry: &MacroRegistry` — **immutable**, so **nothing can register mid-expansion**.

`defservice` emits ONE `do` (`wat/service.wat:1236-1245`):
`(do ~record-def ~state-def … (defn ~serve-name … ~serve-body) … ~@methods …)`.
`expand_form` recurses through that whole `do` — expanding **`serve`'s body before `::Record`/`::State`'s companion
macros register**. So a handler constructing its own minted types keeps a RAW form → `#wat.runtime/UnknownFunction`
at eval.

**Proven by a minimal 40-line defservice A/B** (`scratchpad/expand-order/scratch_expand_order.{wat,rs}` — see
"Acceptance", below):
- `ping` (handler constructs nothing minted; returns `s` unchanged + a `:messages` response — those ARE hoisted early
  via `hoist_surface_messages`) → **PASSES**.
- `bump` (handler constructs its own `(:probe::echo::State :durable (:probe::echo::Record :count 7))`) → dies
  `unknown function: :probe::echo::State`.
- The SAME probe constructs `(:probe::echo::Record :count 0)` at `/start` in the CALLER's world and it WORKS.
  Identical type, identical kwargs form: registered in the caller's world, RAW inside the handler.

THREAD fails / PROCESS passes because the forked child RE-EXPANDS with the companions already hoisted
(`expand.rs:52` states this outright).

## The fix — make registration SEQUENTIAL during expansion (NOT a special case)

The engine already documents the rule at top level (`expand.rs:29-36`): *"Register each such form as it appears so
subsequent forms in the stream can invoke the new macro."* That guarantee holds ONLY at top level today. **Make it
true inside a `do`/`let` body too: a container's children must see earlier siblings' `defmacro`s.**

This was chosen over a defservice-specific two-pass on the four questions. The special-case fails **Obvious**
(why only defservice?), **Simple** (a second accreted arm after `is_defsurface_form`), and **Honest** (it leaves the
promise broken and the next macro that emits-a-defmacro-and-uses-it hits the same wall with NO scream — a silent raw
form → runtime `UnknownFunction`, exactly the silent-failure class this arc has been extirpating). **DO NOT
"simplify" this build back into a defservice special-case.** If you find yourself keying on `defservice`, STOP.

### Rooms (exact)
1. **`expand_form` signature** (`src/macros/expand.rs:343`): `registry: &MacroRegistry` → `&mut MacroRegistry`.
2. **The borrow conflict** (`expand.rs:406`, also `:310`): `if let Some(def) = registry.get(head)` holds a `&MacroDef`
   across the recursive expansion. With `&mut` this conflicts with registering. Resolve by **cloning the `MacroDef`
   (or the fields the expansion needs) and dropping the borrow before recursing** — do NOT reach for interior
   mutability without reporting first.
3. **The child-walk** (`expand.rs:439` and `:452` — the `.map(|c| expand_form(c, …)).collect()` sites): for a
   `do`/`let` **body**, replace the `.map()` with a **sequential loop**: expand child → register any `defmacro` it
   carries → expand the next child. Reuse the EXISTING registration path — `hoist_defmacros_from_container`
   (`:246`, which already does `registry.register(def)?` at `:272`) and `is_do_or_let_containing_defmacro` (`:208`) /
   `container_body_start`. Do NOT hand-roll a second registration path.
4. **Callers** (the whole ripple — small): `expand.rs:39, 335, 383, 409, 429, 439, 452` + `src/macros/tests.rs:38`.

### The DELETION probe (do this — it tests whether the rule is right)
If registration is sequential during expansion, the `defsurface` special-case may be **unnecessary**:
`hoist_surface_messages` (`:159`) + `is_defsurface_form` (`:63`) + their arm in `expand_all` (`:41-56`) exist to fake
this very guarantee for `:messages`. **Try deleting them.** If the floor stays green without them, DELETE them — the
correct mechanism composes what exists and annihilates the scaffolding built to fake it. If they are still needed,
**report exactly WHY** (that is information about whether the rule is right, and it is a finding either way). Do not
delete them if anything goes red — report instead.

## STOP triggers (report, do not guess)
1. **Scoping:** registering defmacros from a `do`/`let` at ANY nesting depth may over-register (a macro defined deep
   in a fn body leaking to the global registry). If the correct depth/scope is ambiguous — STOP and report; this is a
   design call for the orchestrator, not a guess.
2. **Borrow:** if cloning the `MacroDef` cannot resolve the `&mut` conflict cleanly — STOP and report the exact
   error; do not introduce `RefCell`/`unsafe` unilaterally.
3. If you find yourself special-casing `defservice` — STOP (see above); that is the rejected design.

## Acceptance / Gate (report each, with captured evidence)
- `cargo build --release` clean.
- **The A/B probe** (`scratchpad/expand-order/scratch_expand_order.{wat,rs}` — copy into `tests/services/`, run,
  and PROMOTE it as the committed regression test, e.g. `tests/services/probe_arc294_expand_order.{wat,rs}`):
  `ping` stays GREEN **and `bump` turns GREEN** (it currently dies `unknown function: :probe::echo::State`).
- **WHOLE FLOOR** `cargo nextest run --release` captured to a temp file. Baseline is **49 failing (48 failed /
  1 timed out) at HEAD `26fb1c5e`**. **ZERO NEW failures** is the bar. Expect the `deftest_*`/service cluster (~15)
  + `smem_roundtrip` + `sqlite_store_differential` to clear → roughly **~32**.
- Report the EXACT before/after failing-SET diff. Extract names by stripping ANSI **and** the `( n/total)` progress
  counter (which can carry a **leading space**) before `comm` — a raw grep compares timings and manufactures phantom
  regressions.
- Report whether the `defsurface` special-case could be deleted, and if not, why.

## Method
Build/test ONCE to a temp file under `/tmp/claude-1000/-home-watmin-work-holon/ff83d181-261b-498c-8928-82c73028b60c/scratchpad/`,
grep the FILE. A mid-edit rustc/rust-analyzer diagnostic is a PHANTOM — trust a clean `cargo build --release`.
Commit NOTHING; leave the tree for the orchestrator to weigh.

## Report back (raw facts + file:line, not narrative)
1. The A/B probe result (`bump` green?).
2. Diff summary: files + line counts.
3. The whole-floor before/after failing-SET diff, computed exactly as above.
4. The deletion-probe outcome.
5. Any STOP trigger hit, with the exact error.
Your final message is the data the orchestrator weighs — do not claim green without captured evidence. A claim about
WHY something fails must come from reading that failure, not from assuming its class.
