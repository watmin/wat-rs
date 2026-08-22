# BRIEF — identity 2b: give each ROLE its own binding

Source of truth: `TABLE-defservice-type-name-sites.md`. DESIGN:
`DESIGN-STONE-the-angle-string-is-not-a-type-identity.md`.

## ⚠ FIRST — the table's line numbers are STALE

Stone **2a** deleted six dead bindings from `wat/service.wat` (13 lines, at the old
`:868 :888 :936 :944 :1889 :2594`). **Every line number in the table below those points has
shifted up.** Re-locate every site **by binding name**, never by the recorded line. If you find a
site by line number and it does not contain the name the table says, the table is right and the
number is stale.

## The work, in one paragraph

Seven bindings in `defservice` are each consumed in more than one ROLE. One binding cannot serve two
roles once the roles need different NODE SHAPES, so give each role its own binding. **This stone
changes no emitted output** — every new binding holds exactly what the single binding holds today.
It is preparation: 2c then changes only the ANNOTATION ones.

## The seven

| binding | roles | (stale) evidence sites |
|---|---|---|
| `service-op-decl-kw` | DECL-NAME + RUNTIME-ARG | `defenum` name; `retag-op` arg |
| `state-ty` | DECL-NAME + ANNOTATION | `defstruct` name; 5 param/return/field types |
| `record-ty` | DECL-NAME + ANNOTATION | `defrecord` name; 6 annotations |
| `admin-ty` | DECL-NAME + ANNOTATION + RUNTIME-ARG | `defenum` name; param type; `self-peer` |
| `status-ty` | DECL-NAME + ANNOTATION + RUNTIME-ARG | `defenum` name; param type; `self-peer` |
| `handle-name` | DECL-NAME + ANNOTATION | `defstruct` name; 2 return types |
| `selectable-entry-ty` | ANNOTATION + CTOR-ARG | fn param type; `Vector`'s element-type arg |

★ `admin-ty` and `status-ty` are the sharp pair — three roles each, and both reach RUNTIME-ARG
through the **same** `(:wat::program::self-peer ~status-ty ~admin-ty)` call.

## The three shapes each role will eventually need — context, NOT this stone's work

```
ANNOTATION   ONE node, a reference form         (Head :- [args])
DECL-NAME    a base keyword + a SPLICED binder  (defenum ~base ~@binder …)
RUNTIME-ARG  the keyword, unchanged
```

The DECL-NAME shape already has a proven exemplar in the tree — `wat/Record.wat:195`:

```clojure
(:wat::core::recordtype ~fqdn ~@binder :wat::core::Record [~@field-ch])
```

`binder` there is a vector of nodes, empty when monomorphic, spliced with `~@`. **Copy that shape
when 2c/2d arrive. Do not build it now.**

⚠ **Only ANNOTATION's destination is RULED.** What DECL-NAME and RUNTIME-ARG eventually become is
**not yet decided** — that is a later ruling, and this brief deliberately does not anticipate it.

## What "done" looks like

1. Each of the seven bindings is replaced by one binding **per role it serves**, named so the role is
   readable at the use site (e.g. `admin-ty-decl` / `admin-ty-ann` / `admin-ty-runtime`).
2. Every use site references the binding for **its** role.
3. ★ **Every new binding is initialised to exactly what the original held.** Byte-identical
   expressions. No conversion, no `keyword/to-type-form-colon`, no shape change.
4. `macroexpand` of a representative `defservice` is **byte-identical** before and after. This is the
   acceptance row that matters — see below.
5. The floor is unchanged at **4854/4854**.

## ★ THE ACCEPTANCE ROW — a before/after expansion diff

This stone is behaviour-neutral **by construction**, so the honest proof is that the generated code
does not move:

```
BEFORE (stash nothing — build from HEAD):  macroexpand a defservice form, save the output
AFTER:                                      the same, and `diff` them
```

`wat/cache.wat`'s `lru-svc` (parametric, `:- [K V]`) and one monomorphic service both. **The diff
must be empty.** A green floor alone does not prove this — the floor would also be green if you had
quietly converted something the tests do not reach.

⚠ `wat/service.wat` is the stdlib, baked in by `include_str!` at RUST-compile time, so a
`cargo build --bin wat` is required between the edit and any `macroexpand`. That is why the BEFORE
capture must happen **first**.

## Boundaries

- Do NOT run `scripts/floor.sh` or a full `cargo nextest` — the orchestrator measures centrally.
- Do NOT commit, push, stash, revert or amend. Leave everything in the working tree.
- Do NOT convert any node. No `keyword/to-type-form-colon`. No `:-` anywhere new.
- Touch `wat/service.wat` only.
- Do not "tidy" adjacent bindings — the single-role ones stay exactly as they are.

## Your own checks

`cargo build --bin wat`, then `target/debug/wat` running a small `macroexpand` script. Prefix long
commands with `systemd-run --user --scope -q -p MemoryMax=16G -p MemorySwapMax=0 timeout 900`.
Diagnostics go to **stderr** — judge by exit code AND empty output, never grep alone.

## STOP triggers — ship nothing further and report

- **STOP-1.** If the before/after expansion diff is NOT empty, STOP and report the diff verbatim.
  Non-empty means the split changed what is emitted, which this stone must not do.
- **STOP-2.** If a use site's role is ambiguous — it reads as neither the annotation, the declaration
  name, nor a runtime argument — STOP and report it. The table has an `OTHER` column for a reason,
  and a guessed role becomes a wrong conversion in 2c.
- **STOP-3.** If splitting a binding requires touching anything outside `wat/service.wat`, STOP.

## Your report

The diff. The before/after expansion diff for both services (state plainly that it is empty, or paste
it). Which sites you re-located by name because the table's line number was stale. Any role you found
ambiguous. What surprised you.
