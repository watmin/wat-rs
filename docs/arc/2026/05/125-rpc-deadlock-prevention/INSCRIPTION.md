# Arc 125 — RPC deadlock prevention (TYPE-PRECISE) — INSCRIPTION

**Status:** **WITHDRAWN 2026-05-01** in favor of arc 126.
**Closure:** 2026-05-03 (paperwork).

---

## Why withdrawn

Arc 125 proposed a compile-time check sibling to arc 117's `ScopeDeadlock`:

> At every `:wat::core::let*` binding-block (or function-call site), if a binding (or argument) of type `Sender<T>` is sibling/co-located with a binding (or argument) of type `Receiver<T>` for the SAME T after alias expansion, fire `CheckError::RpcDeadlock`.

Arc 126 superseded this with a structurally simpler approach: detect the channel-pair shape at construction sites rather than at binding sites. The four questions (obvious / simple / honest / good UX) preferred 126's check over 125's binding-walker — the construction-site check is more obvious in error messages (points at the pair's birth) and simpler to implement (no T-equality after alias expansion).

## Discipline preserved

Per the project convention: **rejected proposals stay; sequential numbering; no v1/v2.** Arc 125's DESIGN remains on disk as the honest record of an architectural alternative that was considered and overruled by 126. Future readers see the considered space, not a curated optimum.

## References

- `docs/arc/2026/05/125-rpc-deadlock-prevention/DESIGN.md` (the withdrawn proposal)
- `docs/arc/2026/05/126-channel-pair-deadlock-prevention/INSCRIPTION.md` (the shipped sibling)

---

**Arc 125 — closed as withdrawn.**
