# NOTE — `edn::write` PANICS on a value it cannot tag, and the failure channel already exists

**Filed:** 2026-08-29, by the grok-rete agent, at the builder's direction
(*"another arc 109 note about the panic on unknown tag"*).
**Home:** arc 109 — `src/edn_shim.rs` is its territory, and this is not rete.
**Status:** ✅ **FIXED 2026-08-29** (builder: *"if you can fix this now - do it"*). This note is
kept as the record of what was wrong and what was deliberately NOT touched. Sibling of
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
`panic_hook.rs`, `freeze.rs`, `distribution/mod.rs`, `distribution/mcp.rs`. `value_to_edn` (`:3659`) was a second public entry with the same shape.
⚠ **I measured it as having ZERO callers and that was WRONG** — the grep covered `src/` and
`crates/` and missed `tests/`, where it had **five**. It is now deleted and those five call
`value_to_edn_with(v, None)`, so there is one door instead of two. The lesson is the measurement,
not the deletion: a call-site count that does not include the test tree is not a call-site count.

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


---

## WHAT WAS ACTUALLY DONE (2026-08-29)

`value_to_edn_with` and `value_to_json_natural` return `Result<_, RuntimeError>`; the `panic!` is
gone. `RuntimeError` was chosen over `EvalBreak` because it converts INTO `EvalBreak`, so `?` works
in both kinds of caller.

**The doctrine survived the conversion, and a test is why.** Returning the inner diagnostic alone
would have kept the located `TypeMismatch` and silently dropped the *"the algebra never crosses the
wire in any form, per DESIGN-STONE-294.j"* sentence the panic carried.
`row7_bare_bundle_raises_on_encode_never_falls_back` asserts the mechanism is named and caught it.
The diagnostic now embeds the inner message, appends the doctrine, and reuses the inner span.

**Three call sites genuinely cannot propagate, and they get a named lossy door**
(`value_to_edn_string_lossy`), which renders `#wat.edn/Unencodable {:type … :reason …}` instead of
aborting: the STOP-protocol path (`freeze.rs`, returns `Vec<Value>`), the process-crash envelope
(`runtime.rs`/`process/verbs.rs`), the spawn label and the MCP renderer, plus `EdnRepresentable::
to_wire` whose signature the trait fixes. **The panic hook degrades PER ITEM** so one unencodable
value in a death chain does not cost the rest of the chain. The door's doc says outright that it is
not for ordinary callers — reaching for it to avoid threading an error is how the original panic got
written.

**One site refused the lossy door on purpose.** `try-send'`'s `with_ref` callback returns a bare
outcome, so `?` had nowhere to go — but a lossy encode there would have put an
`#wat.edn/Unencodable` marker **on the wire as if it were the payload**. The encode was hoisted
above the closure instead, where the error propagates. Silently transmitting a marker is worse than
either alternative.

**Gate:** `tests/diagnostics/probe_edn_write_unencodable_is_a_diagnostic.rs` — mutation-proven, and
it pins the GOOD door's exact bytes as well, so making the encoder return `Err` for everything
cannot pass it.

Floor 5152/5152, clippy silent.
