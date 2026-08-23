# BRIEF — `<K,V>` unexpressible ANYWHERE, minted names included

**The wall is already in the working tree, uncommitted, and the tree is RED by construction — 3034 of
4893.** That is the deliverable of the wall, not a problem with it: the builder's acceptance criterion
is *"defservice must blow up with the appropriate error … unless those callers fail on illegal syntax,
we've failed."* They fail. Your job is the repair that makes them stop needing to.

Read `DESIGN-STONE-the-minted-name-is-walled-too.md` first — it carries every measurement below.
Copy the report shape of `SCORE-STONE-the-last-comma-lives-in-a-symbol.md`.

⚠ **Work against the DIRTY tree. Do NOT revert, stash, or rebuild from main.** The wall in
`src/runtime.rs` + `src/edn_shim.rs` is your foundation. This is the atomic-commit pattern: the
orchestrator commits wall and repair together once the floor is green.

## STEP 1 — the probe that licenses everything else. Do this FIRST.

The repair below rests on one claim: **the explicit type application `defservice` mints is inert, and
inference already does the whole job.** Three measurements agree (all in the DESIGN), the strongest
being: discard every successfully-parsed type argument and the floor stays **4893/4893 green**.

**Green proves nothing contradicted it.** So before you delete anything:

> **Try to construct a program where explicit type application and inference DIVERGE** — where binding
> a surface method's type params from the minted `<…>` suffix gives a different answer (or an error)
> than letting inference find them. A parametric `defservice` whose usage contradicts its declared
> `Op`/`Reply` is the obvious shape to attempt.

- **If you can build one:** STOP. Report it with the program and both answers. The repair is then a
  different stone and the wall must wait.
- **If you cannot:** say exactly what you tried and why each attempt collapsed. That negative result
  is what licenses the deletion, and it belongs in your report as the load-bearing finding.

## STEP 2 — the minting sites stop building angle names

The waterfall masks all but the first. Fix, rebuild, read the next. Known from the census:

```
wat/service.wat:942     Cache::Op<K,V> / Cache::Reply<K,V>   ← the first scream
wat/service.wat:2375    wat::spawn::Locus/launch<…5 args…>
wat/core.wat            :wat::core::keyword/of — a macro whose whole job is minting this
                        (it has ONE caller: tests/macros/probe_arc249_4_rehome_in_wat_kw_of_tmpl.wat)
```

Per step 1's measurement the replacement is **the bare name** — `Locus/launch`, not
`Locus/launch<…>` and not a type-application form. Nothing consumes the suffix that inference does not
already supply.

⚠ `wat/service.wat:502`'s `proto-tp` builds the `<…>` suffix string that several of these splice.
Its own comment states **the identity property**: a monomorphic surface has `proto-tp` = `""` and
`proto-base` IS the name. Read that comment before editing — the monomorphic path is already the shape
you are extending to every path.

⚠ **A stdlib `.wat` edit is INVISIBLE until you rebuild** (`include_str!` at Rust-compile time).

## STEP 3 — the controls

A negative control proving the refusal and a positive twin proving what survives. Both halves, or the
wall is unfalsifiable:

| | what | expected |
|---|---|---|
| 1★★ | `(keyword/from-string "my::Map<K,V>")` | ⛔ refused, message names `:-` |
| 2★★ | `(keyword-node ":Vec<T>")` | ⛔ refused |
| 3★★★ | `(keyword/from-string "wat::core::i64::<")` | ✅ minted — the OPERATOR survives |
| 4★★★ | `(keyword/from-string "foo/bar")` · `"wat::kernel::Peer'"` | ✅ minted |
| 5★★ | a `defservice` still expands, checks, and DISPATCHES | ✅ a value comes back |

**Rows 3 and 4 decide it.** Rows 1–2 go green for a wall that refuses every minted name — which would
take the whole stdlib with it. Only operator and ordinary names still minting proves the predicate
matched the type-head and nothing else.

## STOP triggers

- **STOP-1 — step 1 finds a divergence.** Report it and ship nothing.
- **STOP-2 — a minting site cannot emit a bare name** because something downstream genuinely needs the
  suffix. That contradicts step 1's measurement; report the site and what needs it.
- **STOP-3 — the waterfall stops falling.** If the failure count plateaus above zero on a cause that is
  not a minting site, report the arm verbatim; something other than minting is in the path.
- **STOP-4 — you find yourself widening the wall's predicate** to let a stdlib name through. The
  predicate is the lexer's own; if it refuses something the lexer accepts, that is a real finding —
  report it rather than loosening it here.

## Boundaries

- `wat/service.wat`, `wat/core.wat`, whatever the waterfall names, and new controls under `tests/`.
- **Do NOT touch the wall's predicate** (`angle_type_head_in_name`) or its message
  (`angle_minted_name_reason`) in `src/runtime.rs` — that is the specification; the callers move.
- **Do NOT delete the angle PARSERS** — `split_type_params`, `split_name_and_type_params`,
  `split_method_name_type_params`, `canonical_callable_name`, `check.rs:5159`'s arm. They are dead
  once nothing mints, but proving that needs a green floor first. Sibling stone.
- **Do NOT retire `keyword/of`** — same reason; it goes with the purge.
- Do NOT commit, push, stash or amend. Keep the git index EMPTY: no `git add`, no
  `git checkout <ref> -- <path>` (it STAGES).
- The orchestrator runs the full floor and clippy centrally. Use `./target/release/wat --check <file>`
  (~0.2s) and scoped `cargo nextest run --release -E '...'`.

Build with `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 timeout 3000 cargo build --release`.
Read exit codes DIRECTLY — never through a pipe, never after a trailing `; echo`.
`cargo wat` uses the STALE installed binary; always use `./target/release/wat`.

## Your report

**Step 1 first and at length** — what you tried to make diverge, and what happened. Then: the minting
sites the waterfall surfaced, in the order it surfaced them, and what each became. Rows 1–5 verbatim
in one run, refusals and survivals together. Any STOP that fired, with the arm captured verbatim
BEFORE you diagnosed it. What surprised you.
