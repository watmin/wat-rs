(ns wat-edn.core
  "wat-edn-clj — Clojure-side bridge to the wat-edn EDN format.

  Three parts:
    1. Default reader for the wat.* tag namespace (no setup needed)
    2. Schema-driven generators (load-types!, gen, read-typed, validate)
    3. Variant helpers (some-of/none-of/ok-of/err-of) and writers

  Schema lives in `.wat` files — the same files wat-rs's type
  checker consumes. Clojure consumers point load-types! at them
  and get type-checked generators back."
  (:refer-clojure :exclude [read])
  (:require [clojure.edn :as edn]
            [clojure.java.io :as io]
            [wat-edn.scanner :as scanner]))

;; ─── Reader: default fn for wat.* tags ──────────────────────────

(defn- starts-with? [^String s ^String prefix]
  (.startsWith s prefix))

(defn wat-default-reader
  "Default tag-handler for the wat.* namespace. Strips the type
  tag for collection wrappers (their body is already idiomatic
  Clojure data); wraps sum variants for identity preservation;
  passes any unknown tag through as a tagged-literal so consumers
  can pattern-match or ignore."
  [tag body]
  (let [s (str tag)]
    (cond
      (or (starts-with? s "wat.core/Vec")
          (starts-with? s "wat.core/HashMap")
          (starts-with? s "wat.core/HashSet")
          (starts-with? s "wat.holon/")
          (starts-with? s "wat.scalar/"))
      body

      (starts-with? s "wat.core/Some")
      (with-meta [::some body] {:type ::variant})

      (starts-with? s "wat.core/None")
      (with-meta [::none] {:type ::variant})

      (starts-with? s "wat.core/Ok")
      (with-meta [::ok body] {:type ::variant})

      (starts-with? s "wat.core/Err")
      (with-meta [::err body] {:type ::variant})

      :else (tagged-literal tag body))))

;; ─── Type registry ─────────────────────────────────────────────

(def ^:private types-registry
  "{tag-symbol → {field-keyword → type-spec-string}}
  Populated by load-types!. Empty by default."
  (atom {}))

(defn list-types
  "Return a sorted vector of all currently-registered tag symbols."
  []
  (vec (sort (keys @types-registry))))

(defn type-fields
  "Return the {field-keyword → type-spec} map for `tag-symbol`,
  or nil if the type isn't registered."
  [tag-symbol]
  (get @types-registry tag-symbol))

(defn clear-types!
  "Empty the type registry. Useful for tests."
  []
  (reset! types-registry {}))

(defn load-types!
  "Load type declarations from one or more `.wat` files (paths
  on the local filesystem). Each (:wat::core::struct ...) form
  is parsed and added to the registry, keyed by its EDN tag form
  (last `::` → `/`, preceding `::` → `.`).

  Idempotent: re-loading the same file replaces prior entries
  for those types; loading additional files extends the registry."
  [& paths]
  (doseq [path paths]
    (let [structs (scanner/extract-from-file path)]
      (doseq [{:keys [tag fields]} structs]
        (swap! types-registry assoc tag fields))))
  nil)

;; ─── Type validation ───────────────────────────────────────────
;;
;; v0.1: validate primitive types strictly; treat collections and
;; user-defined types as opaque (any value accepted). The strict
;; primitive check catches the common typo class.

(defn- type-matches?
  "True if `value` plausibly matches `type-spec` (a wat type
  string like \"Keyword\", \"i64\", \"Vec<...>\", etc.)."
  [type-spec value]
  (case type-spec
    "Keyword"  (keyword? value)
    "String"   (string? value)
    "i64"      (integer? value)
    "f64"      (or (float? value) (decimal? value) (integer? value))
    "bool"     (boolean? value)
    "Bytes"    (bytes? value)
    ;; Generic / unknown — accept anything for v0.1.
    true))

(defn- check-fields
  "Validate `m` against the registered schema for `tag`.
  Returns a vector of {:field :expected :got} for each mismatch.
  Empty vector means valid."
  [tag m]
  (let [schema (get @types-registry tag)]
    (when-not schema
      (throw (ex-info (str "unknown type " tag " — call load-types! first")
                      {:tag tag :registered (keys @types-registry)})))
    (vec
      (for [[field expected] schema
            :let [v (get m field ::missing)]
            :when (or (= v ::missing)
                      (not (type-matches? expected v)))]
        {:field field
         :expected expected
         :got (if (= v ::missing) ::missing v)}))))

(defn validate
  "Throw ex-info if `m` doesn't match the schema for `tag`.
  Returns `m` unchanged on success."
  [tag m]
  (let [errors (check-fields tag m)]
    (when (seq errors)
      (throw (ex-info (str "validation failed for " tag)
                      {:tag tag :errors errors :got m})))
    m))

;; ─── Generators ─────────────────────────────────────────────────

(defn gen
  "Build a tagged-literal for `tag-symbol` with body `m`. Validates
  fields against the registered schema before constructing.
  Throws on type mismatch or missing fields.

  ```
  (gen 'enterprise.config/SizeAdjust
       {:asset :BTC :factor 1.5})
  ```"
  [tag-symbol m]
  (validate tag-symbol m)
  (tagged-literal tag-symbol m))

(defn emit
  "Build a tagged-literal as `gen` does, then `pr-str` it to a
  String — the EDN bytes the wat side will read."
  [tag-symbol m]
  (binding [*print-dup* false]
    (pr-str (gen tag-symbol m))))

;; ─── Read API ──────────────────────────────────────────────────

(defn read
  "Read one EDN form from a reader, using the wat default-fn
  for any wat.* tag. Other tags become Clojure tagged-literals."
  ([rdr] (read rdr ::eof))
  ([rdr eof]
   (edn/read {:readers {}
              :default wat-default-reader
              :eof     eof}
             rdr)))

(defn read-str
  "Parse a single EDN form from a string."
  [s]
  (edn/read-string {:readers {}
                    :default wat-default-reader}
                   s))

(defn read-stream
  "Read all top-level EDN forms from a reader."
  [rdr]
  (let [pbr (if (instance? java.io.PushbackReader rdr)
              rdr
              (java.io.PushbackReader. rdr))]
    (loop [out []]
      (let [v (read pbr ::done)]
        (if (= v ::done)
          out
          (recur (conj out v)))))))

(defn read-typed
  "Parse an EDN string, expect a tagged value of `tag-symbol`,
  validate the body against the registered schema, return the
  body map. Throws on type mismatch, missing fields, or wrong
  tag."
  [tag-symbol s]
  (let [parsed (read-str s)]
    (cond
      (not (tagged-literal? parsed))
      (throw (ex-info (str "expected #" tag-symbol " value, got non-tagged")
                      {:expected tag-symbol :got parsed}))

      (not= tag-symbol (:tag parsed))
      (throw (ex-info (str "expected #" tag-symbol ", got #" (:tag parsed))
                      {:expected tag-symbol :actual (:tag parsed)}))

      :else
      (validate tag-symbol (:form parsed)))))

;; ─── Write API ─────────────────────────────────────────────────

(defmethod print-method ::variant
  [v ^java.io.Writer w]
  (let [kind (first v)]
    (case kind
      ::some (do (.write w "#wat.core/Some ")
                 (print-method (second v) w))
      ::none (.write w "#wat.core/None nil")
      ::ok   (do (.write w "#wat.core/Ok ")
                 (print-method (second v) w))
      ::err  (do (.write w "#wat.core/Err ")
                 (print-method (second v) w)))))

(defn write-str
  "Serialize a Clojure value as EDN. Variant helpers
  (some-of/none-of/ok-of/err-of) emit as wat.core/Some etc.
  Tagged-literals built via `gen` round-trip."
  [v]
  (binding [*print-dup* false]
    (pr-str v)))

(defn print-line!
  "Write a value to a writer as one EDN line."
  [^java.io.Writer w v]
  (.write w (write-str v))
  (.write w "\n")
  (.flush w))

(defn append-file!
  "Append a value to an EDN file as one line."
  [path v]
  (with-open [w (io/writer path :append true)]
    (print-line! w v)))

;; ─── Variant helpers ───────────────────────────────────────────

(defn some-of  [x] (with-meta [::some x] {:type ::variant}))
(defn none-of  []  (with-meta [::none]   {:type ::variant}))
(defn ok-of    [x] (with-meta [::ok x]   {:type ::variant}))
(defn err-of   [e] (with-meta [::err e]  {:type ::variant}))

(defn some-variant? [x] (and (vector? x) (= ::some (first x))))
(defn none-variant? [x] (and (vector? x) (= ::none (first x))))
(defn ok-variant?   [x] (and (vector? x) (= ::ok   (first x))))
(defn err-variant?  [x] (and (vector? x) (= ::err  (first x))))

(defn unwrap-some [x] (when (some-variant? x) (second x)))
(defn unwrap-ok   [x] (when (ok-variant?   x) (second x)))
(defn unwrap-err  [x] (when (err-variant?  x) (second x)))
