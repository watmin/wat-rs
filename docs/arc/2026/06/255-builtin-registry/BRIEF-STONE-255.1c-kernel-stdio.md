# BRIEF — STONE 255.1c-kernel-stdio: carve the six stdio verbs into home #3

Read `DESIGN-STONE-255.1c-kernel-stdio.md` first — it explains why this is 6 arms and not 49, and
where the handler bodies actually live.

## THE WORK, in one paragraph

`src/intrinsic/time.rs` is home #2 and your template. Build `src/intrinsic/kernel_stdio.rs` the same
way for exactly six verbs — `println`, `pprintln`, `eprintln`, `epprintln`, `readln'`, `read-frame` —
declare `mod kernel_stdio;` in `src/intrinsic/mod.rs`, and delete the six arms you carved. **Every
one is `@Purity Effectful`** — the registry's first. Handler bodies live in `crate::services::`, not
inline. Floor held.

## ROOMS — read in this order

1. **`src/intrinsic/time.rs`** — home #2, the closest template to what you are building. Note the
   module doc, the per-fn `///` doc-contract block, the `#[wat_intrinsic("<fqdn>")]` attribute, and
   the positional `&WatAST` params before the `env`/`sym`/`span` tail.
   ⚠ **The `///` block is the USER-FACING body that `render-doc` prints.** Maintainer rationale goes
   in `//`, never `///` — that mistake shipped and was caught by goldens earlier today.
2. **`src/runtime.rs:5704–5714`** — the six arms. Read them: each delegates, e.g.
   `":wat::kernel::println" => crate::services::eval_kernel_println(args, list_span, env, sym).map_err(Into::into)`.
   Note `readln'` carries a `rune:lint(retired-name)` and an optional leading cap arg; `read-frame`
   is the raw-frame sibling.
3. **`src/services/`** (`mod.rs`, `verbs.rs`, `client.rs`) — **where the six bodies actually live.**
   Find each `eval_kernel_*` fn. This directory is in your blast radius.
4. **`src/intrinsic/mod.rs:45–198`** — the enums your doc tags lower into (`Kind`, `DefinedIn`,
   `Layer`, `RuntimeCategory`, `RuntimePurity`, `RuntimeDeterminism`, `Arity`). **These exist. Mint
   nothing.** Also where `mod kernel_stdio;` is declared — without it the `inventory::submit!`s never
   link and the registry stays empty.
5. **`src/intrinsic/mod.rs:601`** — `pure_declared_matches_is_effectful_op`, the cross-check that has
   never yet seen an `Effectful` row. Read it so you know what it compares.
6. **`src/runtime.rs:25164`** — `is_effectful_op`. It classifies **by prefix**:
   `head.starts_with(":wat::kernel::")` ⇒ effectful. So it already has an opinion about all six.

## THE POINT OF THIS STONE

Every registered row today is **48 `Pure` / 2 `Preserving` / 0 `Effectful`**. These six are the first
`Effectful` registrations, and they make `pure_declared_matches_is_effectful_op` a real gate instead
of one that has only ever agreed with itself.

**Declare `@Purity Effectful` on all six from the body, and let the cross-check meet it.** If the
cross-check disagrees with a declaration you believe is correct from reading the body, **that is
STOP-2 — report it.** Do not adjust the declaration to make the test pass, and do not touch
`is_effectful_op`. Two independently-derived answers agreeing is the entire value; making one copy
the other destroys it.

`@Determinism`: decide each from its body. Reading fd 0 is not repeatable; writing fd 1 returns the
same value regardless. State your reasoning per verb.

`@Category`: reuse an existing variant. **If none of the six fits, that is STOP-3** — report the verb
and what it does. Do not mint one; `Clock` and `Arithmetic` were minted by builder ruling, not by a
rider deciding mid-strike.

## BLAST RADIUS

`src/intrinsic/kernel_stdio.rs` (new) · `src/intrinsic/mod.rs` (one `mod` line) · `src/runtime.rs`
(six arms deleted) · `src/services/` (only if a moved body leaves a genuinely unused fn — mirror what
home #2 did with `src/time.rs`). **Nothing else. No other kernel concern. No new enum variant. No
change to `is_effectful_op`, the resolver, the checker, or any `.wat`.**

## STOP TRIGGERS — each means ship nothing, report the gap

**STOP-1 — a body cannot move unchanged.** These delegate into the service tier; if a handler reaches
for something only `dispatch_keyword_head_value`'s local scope has, the seam is wrong. Report which.

**STOP-2 — declared purity and `is_effectful_op` disagree.** Report both answers and the body that
decides it. Change neither.

**STOP-3 — no existing `@Category` variant fits.** Report the verb and what it does.

**STOP-4 — routing changes.** stdio goes through the service tier, and this project has a documented
thread-vs-process stdio asymmetry (a body calling `println` in a thread shares the parent's fd 1).
Registration must not change *which* fn runs — same fn, reached through the registry instead of a
literal arm. If any stdio behaviour differs (output appearing somewhere new, a hermetic test's
captured stdout changing), stop.

**STOP-5 — the floor moves.** Capture the failing test whole and verbatim per the red protocol. Do
not re-run to see whether it clears.

## THE GATE

1. `cargo build --release` — exit 0.
2. `cargo clippy --release --all-targets` — zero warnings, **no `#[allow(dead_code)]`** in the new module.
3. **The Effectful proof.** Run the built binary and paste the actual output:
   `(:wat::runtime::metadata-of :wat::kernel::println)` → `:purity` must read **`Effectful`**.
   Also paste one `Pure` row for contrast (e.g. `:wat::time::to-iso8601`). **A stone where every row
   still reads `Pure` has not done the thing it exists to do.**
4. **Arms gone**: `grep -cE '^\s+":wat::kernel::(println|pprintln|eprintln|epprintln|readln.|read-frame)" *=>' src/runtime.rs` → **0**.
   (Anchor on `=>`. A bare substring grep also matches type-name string literals and error text —
   that imprecision cost a delta on home #2.)
5. `git diff --stat` — the four paths named above and nothing else.
6. Floor: **not yours.** The orchestrator runs `scripts/floor.sh` centrally and weighs by its own re-run.

Run everything **FOREGROUND** and block on it. **You are a rider, not the orchestrator: ending your
turn ENDS you** — nothing wakes you, no notification is coming. Your turn ends when the numbers are in
your hands, not when a command is launched.

⚠ **Rebuild between every source change and every run.** A restored source with a stale binary reports
the previous build; that exact error produced a false reading twice today.

## A PRIOR RESULT TO COPY FOR SHAPE

`25c1f452` (255.1c-time, home #2). Copy its register: every scorecard row scored against what was
actually observed, the orientation sketch overridden from the bodies with the disagreement stated
plainly, and the brief's own errors named as the brief's rather than smoothed over.
