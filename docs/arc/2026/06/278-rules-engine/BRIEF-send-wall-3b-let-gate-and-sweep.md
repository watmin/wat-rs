# BRIEF — send' wall Phase 3b: the `let`-`_` discard gate + the swallow sweep (the wall made WHOLE)

> **Tier:** sonnet shadowdancer. **Arc:** 278 send'-wall Phase 3 (see `DESIGN-send-outcome-wall.md`, R57
> `IGNORANTIAM DELEMVS`). Phase 3a (committed `186ffb91`) walls a discarded `send'`/`try-send'` outcome in the
> `do`-non-final position. **3b closes the OTHER discard door**: a `send'` outcome bound to `_` in a `let`
> binding vector — `(:wat::core::let [_ (:wat::kernel::send' p m)] …)` — is the same swallow, currently legal.
> This strike makes it a compile error too, so a `send'` swallow is unrepresentable in BOTH positions.
> **When 3b lands green, the send'-wall is WHOLE.**

## The disconfirming probe is ALREADY DRAWN + confirmed RED (do NOT touch it)
`tests/services/probe_arc278_send_outcome_must_use_wall_let.wat.bad` (a `let [_ (send' p 42)] nil`) +
`tests/services/probe_arc278_send_outcome_must_use_wall.rs::discarded_send_outcome_in_let_underscore_is_compile_error`.
Run NOW = RED (the `.bad` compiles clean → `expect_err` panics — the gap). It turns GREEN when the gate lands and
asserts `MalformedForm { head == ":wat::core::let", reason contains ":wat::kernel::SendOutcome" && "must be faced" }`.

## THE ORDER IS LOAD-BEARING: SWEEP FIRST, THEN THE GATE
If the gate lands first, all 41 pre-existing `_`-bound `send'` sites flip RED at once. So: **Move 1 (the sweep,
a codemod) faces every site → floor stays green** (a faced `send'` types as `nil`); **Move 2 (the gate) then
lands green** (nothing left un-faced). Both in this one strike; do NOT commit (the orchestrator weighs + banks).

## Move 1 — the SWEEP: a wat-fix codemod (NEVER hand-edit the `.wat` — R21)
Write `wat-scripts/fixes/face-underscore-bound-send-prime.wat`. **Copy the SHAPE of
`wat-scripts/fixes/wrap-client-method-match-in-recvoutcome.wat`** (the span-based `fix-text-apply` codemod:
`ast->children`/`ast-span`/`ast-end-span`, per-file `migrate`, stdin-path driver, idempotent matcher).

**The transform (uniform, order-preserving, handles first-pair AND mid-vector `_`):** for every `_`-bound
`send'` RHS inside a `:wat::core::let` binding vector, WRAP the `send'` call in the SendOutcome facing match —
its type becomes `nil`, so the outcome is faced and the gate passes:

```clojure
;; _ (:wat::kernel::send' X Y)   →   _ (:wat::core::match (:wat::kernel::send' X Y)
;;                                       (:wat::kernel::SendOutcome::Sent      nil)
;;                                       (:wat::kernel::SendOutcome::Closed    nil)
;;                                       ((:wat::kernel::SendOutcome::Lost _c) nil))
```
(These are all fire-and-continue probe/test sends whose outcome was discarded → all three arms → `nil`. The arm
shape is the exemplar `wat/service.wat:995-997`. `Sent`/`Closed` are bare keyword patterns; `Lost` binds `_c`.)

**Span edits (two inserts around the `send'` call node, mirror `wrap-edits` in the exemplar):**
- at the send'-call **start-offset**: `(:wat::core::match `
- at the send'-call **end-offset**: ` (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) ((:wat::kernel::SendOutcome::Lost _c) nil))`

**The matcher** — walk `:wat::core::let` nodes; take the binding vector (child[1], `ast-kind` "vector"); iterate
its children as `[name0 rhs0 name1 rhs1 …]` pairs (even index = name, +1 = rhs); for each pair where **`name` is
the symbol `_`** AND **`rhs` is a list whose head keyword is `:wat::kernel::send'`**, emit the wrap on `rhs`.
The generic recursion (`seq-edits` over `ast->children`) still descends everywhere else. **Idempotent by
construction:** after a wrap the rhs head is `:wat::core::match` (not `:wat::kernel::send'`) → re-run skips it.

**Apply discipline (mandatory, R21 / `wat/fix.wat` header):**
1. **Dry-run on a `/tmp` copy of the 26 files + `diff`** — confirm the diff is EXACTLY the wrap (nothing else moved).
2. Then apply to all 26 paths (the driver reads an EDN vector of paths on stdin):
   `printf '[<all 26 paths>]\n' | ./target/release/wat ./wat-scripts/fixes/face-underscore-bound-send-prime.wat`
3. Re-run once → **0 changes** (prove idempotent).

**The 26 files** (41 sites; regenerate with `grep -rln --include=*.wat -E '(\[|[[:space:]])_ \(:wat::kernel::send'"'" .`):
```
tests/channel/probe_arc209_connection_primitive.wat
tests/comms/probe_arc209_bound_listener.wat
tests/comms/probe_arc209_c0b1b_select_listener.wat
tests/kernel/peer_select_prime_process.wat
tests/process/probe_arc209_structured_peer_death_process.wat
tests/services/probe_arc170_c1_kwargs_bracket.wat
tests/services/probe_arc209_c0b3aii_process_service_loop.wat
tests/services/probe_arc209_c0b3bb_bounced_bounced.wat
tests/services/probe_arc209_c0b3bb_bounced.wat
tests/services/probe_arc209_c0b3bc_post_spawn_bogus_accessor.wat
tests/services/probe_arc209_c0b3bc_post_spawn_thread.wat
tests/services/probe_arc209_c0b3bc_post_spawn.wat
tests/services/probe_arc272_6b_state_over_lineage.wat
wat-scripts/probes/arc-170/probe-bracket-process-runner.wat
wat-scripts/probes/arc-170/probe-child-inherits-defns.wat
wat-scripts/probes/arc-170/probe-generic-shipped.wat
wat-scripts/probes/arc-170/probe-s1-fn-forms.wat
wat-scripts/probes/arc-170/probe-s1-impure-gate.wat
wat-scripts/probes/arc-170/probe-s1-named.wat
wat-scripts/probes/arc-170/probe-s3-process-runner.wat
wat-scripts/probes/arc-170/probe-s3b-astsplice.wat
wat-scripts/probes/arc-170/probe-s3b-crux-fnforms-closure.wat
wat-scripts/probes/arc-170/probe-s3c-rendezvous.wat
wat-scripts/scratch-pad/probe-optA-retag.wat
wat-scripts/scratch-pad/probe-selectables-homogeneity.wat
wat-scripts/scratch-pad/probe-timer-as-peer.wat
```
NB two files have `_` at a non-first pair (`probe-s1-named.wat` has `[i (recv' …) _ (send' …)]` AND
`_ (send' w 1) _ (send' w 2)`) — the pair-walk handles any even index + multiple `_`-sends per vector.

## Move 2 — the GATE: re-add the deferred `_`-wildcard arm
`src/check.rs`, `process_let_binding`, the **bare-symbol binder arm** (~`:12571`). The deferral note (~`:12575`,
"Arc 278 Phase 3 — STOP-2 (in spirit)") explicitly says to **re-add this arm once the sweep lands**. After the
`rhs_ty` is computed, before `new_bindings.insert(name, ty)`:

```rust
// Arc 278 Phase 3b — the `let`-`_` must-use gate (twin of infer_do's do-non-final gate):
// a `_`-bound must-use outcome (send'/try-send') is a swallow → located compile error.
if ident.as_str() == "_" {
    if let Some(ty) = &rhs_ty {
        let resolved = apply_subst(ty, subst);
        if is_must_use_type(&resolved) {
            push_must_use_error(&mut binding_errors, rhs.span(), ":wat::core::let", &format_type(&resolved));
        }
    }
}
```
Use `ident.as_str() == "_"` (the RAW ident, per the note — NOT the `env_key`'d `name`). Reuse the EXISTING
`is_must_use_type` + `push_must_use_error` helpers (do NOT invent new ones) so the error is identical in shape to
the do-gate's, just `head == ":wat::core::let"`. **Update the deferral-note comment** to "re-added in 3b — the
sweep landed (`face-underscore-bound-send-prime.wat`)" so the record stops saying "NOT shipped here".

## STOP triggers
- **STOP-0:** after both moves, the floor is NOT 0-failed → report which tests + why (a real un-faced swallow the
  gate caught, or a codemod ripple). Do NOT mass-edit.
- **STOP-1:** the dry-run diff shows ANY change beyond the intended `_`-bound-send' wrap (a wrong node wrapped, a
  span off-by-one, a non-`_` binding touched) → STOP, report, do not apply.
- **STOP-2:** the gate arm fires on something OTHER than a `_`-bound must-use RHS (e.g. a named binding, a faced
  send') → STOP; a faced send' types as `nil` (not must-use) so it must NOT fire.

## Verify (report; the orchestrator WEIGHS by its own `--release` re-run)
1. `cargo build --release` clean.
2. `./target/release/wat --check <a swept file>` clean (e.g. `tests/services/probe_arc209_c0b3bb_bounced.wat`).
3. **Whole floor `cargo nextest run --release`** — report the **Summary line** (target 0 failed).
4. Both RED-gate probes GREEN:
   `probe_arc278_send_outcome_must_use_wall::{discarded_send_outcome_in_do_non_final_is_compile_error,
   discarded_send_outcome_in_let_underscore_is_compile_error}`.
5. `git diff --stat` (expect: the codemod + 26 `.wat` + `src/check.rs`; the probe files are already staged by the
   orchestrator).

## Deliverable (do NOT commit — the orchestrator banks after its own weigh)
The recorded codemod `wat-scripts/fixes/face-underscore-bound-send-prime.wat`; the 26 swept `.wat`; the gate arm
+ updated note in `src/check.rs`. Report: (1) the codemod's final form + the dry-run diff summary; (2) the
idempotency re-run (0 changes); (3) the floor Summary read by you; (4) both probes green; (5) `git diff --stat`.

## Blast radius
`wat-scripts/fixes/face-underscore-bound-send-prime.wat` (new), the 26 `.wat` above, `src/check.rs`
(`process_let_binding` — the one arm + the note). NO `src/types.rs`, NO `src/runtime.rs`, NO other checker sites.
Scratch logs → `/tmp/claude-scout/`.
