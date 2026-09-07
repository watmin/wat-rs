;; probe-a-fold-accumulator-can-be-a-struct.wat — arc 278.
;;
;; WHY. `the queue reports time inside its store` failed to PARSE, not to
;; design. Threading one more i64 through sqs.wat's three waiter folds meant
;; nesting `(Tuple sc ns)` into the accumulator's third slot, because
;; ⛔ `:wat::core::fourth` DOES NOT EXIST — zero occurrences anywhere in wat/ or
;; wat-scripts/, while `third` is used in wat/sqlite.wat:82 and wat/fix.wat:376.
;;
;; Nesting shifts paren depth at every use site. In sqs.wat, where lines reach
;; 145 chars and indent reaches 100 columns, grok reported: "UnclosedParen with
;; Continue=4; UnexpectedRParen with Continue=5. NO INTEGER WORKS."
;;
;; ★ So the defect class is: THREADING ONE MORE VALUE THROUGH A FOLD REQUIRES
;; PAREN SURGERY. A named-field aggregate removes it — adding a field changes
;; nothing at any use site.
;;
;; ⚠ `defrecord` CANNOT do this. The accumulators carry a Peer, and a record is a
;; pure aggregate:
;;   ImpureFieldInPureAggregate (arc 293.W): pure aggregate may only hold pure
;;   fields — "a record or holon holding a struct field could never cross — it
;;   must not exist."
;;
;; ★★ `defstruct` CAN: a struct is the impure-capable aggregate — it stays in
;; shared memory and never crosses, which is exactly what a local fold
;; accumulator does. This probe is the worked reference.
(:wat::core::defstruct :probe::Acc
  [peer  <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
   kept  <- :wat::core::i64
   calls <- :wat::core::i64
   ns    <- :wat::core::i64])

(:wat::core::defn :probe::step
  [a <- :probe::Acc  i <- :wat::core::i64] -> :probe::Acc
  ;; a fifth field would land here with NO paren change at any call site
  (:probe::Acc
    :peer  (:probe::Acc/peer a)
    :kept  (:wat::i64::+ (:probe::Acc/kept a) i)
    :calls (:wat::i64::+ (:probe::Acc/calls a) 1)
    :ns    (:wat::i64::+ (:probe::Acc/ns a) (:wat::i64::* i 10))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  ;; No live peer is needed to prove the SHAPE. `:probe::step` type-checks:
  ;; a struct accumulator holding a Peer, four named accessors, threaded
  ;; through a step function. A fifth field lands with NO paren change.
  (:wat::kernel::println "defstruct accumulator: declared, accessors typed, step threads it"))
