;; Stone 255.87 group A #1 — a symbol declaration name is its canonical keyword,
;; and a surface name in an extend-type binder tail is that same identity.
;;
;; Measured before the cure: defrecord, defenum, defsurface, defmacro, and defn
;; with a symbol name already register under `:probe::a1::…`. The miss is the
;; binder tail. `(extend-type :- [] Box Cap)` sent `probe.a1/Cap` through
;; `normalize_form`, which accepts only a call head. A surface is not one.
;; The same tail is what `defservice` emits for `wat.capability/Capability`.

(wat.core/defrecord probe.a1/Rec
  [x :- wat.type/i64])
(wat.core/defenum probe.a1/En wat.enum/Pure
  :A
  :B)
(wat.core/defsurface probe.a1/Surf :nature wat.type/Struct
  :features
  [(m [self :- probe.a1/Surf] :- wat.type/nil)])
(wat.core/defmacro probe.a1/mac [] :- wat.type/i64
  7)
(wat.core/defn probe.a1/inc [x :- wat.type/i64] :- wat.type/i64
  (wat.i64/+ x 1))

(:wat::core::defrecord :probe::kw::Rec
  [x <- wat.type/i64])
(:wat::core::defn :probe::kw::inc [x <- wat.type/i64] -> wat.type/i64
  (:wat::i64::+ x 1))

(wat.core/defsurface probe.a1/Cap :nature wat.type/Struct
  :features
  [(grant [self :- probe.a1/Cap] :- wat.type/nil)])
(wat.core/defrecord probe.a1/Box
  [n :- wat.type/i64])
(wat.core/extend-type :- [] probe.a1/Box probe.a1/Cap
  (grant [self] nil))

(:wat::core::defsurface :probe::KwCap :nature :wat::type::Struct
  :features
  [(grant [self <- :probe::KwCap] -> :wat::type::nil)])
(:wat::core::defrecord :probe::KwBox
  [n <- wat.type/i64])
(:wat::core::extend-type :- [] :probe::KwBox :probe::KwCap
  (grant [self] nil))

(:wat::core::defsurface :probe::Api :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Api::GetRequest [])
   (:wat::core::defenum :probe::Api::GetResponse :wat::enum::Pure
     :Ok [value <- wat.type/i64]
     :RequestTooLarge [bytes <- wat.type/i64 cap <- wat.type/i64]
     :RequestMalformed [path <- (wat.type/Vector :- [wat.type/String])
                        expected <- wat.type/String
                        got <- wat.type/String])]
  :features
  [(get [self <- :probe::Api req <- :probe::Api::GetRequest] -> :probe::Api::GetResponse
     :max-request-bytes 524288)])

(:wat::service::defservice :probe::a1svc
  :satisfies :probe::Api
  :durable [count <- wat.type/i64]
  :ephemeral []
  :impls
  [(get [s ctx req]
     (:wat::service::Outcome.Reply
       {:state s
        :reply (:probe::Api::GetResponse.Ok {:value 1})}))] )

(wat.core/defn probe/hold [] :- wat.type/i64
  (wat.i64/+
    (wat.core/if (wat.runtime/is-type? :probe::a1::Rec) 1 0)
    (wat.i64/+
      (wat.core/if (wat.runtime/is-type? :probe::a1::En) 2 0)
      (wat.i64/+
        (wat.core/if (wat.runtime/is-type? :probe::a1::Surf) 4 0)
        (wat.i64/+
          (wat.core/if (wat.runtime/is-type? :probe::a1::Cap) 8 0)
          (wat.i64/+
            (wat.core/if (wat.runtime/is-type? :probe::a1::Box) 16 0)
            (wat.i64/+
              (wat.core/if (wat.runtime/is-type? :probe::kw::Rec) 32 0)
              (wat.i64/+
                (wat.core/if (wat.runtime/is-type? :probe::KwCap) 64 0)
                (wat.i64/+
                  (wat.core/if (wat.runtime/is-type? :probe::Api) 128 0)
                  (wat.i64/+
                    (wat.core/if (wat.runtime/is-type? :probe::a1svc::State) 256 0)
                    (wat.i64/+
                      (wat.core/if (wat.core/= (probe.a1/inc 41) 42) 512 0)
                      (wat.i64/+
                        (wat.core/if (wat.core/= (:probe::kw::inc 41) 42) 1024 0)
                        (wat.i64/+
                          (wat.core/if (wat.core/= (probe.a1/mac) 7) 2048 0)
                          (wat.core/if (wat.core/= (:probe::a1::mac) 7) 4096 0))))))))))))))
