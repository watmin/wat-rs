# BRIEF — the last comma lives in a SYMBOL

⚠ **The turbofish is already dead.** `575f8fb08` killed it — `Mk/mk<i64,i64>` is a keyword whose body
carries a comma and no longer lexes; its probe is now the `.wat.bad` proving the refusal. **Do not
re-fight it.**

What survived the comma strike is the **symbol** path. `mk<S,R>` in a `defsurface :features` method
declaration is a SYMBOL, not a keyword, and the wall was keyword-scoped. Measured on the current
build: it lexes, registers as bare `mk`, and dispatches (returns 7).

**This is the last comma-bearing construct in the language.**

## The population — FOUR sites, two of them stdlib

```
wat/spawn.wat:383    (launch<S,R,St,Sh,Lu> [self <- :wat::spawn::Locus …
wat/spawn.wat:409    (spawn-runner<D,I,O,W> [self <- :wat::spawn::Locus …
tests/program/probe_arc170_edn_bridge_unspellable__lexemes.wat:27   (mk<S,R> …
tests/types/probe_arc271_multi_param_generic_method.wat:6           (combine<A,B> …
```

## The work — a wall and a door, same shape as every stone this arc

**1. The wall.** `crates/wat-reader`'s lexer tracks angle-depth for symbols the same way it did for
keywords; the comma strike walled the keyword path only. Wall the symbol path too, with the same
message. ⚠ Find it yourself and confirm it is the *symbol* branch — `crates/wat-edn` has its own
independent lexer and `src/lexer.rs` is a bare re-export of `wat-reader`. **A previous brief named
one reader and the corpus is compiled by the other; that cost a whole round.**

**2. The door.** `src/types/surface.rs:193`, `split_method_name_type_params` — it reads `name<A,B>`
by `name.find('<')`. Teach it the binder instead:

```clojure
(launch<S,R,St,Sh,Lu> [args] -> ret)      ;; before
(launch :- [S R St Sh Lu] [args] -> ret)  ;; after — siblings, NO parens
```

That is **γ-i's shape**, which taught `defn`/`fn` the same binder — copy it. The data model is
already there: `SurfaceMember::Method` carries `type_params: Vec<String>`.

**3. Migrate the four sites.** Four is a hand edit, not a codemod — R21's threshold is 10+.

## STOP triggers

- **STOP-1 — if `:features` cannot express the binder** after you teach the door, STOP and report the
  exact form you tried. A method-member grammar that cannot take a param-spec is a substrate finding.
- **STOP-2 — if the symbol wall breaks a legitimate symbol.** `<` `>` `/` `'` are all legal symbol
  characters (`is_symbol_continue`) — you are refusing the COMMA only, not the brackets.
- **STOP-3 — if a fifth site appears.** Fix it if it is the identical shape; report anything else.

## Acceptance

| # | what | expected |
|---|---|---|
| 1★ | the binder is accepted | `(launch :- [S R St Sh Lu] [args] -> ret)` parses; the method registers under bare `launch` |
| 2★★ | the comma is refused in a symbol | `(mk<S,R> …)` → lex error naming the comma |
| 3★★ | a legal symbol still lexes | `:wat::kernel::Peer'`, `foo/bar`, `a<b` — untouched |
| 4★ | dispatch still works | a `defsurface` + `extend-type` + call round-trip returns its value |
| 5 | the floor | `scripts/floor.sh` green |
| 6 | clippy | 0 under `-D warnings` |

**Row 3 decides it.** Row 2 goes green for a lexer that refuses `<` or `>` or every symbol. Only
ordinary symbols still lexing proves you refused the **comma** and nothing else — the same pairing
that made the keyword strike meaningful.

## Boundaries

- `crates/wat-reader/` (the symbol branch), `src/types/surface.rs` (the door), and the four sites.
- Do NOT touch the keyword wall or the `.wat.bad` shipped in `575f8fb08`.
- `scripts/floor.sh` IS allowed — it is row 5.
- Do NOT commit, push, stash or amend. Keep the index EMPTY: no `git add`, no
  `git checkout <ref> -- <path>` (it STAGES).

Prefix long commands with `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 timeout 3000`.
Read exit codes DIRECTLY — never through a pipe, never after a trailing `; echo`.

## Your report

Rows 2 and 3 verbatim, together. Which reader/branch you walled and how you confirmed it was the one
the corpus uses. The four migrated sites. Whether `:features` took the binder or hit STOP-1. The
floor's Summary line. What surprised you.
