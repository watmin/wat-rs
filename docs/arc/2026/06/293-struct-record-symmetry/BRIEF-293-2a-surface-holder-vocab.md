# BRIEF — 293 item-2a: surface `:holder` takes the holder-root symbol; the `user-env` portable surface

**The work, in one paragraph.** A surface's `:holder` bound currently hand-matches three **magic shorthand**
keywords (`:struct` / `:record` / `:holon-record`, `surface.rs:322`) — a second, redundant spelling of the holder,
parallel to the real holder-root symbols (`:wat::core::Struct` / `:wat::Record` / `:wat::holon::Record`). Annihilate
the shorthand: add **`Holder::from_root_keyword()`** (the strict inverse of the existing `root_keyword()`), route the
`:holder` parser through it so it takes the **holder-root symbol**, and — because `from_root_keyword` is now THE
canonical reverse map — collapse the two *other* reverse maps into it (`root_holder_of` and the `HOLDER_ROOTS` const
guard the inheritance strike added). Then mint the first real consumer: a 0-member `:holder` surface
`:wat::spawn::user-env` ("any portable aggregate"), and retype `program::Env.user.program` from the bare
`:wat::Record` to it (the field ships to spawned peers → it must be ≥ a record, not a struct, not any `Value`).

## THE ONE CONTRACT DECISION (pinned)
`Holder::from_root_keyword(kw: &str) -> Option<Holder>` is the **single** keyword→holder map (strict inverse of
`root_keyword`): `:wat::core::Struct`→`Some(Struct)`, `:wat::Record`→`Some(Record)`, `:wat::holon::Record`→
`Some(HolonRecord)`, anything else→`None`. EVERY site that turns a holder-root keyword into a `Holder` calls it. The
surface `:holder` magic shorthand and the loose `root_holder_of` `_=>Record` arm both DIE.

> **NOTE on the symbol:** the record holder-root is **`:wat::Record`** (the current name). The `:wat::Record →
> :wat::core::Record` rename is the REST of item 5 — explicitly OUT of scope here. Use `:wat::Record`.

## Read in order (rooms — grounded this session)
1. **`src/types.rs:136` `impl Holder`** (`root_keyword` is at :142-147) — ADD `pub fn from_root_keyword(kw: &str) ->
   Option<Holder>` immediately after `root_keyword`, the strict inverse per the contract above.
2. **`src/types/surface.rs:317-349`** — the `:holder` parse. Replace the `match v.as_str() { ":struct" => …,
   ":record" => …, ":holon-record" => … }` block with `Holder::from_root_keyword(v)`; on `None`, return the existing
   `MalformedDecl` shape with reason: `":holder value must be a holder-root symbol (:wat::core::Struct, :wat::Record,
   or :wat::holon::Record); got {v}"`. (Keep the outer "must be a keyword" arm for a non-keyword node.)
3. **`src/types.rs:2124 root_holder_of`** + **`~2196-2211`** (the `let holder = root_holder_of(&parent);` line and the
   `const HOLDER_ROOTS … if !HOLDER_ROOTS.contains(…) { return Err(inheritance-reject) }` guard the inheritance strike
   added) — collapse BOTH into one: `let holder = Holder::from_root_keyword(&parent).ok_or_else(|| TypeError { span:
   decl_span.clone(), kind: TypeErrorKind::MalformedDecl { head: head.into(), reason: format!("parent '{}' is not a
   holder-root; inheritance is unsupported — reuse a shape via surface-splice `[~@:Surface …]`", parent) } })?;`
   Then DELETE the `root_holder_of` fn (its only caller is gone) and the `HOLDER_ROOTS` const. (Net: the inheritance
   rejection + the holder derivation are now ONE `from_root_keyword` call — no separate guard, no `_=>Record` leak.)
4. **`tests/types/probe_arc293_holder_bound_accept.wat:7` + `probe_arc293_holder_bound_reject.wat:7`** — migrate
   `:holder :holon-record` → `:holder :wat::holon::Record`. Update the prose mentions of `:holder :holon-record` in
   `tests/types/probe_arc293_holder_bound.rs` (comments only) to the new symbol.
5. **`wat/program.wat`** — (a) BEFORE the `:wat::program::Env` defrecord (line 39), declare the surface:
   `(:wat::core::defsurface :wat::spawn::user-env :holder :wat::Record [])`; (b) change line 46
   `user.program <- :wat::Record` → `user.program <- :wat::spawn::user-env`. Update the file's top comment to mention
   the portable-env surface.

## Implementation sketch
- Add `from_root_keyword` → route surface (room 2) + parse_aggregate (room 3) through it → delete the two dead reverse
  maps. `cargo build`. Migrate the 2 `:holder` fixtures (room 4). Add the surface + retype the field (room 5).
- The new RED probe `tests/types/probe_arc293_holder_root_symbol.{rs,wat}` (+ `_bad.wat`) is already committed-RED and
  goes GREEN when room 2 lands.

## Gate (EXPECTATIONS — fixed before the strike)
| what | command | expected |
|---|---|---|
| holder-root-symbol probe GREEN | `cargo nextest run --release -p wat surface_holder_root_symbol` | 2 passed |
| holder-bound fixtures still green | `cargo nextest run --release -p wat probe_arc293_holder_bound` | all pass |
| program-env intact (the consumer) | `cargo nextest run --release -p wat arc259 arc258 arc211` | all pass |
| magic shorthand gone | `grep -n '":record"\|":holon-record"\|":struct" *=>' src/types/surface.rs` | no holder-value matches |
| whole gate, floor 0 | `cargo nextest run --release` | `0 failed` |

Runtime estimate: 15–25 min. Trap-door: the default `user.program` value satisfying the new surface (STOP-DEFAULT-ENV).

## STOP triggers (rejection criteria — surface, do not improvise)
- **STOP-DEFAULT-ENV:** the default `user.program` (the `EmptyEnv` that `(thread)` installs, and the value the
  spawn-injection at `src/kernel/spawn.rs:613` binds) MUST satisfy `:wat::spawn::user-env` — i.e. it must be a Record
  (or holon). If an `arc259`/`arc211` program-env test breaks because the default env isn't a record, STOP and report
  exactly what the default `user.program` value is — do NOT loosen the surface to fix it.
- **STOP-OTHER-MAGIC:** touch ONLY the surface `:holder` site and the `parse_aggregate` reverse-map. Do NOT touch the
  other holder-keyword sites (`value.rs:1120`, `runtime.rs:6705`, `observe.rs:326`) or the `:wat::Record` →
  `:wat::core::Record` rename — those are the rest of item 5, out of scope.
- **STOP-SURFACE-LOAD:** `:wat::spawn::user-env` must be declared BEFORE `program::Env` references it (same file,
  earlier line). If load-order rejects it, STOP and report — do not move it to a later-loading file.

## Blast radius (bounded)
`src/types.rs` (add `from_root_keyword`, delete `root_holder_of` + `HOLDER_ROOTS`, one call-site rewrite),
`src/types/surface.rs` (the `:holder` match), 2 `:holder` test fixtures + 1 `.rs` comment, `wat/program.wat`
(1 surface decl + 1 field retype + comment). No new types beyond the surface. The other 4 magic sites untouched.

## You are a LEAF
Do NOT spawn subagents. Work only in `/home/watmin/work/holon/wat-rs/`. Verify `pwd` first; any `.claude/worktrees/`
path is illegal. Use `cargo nextest run` (NEVER `cargo test`). Do NOT commit. If the strike exceeds this brief, STOP
and report.

## Pairs
`AGGREGATE-MODEL.md` §6 (the `:holder` bound is a holder-root keyword, not a magic symbol) ·
`tests/types/probe_arc293_holder_root_symbol.{rs,wat}` (the RED gate) · `CLOSE-SEQUENCE-293-294.md` item 2a.
