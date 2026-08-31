;; 255 — struct-field is a CONSTANT PROJECTION: same input, same answer.
;;
;; The evidence behind DESIGN-STONE-the-record-family.md's ruling that
;; :wat::core::struct-field / struct-new are Pure ∧ Deterministic.
;;
;; Builder, 2026-08-31: "a function to say 'give me this field's value' is
;; constant when applied to a constant input?"  — yes. This shows it against a
;; struct holding a LIVE, MUTABLE handle, which is the hardest case for the claim.
;;
;; EXPECTED OUTPUT:
;;   "plain field: read twice, equal? true"
;;   "handle len when FIRST read: 0"
;;   "handle len via the SECOND read: 2"
;;   "handle len via the FIRST read, now: 2"
;;
;; The last two lines are the proof: the handle read BEFORE the mutations and the
;; handle read AFTER report the same length — one object, handed back identically
;; both times. Nothing about the READ varied; what moved was behind the handle.
;;
;; ⛔ THE REFUSALS CANNOT LIVE HERE — a committed .wat must load, and these panic at
;; run. Demonstrated out-of-tree 2026-08-31, verbatim, via :wat::rete::compile-all:
;;
;;   (where (i64::> (:wat::kernel::pid) 3))              [negative control]
;;     => "compile-condition: where expr is not pure — ':wat::kernel::pid' is not pure"
;;   (where (i64::> (:u::R/v ?r) 3))            record    => COMPILED-OK
;;   (where (i64::> (struct-field ?r 0) 3))               [BEFORE this stone]
;;     => "compile-condition: where expr is not pure — ':wat::core::struct-field' is not pure"
;;   (where (i64::> (:u::Conn/fd ?c) 3))        struct    => "':u::Conn/fd' is not pure"
;;   (where (i64::> (Lru::len (Lru::new 4)) 0))
;;     => "where expr is not pure — ':rust::cache::Lru::len' is not pure"   <- THE GUARD
;;
;; That last one is why admitting struct-field opens no hole: a handle IN HAND inside
;; a fence is inert. Every verb that could DO anything with it is refused one step
;; later by the recursive walk. The projection was never the hazard.
;;
;; Also measured: a record may not HOLD a struct field at all —
;;   #wat.type/ImpureFieldInPureAggregate, "containment rule (arc 293.W)" — so the only
;; way a struct reaches a fence is as a direct fact, which it can be (COMPILED-OK).

(:wat::core::defstruct :u::Box
  [cache <- (:wat::cache::Lru :- [:wat::core::i64 :wat::core::i64])
   tag   <- :wat::core::i64])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [b     (:u::Box :cache (:wat::cache::Lru::new 8) :tag 42)
     t1    (:wat::core::struct-field b 1)
     t2    (:wat::core::struct-field b 1)
     hA    (:wat::core::struct-field b 0)          ;; read BEFORE any mutation
     lenA  (:wat::cache::Lru::len hA)
     _     (:wat::cache::Lru::put hA 1 100)
     _2    (:wat::cache::Lru::put hA 2 200)
     hB    (:wat::core::struct-field b 0)          ;; read AFTER two mutations
     lenB  (:wat::cache::Lru::len hB)
     lenA2 (:wat::cache::Lru::len hA)]
    (:wat::kernel::println (:wat::string::concat "plain field: read twice, equal? "
      (:wat::core::bool::to-string (:wat::core::= t1 t2))))
    (:wat::kernel::println (:wat::string::concat "handle len when FIRST read: "
      (:wat::i64::to-string lenA)))
    (:wat::kernel::println (:wat::string::concat "handle len via the SECOND read: "
      (:wat::i64::to-string lenB)))
    (:wat::kernel::println (:wat::string::concat "handle len via the FIRST read, now: "
      (:wat::i64::to-string lenA2)))))
