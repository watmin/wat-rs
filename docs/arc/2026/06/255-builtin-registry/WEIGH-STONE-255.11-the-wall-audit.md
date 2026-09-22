# WEIGH — STONE 255.11: ACCEPTED. ⛔⛔ It found a LIVE CAPABILITY ESCAPE at the row I told it to skip.

Weighed against `0817306f9` by independent re-run, **building both binaries**. `git status
--porcelain -- '*.wat'` = **0**. Not pushed.

## ⛔⛔ THE ORCHESTRATOR WAS WRONG AND THE BRIEF WAS DANGEROUS

My FINDING said **`✅ :restricted-to — INTACT`** and **`⛔ Do not re-litigate`**. ⛔ **Had that been
obeyed, a live capability escape would still be in the tree.** The rider ignored it correctly.

**Reproduced here, two binaries from one tree:**

```
PRE-CURE  (:wat::core::quote (:my::kernel::restricted-fn 7))  → rc=1  DefRestrictedCallerNotAllowed
PRE-CURE  (:wat::core::quote (my.kernel/restricted-fn   7))  → rc=0  ⛔⛔ ESCAPED
POST-CURE both spellings                                      → rc=1, same error
POST-CURE a PERMITTED caller, symbol mention                  → rc=0  ✅ NOT over-restricted
```

⭐ **Why my four probes passed: a BACKSTOP, not the wall.** `normalize_symbol_refs` (step 7) rewrites
symbol→keyword in **CODE** positions, so a symbol-spelled *call head* arrived already canonicalized.
⛔ **A DATA position is never normalized**, and **arc 198's own ruling is that a restriction governs
MENTION, not head position.** I probed one position and wrote a verdict about a wall.

⛔⛔ **AND MY "different mechanism, unprobed" LINE WAS WRONG IN KIND.** The Rust-side
`#[restricted_to(…)]` fences drain into the **same `binding_metadata` map**, read by the **same
walker**. Measured pre-cure, mentioned from `:user::` code:

| fence | keyword | symbol |
|---|---|---|
| `wat.io.IOWriter/from-fd` — raw-fd writes | rc=1 | ⛔⛔ **rc=0** |
| `wat.kernel/close` — resource close | rc=1 | ⛔⛔ **rc=0** |

**Both cured and verified by me.**

## ⭐ THE AUDIT DID ITS JOB

**26 walls. 5 FAIL-OPEN on a name spelling. 4 cured, 1 reported** (curing `is_quasiquote_form` would
make it **more permissive** — it is a **ROUTER**: teaching it the symbol spelling would let a
symbol-spelled quasiquote body **skip both** the hygiene check and the F5 purity gate). ⭐ **That is
the cure/report line drawn exactly where the brief drew it**, and the residual asymmetry — the
keyword path is now the weaker one — is disclosed and handed up as **the builder's call.**

⭐ **Gate E needed TWO cures, not one:** curing `quasiquote_inner` alone left the wall silent because
the binder scan's own `let`/`fn` head test was keyword-only too. **Found by measuring, not assumed.**

## Gates re-derived

| gate | mine |
|---|---|
| floor | **5968 / 5968**, 22 skipped, **no `ARM.txt`** |
| clippy | exit 0 |
| census | `no STOP-8`, **212 = 212**, 0 rc changes |
| the escape, both directions + positive control | ⭐ **verified above, on two binaries** |
| `.wat` converted | **0** |

## ⭐⭐ THE BEST DISCLOSURE IN THE ARC

> *"The delta and the census are unchanged to the digit, and that is the point, not reassurance…
> No file in the 179 sample mentions a restricted binding in a data position, hands a mutation form
> to `eval-ast!`, or nests an explicit quasiquote. **A gate that cannot move is not evidence the diff
> is safe.**"*

⛔ **This is the correct reading and I want it kept.** Delta 4→4 and census 212→212 prove **nothing**
about this stone. The evidence is the hand-written probes and the two-binary pairs — **which is
exactly why the brief demanded a positive control per row rather than a headline number.**

## ⛔ IT CORRECTED THREE PRIOR STONES AND ME, FIVE TIMES

1. ⭐⭐ **My `INTACT` row** — above. **Retracted; the document is renamed
   `FINDING-RETRACTED-the-whitelists-were-NOT-intact.md`.**
2. **My seed list counts four NON-walls** — three are `#[test]` fns inside `mod tests`,
   `refuse_export_without_arm` is an **error constructor** that decides nothing. ⛔ **And it MISSED
   the wall that mattered:** `walk_for_restricted_call` is not named `refuse_*`/`validate_*`.
   ⭐ **A name grep cannot enumerate walls. I was told this shape twice and did it again.**
3. **255.9's mutation disposition is wrong** — "no mutation head escapes both walls"; **two do**
   (`set-redef!`, `set-eval-redef!`). Its three probes happened to pick backstopped ones.
   ⛔ *"Currently harmless is not a disposition."*
4. **255.10's quasiquote census row is wrong** — "LOUD"; it is **SILENT**.
5. Floor is 5968, not 5963.

⭐⭐ **THE DEEPEST FINDING — 255.9 AND 255.10 BOTH WROTE THE PIPELINE RULE CORRECTLY AND APPLIED ONLY
ITS FIRST HALF.** *"Post-step-7 ⇒ unreachable"* holds only for the **CODE** half of a wall's input.
The capability wall is a stage-8 site **both stones would have marked UNREACHABLE — and it was live.**
⛔ **There is STILL no gate on pass ordering. Recommended for the THIRD time, now with a worked
example of the reasoning failing in production.**

**Nineteen corrections across seventeen stones.**

## Declared unprobed — not papered over

`digest-load!`'s verify markers: refusal symmetric, ⛔ **but NO POSITIVE CONTROL — a verified load was
never made to ACCEPT.** ⭐ **That is the "a wall that refuses everything looks intact" failure mode,
declared rather than hidden** — the same mistake I made, caught by its own author.
Also unprobed: `rete/purity.rs::refuse_core_structural_on_multi` (FAIL-OPEN **by shape**, no reaching
probe built) · `ScopedLoader` path containment · `config.rs` setter discipline and
`validate_def_position_with_wrapper` (refusal arms never fired in **either** spelling).
§2.2's twelve "FAIL-CLOSED by construction" rows are **readings, not probes.**

## VERDICT

**ACCEPTED.** Pushing.

⛔ **The two standing instrument recommendations are now THREE stones old, and this stone is the
argument for both:** the RECOVERY column (printed by hand three stones running, *"one `awk` clause,
belongs in `scripts/`"*) and **a gate on pass ordering** — whose absence is what let two consecutive
stones mark a live wall unreachable.

Queue: the `wat.type` denotation stone (`register_validated` + the two runtime tables — closes the
8d-ii blocker **and** the last 2 delta files) · the 416-test rete remainder · the shape-B/D sweep ·
`:wat::keyword::canonical-identity` · 255.8's wrong-join hole · 8d-ii (4th draw) · 8d-iii.
⛔ **For the builder:** `is_quasiquote_form`'s residual asymmetry, and the two mutation heads
(`set-redef!`/`set-eval-redef!`) whose blast radius is nil **today**.
