;; Arc 170 gap J — `each` + kwargs tail (the success-gate's "each+tail" proof).
;;
;; `map`/`each` share the identical `:name val` tail-parse (wat/bracket.wat) — this proves the
;; kwargs-provisioning layer rides `each`, not just `map`: a durable-counter service is
;; granted+dialed via `each`'s OWN tail (no `process/uses`, no `bracket/uses`/`uses'` — all
;; retired this stone), each worker's side effect (increment) fires for every item, `each`
;; itself returns nil, and the counter's durable state afterward equals the item count (every
;; item incremented exactly once — no double-count, no drop).
;;
;; This test FORKS processes (the counter service + N pool workers) — run --test-threads=1:
;; cargo nextest run -p wat -E 'test(/probe_arc170_gapj_each_kwargs/)' --test-threads=1

(:wat::core::defsurface :probe::Counter :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :probe::Counter::GetRequest       [])
   (:wat::core::defrecord :probe::Counter::GetResponse       [value <- :wat::core::i64])
   (:wat::core::defrecord :probe::Counter::IncrementRequest  [n <- :wat::core::i64])
   (:wat::core::defrecord :probe::Counter::IncrementResponse [value <- :wat::core::i64])]
  :features
  [(get       [self <- :probe::Counter  req <- :probe::Counter::GetRequest]       -> :probe::Counter::GetResponse)
   (increment [self <- :probe::Counter  req <- :probe::Counter::IncrementRequest] -> :probe::Counter::IncrementResponse)])

(:wat::service::defservice :probe::counter
  :satisfies :probe::Counter
  :durable [count <- :wat::core::i64]
  :ephemeral []
  :impls
  [(get [s req]
     (:wat::service::Outcome::Reply s
       (:probe::Counter::GetResponse
         (:probe::counter::Record/count (:probe::counter::State/durable s)))))
   (increment [s req]
     (:wat::core::let [c (:wat::core::i64::+
                           (:probe::counter::Record/count (:probe::counter::State/durable s))
                           (:probe::Counter::IncrementRequest/n req))]
       (:wat::service::Outcome::Reply
         (:probe::counter::State (:probe::counter::Record c))
         (:probe::Counter::IncrementResponse c))))])

;; kwargs work-fn: item positional, `counter` a dialed `:key` kwarg (grant+Setup ride `each`'s
;; own tail). The side effect is the increment; the return value is discarded by `each`.
(:wat::core::defn :probe::record-hit
  [item <- :wat::core::String
   & [counter <- :wat::kernel::Peer'<probe::Counter::Op,probe::Counter::Reply>]]
  -> :wat::core::i64
  (:probe::Counter::IncrementResponse/value
    (:probe::Counter/increment counter (:probe::Counter::IncrementRequest 1))))

;; `:probe::run` (a non-main defn — no `:user::main`; only freezes + is called directly).
;; Returns (each's own return value, the counter's final durable count) so the Rust driver can
;; assert BOTH halves of the success gate: `each` returns nil, and every item's side effect fired.
(:wat::core::defn :probe::run [] -> :(wat::core::nil,wat::core::i64)
  (:wat::core::let
    [h        (:probe::counter/start :locus (:wat::spawn::process) :record (:probe::counter::Record 0))
     each-out (:wat::bracket::each (:wat::spawn::process) ["a" "b" "c" "d" "e"] :probe::record-hit :counter h)
     c        (:wat::kernel::connect' (:probe::counter::Handle/addr h))
     r        (:probe::Counter/get c (:probe::Counter::GetRequest))]
    (:wat::core::Tuple each-out (:probe::Counter::GetResponse/value r))))
