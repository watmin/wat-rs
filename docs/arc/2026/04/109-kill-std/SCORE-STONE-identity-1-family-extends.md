# SCORE — identity stone 1 of 3: `family_extends` gets its own door. SHIPPED.

Two flights. Flight 1 implemented A-i, fired STOP-1, and found the mechanism; the builder ruled S2;
flight 2 undid most of flight 1 and kept one part. Every row re-verified by the orchestrator.

| # | row | result |
|---|---|---|
| 1 | ★ both negative controls PASS (`arc170` swap gate · `118.B1a` swap gate) | ✅ 537/537 scoped |
| 2 | `satisfies_bare_surface` / `format!("{surface}<")` gone | ✅ grep empty |
| 3 | `family_extends` has ONE impl; 4 sites route through it | ✅ |
| 4 | `transport_satisfier_heads` restored to three keys | ✅ |
| 5 | `extend-type` accepts a FORM parent `(:Proto :- [T])` | ✅ kept from flight 1 |
| 6 | `is_subtype` signature + 30 call sites unchanged | ✅ |
| 7 | floor | ✅ **4854/4854**, 0 FAIL, 19 skipped |
| 8 | clippy `-D warnings` | ✅ 0 |

## ★ What the stone actually delivers — and the proof it delivers no more

The rider evidenced rows 2-3 with **greps**, which show existence, not behaviour. The behavioural
question is whether `family_extends` differs from what it replaced:

```
OLD  satisfies_bare_surface:   p == surface || p.starts_with(&format!("{surface}<"))
NEW  family_extends:           split_type_params_pub(&p).0 == sup_base
```

These agree **exactly** when `sup` is bare, and diverge only when `sup` is PARAMETRIC — where the new
one is strictly more permissive. So the safety question is what the callers pass. Measured, all four:

```
check.rs:15333    ep       — a TypeExpr::Path from the (Parametric, Path) arm     bare
check.rs:15440    &bare    — literally parametric_head_fqdn(eh)                   bare
runtime.rs:8965   surface  — let surface = parametric_head_fqdn(p)                bare
runtime.rs:9011   surface  — let surface = parametric_head_fqdn(head)             bare
```

**Every caller passes a bare `sup`, which is precisely the case where old and new agree.** So this is
**provably behaviour-neutral**: a fake becomes a named function, and nothing moves. That is what the
DESIGN promised and the whole of what shipped.

★ Stating *why* it is neutral is worth more than observing that the tests pass — a green suite is
equally consistent with "neutral" and with "the tests do not reach it."

## The two flights

**Flight 1** implemented A-i faithfully, hit two red **negative controls**, and stopped. Its analysis
was correct and I verified it independently: `assignable`'s fast path was sound only because
`is_subtype` compared full strings — that comparison did two jobs, and base-stripping deleted the
second silently. It also correctly refused to special-case past the reds. **The approach was wrong;
the flight was not.**

**Flight 2** reverted three of four changes, kept the `extend-type` FORM acceptance, and added the
door. It also caught a trap of its own making and reported it: `family_extends`'s doc comment quoted
the literal `format!("{surface}<")` it was replacing, which **self-triggered row 2's acceptance
grep**. It reworded rather than reporting a green it had not earned.

And it deleted flight 1's scratch probe after checking it — that probe asserted A-i's post-strip
behaviour, which S2 makes false, so it would have tripped `wat_scripts_fixes_load.rs`. It verified
the failure before deleting rather than assuming.

## What remains, unchanged and named

- `transport_satisfier_heads`' three-key guess and `transport_edge_keys`' hardcoded `["T","Xt"]`
  last-arg rewriting **stay**. They guess at EXACT keys, which S2 keeps, so they are ugly and sound.
  Removing them requires the fast path to stop needing exact keys — its own stone, its own ruling.
- The `<T>` vs `<?454>` mismatch stays fixed where it always was: Stone 118.3-B's `else` branch.
- `defservice`'s 53 sites are stone 2. The three one-offs are stone 3.
