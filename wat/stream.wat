;; wat/stream.wat — arc 296 J: `:wat::stream::NextOutcome`, declared in wat.
;;
;; Formerly a hand-written `EnumDef` literal in `src/types.rs`. wat is the
;; source of truth; Rust consumes it via `wat_enum_register_from!`.
;;
;; Load order: after `wat/core.wat`.

;; (:wat::stream::NextOutcome :- [T]) — Arc 118.11a (stone A of two, "mint next +
;; NextOutcome", DESIGN-STONE-118.11a). The matchable outcome of
;; `:wat::stream::next`, the single-force pull primitive that replaces the
;; three-force `empty?`/`first`/`rest` walk protocol (measured: 15 user-code
;; calls for 5 elements without the memo; the memo patches the count but pins
;; the whole realized chain in memory, +297 B/element). `next` forces exactly
;; one cell (`crate::stream::realize`, WHNF) and returns both halves in one
;; shot — nothing to dedupe, so no cache is needed:
;;   :Item      [value <- T, rest <- (Stream :- [T])] — the forced head + the
;;                                                 undrained tail, together.
;;   :Exhausted []                              — the named end.
;; Parametric in T exactly as `(RecvOutcome :- [O])` above (the copied exemplar) —
;; and for the identical reason: the nullary `Exhausted` variant is the
;; documented hazard in `check.rs` (the un-parametrized-nullary-variant
;; unify failure) unless the enum itself carries `type_params`. IMPURE like
;; `(RecvOutcome :- [O])`, not `SendOutcome`/`CloseOutcome`: T is a caller-supplied
;; element type that MAY be a live resource (a `Stream` of open peers, say),
;; so a blanket `Pure` marking would lie about what crosses. This stone is
;; purely additive — no existing verb moves, no call site migrates onto `next` yet.
;; (Stone 118.B3 has since DELETED the `forced: OnceLock` memo this comment used to say was
;; untouched; the migration it anticipated happened in 118.B2/B2b. Both are done.)
(:wat::core::defenum :wat::stream::NextOutcome :- [T] :wat::enum::Impure
  :Item [value <- :T  rest <- (:wat::stream::Stream :- [:T])]
  :Exhausted)
