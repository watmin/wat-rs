(ns wat-edn.scanner
  "Tiny scanner that extracts type declarations from `.wat` source.
  Recognizes only `(:wat::core::struct ...)` forms — the surface
  Clojure consumers care about. Everything else (functions, macros,
  imports) is skipped.

  Scope, not full wat. The format is a header file:
  same artifact wat-rs's type checker reads, exposed to Clojure
  for typed read/write/gen.

  The wat language's `::` namespace separator collides with
  Clojure's reader, so we hand-roll a small scanner rather than
  delegating to clojure.edn. ~150 LOC, no dependencies."
  (:require [clojure.string :as str]))

;; ─── Tokenizer ──────────────────────────────────────────────────

(defn- whitespace? [c]
  (or (Character/isWhitespace ^char c) (= \, c)))

(defn- ident-char? [c]
  ;; Characters that can appear inside a symbol/keyword body in wat.
  (or (Character/isLetterOrDigit ^char c)
      (#{\- \_ \. \? \! \* \+ \= \< \> \: \/ \&} c)))

(defn- skip-trivia
  "Advance index past whitespace, commas, and `;`-to-EOL comments."
  [^String s i]
  (let [n (.length s)]
    (loop [i i]
      (cond
        (>= i n) n
        (whitespace? (.charAt s i)) (recur (inc i))
        (= \; (.charAt s i))
        (let [j (.indexOf s "\n" i)]
          (recur (if (neg? j) n (inc j))))
        :else i))))

(defn tokenize
  "Tokenize `.wat` source into a vector of tokens. Tokens are:
    :lparen          — `(`
    :rparen          — `)`
    {:kw \"...\"}    — keyword body (without leading `:`)
    {:sym \"...\"}   — symbol body
  Strings and other literals aren't relevant to type extraction
  and are skipped."
  [^String s]
  (let [n (.length s)
        out (transient [])]
    (loop [i (skip-trivia s 0)]
      (if (>= i n)
        (persistent! out)
        (let [c (.charAt s i)]
          (cond
            (= \( c)
            (do (conj! out :lparen) (recur (skip-trivia s (inc i))))

            (= \) c)
            (do (conj! out :rparen) (recur (skip-trivia s (inc i))))

            (= \" c)
            ;; Skip strings — not relevant for type extraction.
            (let [j (loop [j (inc i)]
                      (cond
                        (>= j n) n
                        (= \\ (.charAt s j)) (recur (+ j 2))
                        (= \" (.charAt s j)) (inc j)
                        :else (recur (inc j))))]
              (recur (skip-trivia s j)))

            (= \: c)
            (let [j (loop [j (inc i)]
                      (if (and (< j n) (ident-char? (.charAt s j)))
                        (recur (inc j))
                        j))]
              (conj! out {:kw (subs s (inc i) j)})
              (recur (skip-trivia s j)))

            :else
            (let [j (loop [j i]
                      (if (and (< j n) (ident-char? (.charAt s j)))
                        (recur (inc j))
                        j))]
              (when (> j i)
                (conj! out {:sym (subs s i j)}))
              (recur (skip-trivia s (max (inc i) j))))))))))

;; ─── Parser ─────────────────────────────────────────────────────

(defn- find-paren-balanced-end
  "Given tokens and start index pointing at `:lparen`, find the
  index of the matching `:rparen`. Returns nil if unbalanced."
  [tokens start]
  (loop [i (inc start) depth 1]
    (cond
      (>= i (count tokens)) nil
      (= :lparen (nth tokens i)) (recur (inc i) (inc depth))
      (= :rparen (nth tokens i)) (if (= 1 depth)
                                   i
                                   (recur (inc i) (dec depth)))
      :else (recur (inc i) depth))))

(defn- wat-path->edn-tag
  "Convert a wat keyword-path body like `enterprise::config::SizeAdjust`
  into the EDN tag form `enterprise.config/SizeAdjust` (last `::`
  becomes `/`, preceding `::` become `.`)."
  [^String body]
  (let [last-idx (.lastIndexOf body "::")]
    (if (neg? last-idx)
      body
      (str (.replace (subs body 0 last-idx) "::" ".")
           "/"
           (subs body (+ last-idx 2))))))

(defn- parse-field-form
  "Parse a single (field-name :Type) form. Returns
  [field-keyword type-spec next-index] or nil."
  [tokens i]
  (when (= :lparen (nth tokens i nil))
    (let [name-tok (nth tokens (+ i 1) nil)
          type-tok (nth tokens (+ i 2) nil)
          close    (nth tokens (+ i 3) nil)]
      (when (and (map? name-tok) (:sym name-tok)
                 (map? type-tok) (:kw type-tok)
                 (= :rparen close))
        [(keyword (:sym name-tok))
         (:kw type-tok)
         (+ i 4)]))))

(defn- parse-struct-form
  "Given tokens starting at the struct's outer `:lparen`, parse the
  full form. Returns {:tag-symbol ... :fields {field type ...}} or
  throws on malformed input."
  [tokens start]
  (let [end (find-paren-balanced-end tokens start)
        ;; Layout: ( :wat::core::struct :path::Name (field :Type) ...)
        ;;         ^start                ^marker     ^bodies     ^end
        marker (nth tokens (+ start 1))
        path   (nth tokens (+ start 2))]
    (when-not (and (map? marker)
                   (= "wat::core::struct" (:kw marker)))
      (throw (ex-info "expected :wat::core::struct marker"
                      {:got marker :index start})))
    (when-not (and (map? path) (:kw path))
      (throw (ex-info "expected :path::TypeName after struct marker"
                      {:got path :index (+ start 2)})))
    (let [tag (wat-path->edn-tag (:kw path))
          fields (loop [i (+ start 3) acc {}]
                   (if (or (>= i end) (= i end))
                     acc
                     (if-let [[fname ftype next-i] (parse-field-form tokens i)]
                       (recur next-i (assoc acc fname ftype))
                       (recur (inc i) acc))))]
      {:tag (symbol tag)
       :fields fields})))

(defn extract-structs
  "Return a vector of {:tag :fields} maps, one per
  (:wat::core::struct ...) form found in `wat-source`."
  [^String wat-source]
  (let [tokens (tokenize wat-source)
        n (count tokens)]
    (loop [i 0 acc []]
      (cond
        (>= i n) acc

        ;; Match `( :wat::core::struct ...`
        (and (= :lparen (nth tokens i))
             (= "wat::core::struct"
                (:kw (nth tokens (inc i) nil))))
        (let [end (find-paren-balanced-end tokens i)
              parsed (parse-struct-form tokens i)]
          (recur (inc end) (conj acc parsed)))

        :else
        (recur (inc i) acc)))))

(defn extract-from-file
  "Read a `.wat` file from disk and extract its struct declarations."
  [path]
  (extract-structs (slurp path)))
