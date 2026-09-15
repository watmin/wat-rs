;; where-inline-keyword.clj — the Clara half of the KEYWORD/ENUM CONSTANT axis.
;;
;; ── WHY CLARA IS THE RIGHT ARBITER HERE, AND WHAT IT SHOWS US ────────────────────────────────
;;
;; Our side refused `(keyword::= :tag :alpha)` inline for the life of the engine, because a keyword
;; in operand position was read as a FIELD REFERENCE unconditionally — so a keyword or enum
;; CONSTANT could not be written there at all.
;;
;; Clojure has no such collision, and that is the interesting part: a FIELD is a bare symbol
;; (`tag`) and a CONSTANT is a keyword (`:alpha`), so the two readings are lexically distinct and
;; nothing has to be inferred. Rows 5-6 below make that explicit — `(= tag beta)` compares two
;; FIELDS, and it is written differently from `(= tag :beta)` which would compare against a
;; constant. That is precisely the distinction the EDN / symbol-head migration will give us; until
;; then our rule recovers the same answer by asking whether the keyword names a declared field.
;;
;; ⚠ CLARA HAS NO ENUM TYPE. Rows 3-4 mirror the SEMANTICS — a closed set of tags compared for
;; equality — with a Clojure keyword standing in for the enum variant. The .clj twin has always
;; been a semantic mirror rather than a syntactic one (Clara has no `where` fence either; `:test`
;; is the mirror of that), and what is being compared is the derived SET, which is identical.
;;
;; `mk-session` takes an EXPLICIT production list per row — never the namespace symbol — so the six
;; rules do not collapse into one session and union their derived sets.

(ns where-inline-keyword
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]))

(def items 210)

(defrecord Req [k tag beta grade])
(defrecord Hit [k])

;; ROW 1 — INLINE keyword constant. Our side refused this outright until 2026-08-28.
(defrule inline-kw
  [Req (= ?k k) (= tag :alpha)]
  => (insert! (->Hit ?k)))

;; ROW 2 — FENCE via :test. This position always worked on our side.
(defrule fence-kw
  [Req (= ?k k) (= ?t tag)]
  [:test (= ?t :alpha)]
  => (insert! (->Hit ?k)))

;; ROW 3 — INLINE enum constant. On our side `:wik::G::Hi` carries `::` and could never have been
;; a field name at all; here it is a plain keyword standing for the same variant.
(defrule inline-enum
  [Req (= ?k k) (= grade :Hi)]
  => (insert! (->Hit ?k)))

;; ROW 4 — FENCE.
(defrule fence-enum
  [Req (= ?k k) (= ?g grade)]
  [:test (= ?g :Hi)]
  => (insert! (->Hit ?k)))

;; ROW 5 — ⛔ THE FIELD WINS. `beta` is a FIELD here, spelled as a bare symbol, so this compares
;; `tag` against the field `beta` — never against the constant `:beta`. Our side writes the same
;; comparison as `(keyword::= :tag :beta)` and must reach this same answer; the seed makes the two
;; readings differ (field-reading selects all 210, constant-reading would select 110).
(defrule inline-shadow
  [Req (= ?k k) (= tag beta)]
  => (insert! (->Hit ?k)))

;; ROW 6 — FENCE, the same comparison through bindings.
(defrule fence-shadow
  [Req (= ?k k) (= ?t tag) (= ?b beta)]
  [:test (= ?t ?b)]
  => (insert! (->Hit ?k)))

(defquery hit-q [] [?fact <- Hit])

(def rows
  [[1 "inline-kw"     inline-kw]
   [2 "fence-kw"      fence-kw]
   [3 "inline-enum"   inline-enum]
   [4 "fence-enum"    fence-enum]
   [5 "inline-shadow" inline-shadow]
   [6 "fence-shadow"  fence-shadow]])

(def seeds
  (mapv (fn [i]
          (->Req i
                 (if (< i 100) :alpha :beta)
                 (if (< i 100) :alpha :beta)
                 (if (< i 60) :Hi :Lo)))
        (range items)))

(defn run-row [[row nm rule]]
  (let [session (apply insert (mk-session [rule hit-q] :cache false) seeds)
        codes   (sort (map #(:k (:?fact %)) (query (fire-rules session) hit-q)))]
    ;; Mirrors the wat side's `render-ints` fold EXACTLY — one leading space per element.
    (str "row " row " " nm " n=" (count codes) " ->"
         (apply str (map #(str " " %) codes)))))

;; `prn`, not `println`: matches wat's :wat::kernel::println EDN-quoting of Strings.
(defn -main [& _] (doseq [r rows] (prn (run-row r))))

(-main)
