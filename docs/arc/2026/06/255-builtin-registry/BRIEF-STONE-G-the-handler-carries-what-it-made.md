# STONE G — the handler carries what it made

DRAWN 2026-08-27 against `732efa3b5`. **Strike this BEFORE the holon home.**
**PRIOR ART — read first:** `git log -1 22453b9b6` (Stone E-iv, which OPENED this ruling and
recorded the exact before/after this stone must reverse).

**Builder's ruling, 2026-08-27:** *"update the signature then — every heretic reveals themselves."*

## The defect, in one type

```rust
// src/intrinsic/mod.rs:152
pub(crate) type NativeHandler =
    fn(&[WatAST], &Span, &Environment, &SymbolTable) -> Result<Value, EvalBreak>;
```

A bare `Value`. Provenance lives on a `TrackedValue` (`src/value/observe.rs:47`), so a registry
handler **physically cannot** stamp `Provenance::RuntimeBuilt { producer, call_span }` — "this value
was manufactured, by that verb, there." A hand-written dispatch arm in `runtime.rs` can. Routing a
producer into a home therefore downgrades it to `SymbolBound`, a fact about the *binding site*
rather than about what made the value. **Not a bug — a missing field in a function type.**

## ⛔ THE ONE CONTRACT DECISION — default-preserving, opt-in for producers

**There is ONE choke point, not 250.** Every registered handler reaches the registry through a
macro-generated shim (`crates/wat-macros/src/wat_intrinsic.rs:439-444`, `handler: #shim_ident` at
`:453`). The return type is written in exactly **two** places: that `quote!` and the alias above.

```
NativeHandler          -> Result<TrackedValue, EvalBreak>
the generated shim     wraps a handler that returns a bare `Value` as
                       TrackedValue::new(v, Provenance::Unknown)      <- TODAY'S BEHAVIOUR, unchanged
a handler that WANTS provenance returns a TrackedValue itself; the macro SNIFFS which form it got
```

Mirror the sniff the macro already performs on arguments — `SniffedArgs::Exact(names)` vs variadic
(`wat_intrinsic.rs:54-71`). Same shape, applied to the return type. **STOP-1** is requiring all 250
handlers to change; the default arm must keep every existing handler compiling untouched.

## ★ THE PROOF THIS WORKS IS ALREADY WRITTEN, IN E-iv's OWN COMMIT

Stone E-iv recorded exactly what it lost, from the golden whose NAME is `renders_runtime_built_keyword`:

```
BEFORE   "(built by :wat::core::keyword/from-string ...)"
         :provenance RuntimeBuilt { :producer "…/from-string" :call-span … }
AFTER    "(bound from tests/…/p2.wat:4:8 ...)"
         :provenance SymbolBound { :binding-span … :head-span … }
```

**This stone must put that golden BACK.** Re-stamp the four keyword producers in
`src/intrinsic/keyword.rs`, then find arc 233's guards that E-iv rewrote to match the degraded
behaviour — they carry honest `⚠ REGRESSED (honestly, not silently)` comments naming the mechanism —
and restore them to asserting `RuntimeBuilt`. **A guard rewritten to match degraded behaviour is a
green test that no longer proves what it was built to prove**; un-rewriting it is the acceptance row.

## Why this blocks three drawn homes, not just holon

```
src/edn/render.rs          17   producers building TrackedValue{RuntimeBuilt}  — HOME-5 DRAWN, UNBUILT
src/edn/error.rs            2
src/runtime.rs              2
src/value/environment.rs    2
src/value/observe.rs        4   the Provenance definition site itself
src/intrinsic/keyword.rs    1   the one that survived a carve — and it survived DOWNGRADED
```

`src/intrinsic/{edn,load,host}` are all **NOT BUILT** though HOME-5/6/7 are drawn. Carving edn
through today's registry drops all seventeen. Holon is worse still: 26 binding fns, several of them
constructors (`eval_holon_atom_constructor`, `wrap_holon_as_atom`, `build_holon_hologram`).

## Rooms

```
src/intrinsic/mod.rs:152                     the NativeHandler alias
crates/wat-macros/src/wat_intrinsic.rs:439   the generated shim's signature
crates/wat-macros/src/wat_intrinsic.rs:444   its return type
crates/wat-macros/src/wat_intrinsic.rs:54-71 SniffedArgs — the sniff pattern to MIRROR
crates/wat-macros/src/wat_intrinsic.rs:453   handler: #shim_ident
src/value/observe.rs:22,47                   Provenance + TrackedValue
src/intrinsic/keyword.rs                     the four producers to re-stamp
```

## STOP triggers — each REJECTS

1. **STOP-1 — you would edit all 250 handlers.** The default arm keeps them compiling untouched.
2. **STOP-2 — the sniff cannot distinguish the two return forms.** Report it; do not guess by name.
3. **STOP-3 — you would weaken an arc 233 guard to make it pass.** They are being RESTORED, not adjusted.
4. **STOP-4 — a room's line does not hold.** Written against `732efa3b5`.

## Acceptance — every row measures a MECHANISM

```bash
# 1. every existing handler still compiles UNTOUCHED — the default arm holds.
cargo build --release --all-targets
git diff --stat -- src/intrinsic/   # only keyword.rs (the re-stamp) should appear

# 2. ★ the golden E-iv lost comes BACK — RuntimeBuilt, not SymbolBound. Paste it.
cargo nextest run --release -E 'test(renders_runtime_built_keyword)'

# 3. ★ arc 233's rewritten guards are RESTORED. Find every `⚠ REGRESSED` comment E-iv left,
#    show the guard now asserting RuntimeBuilt, and DELETE the comment (it is no longer true).
grep -rn 'REGRESSED (honestly' tests/ src/    # must be 0 after

# 4. ★ NON-VACUITY — break the door. Make one re-stamped producer return the default
#    (Provenance::Unknown) and show row 2 FAIL naming the mismatch; restore; show it pass.

# 5. metadata-of can now answer "is this a producer?" for a registered intrinsic.
#    Show it for one re-stamped keyword verb.
```

## Report back with

Each row's actual output. Row 2's golden verbatim, before and after. Every `⚠ REGRESSED` comment you
deleted and the guard you restored. Row 4's both outcomes. What the sniff does when it meets each
return form. Anything this brief got wrong; what you did NOT do, and why.
