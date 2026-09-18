# DESIGN — the row about rotted claims has itself rotted, in every citation

## Why

**F2's seven bullets**, the last substantive block of the 2026-08-30 vigilia. Audited against the tree
at HEAD `6db874fc9` — **every one of the row's own citations has drifted**, and its headline count is
wrong in both directions.

| bullet | the row says | measured |
|---|---|---|
| 6 | *"83 of 207 stones name `src/rete/kernel.rs`"* | **39 of 487.** Corpus more than doubled; hits more than halved |
| 3 | `purity.rs:216` + `completeness_gate` at `:2093` | claim **LIVE**; the gate is at **`:2115`** |
| 4 | `delta.rs:391` | **`:400`** |
| 7 | *"four cells"* at `:419,446` | **`:436,539`** |
| 2 | heading cited by `rust_deps/cache.rs:70` exists nowhere else | **CONFIRMED** — grep finds it in the source line and the work-list row, nothing more |

**The row about rotted claims is an instance of itself.** That is not irony to enjoy; it is the
measurement that says line-numbered prose rots faster than anyone re-reads it — which is what C14 and
F2-e each concluded independently, and why both landed on *prefer a symbol*.

## ⛔ THE DATED-STONE PRECEDENT DOES NOT TRANSFER, AND THE DISTINCTION IS THE SPINE

C14's rider left three dated stones alone that quote 80,200 as a call count: **true when measured**,
dated, and therefore a record rather than a lie. Correct.

**26 of the 39 stones naming `src/rete/kernel.rs` carry a dated `Origin (` header.** The precedent
looks like it applies. It does not:

> **A dated MEASUREMENT is still true. A dead POINTER is still dead.**
> *"It was 80,200 in August"* retains its meaning forever. *"Go look at `src/rete/kernel.rs`"*
> retains none — the file has not existed since 2026-08-20.

A measurement records what was; a citation promises where to look. Only one of those survives the
thing it names.

## The contract decision, pinned

**F0 governs the counts; the deferred-34 lesson governs the paths.**

- **Counts → the command that derives them, never a corrected number.** Bullet 6 does not become
  *"39 of 487"* — that rots in a week, exactly as *"83 of 207"* did. It becomes the `grep` that
  answers it.
- **Paths → verify or delete, never re-point on a name match.** `src/rete/kernel.rs` split into
  `src/rete/kernel/` on 2026-08-20. A citation that meant *the module* re-points; one that meant a
  specific item must be verified to that item, or the path goes and the prose keeps what is still
  true. **The deferred-34 strike proved the plausible target is the danger.**
- **The row's own drifted citations are cured with SYMBOLS, not corrected lines.** Fixing `:2093` to
  `:2115` buys until the next edit above it.

## Out of scope = REJECTED

- **The 114 dated stones' measurements.** Not touched. Dated figures are records.
- **Bullets already struck** (the codemod ✅ and the retirement ⛔). Closed on the merits; re-opening
  them is not this strike's.
- **A gate for in-range citation drift.** Probed and abandoned on the four questions: the pairing is
  semantic — prose carries negations (*"NOT `classify_clause`"*) and attributions
  (*"`where_tree.rs`'s `exec_dim`"*) that a line-local checker inverts. Rowed, not built.
