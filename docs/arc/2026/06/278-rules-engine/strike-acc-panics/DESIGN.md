# DESIGN-STONE — a wire-reachable invariant may not be spelled `panic!`

> **Origin (2026-08-30).** Vigilia Class A2, found by `circumspicere` cast last: *"a host `panic!`
> whose licence is a compile-time proof that the import door never runs."* Sequenced deliberately
> AFTER Class A1's graph wall, because the wall changes which malformed shapes can reach these
> sites — measuring their reachability first would have measured the wrong surface.

## Why

`acc.rs:57` documents `acc_var_i64` as *"Panics on an unbound var or a non-i64 value (a
**compile-time-impossible** shape)"*, and carries
`rune:struere(invariant-coupling) — AccFold compile proved i64`.

**That proof is `build_rete_arm`'s. `import_export` does not run it.** `unpack_fold`'s `:sum` arm
is `AccFold::Sum(expect_at(&items, 1, …)?.clone())` — the key is an arbitrary `Value` off the wire,
with no check that it is a keyword, that any condition binds it, or that its values are `i64`.
`import_export` interns those folds directly, and the accumulate pass reads them.

`export.rs:15-17` states the file's own law: *"it consumes bytes some other process wrote, and
**every one of them can be a lie**."* A rune that names the compiler is naming the wrong door.

## The measurement — driven, not read

Class A1's wall does **not** close this: it validates node **edges**; folds are a side table, which
that stone affirmatively cut from scope. So the reachability had to be driven, and no fixture in
the corpus could do it — `probe_arc278_derived_exists_acc.wat` pairs an accumulate with an export
but its fold is `acc::count`, which carries **no key**; every keyed-fold fixture calls the fold
library directly rather than through a rule's `:from`, so builds no Accumulate node. A fixture was
written (`probe.wat.txt`) and driven at HEAD `788e5b66d`:

```
thread '…::import_refuses_a_fold_key_no_condition_binds' panicked at src/rete/kernel/fire/acc.rs:72:28:
accumulate: var wat__core__keyword("?no-condition-binds-this") not in packed slot_keys

A WIRE VALUE PANICKED THE HOST. Importing an Export whose :sum fold key is a keyword no
condition binds, then firing, unwound the process instead of refusing.
```

There is no `catch_unwind` on the program path (only `run-sandboxed`, `freeze.rs:187`, and
`distribution/mcp.rs:267`), so a wat caller gets a host unwind with **no span and no rule named** —
the exact failure class `DEFAULT_MAX_FIRE_ROUNDS` and `alloc_counter` were built to remove.

## ⚠ WHICH ARM THE PROBE REACHES — read this before prescribing a mutation

`acc_var_i64` has **five** panic arms and `acc.rs` has **nine**:

| line | arm | reached by the probe? |
|---|---|---|
| `:64` | bound, non-i64 | no |
| `:65` | unbound in element bindings | no |
| **`:72`** | **var not in packed `slot_keys`** | **YES — this is the one that fired** |
| `:76` | packed field missing | no |
| `:83` | packed row missing for fact | no |
| `:129` | packed row missing (slot path) | no |
| `:139`,`:140`,`:142` | slot bound non-i64 / filler id missing / slot missing | no |

The probe takes the **packed** path (`el.binds.len == 0`), so it lands at `:72` and never touches
the `Bindings::get` arms. **One probe proves one arm.** This is the lesson A1's strike paid for:
its brief prescribed one mutation for a three-rule wall and the prescribed mutation could not
redden the probe, because the probe's fixture only ever reached a different arm.

## The algorithm

Give `acc_var_i64` and its siblings the shape `driver_of` (`fire/mod.rs:235-244`) already has for
the same class of missing-id: return `Result<_, EvalBreak>` carrying a `MalformedForm` that names
the fold, the key, and the element — and let the accumulate pass propagate it with `?`. The pass
already returns `Result`, so the propagation is mechanical.

## ★ THE ONE CONTRACT DECISION

**The refusal names the DOOR, not the compiler.** Every message and every surviving rune must say
that this state is reachable *from an imported network*, never that it is "compile-time
impossible" — because that sentence is what licensed the panic and it is false. A `rune:struere`
that still cites `AccFold compile proved i64` after this strike is the defect re-committed in the
cure, which this arc has done before and recorded.

## Blast radius

`src/rete/kernel/fire/acc.rs` and its callers in `src/rete/kernel/fire/pass/accumulate.rs`. Plus
the probe pair into `tests/rete/`. **No wire-format change, no version bump, no new types beyond
the `Result` in existing signatures.**

## Out of scope — AFFIRMATIVELY CUT

- **`fire/mod.rs:1400,1406,1415,1615-1628`** (`key_of` / `key_of_el`, eight more arms of the same
  class). Same shape, different pass, and `fire/mod.rs:1603` carries its own justification block
  that must be read and re-judged rather than swept along. **Its own strike**, and it should copy
  this one once this one has proven the propagation is mechanical.
- **Validating fold keys at the import door** — i.e. proving in `import_export` that a fold's key
  is bound by its accumulate's own conditions. Tempting, and it is the *other* honest fix. Cut
  because it needs the condition set, which import has not assembled at fold-read time, and
  because a refusal at fire time still satisfies the law (no host panic) while a wall at import
  time is a bigger, separate design. If it is wanted later it is additive, not a replacement.
- **A6 (unbounded `unpack_expr` recursion) and A7 (O(N²) import, missing ceiling calls).**
