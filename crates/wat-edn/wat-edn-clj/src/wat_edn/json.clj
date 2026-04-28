(ns wat-edn.json
  "EDN ↔ JSON conversion. Mirrors the wire convention used by the
  Rust side (see crates/wat-edn/src/json.rs).

  Sentinel-key tagged objects preserve EDN type fidelity through
  JSON's smaller type system:

    EDN value          JSON shape
    ─────────────────  ────────────────────────────────────────────
    nil                null
    true / false       true / false
    integer            number  (or string if outside JS-safe range)
    bigint             {\"#bigint\": \"123N\"}
    float              number
    NaN / ±Inf         {\"#float\": \"nan|inf|neg-inf\"}
    bigdec             {\"#bigdec\": \"3.14M\"}
    string             string
    char               {\"#char\": \"X\"}
    keyword            \":foo\" / \":ns/foo\"   (colon discriminator)
    symbol             {\"#symbol\": \"foo\"}
    list / vector      array (round-trips as vector)
    map (string keys)  object {\"k\": v, ...}
    map (other keys)   object — non-string keys serialized as EDN
    set                {\"#set\": [...]}
    tagged             {\"#tag\": \"ns/name\", \"body\": ...}
    inst               {\"#inst\": \"2026-04-28T16:00:00Z\"}
    uuid               {\"#uuid\": \"550e8400-...\"}"
  (:require [cheshire.core :as cheshire]
            [clojure.edn :as edn]
            [clojure.string :as str]
            [wat-edn.core :as wat])
  (:import [java.util Date UUID]
           [java.time Instant]
           [java.time.format DateTimeFormatter]))

;; JS-safe integer range (Number.MAX_SAFE_INTEGER == 2^53 - 1).
(def ^:private safe-int-max 9007199254740991)
(def ^:private safe-int-min -9007199254740991)

(declare edn->json json->edn)

;; ─── EDN → JSON ────────────────────────────────────────────────

(defn- inst->iso8601 [^Date d]
  (-> d (.toInstant) (.toString)))

(defn- map-key->json-key [k]
  ;; String keys pass through. Anything else gets EDN-stringified
  ;; so the reader can parse it back via clojure.edn/read-string.
  (if (string? k)
    k
    (binding [*print-dup* false] (pr-str k))))

(defn edn->json
  "Convert a Clojure value (typically read via wat-edn-clj or
  pure clojure.edn) into a Clojure data structure that cheshire
  can serialize as JSON. Wire convention preserves type fidelity."
  [v]
  (cond
    (nil? v) nil
    (boolean? v) v

    ;; integers
    (integer? v)
    (if (and (<= v safe-int-max) (>= v safe-int-min))
      v
      (str v))

    ;; floats
    (float? v)
    (cond
      (Double/isNaN v) {"#float" "nan"}
      (Double/isInfinite v)
      {"#float" (if (neg? v) "neg-inf" "inf")}
      :else v)

    (decimal? v) {"#bigdec" (str v "M")}

    (string? v) v

    (char? v) {"#char" (str v)}

    (keyword? v)
    (str v)  ; includes leading `:`

    (symbol? v) {"#symbol" (str v)}

    (instance? Date v) {"#inst" (inst->iso8601 v)}

    (instance? UUID v) {"#uuid" (str v)}

    ;; Variants are vectors with `:type ::variant` metadata; check
    ;; them BEFORE the generic vector branch.
    (wat/some-variant? v) {"#tag" "wat.core/Some" "body" (edn->json (second v))}
    (wat/none-variant? v) {"#tag" "wat.core/None" "body" nil}
    (wat/ok-variant? v)   {"#tag" "wat.core/Ok"   "body" (edn->json (second v))}
    (wat/err-variant? v)  {"#tag" "wat.core/Err"  "body" (edn->json (second v))}

    (tagged-literal? v)
    {"#tag" (str (:tag v))
     "body" (edn->json (:form v))}

    (set? v) {"#set" (mapv edn->json v)}

    (vector? v) (mapv edn->json v)
    (sequential? v) (mapv edn->json v)  ; lists / seqs collapse to arrays

    (map? v)
    (reduce-kv (fn [acc k val]
                 (assoc acc (map-key->json-key k) (edn->json val)))
               {}
               v)

    :else
    (throw (ex-info "edn->json: unsupported value type"
                    {:value v :class (class v)}))))

(defn to-json-string
  "Convert a Clojure value to a compact JSON string."
  [v]
  (cheshire/generate-string (edn->json v)))

(defn to-json-pretty
  "Convert a Clojure value to a pretty-printed JSON string."
  [v]
  (cheshire/generate-string (edn->json v) {:pretty true}))

;; ─── JSON → EDN ────────────────────────────────────────────────

(defn- string->edn-value [s]
  ;; Strings starting with `:` are EDN keywords; otherwise plain.
  (if (and (string? s) (str/starts-with? s ":"))
    (let [body (subs s 1)]
      (if (str/blank? body)
        s  ; treat ":" alone as plain string
        (try (edn/read-string s)
             (catch Throwable _ s))))
    s))

(defn- parse-map-key
  "Try to parse a JSON object key as EDN; fall back to string."
  [k]
  (let [c (when (seq k) (first k))]
    (if (and c (#{\: \[ \{ \( \# \"} c))
      (try (edn/read-string k)
           (catch Throwable _ k))
      k)))

(defn- coerce-number
  "cheshire returns `Integer` for small JSON ints, but Clojure
  literal `1` is `Long`. They compare equal via Clojure's `=` but
  `TaggedLiteral.equals` (and other Java-side equals) treats them
  as different. Coerce all integer values to `Long` for consistent
  cross-language equality."
  [n]
  (cond
    (instance? Long n) n
    (instance? Integer n) (long n)
    (instance? Short n) (long n)
    (instance? Byte n) (long n)
    :else n))

(defn json->edn
  "Convert a Clojure data structure produced by cheshire (parsed
  from JSON) back into idiomatic Clojure/EDN. Recognizes the
  sentinel-key conventions emitted by edn->json."
  [v]
  (cond
    (nil? v) nil
    (boolean? v) v
    (number? v) (coerce-number v)
    (string? v) (string->edn-value v)

    ;; cheshire returns persistent vectors for JSON arrays, but also
    ;; covers lazy seqs / arrays-as-lists; sequential? is safer.
    (sequential? v) (mapv json->edn v)

    (map? v)
    (cond
      ;; Single-key sentinels
      (and (= 1 (count v)) (contains? v "#bigint"))
      (let [s (get v "#bigint")
            trimmed (if (str/ends-with? s "N") (subs s 0 (dec (count s))) s)]
        (BigInteger. trimmed))

      (and (= 1 (count v)) (contains? v "#bigdec"))
      (let [s (get v "#bigdec")
            trimmed (if (str/ends-with? s "M") (subs s 0 (dec (count s))) s)]
        (bigdec trimmed))

      (and (= 1 (count v)) (contains? v "#float"))
      (case (get v "#float")
        "nan"     (Double/NaN)
        "inf"     (Double/POSITIVE_INFINITY)
        "neg-inf" (Double/NEGATIVE_INFINITY)
        (throw (ex-info "invalid #float sentinel" {:value v})))

      (and (= 1 (count v)) (contains? v "#char"))
      (let [s (get v "#char")]
        (when-not (= 1 (count s))
          (throw (ex-info "#char body must be one character" {:value v})))
        (.charAt s 0))

      (and (= 1 (count v)) (contains? v "#symbol"))
      (symbol (get v "#symbol"))

      (and (= 1 (count v)) (contains? v "#set"))
      (set (map json->edn (get v "#set")))

      (and (= 1 (count v)) (contains? v "#inst"))
      (Date/from (Instant/parse (get v "#inst")))

      (and (= 1 (count v)) (contains? v "#uuid"))
      (UUID/fromString (get v "#uuid"))

      ;; Two-key tagged element
      (and (= 2 (count v))
           (contains? v "#tag")
           (contains? v "body"))
      (let [tag-s (get v "#tag")
            body  (json->edn (get v "body"))]
        (cond
          (= "wat.core/Some" tag-s) (wat/some-of body)
          (= "wat.core/None" tag-s) (wat/none-of)
          (= "wat.core/Ok"   tag-s) (wat/ok-of body)
          (= "wat.core/Err"  tag-s) (wat/err-of body)
          :else (tagged-literal (symbol tag-s) body)))

      ;; Plain map: parse non-string-looking keys as EDN
      :else
      (reduce-kv (fn [acc k val]
                   (assoc acc (parse-map-key k) (json->edn val)))
                 {}
                 v))))

(defn from-json-string
  "Parse a JSON string into a Clojure value, with sentinel
  keys reconstructed back to their EDN types."
  [^String s]
  (json->edn (cheshire/parse-string s)))
