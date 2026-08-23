;; Fixture: `:wat::kernel::Thread` must parse as a registered parametric type
;; head (mirror `Sender<T>`/`Receiver<T>`). Arc 109 ③ retired the angle-bracket
;; reference spelling `Thread<I,O>` `parse_type_expr` (a &str -> TypeExpr fn,
;; pub outside the crate) used to exercise directly; the surviving reference
;; spelling `(Head :- [args])` only parses from a structural WatAST::List node,
;; which requires going through the freeze pipeline from outside the crate —
;; so this fixture stands in for the direct parse_type_expr call the probe
;; used to make. Same instantiation `probe2.wat`'s already-proven return-type
;; annotation uses (`(:wat::kernel::Thread :- [:wat::core::i64 :wat::core::i64])`).
(:wat::core::typealias :probe::arc214::ThreadPeer
  (:wat::kernel::Thread :- [:wat::core::i64 :wat::core::i64]))
