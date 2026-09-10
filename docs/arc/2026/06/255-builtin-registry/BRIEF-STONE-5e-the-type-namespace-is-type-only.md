# BRIEF — STONE ⑤-E: `:wat::type::` is a TYPE-ONLY namespace

Group **E** of `DESIGN-STONE-5-the-blanket-dies-closing-the-seven.md`. Small — about six lines in
one file — and it lands GREEN under the blanket; the deletion is the next stone.

## The gap stone ② left

Stone ② taught `normalize` that the HEAD of a `(Head :- [T …])` form may be a type, and gated that
on an `also_accept_type` flag passed only from the binder-head position. **A bare `wat.type/X`
symbol in an ANNOTATION position is not a binder head**, so it never reaches that acceptance:

```
tests/resolve/probe_arc251_stone2_type_namespace.wat
  (:wat::core::defn :user::inc [x <- wat.type/i64] -> wat.type/i64 (:wat::i64::+ x 1))
                                     ^^^^^^^^^^^^     ^^^^^^^^^^^^
```

These are `WatAST::Symbol`s in type positions. With the blanket gone they are refused by
`resolve_namespaced_symbol`, which asks `is_resolvable_call_head` — a CALL question — about a TYPE.

## The rule, and why it needs no position tracking

> **`:wat::type::` is a TYPE-ONLY namespace: a name under it is NEVER a call head, in any
> position.** So a `:wat::type::` candidate is asked the TYPE question — `TypeEnv::is_known_type`,
> stone ②'s one door, which canonicalizes `:wat::type::X` → `:wat::core::X` internally —
> regardless of `also_accept_type`.

**The namespace IS the position.** `normalize` carries no position context and does not need any.

## The measured population

Every bare (annotation-position) `wat.type/*` symbol in the corpus, and the door's verdict:

```
<- / -> wat.type/i64      is-type? true      ✓ accept
        wat.type/f64      is-type? true      ✓ accept
        wat.type/bool     is-type? true      ✓ accept
        wat.type/String   is-type? true      ✓ accept
        wat.type/Bogus    is-type? FALSE     ⛔ REFUSE — and it must stay refused
```

`wat.type/Bogus` is stone ②'s own negative fixture. The door discriminates it correctly today; this
change must not start accepting it.

⚠ `:wat::type::Infer` never appears in the dot spelling and `is-type?` answers **false** for it —
it is a type-position MARKER (`src/types.rs:74`), not a type. It lives only inside `:- [...]`
argument vectors, which stone ② exempts from validation entirely, so this rule never sees it.
**Do not add a special case for it.** If you find yourself wanting one, that is STOP-3.

## Read in order

1. `src/resolve/normalize.rs:~440` — `resolve_namespaced_symbol`, and stone ②'s
   `also_accept_type` acceptance directly below the call-head check. Your rung goes beside it.
2. `src/types.rs` — `TypeEnv::is_known_type`, the ONE DOOR. It already canonicalizes; do not
   re-implement the `:wat::type::` → `:wat::core::` mapping anywhere.
3. `tests/resolve/probe_arc255_the_type_position_has_its_own_authority.rs` — 13 rows from stone ②,
   including the `Bogus` fixtures. Read the header.

## Implementation sketch

Beside stone ②'s acceptance, NOT inside it:

```rust
// `:wat::type::` is TYPE-ONLY — a name under it is never a call head, in ANY position, so it is
// asked the TYPE question regardless of `also_accept_type`. The namespace IS the position.
if primary.starts_with(":wat::type::")
    && sym.types().is_some_and(|t| t.is_known_type(&primary))
{
    return Ok(WatAST::Keyword(primary, span.clone()));
}
```

On a miss it must FALL THROUGH to the existing `UnresolvedReference` — do not invent a new
diagnostic, and do not return early on failure.

## Blast radius

`src/resolve/normalize.rs` only. No new public API. No change to `is_known_type`, `walk.rs`, or the
registry.

## Acceptance

```
cargo nextest run --release -E 'test(probe_arc255_the_type_position)'     13 rows, all pass
cargo nextest run --release -E 'test(probe_arc251_stone2_type_namespace)' unchanged
target/release/wat --check tests/resolve/probe_arc251_stone2_type_namespace.wat   exit 0
```

★ **No row can show this stone "working" by going green** — the blanket still accepts these names
today, so every one of them already passes. The rows are CONTAINMENT: they prove the change did not
widen past its namespace. The stone's real proof is a measurement the orchestrator runs centrally —
the gate census losing `:wat::type::i64`.

## STOP triggers — each is a REJECTION. Ship nothing; report.

**STOP-1.** If `wat.type/Bogus` starts resolving — STOP. The door must still refuse an unknown name
under a known namespace; a namespace-wide accept would be a new blanket, one namespace narrower.

**STOP-2.** If you re-implement the `:wat::type::` → `:wat::core::` canonicalization — STOP.
`is_known_type` owns it, precisely so no caller can forget it.

**STOP-3.** If you find yourself special-casing `:wat::type::Infer` — STOP and report why you
needed to. It should never reach this rung.

**STOP-4.** If any of stone ②'s 13 rows changes verdict — STOP with the verbatim block.

**STOP-5.** If the change wants a second file — STOP and name it.

## Tier

You edit and report. Run the three acceptance commands, foreground, and nothing else. **Do NOT run
`scripts/floor.sh` or `cargo clippy`** — the orchestrator runs those centrally, once, on a quiescent
tree. **Do NOT commit.**

Work in `/home/john/work/holon/wat-rs`; verify with `pwd` first. Any path containing
`.claude/worktrees/` is harness state and must not be operated on.
