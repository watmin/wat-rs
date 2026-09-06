;; probe-what-a-large-fn-costs.wat
;;
;; The committed closure probes used TINY bodies (`fn [] nil`).
;; sqs.wat's `once-put` / `once-del` are large nested matches, and
;; `eval_fn` clones the body AST (`synthesize_fn_body`) on every
;; construction. If cost scales with body size, the tiny probes
;; understated the send-arm closures and this is the 934 ms.
;;
;; Also: DESIGN remaining hypothesis — a large continuation sitting
;; in a `let` after dummy bindings vs inside an `if` arm.
;;
;; None of these CALL the constructed fns. Construction only.
;; Match-on-keyword is illegal, so the large body is nested let/if.

(:wat::config::set-redef! true)

(:wat::core::defn :lf::now [] -> :wat::core::i64
  (:wat::time::epoch-nanos (:wat::time::now)))

(:wat::core::defn :lf::us [a <- :wat::core::i64 b <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::/ (:wat::i64::- b a) 1000))

(:wat::core::defn :lf::ns-op [a <- :wat::core::i64 b <- :wat::core::i64 n <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::/ (:wat::i64::- b a) n))

;; TINY — the killed-hypothesis control.
(:wat::core::defn :lf::tiny [acc <- :wat::core::i64 i <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let
    [nap  (:wat::core::fn [] -> :wat::core::nil nil)
     once (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::bool (:wat::i64::> x 0))]
    (:wat::i64::+ acc 1)))

;; LARGE-BODY — once-put-ish nested if/let, never called.
(:wat::core::defn :lf::large-body [acc <- :wat::core::i64 i <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let
    [nap  (:wat::core::fn [] -> :wat::core::nil
            (:wat::core::let
              [a (:wat::i64::+ i 1)
               b (:wat::core::format "nap-{a}" :a a)]
              nil))
     once (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::bool
            (:wat::core::if (:wat::i64::= x 0)
              false
              (:wat::core::if (:wat::i64::= x 1)
                true
                (:wat::core::if (:wat::i64::= x 2)
                  (:wat::kernel::assertion-failed! "lf Constraint" :wat::core::None :wat::core::None)
                  (:wat::core::if (:wat::i64::= x 3)
                    (:wat::kernel::assertion-failed! "lf Fatal" :wat::core::None :wat::core::None)
                    (:wat::core::if (:wat::i64::= x 4)
                      (:wat::kernel::assertion-failed! "lf RequestTooLarge" :wat::core::None :wat::core::None)
                      (:wat::kernel::assertion-failed! "lf RequestMalformed" :wat::core::None :wat::core::None)))))))]
    (:wat::i64::+ acc 1)))

;; LARGE-TYPE — tiny body, nested parametric type like once-put's Peer.
(:wat::core::defn :lf::large-type [acc <- :wat::core::i64 i <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let
    [once (:wat::core::fn
            [st <- (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String])]
            -> :wat::core::bool
            false)]
    (:wat::i64::+ acc 1)))

;; LARGE CONTINUATION inside an if arm (old send shape analogue).
(:wat::core::defn :lf::body-in-if [acc <- :wat::core::i64 i <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::i64::>= i 0)
    (:wat::core::let
      [a (:wat::i64::+ i 1) b (:wat::i64::+ i 2) c (:wat::i64::+ i 3)
       d (:wat::i64::+ i 4) e (:wat::i64::+ i 5) f (:wat::i64::+ i 6)
       g (:wat::core::format "{a}-{b}-{c}" :a a :b b :c c)
       h (:wat::core::count (:wat::core::range 0 8))
       s1 (:wat::i64::+ a b)
       s2 (:wat::i64::+ s1 c)
       s3 (:wat::i64::+ s2 d)
       s4 (:wat::i64::+ s3 e)
       s5 (:wat::i64::+ s4 f)
       s  (:wat::i64::+ s5 h)]
      (:wat::i64::+ acc s))
    acc))

;; Same continuation AFTER a dummy `_ok` let (new send shape analogue).
(:wat::core::defn :lf::body-in-let [acc <- :wat::core::i64 i <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let
    [_ok (:wat::core::if (:wat::i64::>= i 0) nil nil)
     a (:wat::i64::+ i 1) b (:wat::i64::+ i 2) c (:wat::i64::+ i 3)
     d (:wat::i64::+ i 4) e (:wat::i64::+ i 5) f (:wat::i64::+ i 6)
     g (:wat::core::format "{a}-{b}-{c}" :a a :b b :c c)
     h (:wat::core::count (:wat::core::range 0 8))
     s1 (:wat::i64::+ a b)
     s2 (:wat::i64::+ s1 c)
     s3 (:wat::i64::+ s2 d)
     s4 (:wat::i64::+ s3 e)
     s5 (:wat::i64::+ s4 f)
     s  (:wat::i64::+ s5 h)]
    (:wat::i64::+ acc s)))

(:wat::core::defn :lf::run [] -> :wat::core::String
  (:wat::core::let
    [xs (:wat::core::range 0 8000)
     n  (:wat::core::count xs)
     t0 (:lf::now)  _a (:wat::core::foldl :lf::tiny        0 xs)
     t1 (:lf::now)  _b (:wat::core::foldl :lf::large-body  0 xs)
     t2 (:lf::now)  _c (:wat::core::foldl :lf::large-type  0 xs)
     t3 (:lf::now)  _d (:wat::core::foldl :lf::body-in-if  0 xs)
     t4 (:lf::now)  _e (:wat::core::foldl :lf::body-in-let 0 xs)
     t5 (:lf::now)]
    (:wat::core::format
      "n={n};tiny-us={a};large-body-us={b};large-type-us={c};body-in-if-us={d};body-in-let-us={e};tiny-ns/op={ta};large-body-ns/op={tb};large-type-ns/op={tc};let-minus-if-ns/op={lm}"
      :n n
      :a (:lf::us t0 t1) :b (:lf::us t1 t2) :c (:lf::us t2 t3)
      :d (:lf::us t3 t4) :e (:lf::us t4 t5)
      :ta (:lf::ns-op t0 t1 n) :tb (:lf::ns-op t1 t2 n) :tc (:lf::ns-op t2 t3 n)
      :lm (:wat::i64::/ (:wat::i64::- (:wat::i64::- t5 t4) (:wat::i64::- t4 t3)) n))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:lf::run)))
