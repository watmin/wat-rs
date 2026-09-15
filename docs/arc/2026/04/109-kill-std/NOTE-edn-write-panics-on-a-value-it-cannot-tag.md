# NOTE — `edn::write` PANICS on a value it cannot tag, and the failure channel already exists

**Filed:** 2026-08-29, by the grok-rete agent, at the builder's direction
(*"another arc 109 note about the panic on unknown tag"*).
**Home:** arc 109 — `src/edn_shim.rs` is its territory, and this is not rete.
**Status:** OPEN. Nothing started. Sibling of
`NOTE-the-cache-lru-panics-on-a-value-that-arrives-from-durable-storage.md`, and the same class:
a `panic!` where the substrate can already carry an error.
**Ground:** `grok-rete` @ `4e2043cc2`. Reached from ordinary wat, not constructed in Rust.

---

## The panic

`src/edn_shim.rs:4262`, inside `value_to_edn_with`'s holon arm:

```rust
Err(e) => panic!(
    "cannot encode HolonAST to the wire — {e} — the algebra \
     (Bind/Bundle/Atom/Permute/Blend) never crosses the wire in any form, per \
     DESIGN-STONE-294.j; only DATA and the two directives (Thermometer/SlotMarker) do"
),
```

**Reached from a two-line wat program:**

```
(:wat::edn::write (:wat::holon::to-holon (:wat::core::Vector :wat::core::i64 1 2 3)))
    -> "#wat/holon [1 2 3]"
(:wat::edn::write #holon [1 2 3])
    -> PANIC
```

Same data. The first goes through the value lift and is classifier-wrapped; the second goes through
the wat source reader macro and is a bare, unclassified `Bundle`, which the encoder cannot tag.
(That divergence is a separate, DEFERRED defect —
`~/work/NOTE-holon-classifier-contract-is-unenforced-and-the-holon-tag-breaks-it.md`. **This note is
not about which holon is right; it is about what happens when the encoder meets one it cannot
name.**)

## ★ THE ROOT — the failure channel exists and stops ONE FUNCTION SHORT

```
eval_edn_write(...)      -> Result<Value, RuntimeError>     ← HAS a failure channel
  value_to_edn_with(...) -> OwnedValue                      ← HAS NONE. panics instead.
```

`src/edn_shim.rs:64` already returns `Result`. `src/edn_shim.rs:3803` does not, so when the holon
arm meets a value it cannot tag, there is nowhere for the failure to GO except the process. The
caller is one `?` away from rendering it as a located diagnostic like every other runtime error.

**This is not "we cannot report it" — it is "we did not thread it".** Exactly the shape the LRU note
records: there the stated blocker (a macro that could not marshal `Result`) had silently stopped
being true; here there was never a blocker at all, only a signature.

## Why a panic is the wrong shape here specifically

- **It is a data-dependent failure, not an invariant violation.** The value comes from the user's
  program. Encoding a holon the encoder cannot classify is the same category as writing a record
  whose type is not registered — and that path returns an error.
- **A panic cannot be caught, so it cannot be handled.** A service serialising a user-supplied value
  takes the whole process down; there is no `Result` for a caller to match, no fallback, no
  diagnostic with a span pointing at the offending call.
- **It escapes as a `Panic`, not a `RuntimeError`.** The user sees
  `#wat.kernel.LociDiedError/Panic` with a Rust `file:line`, rather than a wat-located diagnostic
  naming the op — so the ruin does not teach in the substrate's own idiom.

## The three sibling panics on this path are NOT this class — verified, do not bundle them

| site | what | verdict |
|---|---|---|
| `edn_shim.rs:2823`, `:3563` | *"def and its own match arm disagree"* | **Correct.** A genuine internal invariant; unreachable without a substrate bug. |
| `edn_shim.rs:2850` `struct_tag_for`, `:4180`/`:4189` `tag_from_type_path` | type path has no `::` namespace | **Correct, and structurally unreachable.** Driven: `(:wat::core::defrecord :Foo …)` is refused at LOAD by `#wat.macro/UnnamespacedName` — *"top-level name ':Foo' is not namespaced"*. The macro layer makes the input unrepresentable, so the panic guards something that cannot arrive. |

**Only `:4262` takes a value the user program supplies.** Converting the other four would be churn
against guards that are already right, and would weaken the signal that they are invariants.

## Cost

`value_to_edn_with` has **40 call sites** — 23 inside `edn_shim.rs` itself, then `runtime.rs` (6),
`services/verbs.rs` (4), `capability/registry.rs` (2), and one each in `process/verbs.rs`,
`panic_hook.rs`, `freeze.rs`, `distribution/mod.rs`, `distribution/mcp.rs`. `value_to_edn` (`:3659`)
is a second public entry with the same shape and **zero** callers — check whether it should simply
go rather than be converted alongside.

⚠ **`panic_hook.rs` is the one to think about before starting.** An encoder that can now fail, called
from the panic hook, is a failure inside failure-reporting. Whatever it does there must not be able
to recurse.

## What NOT to do

- **Do not "fix" it by encoding unclassified algebra as something plausible.** A bare `Bundle`
  rendered as a vector would silently produce a wire value that reads back as different data — the
  encoder's whole job is to refuse that. The refusal is right; only its MECHANISM is wrong.
- **Do not convert the other four panics for symmetry** — see the table; two are invariants and two
  guard an input the macro layer has already made unrepresentable.
- **Do not leave it as a comment.** Its sibling in this arc was deferred in prose for a month and
  its stated reason went stale unnoticed. This note is the re-readable row; if it is ruled LEAVE,
  write that here.

## Citations

| what | where |
|---|---|
| the panic | `src/edn_shim.rs:4262` |
| the caller that already returns `Result` | `src/edn_shim.rs:64` (`eval_edn_write`) |
| the encoder with no failure channel | `src/edn_shim.rs:3803` (`value_to_edn_with`) |
| second public entry, zero callers | `src/edn_shim.rs:3659` (`value_to_edn`) |
| invariant panics — correct | `src/edn_shim.rs:2823`, `:3563` |
| namespace panics — correct, unreachable | `src/edn_shim.rs:2850`, `:4180`, `:4189` |
| why the two holon doors differ (DEFERRED, separate) | `~/work/NOTE-holon-classifier-contract-is-unenforced-and-the-holon-tag-breaks-it.md` |
| the sibling panic-vs-error ruling in this arc | `docs/arc/2026/04/109-kill-std/NOTE-the-cache-lru-panics-on-a-value-that-arrives-from-durable-storage.md` |
