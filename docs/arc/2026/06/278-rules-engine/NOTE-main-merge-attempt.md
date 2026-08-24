# NOTE — merging `origin/main` into `grok-rete`: attempted, backed out, what blocks it

**2026-08-24.** Attempted at the builder's direction, aborted cleanly, branch
returned to `ce6ab4319` (in sync with origin, builds, cohort 363/363). Nothing
was pushed. This note exists so the next attempt starts from what was learned
rather than re-deriving it.

## The shape

`origin/main` is **170 commits** ahead. Its top commit is
`curare: E is BLOCKED on the rete merge`, so the merge is expected from both
sides. Those commits are arc 109, *"annihilate the angle bracket"*:

```
retired:   :wat::core::PersistentVector<wat::core::i64>
surviving: (:wat::core::PersistentVector :- [:wat::core::i64])
```

`git merge origin/main` produces **12 conflicts**. The builder's read — that the
contention is only this syntax change — is very nearly right; the exceptions are
listed below and all resolve in our favour.

## The 12 conflicts and how they resolve

Verified against the merge base (`fe7700f0e`) in every case, so "who changed
this" is fact, not guess.

| file | resolution | why |
|---|---|---|
| `src/rete/kernel/session.rs` | **ours** | WE removed `node_kind_label`/`node_record` and added `agg_named_field`; main never touched them. Adjacency artifact. |
| `src/rete/kernel/tests.rs` | ours **+ main's rune** | Same code both sides; main added `// rune:lint(no-angle-type-in-diagnostic)`, which the new lint needs. |
| `src/runtime.rs` | our axis model, main's syntax | Comment only. Ours says `:wat::rete::Axis` (the 4-conjunct fence); main still says `:pure\|:deterministic`, the older 2-axis model. |
| `src/value/value.rs` | **ours — semantic** | Ours is `PVec` (`DESIGN-STONE-promoting-vector`); main is still `rpds::VectorSync`. Ours is newer. |
| `wat/rete/acc.wat` | ours + main's syntax on `els` | Ours introduced the `:wat::rete::GroupByMap` alias; main expanded the inline type. A named type beats an expanded one. |
| `wat/rete/compile.wat` | **ours** | Base had `span <- Option<Location>`; WE dropped the `Option`. Already valid new syntax. |
| `wat/rete/oracle/pass.wat` | main's syntax + our rune | Pure syntax; our `rune:perspicere` comment must survive. |
| `wat/rete.wat` (5 hunks) | **ours** | Includes removing `binding-keys` from two node records — ours, deliberately: base had it, we removed it, and no Rust on our side reads it (the only matches are a test *function name*). |
| `wat/rete/oracle/fire.wat` (6 hunks) | **ours** | Newer engine. |
| `probe_arc278_4a` / `4b` | **ours**, main's comment is stale | We replaced flatten-production-memory with a query-based harvest and document it in the line below; main's comment describes the approach we removed. |
| `probe_arc278_P12c` | ours + main's syntax | We refactored three functions onto shared helpers; main kept the long inline forms. |

⚠ **Do NOT resolve the `.wat` conflicts with `git checkout --ours`.** That takes
the WHOLE file and discards main's auto-merged migration of every line that did
not conflict — it turns 17 hunks into 24 whole files needing migration. Resolve
hunk by hunk. (Learned the expensive way.)

## What actually blocks it: a bootstrap cycle the STASH-DANCE does not cover

After resolving, `cargo build` fails: a proc macro (`wat_field_names_from!`)
parses `wat/rete.wat` **at compile time**, and the new lexer refuses turbofish.
So the corpus must be migrated before the binary exists — and
`wat-scripts/fixes/angle-brackets-to-binder.wat`, the recorded codemod for
exactly this, needs a working binary to run.

`wat/fix.wat`'s BOOTSTRAP note anticipates this and prescribes the STASH-DANCE:
revert the Rust wall, build, run the codemod, restore. Four things were tried:

1. **Run the codemod with the pre-merge binary.** No — the codemod is written in
   main's new syntax (`[arg :-> ret]`), which the old lexer cannot read.
2. **Lift the lexer walls** (`AngleTypeHeadInName` ×2, `CommaInSymbolBody`,
   `CommaInKeywordBody`) and rebuild. Gets further — compiles — but startup then
   fails in `src/types.rs`, the second door.
3. **Freeze main's already-migrated stdlib** so the binary can start, then run
   the codemod on our files. Fails: our branch MOVED `StratifyAcc` from
   `oracle/fire.wat` into `oracle/stratify.wat`, so main's `fire.wat` plus our
   `stratify.wat` is a duplicate type declaration. And `stratify.wat` is
   ours-only, so main has no migrated copy of it to borrow.
4. **Lift the `types.rs` door too.** This is where it genuinely breaks:
   **81 type-check errors**, all of the form

   ```
   expects :wat::core::HashMap<wat::core::String,wat::core::i64>
   got     (:wat::core::HashMap :- [:wat::core::String :wat::core::i64])
   ```

### The crux, stated precisely

Pre-arc-109, `Head<args>` lexed as ONE keyword that the type parser then **split**
into head + args — making it *equivalent* to `(Head :- [args])`. Arc 109 removed
the rejection **and** the splitting. Lifting only the rejection lets turbofish
parse as a **plain path name**, which is a *different type* from the binder form.

A half-migrated corpus therefore cannot type-check under a walls-lifted binary:
every call across the boundary is a mismatch. **The bootstrap needs the old
SPLITTING logic restored, not merely the rejection removed** — a real temporary
restoration from `src/types.rs` at the merge base, not a one-line guard flip.

## Options for the next attempt

- **A — restore the splitter.** Temporarily reinstate the pre-arc-109 `Head<args>`
  split in `src/types.rs` (from the merge base) alongside the lifted lexer walls,
  build, run the codemod over the whole corpus, restore both files, rebuild, gate.
  Most faithful to the documented dance. Bounded, but it means temporarily
  un-doing a language wall, so it wants the builder's explicit say-so.
- **B — migrate rete's `.wat` on main's side first**, then merge. Sidesteps the
  cycle entirely: main already has a working migrated toolchain, and rete's `.wat`
  is what it is blocked on.
- **C — ask whether arc 109 already has a rete plan.** Main's own commit says E is
  blocked on this merge; the people who wrote the codemod may already intend a
  specific path for rete and know which of A/B they want.

Everything above is reproducible from `ce6ab4319` + `origin/main`; nothing is
lost by having backed out.
