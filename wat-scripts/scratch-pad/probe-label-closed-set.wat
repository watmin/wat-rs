;; probe-label-closed-set.wat — arc 170 closure #6: IS the ps-label set actually CLOSED?
;;
;; SHAPE 2 was ratified OVER SHAPE 1 on exactly one property: `ps` output is a CLOSED set
;; the operator learns once and matches exhaustively — "no caller defines anything to be
;; visible" (wat-scripts/intueri/proc-identity-vocabulary.wat.intueri). wat/spawn.wat's
;; `with-label` doc block asserts that restriction in prose:
;;
;;   "`r` is restricted to the two substrate-owned identity types (`:wat::process::Bracket` |
;;    `:wat::process::Service`) — a CLOSED set … no caller mints its own tag."
;;
;; This probe asks the disk whether anything ENFORCES that sentence. It mints a rogue record
;; that is NOT in the closed set and hands it to `with-label`.
;;
;;   GREEN (this file type-checks) => nothing enforces it. The prose is a CLAIM with no
;;                                    witness, and `ps` output is an OPEN set — SHAPE 1 by
;;                                    the back door, the very thing SHAPE 2 was chosen over.
;;   RED   (located type error)    => the set IS closed; delete this probe and the doc stands.

(:wat::core::defrecord :probe::Rogue [tag <- :wat::core::String])

(:wat::core::defn :probe::mint-rogue-label [] -> :wat::spawn::Locus
  (:wat::spawn::with-label
    (:wat::spawn::process)
    (:probe::Rogue :tag "i-am-not-a-bracket-or-a-service")))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:probe::mint-rogue-label)
    nil))
