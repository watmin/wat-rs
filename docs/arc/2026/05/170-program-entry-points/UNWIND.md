# UNWIND — the descent from 170 to 255, and the climb back

> Breadcrumb (2026-06-22). Why we are deep in arc 255's doc-contract when the
> open arc is **170 (remote programs)**. The reach-stumble recursion: each layer
> is a tool we reached for, found missing or dishonest, and descended to build —
> and *that* tool needed another. Grounded against the arc docs, not recall.

## The descent (each layer a prerequisite of the one above it)

```
arc 170  — REMOTE PROGRAMS / resolve the deadlock        ← the root goal (branch IS arc-170)
   │  to spawn a program on a remote peer you must ship its forms over the wire as
   │  honest EDN. wat's surface isn't clean EDN:
   │     :wat::core::HashMap<wat::core::String,wat::core::i64>   ← not EDN (the <…> generic)
   │     (… -> :T …)                                            ← not EDN (the ascription arrow)
   │     HolonAST-as-data                                       ← not EDN (the holon crutch)
   ▼
arc 251  — TYPES-AS-FORMS / "clojure-faithful surface inversion"   (+ 257 native {} #{} literals)
   │  make wat an honest clojure dialect — everything a real EDN form:
   │     <…> generic  →  (wat.type/HashMap wat.type/String wat.type/i64)
   │     -> :T        →  killed (arc 258); only fn-return survives
   │  └─ arc 258 (instinctive-conditionals) — the `-> :T` kill. Surfaced writing
   │     `fix-source` (251.5a-vi): reached for `cond`, stumbled on its `-> :T`;
   │     the DESIGN names it "papers over a checker limitation (synthesis-only,
   │     no LUB)." The stumble named the arc. (NOT from lint — lint is a sibling.)
   ▼
arc 282  — WAT-FIX OVER RUST  (+ arc 277 wat-lint-fix-fmt)
   │  the swap touches every .wat file → one-shot it with a codemod, don't hand-edit.
   │  282's own DESIGN: "STUB/HORIZON, BLOCKED behind arc 278" — a fact-source FOR
   │  the rules engine, not a standalone machine.
   ▼
arc 278  — RULES ENGINE (rete)
   │  the fact-matching machine the lint auto-fixes (277) and wat-fix (282) run ON.
   │  building it needed reflection + type machinery → which REVEALED…
   ▼
arc 255  — BUILTIN REGISTRY / doc-contract + type-scheme durable fix      ← THE FLOOR (here now)
      the checker's ~1192-ref hardcoded type table, the un-witnessed docs, the
      HolonAST crutches. The registry + doc-contract is the durable fix — AND the
      clean single-source type representation the 251 `<…>→(wat.type/…)` swap must
      rewrite INTO. Both the rete-revealed-mess and the type-swap ride on it.
```

## The climb back (the resolution path)

1. **255** — doc-contract (`if`/`let` prove the special-form half) → 205-intrinsic
   migration → **checker-builds-from-registry / type-scheme single-source** (the
   value-side blocker, four-questioned → Option B: home authors the scheme, doc
   types project from it). *This is the floor that makes a clean type representation exist.*
2. **kill non-fn `-> :T`** (finish 258 — no survivors; readln re-syntaxes, not special)
   + **`<…> → (wat.type/HashMap …)`** (251 type-forms). wat's surface becomes honest EDN.
3. **278 rete green → unblocks 282/277 wat-fix** → one-shot the `.wat` corpus from
   wat-lisp to clojure-lisp.
4. wat is an honest clojure dialect → **real EDN forms on the wire**.
5. EDN-on-wire → **spawn-program' remote leg** (214's local typed-peer extended) →
   **170 resolved.**

## The shape

The substrate forced the hand five levels down: 170 → 251 → 282 → 278 → 255. Every
floor we pour now (the 255 doc-contract today) is a step back up toward shipping a
program to a remote peer as clean EDN. We are not lost in 255 — we are standing on
the floor the whole stack needed.
