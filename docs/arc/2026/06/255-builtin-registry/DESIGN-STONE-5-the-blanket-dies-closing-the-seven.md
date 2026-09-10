# DESIGN — STONE ⑤: the blanket dies, and the seven it was hiding

The deletion is WRITTEN, MEASURED and PRESERVED at commit `d79597b3a` (unpushed). Applied, the
floor waterfalls **133 → 111 → 97** across two fixes already in that commit. This stone closes the
remaining seven families so it can land green.

```rust
-  if is_reserved_prefix(head) { return true; }
+  if crate::intrinsic::registry().contains(head) { return true; }
```

No prefix is consulted — measured, EVERY registry name is reserved-prefixed, so gating on the
prefix would NARROW the blanket rather than delete it and leave a prefix list standing in front of
the registry. And the rung FALLS THROUGH on a miss instead of returning: that single property is
the difference between 97 files refusing and 600.

## ⛔ What the deletion proved, before anything else

**The stdlib was calling verbs the substrate declared nowhere.** `wat/core.wat` called
`:wat::core::List?` three times; the verb had no row, no scheme, only a dispatch arm. Nothing had
ever asked what it was, because the blanket answered first. That one is CLOSED (`9054db083`) — it
carries a `TypeScheme` now, and the `REGISTRY_MEMBERSHIP_GAP_A` ratchet moved with it.

That is the whole thesis of arc 255, demonstrated: *"the undefined-func class dies as a SIDE EFFECT
of fixing the real defect."*

## The seven, each measured

### A · `:wat::program::self-peer` — a FOURTH store · 15 direct, most of the 97 downstream

Called from `wat/bracket.wat` (the stdlib) at three sites. It has a runtime arm
(`eval_program_self_peer`, `runtime.rs:11015`) and a **literal inference arm** at `check.rs:4247`
(`infer_program_self_peer`) — and **no `TypeScheme` and no row**. The checker's special-case arms
are a store my stone-③ fold does not reach and nothing can enumerate.

It takes **two TYPE-shaped arguments** (`is_type_arg_shaped`), so it is a special form, not a call.
⚠ Do NOT give it a scheme — that would be a second authority beside the inference arm. The honest
close is `#[wat_special_form(":wat::program::self-peer")]` with `#[wat_special_form_impl]` pointers
for `role = check` and `role = eval`; **both target functions already exist**.

★ The `every_special_form_carries_check_and_eval_impls` wall REQUIRES those pointers — that is what
refused stone ③a-i, and here the pointers are real rather than absent, which is why this one lands
where that one could not.

### B · `:wat::kernel::abort` — PHANTOM · 3 test fixtures

`grep -rn "kernel::abort" src/` → nothing. No arm, no scheme, no row. Sites are
`tests/reflection/wat_arc201_holon_ast_accessors_{children_parametric,children_sig,first_err_empty}.wat`,
each `(:wat::kernel::abort "message")` in a diverging `match` arm.

**MEASURED replacement:** `(:wat::kernel::assertion-failed! :message "…")` in that exact position
type-checks at **exit 0**. Same disposition, same shape, as stone ④'s `:wat::kernel::panic!`.

### C · `:wat::core::vector` — PHANTOM · 1 fixture, and it DE-VACUIFIES its test

`tests/types/probe_arc256_generic_defclause_c05.wat:5` calls `(:wat::core::vector 1 2 3)` —
lowercase, registered nowhere.

**MEASURED replacement:** `(:wat::core::Vector :- [:wat::core::i64] 1 2 3)` → `"[1, 2, 3]"`, exit 0.
Bare `(:wat::core::Vector 1 2 3)` is REFUSED — the head requires its `:- [T]` param-spec first.

★★ And this is not cosmetic. `c05_parametric_container_clause_checks` asserts *"a clause over a
parametric container head (Vector T) … dispatch matches by head."* It feeds a phantom whose type
the checker resolves to a FRESH VARIABLE, which unifies with `(Vector :- [T])` unconditionally.
**The test passes whether or not head-matching works.** Fixing the phantom is what makes it test
its own claim.

### D · `:wat::core::i64::+` and `::*` — RETIRED spellings · 14 sites in `src/resolve/mod.rs`

`src/remedy/retirement.rs:190` — `retired: ":wat::core::i64::+"` → `replacement: ":wat::i64::+"`.
The live sites are UNIT TESTS inside `src/resolve/mod.rs` using `wat.core/i64::+` as a sample
namespaced symbol to test normalize's data-vs-code boundary.

★ Their subject was a phantom. `normalize_skips_match_pattern_but_rewrites_body` asserts *"the BODY
symbol must be rewritten to its keyword FQDN"* — true under the blanket for ANY name. Migrating to
`wat.i64/+` makes the assertion about a name that exists. The identity is incidental to what these
tests measure, which is why the migration is mechanical and loses nothing.

### E · `:wat::type::i64` — a BARE type-namespace symbol · stone ② reached only the `:-` HEAD

`tests/resolve/probe_arc251_stone2_type_namespace.wat`:

```
(:wat::core::defn :user::inc [x <- wat.type/i64] -> wat.type/i64 (:wat::i64::+ x 1))
```

These are Symbols in ANNOTATION positions — not `:-` heads — so stone ②'s union does not see them.

**The rule, and it is derivable rather than positional: `:wat::type::` is a TYPE-ONLY namespace. A
name under it is never a call head.** So `resolve_namespaced_symbol` accepts a `:wat::type::`
candidate iff `TypeEnv::is_known_type` (stone ②'s one door, which canonicalizes
`:wat::type::X` → `:wat::core::X`) answers yes — in ANY position. The namespace IS the position, so
normalize needs no position tracking it does not have.

### F · `:wat::spawn::ProcessLaunch/bogus-field` — a DELIBERATE negative fixture

Its expectation shifts only WITH the deletion: the bogus field is currently refused downstream and
will be refused at resolve instead. **This one lands WITH G, not before** — its post-deletion
verdict cannot be asserted while the blanket stands.

### G · THE DELETION

Already written and preserved at `d79597b3a`, together with the `:rust::*` rung it needs:

> `:rust::*` has its OWN authority — a rust path is legal iff a `(:wat::core::use! :rust::Type)`
> declaration covers it, and `check_form` asks `UseDeclarations::covers` on the very next lines.
> This predicate has no `use_decls`; answering here would duplicate that question and answer it
> worse. **Not a surviving blanket:** every `:rust::*` name reaching that rung IS validated,
> immediately, by the authority that owns it.

## The order, and why it matters

```
A · B · C · D · E     land GREEN under the blanket — each is a fix that is correct either way
F + G                  land TOGETHER — F's verdict only exists after the deletion
```

Five sixths of this stone is verifiable **before** the irreversible step. That is the point of the
ordering: when G lands, the only thing that can be wrong is G.

## Out of scope = REJECTED

- **The two ratchets** (`probe_arc255_the_blanket_hides_a_phantom_head`). They pin pre-deletion
  behaviour and MUST go red at G. Updating them is G's own proof and belongs in G's commit.
- **Giving `self-peer` a `TypeScheme`.** It has an inference arm; a scheme would be a second
  authority for one question.
- **The dot flip.** After the blanket, per the seam's measured ordering.
- **`type-equal?`'s unreachable angle-bracket branch.** Still open, still recorded at
  `tests/reflection/probe_arc255_ivb2b_verify_examples.rs`. Unrelated to the blanket.
