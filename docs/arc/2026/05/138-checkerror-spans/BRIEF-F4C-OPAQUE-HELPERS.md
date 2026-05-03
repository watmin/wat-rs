# Arc 138 F4c — Sonnet Brief: opaque-cell helpers gain span

**Goal:** add `span: Span` parameter to 5 opaque/cell helpers in src/rust_deps/marshal.rs (rust_opaque_arc, ThreadOwnedCell::ensure_owner, ThreadOwnedCell::with_mut, OwnedMoveCell::take, downcast_ref_opaque). Update ~10 caller sites in src/io.rs (×6) and crates/wat-telemetry-sqlite/src/auto.rs (×4). Internal test callers in marshal.rs use `Span::unknown()` (synthetic test context — fine).

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

**Driver:** the user invoked the no-deferrals rule. F4c is sub-item 3 of 3 in F4 decomposition (final sub-crack). After F4c ships, all four cracks (F1-F4) are closed.

## Read in order

1. `docs/arc/2026/05/138-checkerror-spans/CRACKS-AUDIT.md` — F4 decomposition.
2. `docs/arc/2026/05/138-checkerror-spans/SCORE-F4B-FROMWAT.md` — predecessor identifying these 7 leftover Pattern E sites.
3. `docs/arc/2026/05/138-checkerror-spans/SCORE-F4A-VALUE-HELPERS.md` — same value-helper-with-span pattern.
4. `src/rust_deps/marshal.rs` lines 355-540 (the 5 helpers).
5. `src/io.rs` (search for `with_mut(`) — ~6 caller sites.
6. `crates/wat-telemetry-sqlite/src/auto.rs` lines 267-404 — ~4 caller sites.

## What to do

### 1. Helper signature updates (src/rust_deps/marshal.rs)

```rust
pub fn rust_opaque_arc(
    v: &Value,
    type_path: &'static str,
    op: &'static str,
    span: Span,  // NEW
) -> Result<Arc<dyn Any + Send + Sync>, RuntimeError> { ... }

impl ThreadOwnedCell<T> {
    fn ensure_owner(&self, op: &'static str, span: Span) -> Result<(), RuntimeError> { ... }
    pub fn with_mut<R>(
        &self,
        op: &'static str,
        span: Span,  // NEW (in 2nd position before closure)
        f: impl FnOnce(&mut T) -> R,
    ) -> Result<R, RuntimeError> { ... }
}

impl OwnedMoveCell<T> {
    pub fn take(&self, op: &'static str, span: Span) -> Result<T, RuntimeError> { ... }
}

pub fn downcast_ref_opaque<'a, T: Any>(
    arc: &'a Arc<dyn Any + Send + Sync>,
    type_path: &'static str,
    op: &'static str,
    span: Span,  // NEW
) -> Result<&'a T, RuntimeError> { ... }
```

Each helper body uses the threaded span when constructing RuntimeError. DELETE the 7 `// arc 138: no span — ...` rationale comments at these sites.

### 2. Caller updates

**src/io.rs (~6 sites):** all are inside WatReader/WatWriter trait method bodies. After F3, these trait methods take `span: Span` — that span IS in scope. Pass it to `with_mut`:

```rust
self.state.with_mut(":wat::io::read", span.clone(), |s| { ... })
```

**crates/wat-telemetry-sqlite/src/auto.rs (~4 sites):** these are inside scheme functions with `args: &[WatAST]` in scope. Span source is `args[0].span().clone()` for the receiver opaque (since the receiver is conventionally `args[0]`):

```rust
let inner = rust_opaque_arc(&db_val, TYPE_PATH, OP, args[0].span().clone())?;
let cell: &ThreadOwnedCell<WatSqliteDb> = downcast_ref_opaque(&inner, TYPE_PATH, OP, args[0].span().clone())?;
cell.with_mut(OP, args[0].span().clone(), |db| { ... })
```

**Test callers in marshal.rs (~4 sites in tests module):** use `Span::unknown()` — synthetic test context, no AST source.

### 3. Recursive interaction

`with_mut` body calls `ensure_owner(op)?` internally. After expansion, `with_mut` should pass its received span to ensure_owner: `self.ensure_owner(op, span.clone())?`. This is the recursive update.

## Constraints

- 3 files modified: src/rust_deps/marshal.rs + src/io.rs + crates/wat-telemetry-sqlite/src/auto.rs. NO others.
- NO new variants. NO Display string changes.
- NO commits, NO pushes.
- All 6 existing arc138 canaries continue to pass.
- Workspace tests pass: `cargo test --release --workspace 2>&1 | grep FAILED | grep -v trading` returns empty.
- Pattern E count in marshal.rs production code: 7 → 0 (the 7 helpers identified by F4b's honest delta).

## Reporting back

Compact (~300 words):

1. **Diff stat:** 3 files.
2. **5 helpers updated:** confirm each signature.
3. **Recursive call from with_mut to ensure_owner:** confirm.
4. **Caller distribution:** io.rs (6 sites with span.clone() from trait method param) + auto.rs (4 sites with args[0].span().clone()).
5. **Pre/post Span::unknown() in marshal.rs production code:** target 7 → 0.
6. **Verification:** all 6 canaries + workspace tests.
7. **Honest deltas.**
8. **Four questions** briefly.

## Why this is small

5 helpers + ~10 caller sites + 3 files. Same shape as F4a (helpers + callers in single engagement). F4a ran 14 min for 14 helpers + ~33 callers. F4c is smaller; estimated 8-15 min.
