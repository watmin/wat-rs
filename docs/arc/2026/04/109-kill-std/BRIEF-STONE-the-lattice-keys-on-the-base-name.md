# BRIEF — the subtype lattice keys on the BASE NAME (identity stone 1 of 3)

DESIGN: `DESIGN-STONE-the-angle-string-is-not-a-type-identity.md`. Read it first — it carries the
measurement and the reason this is a DELETION.

## The work, in one paragraph

`subtype_edges` is keyed by a type's FULL rendered name, so an edge registered as
`:sq::Seqable<T>` can never match a call site that renders `:sq::Seqable<?454>` — the args in the key
are a bound variable's NAME. Two helpers exist solely to work around that. Key by the BASE NAME
instead; the two helpers then collapse.

## The shape — one write door, one read door

```
register_subtype(child, parent, span)   types.rs:716   the ONE write door — all 6 callers are in
                                                       this same file (:641 :658 :695 :2526 :3626 :3658)
subtype_parents(name)                   types.rs:738   the ONE read door
is_subtype(sub, sup, env)               types.rs:5564  reflexive `sub == sup`, then walks parents
subtype_edges: HashMap<String,…>        types.rs:479
```

**Strip to base inside `register_subtype`** and every key in the map is a base name by construction.
`is_subtype` then strips its two inputs. That is three touch points, one file.

## The doors already exist — write no new helper

```
split_type_params_pub(s) -> (base, suffix)    src/runtime.rs:14266
TypeExpr::base_fqdn() -> Option<String>       src/types.rs:131   ("One implementation, two doors")
```

⚠ Base extraction is hand-rolled **sixteen** times inline elsewhere in the tree. Consolidating those
is NOT this stone. But the lattice's own extraction must be singular — route all of it through one
door and make that visible, or you reintroduce the inconsistency this removes.

## The two collapses

```
transport_satisfier_heads(head)   types.rs:745   → vec![fq, format!("{fq}<T>"), format!("{fq}<Xt>")]
                                                   guesses THREE keys. Becomes ONE.
                                                   callers: types.rs:754, check.rs:9770

satisfies_bare_surface(sub, surface, env)  types.rs:752  → prefix-matches format!("{surface}<")
                                                   Becomes is_subtype.
                                                   callers: check.rs:15333 :15440, runtime.rs:8915 :8961
```

### ★ THE ONE CONTRACT DECISION

Do these two functions get **deleted** (substituting at their 6 call sites, touching `check.rs` and
`runtime.rs`) or **kept as thin wrappers** (zero call-site churn, but a name that no longer describes
what it does)?

**Take the deletion if the substitution is mechanical.** A function named `satisfies_bare_surface`
whose body is exactly `is_subtype` is a synonym, and synonyms drift. If any call site needs more than
a mechanical substitution, that is STOP-2 — report it rather than inventing a shim.

## ⚠ It must accept BOTH spellings

After ②-iii, `extend-type`'s parent slot is a FORM — `(:wat::core::Seqable :- [T])` — not a keyword
with a `<…>` suffix. Base extraction must take the head from either. That is ②-iii blocker 3's
lattice half, and closing it here is the point.

## What "done" looks like

1. An edge registered from `(extend-type :Vector :Seqable<T>)` is found by a query for
   `:Seqable`, `:Seqable<T>`, and `:Seqable<?N>` alike.
2. The same, with the parent written as a FORM: `(extend-type :Vector (:Seqable :- [T]))`.
3. `transport_satisfier_heads` no longer guesses — one key, no `format!("{fq}<T>")`.
4. `satisfies_bare_surface`'s `format!("{surface}<")` prefix match is GONE.
5. `grep -rn "format!(\"{fq}<\|format!(\"{surface}<" src/` returns nothing.
6. ★ Arc 293's transport machinery still passes — `Handle<Wire>` satisfying a bare `Dialable`.
7. `is_subtype`'s 30 call sites are UNCHANGED — the signature does not move.

## Boundaries

- Do NOT run `scripts/floor.sh` or a full `cargo nextest` — the orchestrator measures centrally.
- Do NOT commit, push, stash, revert or amend. Leave everything in the working tree.
- Do NOT touch `defservice`, `wat/core.wat`, `bracket.wat` or `fix.wat` — those are stones 2 and 3.
- Do NOT consolidate the sixteen inline `find('<')` sites. Named, separate, not yours.
- Write no new base-extraction helper.

## Your own checks

`cargo build --bin wat`, then `target/debug/wat --check <file>` on files under
`wat-scripts/scratch-pad/`, and `cargo nextest run --release -E 'binary_id(wat::types)'` for a scoped
run. Diagnostics go to **stderr** — judge by exit code AND empty output, never grep alone. Prefix long
commands with `systemd-run --user --scope -q -p MemoryMax=16G -p MemorySwapMax=0 timeout 900`.

Delete any scratch `.wat` that must FAIL — `tests/lint/wat_scripts_fixes_load.rs` type-checks
everything under `wat-scripts/`.

## STOP triggers — ship nothing further and report

- **STOP-1.** If stripping the args makes arc 293's transport rows go red — `Handle<Wire>` satisfying
  a bare `Dialable`, `transport_satisfier_heads`' `<Xt>` guess — STOP and report WHICH rows with
  verbatim errors. **That is an anticipated outcome:** it would mean some edge genuinely needs an
  instantiation, and the builder rules on a NAMED exception rather than you inventing one.
- **STOP-2.** If deleting either helper needs more than a mechanical call-site substitution, STOP and
  report the site. Do not add a compatibility shim.
- **STOP-3.** If the base extraction cannot handle the FORM spelling without changing
  `parse_type_node` or the reader, STOP and report.

## Your report

The diff per file. Every done-row with verbatim output. Whether arc 293's transport rows moved. What
surprised you. Anything you inspected and left alone, with the reason.
