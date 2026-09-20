# SCORE — STONE 251.8d-ii: THE BOOTSTRAP — STOP

Branch: `main`. **No conversion committed.** Tree is `wat/` as drawn (`bcf703ed9` + the brief).
**Not pushed.** 8d-iii must not start.

## STOP — the converted stdlib does not load, and the cure is a checker change

The conversion itself is proven. Embedding it is not. The brief: *"If this cannot be done without
a checker change, STOP AND REPORT."* That is this SCORE.

### What the tool did (on copies, R21)

| | |
|---|---|
| path list | `git ls-files \| grep -E '^wat/.*\.wat$'` → **64** (the brief's `git ls-files 'wat/**/*.wat'` is **31** — git `**` does not match `wat/*.wat`; **the runner wins**) |
| `include_str!` | 64 files (rg -c counted 66 *lines*) |
| dry-run | `/tmp/8d-ii/tree`, one batch, **rc=0**, 1296 s |
| timed one file first | `wat/fix.wat` copy: **296.8 s**, rc=0, 1282 lines rewritten |
| all 64 changed | **11889 / 11889** line replacements, 0 unchanged |
| markers | `{:restricted-to [wat.spawn wat.test]}` in `spawn.wat` / `stdio.wat` |
| leftover `(:wat::` | 38 files, **comments only** (plus one comment-on-the-same-line in `rete/compile.wat`) |
| idempotence on the converted tree | second pass **0 content changes** (419 s, hashes match) |

No hand edits. Copies remain under `/tmp/8d-ii/tree`.

### What happened when it was applied

1. Copied tool output onto `wat/`. `git status`: **64 files, only `wat/`**.
2. `cargo build --release` **failed in `wat-doc`**: `wat_enum_from!` (`crates/wat-source-derive`)
   requires a Keyword head `:wat::core::defenum` and a Keyword name. Converted
   `runtime-meta.wat` is `(wat.core/defenum wat.runtime/Purity …)`.
3. A dual-spelling read in `wat-source-derive` (not committed) unblocked **compile**.
4. The **new binary would not start**. Runtime load:

```
#wat.type/MalformedDecl
  malformed :wat::core::defenum declaration: name must be a keyword; got symbol
  wat/core.wat:2125
  (wat.core/defenum wat.core/Option :- [T] wat.enum/Pure …)
```

`src/types.rs:parse_declared_name` (`4684`) matches **only** `WatAST::Keyword`, then stores the
**raw keyword** (`:wat::core::Option`) as the TypeEnv key (`4723: Ok((raw, Vec::new()))`).
A Symbol `wat.core/Option` is refused; even if accepted, it would be a **different key**.

That is a checker/registry change (declaration names + TypeEnv identity), not a spelling flip.
Not smuggled. **STOP.**

### Recovery — proven

```
git checkout -- wat/
git checkout -- crates/wat-source-derive/src/lib.rs
cargo build --release   # exit 0, 22.5 s
```

Working tree clean. The old stdlib is re-embedded. The tool works again.

## What 8d-ii does not decide

- Dual-key TypeEnv / `parse_declared_name` accepting `wat.core/Option`.
- `wat-source-derive` dual-spelling (compile-time sibling of the same gap).
- 8d-iii.

The 8d premise *"both spellings type-check"* holds for **corpus files the checker already
loads**. It does **not** hold for **stdlib `defenum`/`typealias` names** going through
`parse_declared_name`. That is the sixth brief error, and it is load-bearing.

## Walls

- Conversion census on copies: 64/64, second pass 0.
- Floor / workspace clippy / `census.sh --diff`: **not run** (nothing landed).
- Test-count delta: **0** (no commit).
- Do not push. Do not start 8d-iii.
