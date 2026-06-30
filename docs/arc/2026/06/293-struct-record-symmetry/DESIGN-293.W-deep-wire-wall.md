# 293.W — the deep wire wall: the holder's comms boundary, made a TYPE guarantee

> **Status: SCOPED (2026-06-29). The PRIORITY** — pulling the projection-depth thread surfaced a grounded breach of
> §7 / R3. Builder: *this IS core 293* (the holder's categorical comms boundary). Gates K3-revise + K5.
> Pairs `AGGREGATE-MODEL.md` § principle 8 (the CONTAINMENT RULE) + § `to-record` (the projection that depends on it).

## The breach (grounded this session)

`is_portable_type` (`check.rs:13543`) checks a record's portability by its **top holder only** — `Some(Aggregate(a)) =>
a.holder.is_portable()` — and never recurses into field types (it *does* recurse for `Tuple` / `Vector<T>` / `Newtype`;
aggregates + enums are the holdouts; the enum arm carries an explicit exigere-violating `"not yet enforced"`). So a
`Record` carrying a `Struct` field passes the wall, and the runtime codec serializes the struct into a tagged map.
**Proven live** (the disconfirming probe): a child built `(:w::R 7 (:w::S 99))` (a record with a struct field) and the
parent `recv'`'d it across a process peer — `#w/S {:a 99}` reconstructed on the far side. A `Struct` crossed comms.
§7 (*"a Struct crosses NO comms, ever"*) and R3 *SUB SUPERFICIE QUOD ES* (*"the holder is enforced HARD … the same
leak class as a struct … crossing the wire"*) are both violated. R3 is PROBANDUM until this lands.

## The cure — the CONTAINMENT RULE (the top rung, not a runtime patch)

A non-portable field cannot be **reconstructed** from EDN bytes on the far side (you cannot materialize a bound socket —
there is no default value). So a portable container that held one could never be reconstructed → **it must not exist.**

> **A portable aggregate (`Record` / `HolonRecord`) may declare ONLY portable field types.** A `Struct` field is
> ILLEGAL at type declaration. A `Struct` itself still holds anything (in-locus — sockets, caches, nested structs).

This makes *"a struct crosses NO comms"* a **structural guarantee**: a record cannot *hold* a struct → can never
*carry* one across. The illegal state has no form (extirpare's top rung). `is_portable_type` staying shallow then
becomes *correct* (the rule guarantees the depth), and `to-record`'s recursive strip is well-defined (kept fields are
portable by the rule).

## The contract (pinned)

1. **Declaration gate (the core):** registering a `Record` / `HolonRecord` aggregate whose any field type is
   non-portable is a **hard declaration error** (`MalformedDecl` / a typed error in `register_types` / the aggregate
   registration path). "Non-portable field type" = `is_portable_type(field_ty) == false`. (Reuse the existing
   `is_portable_type`; it is the right predicate, just newly *enforced* at declaration instead of only consulted at
   `send'`.) A `Struct` aggregate is unrestricted.
2. **The `recv'` backstop (the untyped top-level path):** `recv'` (`eval_peer_recv_prime`, `runtime.rs:24685`) refuses
   to reconstruct a **bare top-level `Holder::Struct`** value off the wire — the one path the declaration gate can't
   reach (a child `pprintln`s a bare struct, no type to check). A struct shall not *arrive*.
3. **Kill the deferral, rune the legit cases:** the enum arm of `is_portable_type` recurses into variant field types
   too; the genuinely-non-portable substrate service-control enums (`StdOutService::Event` etc. carrying `Receiver<T>`)
   carry an explicit **`// rune:lint(<lint>) — <reason>`** at their declaration site (excusare-auditable), NOT a
   blanket "not yet enforced." The comment dies.

## RED probe

`tests/types/probe_arc293_W_containment.{rs,wat}` — a record declaring a struct field is REJECTED at load:
```clojure
(:wat::core::defstruct :w::Conn [fd <- :wat::core::i64])
(:wat::core::defrecord :w::Bad [tag <- :wat::core::i64  c <- :w::Conn])   ; ILLEGAL — a record cannot hold a struct
```
RED at HEAD: this loads cleanly today (the breach). GREEN after 293.W: the load FAILS with a containment-rule error
naming the offending field. (A second `_bad`-style probe asserting the breach roundtrip now errors is a follow-on once
the gate lands.)

## Blast radius (the existing illegal declarations to surface + fix)

Enforcing the rule will RED any current `Record`/`HolonRecord` that declares a struct field — the corpus must be swept
(each is either a real bug → fix, or a struct-that-should-be-a-struct → the *container* should be a struct). The
breach probe's `:w::R` is the synthetic case; the gate run reveals the real ones (a cascade — normal, the meter to
zero). Service enums with `Receiver<T>` → runed. **Weigh forced-clean; the cascade is the progress meter.**

## Decomposition
- **293.W.1 — the declaration gate** + the `recv'` backstop + the RED probe → GREEN.
- **293.W.2 — the enum recursion + rune the legit service-enum sites** (kills the deferral comment).
- **293.W.3 — sweep the corpus** (any existing record-with-struct-field; cascade to zero).
- Then **K3-REVISE** (annihilate `to-struct` + `$struct`; the pair) → **K5** → showcase graduates.

## Pairs
`AGGREGATE-MODEL.md` § principle 8 + § `to-record` · `CLOSE-SEQUENCE-293-294.md § THE SURFACE KIT` (the pivot banner) ·
`291/STRIKE-4b-struct-state.md` (R8 — the EDN wire wall, the soul/body line) · `feedback` exigere (the deferral) ·
the `rune:lint` exemption scheme (`4ce97de3`; excusare audits the reason).
