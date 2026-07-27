# NOTE — a macro-generated program does not survive the wire (two gaps, both small)

> **Found 2026-07-27**, arc 170 execve step 2d, by the floor. STOP-4 of
> `BRIEF-execve-step2-stream-the-program.md` fired exactly as written.
>
> **This note was rewritten the same day.** Its first version diagnosed a design
> fork ("text vs three candidate shapes") and that was wrong — the builder cut it
> in one line: *"are we not shipping edn forms over the wire?"* The corrected
> diagnosis is below. Steps 2a–2c are green and banked (`425d7624`, `aa5910e6`).

## What actually happened

Step 2d had the child freeze from a **source-text** payload:
`forms → write_wat_source → text → parse_all_with_file → forms`.

Three real consumers broke:

```
probe_arc170_gapj_each_kwargs   probe_arc170_c2_strike1_mixed   probe_arc170_c2_mixed_macro
```

with the checker naming its own invariant:

> `HygieneScopeDivergence`: reference `kwargs` (scope `{}`) is unbound, but a
> binder `kwargs` exists under a different scope `{952}` — *"a macro rebuilt this
> binder from its name instead of reusing the node."* — `<child>` line 26, from
> `wat/bracket.wat:666`

`spawn-program (process)` can be handed forms **produced by a macro**, and those
carry hygiene scopes. The child is the "macro" in that message: parsing rendered
text *is* rebuilding a binder from its name.

## Why this had never surfaced

**Programs have been shipping for months by COW fork — memory, not a wire.** The
child inherits the forms because it inherits the address space. Nothing was ever
serialized, so no encoder ever had to carry hygiene. execve is the first thing
that forces the question.

## The corrected diagnosis — two gaps, neither a design fork

The first version of this note framed three "candidate shapes" (text with a scope
syntax / EDN-encoded AST / resolve hygiene to unique names) as though they were
comparable. They are not. The substrate ships EDN everywhere; the payload should
be EDN-encoded AST, and **the tooling already exists** —
`src/wat_edn_bridge.rs`'s `watast_to_edn` / `edn_to_watast`, both directions,
with `::` handled via `keyword_from_wat_path` / `ns_to_wat_path`.

Source text was never required. That premise was carried, unexamined, from the
first sketch of the design through four commits.

What remains is two concrete gaps:

### Gap 1 — the bridge drops the scope set (new)

```rust
// src/wat_edn_bridge.rs:104
WatAST::Symbol(ident, _) => OwnedValue::Symbol(Symbol::new(ident.as_str())),
```

`Identifier` is `{ name: String, scopes: BTreeSet<ScopeId> }`
(`crates/wat-reader/src/identifier.rs:79`). The encoder takes the name and throws
the scopes away — the same loss as the source printer, by a different route.

Carry them, and carry them back in `edn_to_watast`. One arm plus its inverse.

**The obvious encoding is closed off:** `edn_to_watast`'s own doc says it rejects
a namespaced `Symbol`, so scopes cannot ride as `scope/name`. They need their own
representation — a small wire-format decision, cheap either way.

**Also worth knowing before it confuses someone:** that same doc records that
spans are NOT preserved — every reconstructed node gets `rust_caller_span!()`.
Fine for freezing (its note says so), but a child's diagnostics would point at the
bridge rather than at the program.

### Gap 2 — the `::` ↔ `.` dial is a `replace()`, not a parse (since 2026-05-21)

```rust
// forward   crates/wat-edn/src/vocab.rs:225
let translated = ns.replace("::", ".");
// inverse   src/edn_shim.rs:2783
format!(":{}::{}", ns.replace('.', "::"), name)
```

Lossy and ambiguous in both directions: a name containing a literal `.` comes back
as `::`, and the two spellings stop being distinguishable after a round trip.
Dated by `git log -S` to **2026-05-21, arc 218**.

This is the fourth instance of the class arc 278 named three times in one arc —
*"suspect a string comparison with one side normalized and the other not before
suspecting the type system; a `format!`/`split`/`==` on names is the culprit."*

It is load-bearing here: if the program crosses as EDN-encoded AST, **every
keyword in it goes through this dial**, so the payload's fidelity depends on a
substitution that cannot distinguish the separators.

## The lesson from the check that missed it

Step 2c kept the inherited forms as an "oracle" and had the child compare the
streamed source against `forms_to_source(&forms)` computed from them. Both sides
ran the same printer over the same in-memory forms, so the strings were equal **by
construction**.

```
   tested:      render(forms)          ==  render(forms)      ← trivially true
   needed:      parse(render(forms))   ==  forms              ← the actual claim
```

It proved the *transport* faithful and could not have detected a lossy *round
trip*, because the parent never re-parsed. **An oracle that compares a thing to
itself is not an oracle.** Any future "is the new path faithful?" check must run
the whole conversion and compare against the original.

## Status

**OPEN — step 2d is blocked on Gap 1; Gap 2 is upstream of it.** The pipe is
right; the payload was wrong. 2a–2c stand: the boot wire (derived, registered,
guarded), the transport (chunked, acked, named EOF, driven over real pipes), and
the handshake through both fork sides with a real child booting over it.

The probe (`wat-scripts/scratch-pad/probe-execve-argv-cow-leak.wat`) remains RED
at `2` / `2`, which is correct — nothing has closed the leak yet.
