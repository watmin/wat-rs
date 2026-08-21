;; wat-scripts/scratch-pad/arc109-2iii-fn-bracket-destinations.wat — arc 109 Stone ②-iii.
;;
;; The ②-iii dry-run over wat/ revealed that the codemod ALSO migrates two keyword families the
;; DESIGN's "what this stone does NOT do" list had scoped out: the 42 `Fn(args)->ret` sites and
;; the 49 `:(a,b,c)` tuple-type sites. Both are `type-shaped-keyword?` by that predicate's own
;; documented definition ("a parametric `Head<...>` or a tuple/fn `(...)`"), so excluding them
;; would mean ADDING a discriminator — the exact move the DESIGN forbids.
;;
;; They are not a new spelling: `[arg… :-> ret]` is arc 251.4c's function-type bracket, whose
;; own doc states it "produces the SAME `TypeExpr::Fn` the keyword form yields, so the two
;; spellings unify" (`src/types.rs:parse_fn_type_bracket`), and it is already live in
;; `wat/test.wat:326,371` and `wat/spawn.wat:347`.
;;
;; This probe pins the DESTINATION shapes the codemod actually emitted on wat/, so the claim
;; "legal" is checked rather than reasoned. Each rung is a real transform from the dry-run diff:
;;
;;   :wat::core::Fn(U,T)->U                       -> [U T :-> U]                    (5 sites)
;;   :wat::core::Fn(T)->wat::core::bool           -> [T :-> :wat::core::bool]        (5 sites)
;;   :wat::core::Fn()->wat::core::Record          -> [:-> :wat::core::Record]        (2 sites, NULLARY)
;;   :wat::core::Fn(wat::core::i64,T)->Option<U>  -> [:wat::core::i64 T :-> (:wat::core::Option :- [U])]
;;   :wat::core::Fn(wat::kernel::Peer<S,R>,I)->O  -> [(:wat::kernel::Peer :- [S R]) I :-> O]
;;   :(wat::core::i64,wat::core::i64)             -> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])
;;
;; The nullary rung is the one worth a probe rather than a read: `parse_fn_type_bracket` finds
;; `:->` at position 0 and slices `items[..0]` for the args, which is empty-but-valid only if
;; nothing downstream requires at least one argument.

(:wat::core::defn :user::rung-binary-typevars
  [f <- [U T :-> U] seed <- U x <- T] -> U
  (:wat::core::apply f seed [x]))

(:wat::core::defn :user::rung-pred
  [f <- [T :-> :wat::core::bool] x <- T] -> :wat::core::bool
  (:wat::core::apply f [x]))


;; NULLARY — both real sites (wat/spawn.wat:51, :105) only CARRY the value; neither applies it
;; at the declaration site, so the probe mirrors that: the rung proves the annotation parses and
;; the value threads through a signature, which is exactly what the two migrating sites do.
(:wat::core::defn :user::rung-nullary
  [f <- [:-> :wat::core::Record]] -> [:-> :wat::core::Record]
  f)

(:wat::core::defn :user::rung-nested-ret
  [f <- [:wat::core::i64 T :-> (:wat::core::Option :- [U])] i <- :wat::core::i64 x <- T]
  -> (:wat::core::Option :- [U])
  (:wat::core::apply f i [x]))

(:wat::core::defn :user::rung-nested-arg
  [f <- [(:wat::kernel::Peer :- [S R]) I :-> O] p <- (:wat::kernel::Peer :- [S R]) i <- I] -> O
  (:wat::core::apply f p [i]))

(:wat::core::defn :user::rung-tuple
  [t <- (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::i64
  (:wat::core::tuple-get t 0))
