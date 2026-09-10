# BRIEF — ③a-ii: route the twenty-one variant sites through the door

Anchor: `/home/john/work/holon/wat-rs`. Verify with `pwd` first; any path containing
`.claude/worktrees/` is harness state and must not be operated on — use `git -C <anchor>` for git.

## The work, in one paragraph

A variant's fully-qualified name is composed and decomposed in exactly one place —
`crates/wat-reader/src/identifier.rs`'s `compose_variant` / `decompose_variant` pair. Twenty-one
sites still spell the `::` between an enum and its variant by hand. Repoint each one at the pair.
**Nothing observable changes**: `decompose_variant(n)` is exactly `Some((path(n), leaf(n)))` when
`n` contains `"::"` and `None` otherwise — and `path` returns `""` iff there is no `"::"`, which is
precisely when the door returns `None`. This is the same rewrite that landed on cluster 1
(`src/types.rs`'s `variant_parent_enum`, commit `b0c4eb7d5`) — read that diff first as the shape.

## The door

```rust
pub fn compose_variant(enum_path: &str, variant_name: &str) -> String
pub fn decompose_variant(name: &str) -> Option<(&str, &str)>   // (parent, variant_leaf)
```

## The rooms — read in order, edit in place

### PATTERN G — guard + `path` + `leaf` collapse to one call

`src/record/construct.rs:261,264,265` (`try_eval_enum_map_ctor`) — the guard and both accessors are
three spellings of one question in three lines:

```rust
    if !head.contains("::") { return None; }
    let type_path = wat_reader::identifier::path(head);
    let variant_name = wat_reader::identifier::leaf(head);
```
becomes
```rust
    let (type_path, variant_name) = wat_reader::identifier::decompose_variant(head)?;
```

`src/check.rs:13689,13692` (`literal_enum_variant_ctor`) and `src/check.rs:13896,13899` — the same
collapse; both functions already return `Option`, so `?` is in hand.

`src/closure_extract.rs:1589,1590` — an `if` block, so it becomes an `if let`; only the parent is
used, so bind the leaf as `_`:

```rust
    if let Some((enum_part, _)) = wat_reader::identifier::decompose_variant(name) {
        if let Some(TypeDef::Enum(_)) = state.parent_types.get(enum_part) {
```

### PATTERN R — `rsplit_once("::")` becomes the door

Nine sites, each binding an enum path and a variant leaf. Swap the call, keep the surrounding
`if let` / `match` exactly as it stands:

```
src/rete/expr_ir.rs:818    src/rete/validate.rs:1227    src/rete/validate.rs:1483
src/rete/purity.rs:804     src/check.rs:6881            src/check.rs:7007
src/check.rs:7179          src/check.rs:7436            src/check.rs:7682
```

`head.rsplit_once("::")` → `wat_reader::identifier::decompose_variant(head)`.

### PATTERN P — the separator used AS a predicate

`src/match_arm.rs:134` (`is_namespaced_variant`): `path.contains("::")` →
`wat_reader::identifier::decompose_variant(path).is_some()`.

`src/rete/expr_ir.rs:604`: `k.starts_with(':') && k.contains("::")` →
`k.starts_with(':') && wat_reader::identifier::decompose_variant(k).is_some()`.

### PATTERN M — the mixed site

`src/rete/expr_ir.rs:1321` already composes through the door on the line above and then decomposes
with the **general** accessor. Both halves must be the variant pair:

```rust
    let last = wat_reader::identifier::decompose_variant(name)
        .map_or(name, |(_, v)| v)
        .trim_start_matches(':');
```

`leaf(name)` returns the whole name when there is no `"::"`; `map_or(name, …)` reproduces that
exactly.

### PATTERN C — five hand-rolled compositions the door missed

```
src/runtime.rs:3196   format!("{}::Op::{}", protocol_fqdn, variant)
      -> wat_reader::identifier::compose_variant(&format!("{protocol_fqdn}::Op"), &variant)
src/runtime.rs:3198   format!("{}::Reply::{}", protocol_fqdn, variant)
      -> wat_reader::identifier::compose_variant(&format!("{protocol_fqdn}::Reply"), &variant)
src/runtime.rs:8638   format!(":{}::{}", fv.enum_class, fv.variant)
      -> wat_reader::identifier::compose_variant(&format!(":{}", fv.enum_class), &fv.variant)
src/check.rs:4837     Some(format!("{enum_path}::{variant_leaf}"))
      -> Some(wat_reader::identifier::compose_variant(enum_path, variant_leaf))
crates/wat-doc/src/print.rs:128   wat_fqdn_to_edn_keyword(&format!("{wat_type_path}::{variant}"))
      -> wat_fqdn_to_edn_keyword(&wat_reader::identifier::compose_variant(wat_type_path, variant))
```

`crates/wat-doc` already depends on `wat-reader` (`Cargo.toml:17`). The enum's own type path —
`{protocol}::Op` in rows 1 and 2 — is the PARENT, and its `::` is a namespace separator that stays.

## Blast radius

Those 21 sites and nothing else. **No test file changes. No new types. No signature changes.**
`identifier::path` and `identifier::leaf` are the GENERAL namespace accessors — every other caller
of them is correct and must stay untouched; they split every namespaced name in the substrate.

## STOP triggers — each REJECTS the stone; ship nothing and report

**STOP-1.** A test expectation moves. A moved expectation is a **finding**, not something to
update: it means the rewrite was not behaviour-preserving on some input, and the input is the
deliverable. Report the exact test, the arm that fired, and its whole verbatim stdout+stderr block.

**STOP-2.** Any site above does not have the shape this brief describes. Report the verbatim lines;
do not adapt the sketch to fit.

**STOP-3.** A borrow/lifetime error forces a `.to_string()` where the old code held a `&str`.
Report it — the door returns borrows precisely so it can be a drop-in, and a forced allocation
means one of my line numbers points at something else.

**STOP-4.** You find a **22nd** hand-rolled variant separator while working. Report it; do not fix
it. The census is mine to close, and a site added quietly is a site nobody counted.

## What to run

`cargo build --release` and, per file you touch, a scoped check such as
`cargo nextest run --release -E 'binary_id(wat::rete)'`. **Do not run `scripts/floor.sh` and do not
run clippy** — the orchestrator measures those centrally, once, on a quiescent tree. Run every
command in the FOREGROUND and block on it; your turn ends when the numbers are in your hands, not
when a command is launched. Do not commit. Do not contact any peer.

## Report

The diff per row; anything that surprised you; and for each of the five PATTERN C rows, the
before/after string the site now produces, so the orchestrator can confirm they are byte-identical
today.
