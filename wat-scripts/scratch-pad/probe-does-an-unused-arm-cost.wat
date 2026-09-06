;; probe-does-an-unused-arm-cost.wat
;;
;; SCORE-find-the-934-milliseconds claims: "A match pays for unused arms. Their
;; AST is walked on every evaluation." That is an extraordinary claim about the
;; interpreter — it would apply to every match in the tree.
;;
;; The evidence offered was V4, a CIRCUIT-level variant (+1697ms), which
;; confounds unused-arm size with a duplicated body, a larger file and a changed
;; success path. And probe-what-a-six-arm-match-costs.wat measures arm COUNT with
;; TINY arms — it cannot see an arm-SIZE cost.
;;
;; ONE VARIABLE: the same match, always taking the same first arm. The second arm
;; is tiny in one function and huge in the other. Nothing else differs.
;;
;; Refutation: if tiny and huge cost the same, unused arms are NOT walked and the
;; mechanism in the SCORE is misnamed — whatever V4 measured, it was not this.

(:wat::config::set-redef! true)

(:wat::core::defn :ua::now [] -> :wat::core::i64
  (:wat::time::epoch-nanos (:wat::time::now)))

(:wat::core::defenum :ua::R :wat::enum::Pure
  :Success []
  :Other [])

;; TAKEN arm identical in both. Only the UNUSED arm differs.
(:wat::core::defn :ua::add [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::+ acc x))

(:wat::core::defn :ua::tiny [acc <- :wat::core::i64 i <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::match (:ua::R::Success)
    ((:ua::R::Success) (:wat::i64::+ acc 1))
    ((:ua::R::Other) acc)))

(:wat::core::defn :ua::huge [acc <- :wat::core::i64 i <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::match (:ua::R::Success)
    ((:ua::R::Success) (:wat::i64::+ acc 1))
    ((:ua::R::Other)
      ;; Never evaluated. Deliberately large: nested lets, a nested match,
      ;; formats and vectors — roughly the shape of sqs.wat's `once-put`.
      (:wat::core::let
        [a01 (:wat::i64::+ i 1)   a02 (:wat::i64::+ i 2)   a03 (:wat::i64::+ i 3)
         a04 (:wat::i64::+ i 4)   a05 (:wat::i64::+ i 5)   a06 (:wat::i64::+ i 6)
         v1  (:wat::core::Vector :- [:wat::core::String] "aa" "bb" "cc" "dd")
         s1  (:wat::core::format "{a}-{b}-{c}" :a a01 :b a02 :c a03)
         s2  (:wat::core::format "{a}-{b}-{c}" :a a04 :b a05 :c a06)
         inner (:wat::core::match (:ua::R::Other)
                 ((:ua::R::Success)
                   (:wat::core::let
                     [b01 (:wat::i64::* a01 2) b02 (:wat::i64::* a02 3)
                      b03 (:wat::core::format "{x}" :x b01)
                      b04 (:wat::core::count v1)]
                     (:wat::i64::+ b01 (:wat::i64::+ b02 b04))))
                 ((:ua::R::Other)
                   (:wat::core::let
                     [c01 (:wat::i64::* a03 5) c02 (:wat::i64::* a04 7)
                      c03 (:wat::core::format "{y}-{z}" :y c01 :z s1)
                      c04 (:wat::core::count (:wat::core::range 0 4))]
                     (:wat::i64::+ c01 (:wat::i64::+ c02 c04)))))
         inner2 (:wat::core::match (:ua::R::Other)
                 ((:ua::R::Success)
                   (:wat::core::let
                     [b21 (:wat::i64::* a01 2) b22 (:wat::i64::* a02 3)
                      b23 (:wat::core::format "{x}" :x b21)
                      b24 (:wat::core::count v1)]
                     (:wat::i64::+ b21 (:wat::i64::+ b22 b24))))
                 ((:ua::R::Other)
                   (:wat::core::let
                     [c21 (:wat::i64::* a03 5) c22 (:wat::i64::* a04 7)
                      c23 (:wat::core::format "{y}-{z}" :y c21 :z s1)
                      c24 (:wat::core::count (:wat::core::range 0 4))]
                     (:wat::i64::+ c21 (:wat::i64::+ c22 c24)))))
         inner3 (:wat::core::match (:ua::R::Other)
                 ((:ua::R::Success)
                   (:wat::core::let
                     [b31 (:wat::i64::* a01 2) b32 (:wat::i64::* a02 3)
                      b33 (:wat::core::format "{x}" :x b31)
                      b34 (:wat::core::count v1)]
                     (:wat::i64::+ b31 (:wat::i64::+ b32 b34))))
                 ((:ua::R::Other)
                   (:wat::core::let
                     [c31 (:wat::i64::* a03 5) c32 (:wat::i64::* a04 7)
                      c33 (:wat::core::format "{y}-{z}" :y c31 :z s1)
                      c34 (:wat::core::count (:wat::core::range 0 4))]
                     (:wat::i64::+ c31 (:wat::i64::+ c32 c34)))))
         inner4 (:wat::core::match (:ua::R::Other)
                 ((:ua::R::Success)
                   (:wat::core::let
                     [b41 (:wat::i64::* a01 2) b42 (:wat::i64::* a02 3)
                      b43 (:wat::core::format "{x}" :x b41)
                      b44 (:wat::core::count v1)]
                     (:wat::i64::+ b41 (:wat::i64::+ b42 b44))))
                 ((:ua::R::Other)
                   (:wat::core::let
                     [c41 (:wat::i64::* a03 5) c42 (:wat::i64::* a04 7)
                      c43 (:wat::core::format "{y}-{z}" :y c41 :z s1)
                      c44 (:wat::core::count (:wat::core::range 0 4))]
                     (:wat::i64::+ c41 (:wat::i64::+ c42 c44)))))
         inner5 (:wat::core::match (:ua::R::Other)
                 ((:ua::R::Success)
                   (:wat::core::let
                     [b51 (:wat::i64::* a01 2) b52 (:wat::i64::* a02 3)
                      b53 (:wat::core::format "{x}" :x b51)
                      b54 (:wat::core::count v1)]
                     (:wat::i64::+ b51 (:wat::i64::+ b52 b54))))
                 ((:ua::R::Other)
                   (:wat::core::let
                     [c51 (:wat::i64::* a03 5) c52 (:wat::i64::* a04 7)
                      c53 (:wat::core::format "{y}-{z}" :y c51 :z s1)
                      c54 (:wat::core::count (:wat::core::range 0 4))]
                     (:wat::i64::+ c51 (:wat::i64::+ c52 c54)))))
         inner6 (:wat::core::match (:ua::R::Other)
                 ((:ua::R::Success)
                   (:wat::core::let
                     [b61 (:wat::i64::* a01 2) b62 (:wat::i64::* a02 3)
                      b63 (:wat::core::format "{x}" :x b61)
                      b64 (:wat::core::count v1)]
                     (:wat::i64::+ b61 (:wat::i64::+ b62 b64))))
                 ((:ua::R::Other)
                   (:wat::core::let
                     [c61 (:wat::i64::* a03 5) c62 (:wat::i64::* a04 7)
                      c63 (:wat::core::format "{y}-{z}" :y c61 :z s1)
                      c64 (:wat::core::count (:wat::core::range 0 4))]
                     (:wat::i64::+ c61 (:wat::i64::+ c62 c64)))))
         inner7 (:wat::core::match (:ua::R::Other)
                 ((:ua::R::Success)
                   (:wat::core::let
                     [b71 (:wat::i64::* a01 2) b72 (:wat::i64::* a02 3)
                      b73 (:wat::core::format "{x}" :x b71)
                      b74 (:wat::core::count v1)]
                     (:wat::i64::+ b71 (:wat::i64::+ b72 b74))))
                 ((:ua::R::Other)
                   (:wat::core::let
                     [c71 (:wat::i64::* a03 5) c72 (:wat::i64::* a04 7)
                      c73 (:wat::core::format "{y}-{z}" :y c71 :z s1)
                      c74 (:wat::core::count (:wat::core::range 0 4))]
                     (:wat::i64::+ c71 (:wat::i64::+ c72 c74)))))
         inner8 (:wat::core::match (:ua::R::Other)
                 ((:ua::R::Success)
                   (:wat::core::let
                     [b81 (:wat::i64::* a01 2) b82 (:wat::i64::* a02 3)
                      b83 (:wat::core::format "{x}" :x b81)
                      b84 (:wat::core::count v1)]
                     (:wat::i64::+ b81 (:wat::i64::+ b82 b84))))
                 ((:ua::R::Other)
                   (:wat::core::let
                     [c81 (:wat::i64::* a03 5) c82 (:wat::i64::* a04 7)
                      c83 (:wat::core::format "{y}-{z}" :y c81 :z s1)
                      c84 (:wat::core::count (:wat::core::range 0 4))]
                     (:wat::i64::+ c81 (:wat::i64::+ c82 c84)))))
         inner9 (:wat::core::match (:ua::R::Other)
                 ((:ua::R::Success)
                   (:wat::core::let
                     [b91 (:wat::i64::* a01 2) b92 (:wat::i64::* a02 3)
                      b93 (:wat::core::format "{x}" :x b91)
                      b94 (:wat::core::count v1)]
                     (:wat::i64::+ b91 (:wat::i64::+ b92 b94))))
                 ((:ua::R::Other)
                   (:wat::core::let
                     [c91 (:wat::i64::* a03 5) c92 (:wat::i64::* a04 7)
                      c93 (:wat::core::format "{y}-{z}" :y c91 :z s1)
                      c94 (:wat::core::count (:wat::core::range 0 4))]
                     (:wat::i64::+ c91 (:wat::i64::+ c92 c94)))))
         inner10 (:wat::core::match (:ua::R::Other)
                 ((:ua::R::Success)
                   (:wat::core::let
                     [b101 (:wat::i64::* a01 2) b102 (:wat::i64::* a02 3)
                      b103 (:wat::core::format "{x}" :x b101)
                      b104 (:wat::core::count v1)]
                     (:wat::i64::+ b101 (:wat::i64::+ b102 b104))))
                 ((:ua::R::Other)
                   (:wat::core::let
                     [c101 (:wat::i64::* a03 5) c102 (:wat::i64::* a04 7)
                      c103 (:wat::core::format "{y}-{z}" :y c101 :z s1)
                      c104 (:wat::core::count (:wat::core::range 0 4))]
                     (:wat::i64::+ c101 (:wat::i64::+ c102 c104)))))
         inner11 (:wat::core::match (:ua::R::Other)
                 ((:ua::R::Success)
                   (:wat::core::let
                     [b111 (:wat::i64::* a01 2) b112 (:wat::i64::* a02 3)
                      b113 (:wat::core::format "{x}" :x b111)
                      b114 (:wat::core::count v1)]
                     (:wat::i64::+ b111 (:wat::i64::+ b112 b114))))
                 ((:ua::R::Other)
                   (:wat::core::let
                     [c111 (:wat::i64::* a03 5) c112 (:wat::i64::* a04 7)
                      c113 (:wat::core::format "{y}-{z}" :y c111 :z s1)
                      c114 (:wat::core::count (:wat::core::range 0 4))]
                     (:wat::i64::+ c111 (:wat::i64::+ c112 c114)))))
         inner12 (:wat::core::match (:ua::R::Other)
                 ((:ua::R::Success)
                   (:wat::core::let
                     [b121 (:wat::i64::* a01 2) b122 (:wat::i64::* a02 3)
                      b123 (:wat::core::format "{x}" :x b121)
                      b124 (:wat::core::count v1)]
                     (:wat::i64::+ b121 (:wat::i64::+ b122 b124))))
                 ((:ua::R::Other)
                   (:wat::core::let
                     [c121 (:wat::i64::* a03 5) c122 (:wat::i64::* a04 7)
                      c123 (:wat::core::format "{y}-{z}" :y c121 :z s1)
                      c124 (:wat::core::count (:wat::core::range 0 4))]
                     (:wat::i64::+ c121 (:wat::i64::+ c122 c124)))))
         inner13 (:wat::core::match (:ua::R::Other)
                 ((:ua::R::Success)
                   (:wat::core::let
                     [b131 (:wat::i64::* a01 2) b132 (:wat::i64::* a02 3)
                      b133 (:wat::core::format "{x}" :x b131)
                      b134 (:wat::core::count v1)]
                     (:wat::i64::+ b131 (:wat::i64::+ b132 b134))))
                 ((:ua::R::Other)
                   (:wat::core::let
                     [c131 (:wat::i64::* a03 5) c132 (:wat::i64::* a04 7)
                      c133 (:wat::core::format "{y}-{z}" :y c131 :z s1)
                      c134 (:wat::core::count (:wat::core::range 0 4))]
                     (:wat::i64::+ c131 (:wat::i64::+ c132 c134)))))
         inner14 (:wat::core::match (:ua::R::Other)
                 ((:ua::R::Success)
                   (:wat::core::let
                     [b141 (:wat::i64::* a01 2) b142 (:wat::i64::* a02 3)
                      b143 (:wat::core::format "{x}" :x b141)
                      b144 (:wat::core::count v1)]
                     (:wat::i64::+ b141 (:wat::i64::+ b142 b144))))
                 ((:ua::R::Other)
                   (:wat::core::let
                     [c141 (:wat::i64::* a03 5) c142 (:wat::i64::* a04 7)
                      c143 (:wat::core::format "{y}-{z}" :y c141 :z s1)
                      c144 (:wat::core::count (:wat::core::range 0 4))]
                     (:wat::i64::+ c141 (:wat::i64::+ c142 c144)))))
         inner15 (:wat::core::match (:ua::R::Other)
                 ((:ua::R::Success)
                   (:wat::core::let
                     [b151 (:wat::i64::* a01 2) b152 (:wat::i64::* a02 3)
                      b153 (:wat::core::format "{x}" :x b151)
                      b154 (:wat::core::count v1)]
                     (:wat::i64::+ b151 (:wat::i64::+ b152 b154))))
                 ((:ua::R::Other)
                   (:wat::core::let
                     [c151 (:wat::i64::* a03 5) c152 (:wat::i64::* a04 7)
                      c153 (:wat::core::format "{y}-{z}" :y c151 :z s1)
                      c154 (:wat::core::count (:wat::core::range 0 4))]
                     (:wat::i64::+ c151 (:wat::i64::+ c152 c154)))))
         inner16 (:wat::core::match (:ua::R::Other)
                 ((:ua::R::Success)
                   (:wat::core::let
                     [b161 (:wat::i64::* a01 2) b162 (:wat::i64::* a02 3)
                      b163 (:wat::core::format "{x}" :x b161)
                      b164 (:wat::core::count v1)]
                     (:wat::i64::+ b161 (:wat::i64::+ b162 b164))))
                 ((:ua::R::Other)
                   (:wat::core::let
                     [c161 (:wat::i64::* a03 5) c162 (:wat::i64::* a04 7)
                      c163 (:wat::core::format "{y}-{z}" :y c161 :z s1)
                      c164 (:wat::core::count (:wat::core::range 0 4))]
                     (:wat::i64::+ c161 (:wat::i64::+ c162 c164)))))
         inner17 (:wat::core::match (:ua::R::Other)
                 ((:ua::R::Success)
                   (:wat::core::let
                     [b171 (:wat::i64::* a01 2) b172 (:wat::i64::* a02 3)
                      b173 (:wat::core::format "{x}" :x b171)
                      b174 (:wat::core::count v1)]
                     (:wat::i64::+ b171 (:wat::i64::+ b172 b174))))
                 ((:ua::R::Other)
                   (:wat::core::let
                     [c171 (:wat::i64::* a03 5) c172 (:wat::i64::* a04 7)
                      c173 (:wat::core::format "{y}-{z}" :y c171 :z s1)
                      c174 (:wat::core::count (:wat::core::range 0 4))]
                     (:wat::i64::+ c171 (:wat::i64::+ c172 c174)))))
         inner18 (:wat::core::match (:ua::R::Other)
                 ((:ua::R::Success)
                   (:wat::core::let
                     [b181 (:wat::i64::* a01 2) b182 (:wat::i64::* a02 3)
                      b183 (:wat::core::format "{x}" :x b181)
                      b184 (:wat::core::count v1)]
                     (:wat::i64::+ b181 (:wat::i64::+ b182 b184))))
                 ((:ua::R::Other)
                   (:wat::core::let
                     [c181 (:wat::i64::* a03 5) c182 (:wat::i64::* a04 7)
                      c183 (:wat::core::format "{y}-{z}" :y c181 :z s1)
                      c184 (:wat::core::count (:wat::core::range 0 4))]
                     (:wat::i64::+ c181 (:wat::i64::+ c182 c184)))))
         inner19 (:wat::core::match (:ua::R::Other)
                 ((:ua::R::Success)
                   (:wat::core::let
                     [b191 (:wat::i64::* a01 2) b192 (:wat::i64::* a02 3)
                      b193 (:wat::core::format "{x}" :x b191)
                      b194 (:wat::core::count v1)]
                     (:wat::i64::+ b191 (:wat::i64::+ b192 b194))))
                 ((:ua::R::Other)
                   (:wat::core::let
                     [c191 (:wat::i64::* a03 5) c192 (:wat::i64::* a04 7)
                      c193 (:wat::core::format "{y}-{z}" :y c191 :z s1)
                      c194 (:wat::core::count (:wat::core::range 0 4))]
                     (:wat::i64::+ c191 (:wat::i64::+ c192 c194)))))
         inner20 (:wat::core::match (:ua::R::Other)
                 ((:ua::R::Success)
                   (:wat::core::let
                     [b201 (:wat::i64::* a01 2) b202 (:wat::i64::* a02 3)
                      b203 (:wat::core::format "{x}" :x b201)
                      b204 (:wat::core::count v1)]
                     (:wat::i64::+ b201 (:wat::i64::+ b202 b204))))
                 ((:ua::R::Other)
                   (:wat::core::let
                     [c201 (:wat::i64::* a03 5) c202 (:wat::i64::* a04 7)
                      c203 (:wat::core::format "{y}-{z}" :y c201 :z s1)
                      c204 (:wat::core::count (:wat::core::range 0 4))]
                     (:wat::i64::+ c201 (:wat::i64::+ c202 c204)))))
         inner21 (:wat::core::match (:ua::R::Other)
                 ((:ua::R::Success)
                   (:wat::core::let
                     [b211 (:wat::i64::* a01 2) b212 (:wat::i64::* a02 3)
                      b213 (:wat::core::format "{x}" :x b211)
                      b214 (:wat::core::count v1)]
                     (:wat::i64::+ b211 (:wat::i64::+ b212 b214))))
                 ((:ua::R::Other)
                   (:wat::core::let
                     [c211 (:wat::i64::* a03 5) c212 (:wat::i64::* a04 7)
                      c213 (:wat::core::format "{y}-{z}" :y c211 :z s1)
                      c214 (:wat::core::count (:wat::core::range 0 4))]
                     (:wat::i64::+ c211 (:wat::i64::+ c212 c214)))))
         bulk (:wat::core::foldl :ua::add 0 (:wat::core::Vector :- [:wat::core::i64] inner2 inner3 inner4 inner5 inner6 inner7 inner8 inner9 inner10 inner11 inner12 inner13 inner14 inner15 inner16 inner17 inner18 inner19 inner20 inner21))
         tail (:wat::core::let
                [d01 (:wat::core::count s2) d02 (:wat::i64::+ inner d01)
                 d03 (:wat::core::format "{q}" :q d02)]
                (:wat::i64::+ d02 (:wat::core::count d03)))]
        (:wat::i64::+ acc (:wat::i64::+ tail bulk))))))

(:wat::core::defn :ua::run [] -> :wat::core::String
  (:wat::core::let
    [xs (:wat::core::range 0 8000)
     n  (:wat::core::count xs)
     ;; interleaved, tiny first and last, to absorb any drift
     t0 (:ua::now)  _a (:wat::core::foldl :ua::tiny 0 xs)
     t1 (:ua::now)  _b (:wat::core::foldl :ua::huge 0 xs)
     t2 (:ua::now)  _c (:wat::core::foldl :ua::tiny 0 xs)
     t3 (:ua::now)
     tiny1 (:wat::i64::- t1 t0)
     huge  (:wat::i64::- t2 t1)
     tiny2 (:wat::i64::- t3 t2)
     tiny-avg (:wat::i64::/ (:wat::i64::+ tiny1 tiny2) 2)]
    (:wat::core::format
      "n={n};tiny1-ns/op={a};huge-ns/op={b};tiny2-ns/op={c};huge-minus-tiny-ns/op={d}"
      :n n
      :a (:wat::i64::/ tiny1 n)
      :b (:wat::i64::/ huge n)
      :c (:wat::i64::/ tiny2 n)
      :d (:wat::i64::/ (:wat::i64::- huge tiny-avg) n))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:ua::run)))
