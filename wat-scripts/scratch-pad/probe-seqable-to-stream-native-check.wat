;; probe-seqable-to-stream-native-check.wat — Arc-278 DESIGN-STONE seq-traversal-one-door,
;; Strike 1: sanity-check the native `seqable->stream` beyond the RED-gate wall test.
;;
;; TWO THINGS THE WALL TEST DOESN'T PROVE:
;;   (a) results are IDENTICAL across Vector / List / PersistentVector sources (the door is
;;       shared correctly, not just "fast"), for several delegating verbs.
;;   (b) laziness genuinely survives: forcing only `first` of a `keep` pipeline over a
;;       4000-element Vector must NOT invoke the predicate more than once — a side-effecting
;;       predicate (println) makes the invocation count observable.
;;
;; SAFE: pure collections + one println count, no rete, no forks.
;;   ./target/release/wat wat-scripts/scratch-pad/probe-seqable-to-stream-native-check.wat

(:wat::core::defn :cx::pos? [x <- :wat::core::i64] -> (:wat::core::Option :- [:wat::core::i64])
  (:wat::core::if (:wat::core::>= x 0) (:wat::core::Some x) :wat::core::None))

(:wat::core::defn :cx::build-list [n <- :wat::core::i64] -> (:wat::core::List :- [:wat::core::i64])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::List :- [:wat::core::i64])  i <- :wat::core::i64] -> (:wat::core::List :- [:wat::core::i64])
      (:wat::core::List/conj acc i))
    (:wat::core::List)
    (:wat::core::reverse (:wat::core::range 0 n))))

(:wat::core::defn :cx::build-pv [n <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::i64])  i <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat::core::i64])
      (:wat::core::PersistentVector/conj acc i))
    (:wat::core::PersistentVector)
    (:wat::core::range 0 n)))

;; A side-effecting "predicate" — println's, then always keeps. Lets us COUNT invocations by
;; counting printed lines (rather than eyeballing timing).
(:wat::core::defn :cx::counting-keep [x <- :wat::core::i64] -> (:wat::core::Option :- [:wat::core::i64])
  (:wat::core::let [__ (:wat::kernel::println (:wat::i64::to-string x))]
    (:wat::core::Some x)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [
    n 10
    v  (:wat::core::range 0 n)
    l  (:cx::build-list n)
    pv (:cx::build-pv n)

    ;; (a) cross-container agreement for several delegating verbs.
    keep-v  (:wat::core::into (:wat::core::Vector :wat::core::i64) (:wat::core::keep :cx::pos? v))
    keep-l  (:wat::core::into (:wat::core::Vector :wat::core::i64) (:wat::core::keep :cx::pos? l))
    keep-pv (:wat::core::into (:wat::core::Vector :wat::core::i64) (:wat::core::keep :cx::pos? pv))

    dedupe-v  (:wat::core::into (:wat::core::Vector :wat::core::i64) (:wat::core::dedupe v))
    dedupe-l  (:wat::core::into (:wat::core::Vector :wat::core::i64) (:wat::core::dedupe l))
    dedupe-pv (:wat::core::into (:wat::core::Vector :wat::core::i64) (:wat::core::dedupe pv))

    distinct-v  (:wat::core::into (:wat::core::Vector :wat::core::i64) (:wat::core::distinct v))
    distinct-l  (:wat::core::into (:wat::core::Vector :wat::core::i64) (:wat::core::distinct l))
    distinct-pv (:wat::core::into (:wat::core::Vector :wat::core::i64) (:wat::core::distinct pv))

    map-idx-v  (:wat::core::into (:wat::core::Vector :wat::core::i64) (:wat::core::map-indexed (:wat::core::fn [i <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64 (:wat::i64::+ i x)) v))
    map-idx-l  (:wat::core::into (:wat::core::Vector :wat::core::i64) (:wat::core::map-indexed (:wat::core::fn [i <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64 (:wat::i64::+ i x)) l))
    map-idx-pv (:wat::core::into (:wat::core::Vector :wat::core::i64) (:wat::core::map-indexed (:wat::core::fn [i <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64 (:wat::i64::+ i x)) pv))

    take-nth-v  (:wat::core::into (:wat::core::Vector :wat::core::i64) (:wat::core::take-nth 3 v))
    take-nth-l  (:wat::core::into (:wat::core::Vector :wat::core::i64) (:wat::core::take-nth 3 l))
    take-nth-pv (:wat::core::into (:wat::core::Vector :wat::core::i64) (:wat::core::take-nth 3 pv))

    ;; cond over negated pairwise tests — short-circuits to false at the first disagreement,
    ;; terminal :else is the last pairwise test (cleaner than a nested-if pyramid).
    agree (:wat::core::cond
            ((:wat::core::not= keep-v keep-l) false)
            ((:wat::core::not= keep-v keep-pv) false)
            ((:wat::core::not= dedupe-v dedupe-l) false)
            ((:wat::core::not= dedupe-v dedupe-pv) false)
            ((:wat::core::not= distinct-v distinct-l) false)
            ((:wat::core::not= distinct-v distinct-pv) false)
            ((:wat::core::not= map-idx-v map-idx-l) false)
            ((:wat::core::not= map-idx-v map-idx-pv) false)
            ((:wat::core::not= take-nth-v take-nth-l) false)
            (:else (:wat::core::= take-nth-v take-nth-pv)))

    __ (:wat::kernel::println (:wat::string::concat "cross-container-agree=" (:wat::core::bool::to-string agree)))

    ;; (b) laziness — force ONLY ONE cell of a `keep` pipeline over a BIG Vector. If the
    ;; predicate prints once, only ONE element was ever touched; the pipeline did not
    ;; realize the whole stream to answer the first cell.
    ;; Stone 118.B4-iii — THE WALL: was `(first (keep :cx::counting-keep big))`. `first` no
    ;; longer accepts a Stream (`keep` is lazy, arc 118.2a) — `:wat::stream::next` is the door
    ;; now, and it proves the SAME thing: one `NextOutcome::Item` means one cell realized,
    ;; identical to what `first` used to demonstrate.
    big     (:wat::core::range 0 4000)
    __hdr   (:wat::kernel::println "--- laziness probe: expect exactly ONE line below ---")
    fst     (:wat::core::match (:wat::stream::next (:wat::core::keep :cx::counting-keep big))
              ((:wat::stream::NextOutcome::Item value _rest) value)
              (:wat::stream::NextOutcome::Exhausted
                (:wat::kernel::assertion-failed! "keep: unexpectedly exhausted" :wat::core::None :wat::core::None)))
    __ftr   (:wat::kernel::println "--- end laziness probe ---")]
    (:wat::kernel::println (:wat::string::concat "first=" (:wat::i64::to-string fst)))))
