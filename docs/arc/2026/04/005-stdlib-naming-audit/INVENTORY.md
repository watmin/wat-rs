# wat — Primitive Inventory

**Status:** Pass 1 draft, 2026-04-20. Reflects HEAD at commit
`2cfa40f` (FOUNDATION sweep). Every row below is grep-verified
against `src/runtime.rs`, `src/check.rs`, `src/rust_deps/*`, and
`wat/std/*.wat`.

**Purpose:** the canonical answer to "does this path exist and
how do I call it?" FOUNDATION.md answers the WHY (what kind of
operation is this); INVENTORY answers the WHAT and the HOW.
Updated on every slice that adds or renames a primitive.

**Reading the rows.** Each entry names the path, its type
signature (where known), its kind (form / primitive / macro /
program / struct / enum / typealias / …), and the source file +
line it lands at. When the checker has a typed scheme, its
location is also cited; when dispatch is special-cased, the
`infer_*` function is named.

---

## `:wat::core::*` — language core

### Forms (parse-time / evaluation machinery)

| Path | Signature | Kind | Source |
|---|---|---|---|
| `:wat::core::define` | `(name params -> ret) body` — registers a function at startup | form | `runtime.rs` → `register_defines`; `check.rs` → `infer` refuses as a value |
| `:wat::core::lambda` | `((params -> ret) body)` — runtime-valued function | form | `runtime.rs::eval_lambda`; `check.rs::infer_lambda` |
| `:wat::core::let` | `(((n :T) rhs) …) body` — parallel typed bindings | form | `runtime.rs::eval_let`; `check.rs::infer_let` |
| `:wat::core::let*` | `(((n :T) rhs) …) body` — sequential typed bindings | form | `runtime.rs::eval_let_star`; `check.rs::infer_let_star` |
| `:wat::core::if` | `cond -> :T then else` — typed branch | form | `runtime.rs::eval_if` (`eval_if_tail`); `check.rs::infer_if` |
| `:wat::core::match` | `scrut -> :T (pat body) …` — typed pattern match | form | `runtime.rs::eval_match` (`eval_match_tail`); `check.rs::infer_match` |
| `:wat::core::try` | `<result-expr>` — Ok-unwrap or Err-propagate | form (INSCRIPTION 058-033) | `runtime.rs::eval_try`; `check.rs::infer_try` |
| `:wat::core::quote` | `<ast>` — returns the unevaluated AST as `:wat::WatAST` | form | `runtime.rs::eval_quote` |
| `:wat::core::atom-value` | `(:wat::holon::HolonAST) -> :T` — reads atom literal | primitive | `runtime.rs::eval_atom_value` |
| `:wat::core::use!` | `:rust::Type` — per-program opt-in declaration | form | `runtime.rs`: Unit; `check.rs::infer` |

### Type declarations (compile-time; refused at eval)

| Path | Kind | Source |
|---|---|---|
| `:wat::core::struct` | product type declaration | `types.rs::parse_struct` |
| `:wat::core::enum` | coproduct type declaration | `types.rs::parse_enum` |
| `:wat::core::newtype` | nominal wrapper declaration | `types.rs::parse_newtype` |
| `:wat::core::typealias` | structural alias declaration | `types.rs::parse_typealias` (expansion via `types.rs::expand_alias` and `check.rs::reduce`) |

### Macro + load machinery

| Path | Kind | Source |
|---|---|---|
| `:wat::core::defmacro` | macro registration (incl. variadic `&`) | `macros.rs::parse_defmacro_form` |
| `:wat::core::quasiquote` | template form (used inside defmacro bodies) | `macros.rs::expand_template` |
| `:wat::core::unquote` | `,x` splice-one inside quasiquote | `macros.rs::unquote_argument` |
| `:wat::core::unquote-splicing` | `,@x` splice-list inside quasiquote | `macros.rs::splice_argument` |
| `:wat::core::load!` | build-time module loader | `freeze.rs::resolve_loads` |
| `:wat::core::digest-load!` | digest-verified load | `freeze.rs::resolve_loads` |
| `:wat::core::signed-load!` | signature-verified load | `freeze.rs::resolve_loads` |

### Eval-family (runtime dynamic evaluation)

Return `:Result<wat::holon::HolonAST, :wat::core::EvalError>` per the
2026-04-19 inscription.

| Path | Signature | Source |
|---|---|---|
| `:wat::core::eval-ast!` | `:wat::WatAST -> :Result<wat::holon::HolonAST, EvalError>` | `runtime.rs::eval_form_ast` |
| `:wat::core::eval-edn!` | EDN source → parse → eval | `runtime.rs::eval_form_edn` |
| `:wat::core::eval-digest!` | digest-verified EDN eval | `runtime.rs::eval_form_digest` |
| `:wat::core::eval-signed!` | signature-verified EDN eval | `runtime.rs::eval_form_signed` |

### Arithmetic (strict, no promotion)

| Path | Signature | Source |
|---|---|---|
| `:wat::core::i64::+` | `:i64 × :i64 -> :i64` | `runtime.rs::eval_i64_arith` |
| `:wat::core::i64::-` | `:i64 × :i64 -> :i64` | same |
| `:wat::core::i64::*` | `:i64 × :i64 -> :i64` | same |
| `:wat::core::i64::/` | `:i64 × :i64 -> :i64` (DivisionByZero on 0) | same |
| `:wat::core::f64::+` | `:f64 × :f64 -> :f64` | `runtime.rs::eval_f64_arith` |
| `:wat::core::f64::-` | `:f64 × :f64 -> :f64` | same |
| `:wat::core::f64::*` | `:f64 × :f64 -> :f64` | same |
| `:wat::core::f64::/` | `:f64 × :f64 -> :f64` | same |

### Comparison + boolean

| Path | Signature | Source |
|---|---|---|
| `:wat::core::=` | `∀T. T × T -> :bool` | `runtime.rs::eval_comparison` |
| `:wat::core::<` | same | same |
| `:wat::core::<=` | same | same |
| `:wat::core::>` | same | same |
| `:wat::core::>=` | same | same |
| `:wat::core::not` | `:bool -> :bool` | `runtime.rs::eval_not` |
| `:wat::core::and` | `:bool × :bool -> :bool` (short-circuit) | `runtime.rs::eval_and_or` |
| `:wat::core::or` | `:bool × :bool -> :bool` (short-circuit) | same |

### Collection primitives

| Path | Signature | Source |
|---|---|---|
| `:wat::core::vec` | `:T × T* -> :Vec<T>` | `runtime.rs::eval_list_ctor`; `check.rs::infer_list_constructor` |
| `:wat::core::list` | alias for `:wat::core::vec` | same |
| `:wat::core::tuple` | `T1 × T2 × … -> :(T1,T2,…)` | `runtime.rs::eval_tuple_ctor`; `check.rs::infer_tuple_constructor` |
| `:wat::core::conj` | `∀T. :Vec<T> × T -> :Vec<T>` (immutable append) | `runtime.rs::eval_conj` (INSCRIPTION on 058-026) |
| `:wat::core::first` | polymorphic over tuple / Vec | `check.rs::infer_positional_accessor` (pos 0) |
| `:wat::core::second` | same (pos 1) | same (pos 1) |
| `:wat::core::third` | same (pos 2) | same (pos 2) |
| `:wat::core::rest` | `∀T. :Vec<T> -> :Vec<T>` | `runtime.rs::eval_vec_rest` |
| `:wat::core::empty?` | `∀T. :Vec<T> -> :bool` | `runtime.rs::eval_vec_empty` |
| `:wat::core::length` | `∀T. :Vec<T> -> :i64` | `runtime.rs::eval_vec_length` |
| `:wat::core::reverse` | `∀T. :Vec<T> -> :Vec<T>` | `runtime.rs::eval_vec_reverse` |
| `:wat::core::take` | `∀T. :i64 × :Vec<T> -> :Vec<T>` | `runtime.rs::eval_vec_take` |
| `:wat::core::drop` | `∀T. :i64 × :Vec<T> -> :Vec<T>` | `runtime.rs::eval_vec_drop` |
| `:wat::core::range` | `:i64 × :i64 -> :Vec<i64>` | `runtime.rs::eval_range` |
| `:wat::core::map` | `∀T,U. :Vec<T> × :fn(T)->U -> :Vec<U>` | `runtime.rs::eval_vec_map` |
| `:wat::core::filter` | `∀T. :Vec<T> × :fn(T)->bool -> :Vec<T>` | `runtime.rs::eval_vec_filter` |
| `:wat::core::foldl` | `∀T,Acc. :Vec<T> × :Acc × :fn(Acc,T)->Acc -> :Acc` | `runtime.rs::eval_vec_foldl` |
| `:wat::core::foldr` | `∀T,Acc. :Vec<T> × :Acc × :fn(T,Acc)->Acc -> :Acc` | `runtime.rs::eval_vec_foldr` |

### Internal primitives (auto-generated-access underlayer; users don't call)

| Path | Kind | Source |
|---|---|---|
| `:wat::core::struct-new` | struct construction helper | `runtime.rs::eval_struct_new` |
| `:wat::core::struct-field` | struct field-access helper | `runtime.rs::eval_struct_field` |

### Built-in types registered via `TypeEnv::with_builtins`

| Path | Shape | Source |
|---|---|---|
| `:wat::holon::CapacityExceeded` | struct `{ cost :i64, budget :i64 }` | `types.rs::register_builtin_types` |
| `:wat::core::EvalError` | struct `{ kind :String, message :String }` | same |

**Note.** `:Option<T>` and `:Result<T,E>` are built-in enums but
their declarations live in FOUNDATION's enum examples; they're
dispatched via Value variants directly rather than through the
declared-type registry. The pattern matcher and constructor
dispatch are in `runtime.rs` (see `eval_some_ctor`, `eval_ok_ctor`,
`eval_err_ctor`, `try_match_pattern`).

---

## `:wat::config::*` — ambient startup constants

### Setters (banged; one-shot at entry file)

| Path | Signature | Source |
|---|---|---|
| `:wat::config::set-dims!` | `:i64 -> :()` | `freeze.rs::collect_entry_file` |
| `:wat::config::set-capacity-mode!` | `:wat::core::keyword -> :()` (`:silent` / `:warn` / `:error` / `:abort`) | same |
| `:wat::config::set-global-seed!` | `:i64 -> :()` | same |
| `:wat::config::set-noise-floor!` | `:f64 -> :()` | same |

### Accessors (runtime-readable)

| Path | Signature | Source |
|---|---|---|
| `:wat::config::dims` | `-> :i64` | `runtime.rs::eval_config_dims` |
| `:wat::config::global-seed` | `-> :i64` | `runtime.rs::eval_config_global_seed` |
| `:wat::config::noise-floor` | `-> :f64` | `runtime.rs::eval_config_noise_floor` |

**Note.** `:wat::config::capacity-mode` accessor is spec'd in
FOUNDATION but not currently exposed as a runtime primitive;
the mode is read internally by `:wat::holon::Bundle`. Flagged
in Pass 2 as a referenced-but-not-shipped candidate if user
code needs to observe the mode.

---

## `:wat::holon::*` — algebra core (6 + 2 measurements)

Six vector-producing primitives; two scalar-returning
measurements.

| Path | Signature | Source |
|---|---|---|
| `:wat::holon::Atom` | `∀T. T -> :wat::holon::HolonAST` (typed atoms, 058-001) | `runtime.rs::eval_algebra_atom` |
| `:wat::holon::Bind` | `:wat::holon::HolonAST × :wat::holon::HolonAST -> :wat::holon::HolonAST` | `runtime.rs::eval_algebra_bind` |
| `:wat::holon::Bundle` | `:Vec<wat::holon::HolonAST> -> :Result<wat::holon::HolonAST, wat::holon::CapacityExceeded>` (058-003 INSCRIPTION) | `runtime.rs::eval_algebra_bundle` |
| `:wat::holon::Permute` | `:wat::holon::HolonAST × :i64 -> :wat::holon::HolonAST` | `runtime.rs::eval_algebra_permute` |
| `:wat::holon::Thermometer` | `:f64 × :f64 × :f64 -> :wat::holon::HolonAST` (value min max) | `runtime.rs::eval_algebra_thermometer` |
| `:wat::holon::Blend` | `:wat::holon::HolonAST × :wat::holon::HolonAST × :f64 × :f64 -> :wat::holon::HolonAST` | `runtime.rs::eval_algebra_blend` |
| `:wat::holon::cosine` | `:wat::holon::HolonAST × :wat::holon::HolonAST -> :f64` | `runtime.rs::eval_algebra_cosine` |
| `:wat::holon::dot` | `:wat::holon::HolonAST × :wat::holon::HolonAST -> :f64` | `runtime.rs::eval_algebra_dot` |
| `:wat::holon::presence?` | `:wat::holon::HolonAST × :wat::holon::HolonAST -> :f64` (cosine vs reference) | `runtime.rs::eval_algebra_presence` |

---

## `:wat::kernel::*` — concurrency primitives

| Path | Signature | Source |
|---|---|---|
| `:wat::kernel::make-bounded-queue` | `:T × :i64 -> :(Sender<T>, Receiver<T>)` | `runtime.rs::eval_kernel_make_bounded_queue` |
| `:wat::kernel::make-unbounded-queue` | `:T -> :(Sender<T>, Receiver<T>)` | `runtime.rs::eval_kernel_make_unbounded_queue` |
| `:wat::kernel::send` | `∀T. Sender<T> × T -> :Option<()>` (symmetric with recv, 2026-04-20) | `runtime.rs::eval_kernel_send` |
| `:wat::kernel::recv` | `∀T. Receiver<T> -> :Option<T>` | `runtime.rs::eval_kernel_recv` |
| `:wat::kernel::try-recv` | `∀T. Receiver<T> -> :Option<T>` (non-blocking) | `runtime.rs::eval_kernel_try_recv` |
| `:wat::kernel::drop` | `∀T. Sender<T> | Receiver<T> -> :()` | `runtime.rs::eval_kernel_drop`; `check.rs::infer_drop` |
| `:wat::kernel::select` | `:Vec<Receiver<T>> -> :(i64, :Option<T>)` | `runtime.rs::eval_kernel_select` |
| `:wat::kernel::spawn` | `<fn-path-or-lambda> × args... -> :ProgramHandle<R>` (accepts lambdas since 2026-04-20) | `runtime.rs::eval_kernel_spawn`; `check.rs::infer_spawn` |
| `:wat::kernel::join` | `:ProgramHandle<R> -> R` | `runtime.rs::eval_kernel_join` |

### HandlePool — claim-or-panic discipline

| Path | Signature | Source |
|---|---|---|
| `:wat::kernel::HandlePool::new` | `:String × :Vec<T> -> :HandlePool<T>` | `runtime.rs::eval_handle_pool_new` |
| `:wat::kernel::HandlePool::pop` | `:HandlePool<T> -> T` (panics empty) | `runtime.rs::eval_handle_pool_pop` |
| `:wat::kernel::HandlePool::finish` | `:HandlePool<T> -> :()` (panics on orphans) | `runtime.rs::eval_handle_pool_finish` |

### Signal queries (pollable kernel state)

| Path | Signature | Source |
|---|---|---|
| `:wat::kernel::stopped?` | `-> :bool` | `runtime.rs::eval_kernel_stopped` |
| `:wat::kernel::sigusr1?` | `-> :bool` | `runtime.rs::eval_user_signal_query` |
| `:wat::kernel::sigusr2?` | `-> :bool` | same |
| `:wat::kernel::sighup?` | `-> :bool` | same |
| `:wat::kernel::reset-sigusr1!` | `-> :()` | `runtime.rs::eval_user_signal_reset` |
| `:wat::kernel::reset-sigusr2!` | `-> :()` | same |
| `:wat::kernel::reset-sighup!` | `-> :()` | same |

---

## `:wat::io::*` — stdio gateways

| Path | Signature | Source |
|---|---|---|
| `:wat::io::IOReader/from-bytes` | `:Vec<u8> -> :wat::io::IOReader` (arc 008 slice 2) | `io.rs::eval_ioreader_from_bytes` |
| `:wat::io::IOReader/from-string` | `:String -> :wat::io::IOReader` (arc 008 slice 2) | `io.rs::eval_ioreader_from_string` |
| `:wat::io::IOReader/read` | `:IOReader × :i64 -> :Option<Vec<u8>>` | `io.rs::eval_ioreader_read` |
| `:wat::io::IOReader/read-all` | `:IOReader -> :Vec<u8>` | `io.rs::eval_ioreader_read_all` |
| `:wat::io::IOReader/read-line` | `:IOReader -> :Option<String>` | `io.rs::eval_ioreader_read_line` |
| `:wat::io::IOReader/rewind` | `:IOReader -> :()` | `io.rs::eval_ioreader_rewind` |
| `:wat::io::IOWriter/new` | `() -> :wat::io::IOWriter` | `io.rs::eval_iowriter_new` |
| `:wat::io::IOWriter/to-bytes` | `:IOWriter -> :Vec<u8>` | `io.rs::eval_iowriter_to_bytes` |
| `:wat::io::IOWriter/to-string` | `:IOWriter -> :Option<String>` | `io.rs::eval_iowriter_to_string` |
| `:wat::io::IOWriter/write` | `:IOWriter × :Vec<u8> -> :i64` | `io.rs::eval_iowriter_write` |
| `:wat::io::IOWriter/write-all` | `:IOWriter × :Vec<u8> -> :()` | `io.rs::eval_iowriter_write_all` |
| `:wat::io::IOWriter/write-string` | `:IOWriter × :String -> :i64` | `io.rs::eval_iowriter_write_string` |
| `:wat::io::IOWriter/print` | `:IOWriter × :String -> :()` (unit-returning convenience) | `io.rs::eval_iowriter_print` |
| `:wat::io::IOWriter/println` | `:IOWriter × :String -> :()` (unit-returning + `\n`) | `io.rs::eval_iowriter_println` |
| `:wat::io::IOWriter/writeln` | `:IOWriter × :String -> :i64` (adds `\n`) | `io.rs::eval_iowriter_writeln` |
| `:wat::io::IOWriter/flush` | `:IOWriter -> :()` | `io.rs::eval_iowriter_flush` |
| ~~`:wat::io::write`~~ | **retired 2026-04-21 (arc 008 slice 3)** — superseded by `:wat::io::IOWriter/print` and siblings | — |
| ~~`:wat::io::read-line`~~ | **retired 2026-04-21 (arc 008 slice 3)** — superseded by `:wat::io::IOReader/read-line` | — |

---

## `:wat::load::*` / `:wat::verify::*` / `:wat::eval::*` — verification vocabulary

Used inside `load!` / `digest-load!` / `signed-load!` / `eval-*!`
as first-class keyword arguments.

| Path | Role | Source |
|---|---|---|
| `:wat::load::file-path` | file-path loader mode | `freeze.rs::resolve_loads` |
| `:wat::eval::file-path` | file-path eval source | `runtime.rs::eval_form_edn` |
| `:wat::eval::string` | inline-string eval source | same |
| `:wat::verify::digest-sha256` | digest algorithm marker | `freeze.rs::resolve_loads` |
| `:wat::verify::signed-ed25519` | signature algorithm marker | same |
| `:wat::verify::file-path` | file-path verification payload | same |
| `:wat::verify::string` | inline-string verification payload | same |

---

## `:wat::std::*` — stdlib (macros, defines, programs, types)

### Algebra stdlib (named compositions over algebra core)

| Path | Kind | Source file |
|---|---|---|
| `:wat::holon::Amplify` | macro over Blend | `wat/holon/Amplify.wat` |
| `:wat::holon::Subtract` | macro over Blend (1, -1 weights) | `wat/holon/Subtract.wat` |
| `:wat::holon::Log` | macro over Thermometer with ln transform | `wat/holon/Log.wat` |
| `:wat::holon::Circular` | macro over Blend with cos/sin basis | `wat/holon/Circular.wat` |
| `:wat::holon::Reject` | macro over Blend + dot (Gram-Schmidt reject) | `wat/holon/Reject.wat` |
| `:wat::holon::Project` | macro `Subtract(x, Reject(x, y))` | `wat/holon/Project.wat` |
| `:wat::holon::Sequential` | macro — positional bind-chain | `wat/holon/Sequential.wat` |
| `:wat::holon::Ngram` | macro — n-wise adjacency | `wat/holon/Ngram.wat` |
| `:wat::holon::Bigram` | `Ngram 2` | `wat/holon/Bigram.wat` |
| `:wat::holon::Trigram` | `Ngram 3` | `wat/holon/Trigram.wat` |

### Reserved atom literals

| Path | Kind | Source |
|---|---|---|
| `:wat::std::circular-cos-basis` | atom literal (basis vector for Circular) | referenced from `Circular.wat` |
| `:wat::std::circular-sin-basis` | atom literal (basis vector for Circular) | same |

### Data-structure dispatch helpers

| Path | Signature | Source |
|---|---|---|
| `:wat::core::HashMap` | `:(K,V) × k1 v1 k2 v2 … -> :HashMap<K,V>` constructor | `runtime.rs::eval_hashmap_ctor`; `check.rs::infer_hashmap_constructor`. Moved from `:wat::std::*` → `:wat::core::*` in arc 021 (can't be written in wat). |
| `:wat::core::HashSet` | `:T × items… -> :HashSet<T>` constructor | `runtime.rs::eval_hashset_ctor`; `check.rs::infer_hashset_constructor`. Moved from `:wat::std::*` in arc 021. |
| `:wat::core::get` | polymorphic `get` on HashMap / HashSet / Vec | `runtime.rs::eval_std_get`; `check.rs::infer_get`. Moved from `:wat::std::*` in arc 021. |
| `:wat::core::contains?` | HashMap key-membership test | `runtime.rs::eval_std_contains`. Moved from `:wat::std::*` in arc 021. |
| `:wat::core::assoc` | `HashMap<K,V> × K × V -> HashMap<K,V>` — Clojure's `assoc`, values-up put | `runtime.rs::eval_assoc`; `check.rs::infer_assoc`. Added in arc 020. |
| `:wat::std::member?` | HashSet element-membership test | `runtime.rs::eval_std_member` |

### `:wat::std::list::*` — list combinators

| Path | Signature | Source |
|---|---|---|
| `:wat::std::list::map-with-index` | `∀T,U. :Vec<T> × :fn(T,i64)->U -> :Vec<U>` | `runtime.rs::eval_list_map_with_index` |
| `:wat::std::list::remove-at` | `∀T. :Vec<T> × :i64 -> :Vec<T>` | `runtime.rs::eval_list_remove_at` |
| `:wat::std::list::window` | `∀T. :Vec<T> × :i64 -> :Vec<Vec<T>>` | `runtime.rs::eval_list_window` |
| `:wat::std::list::zip` | `∀T,U. :Vec<T> × :Vec<U> -> :Vec<(T,U)>` | `runtime.rs::eval_list_zip` |

### `:wat::std::math::*` — math primitives

| Path | Signature | Source |
|---|---|---|
| `:wat::std::math::pi` | `-> :f64` | `runtime.rs::eval_math_pi` |
| `:wat::std::math::ln` | `:f64 -> :f64` | `runtime.rs::eval_math_unary` |
| `:wat::std::math::log` | `:f64 -> :f64` (alias for ln) | same |
| `:wat::std::math::cos` | `:f64 -> :f64` | same |
| `:wat::std::math::sin` | `:f64 -> :f64` | same |

### `:wat::std::LocalCache<K,V>` — L1 cache

| Path | Signature / Kind | Source |
|---|---|---|
| `:wat::std::LocalCache<K,V>` | typealias → `:rust::lru::LruCache<K,V>` | `wat/std/LocalCache.wat` |
| `:wat::std::LocalCache::new` | `:i64 -> :LocalCache<K,V>` | same |
| `:wat::std::LocalCache::put` | `:LocalCache<K,V> × K × V -> :()` | same |
| `:wat::std::LocalCache::get` | `:LocalCache<K,V> × K -> :Option<V>` | same |

### `:wat::std::service::*` — spawnable programs

| Path | Kind | Source |
|---|---|---|
| `:wat::std::service::Console` | setup function `(stdout × stderr × count)` → `(HandlePool, driver-handle)` | `wat/std/service/Console.wat` |
| `:wat::std::service::Console/loop` | driver function | same |
| `:wat::std::service::Console/out` | client helper | same |
| `:wat::std::service::Console/err` | client helper | same |
| `:wat::std::service::Console::Message` | typealias `:(i64,String)` | same |
| `:wat::std::service::Console::Tx` | typealias `:Sender<Message>` | same |
| `:wat::std::service::Console::Rx` | typealias `:Receiver<Message>` | same |
| `:wat::std::service::Cache<K,V>` | setup function `(capacity × count)` → `(HandlePool, driver-handle)` | `wat/std/service/Cache.wat` |
| `:wat::std::service::Cache/loop` | driver function | same |
| `:wat::std::service::Cache/loop-step` | inner loop | same |
| `:wat::std::service::Cache/get` | client helper | same |
| `:wat::std::service::Cache/put` | client helper | same |
| `:wat::std::service::Cache::Body<K,V>` | typealias `:(i64,K,Option<V>)` | same |
| `:wat::std::service::Cache::ReplyTx<V>` | typealias `:Sender<Option<V>>` | same |
| `:wat::std::service::Cache::Request<K,V>` | typealias `:(Body, ReplyTx)` | same |
| `:wat::std::service::Cache::ReqTx<K,V>` | typealias `:Sender<Request>` | same |
| `:wat::std::service::Cache::ReqRx<K,V>` | typealias `:Receiver<Request>` | same |

### `:wat::std::stream::*` — CSP pipeline stdlib (058-034 INSCRIPTION)

| Path | Signature / Kind | Source |
|---|---|---|
| `:wat::std::stream::Stream<T>` | typealias `:(Receiver<T>, ProgramHandle<()>)` | `wat/std/stream.wat` |
| `:wat::std::stream::Producer<T>` | typealias `:fn(Sender<T>)->()` | same |
| `:wat::std::stream::spawn-producer` | `:Producer<T> -> :Stream<T>` | same |
| `:wat::std::stream::map` | `:Stream<T> × :fn(T)->U -> :Stream<U>` | same |
| `:wat::std::stream::map-worker` | internal worker | same |
| `:wat::std::stream::filter` | `:Stream<T> × :fn(T)->bool -> :Stream<T>` | same |
| `:wat::std::stream::filter-worker` | internal worker | same |
| `:wat::std::stream::chunks` | `:Stream<T> × :i64 -> :Stream<Vec<T>>` | same |
| `:wat::std::stream::chunks-worker` | internal worker | same |
| `:wat::std::stream::for-each` | `:Stream<T> × :fn(T)->() -> :()` (terminal) | same |
| `:wat::std::stream::for-each-drain` | internal recursion | same |
| `:wat::std::stream::collect` | `:Stream<T> -> :Vec<T>` (terminal) | same |
| `:wat::std::stream::collect-drain` | internal recursion | same |
| `:wat::std::stream::fold` | `:Stream<T> × :Acc × :fn(Acc,T)->Acc -> :Acc` (terminal) | same |
| `:wat::std::stream::fold-drain` | internal recursion | same |
| `:wat::std::stream::inspect` | `:Stream<T> × :fn(T)->() -> :Stream<T>` (1:1 pass-through with side effect, arc 006) | same |
| `:wat::std::stream::inspect-worker` | internal worker | same |
| `:wat::std::stream::flat-map` | `:Stream<T> × :fn(T)->Vec<U> -> :Stream<U>` (1:N expansion, arc 006) | same |
| `:wat::std::stream::flat-map-worker` | internal worker | same |
| `:wat::std::stream::take` | `:Stream<T> × :i64 -> :Stream<T>` (stage — forwards first n, exits via drop cascade, arc 006 slice 2) | same |
| `:wat::std::stream::take-worker` | internal worker | same |
| `:wat::std::stream::from-receiver` | `:Receiver<T> × :ProgramHandle<()> -> :Stream<T>` (trivial tuple-wrap, arc 006 slice 3) | same |

---

## `:rust::*` — surfaced Rust types (via `#[wat_dispatch]`)

| Path | Scope | Kind | Source |
|---|---|---|---|
| `:rust::lru::LruCache<K,V>` | `thread_owned` | struct + methods (`::new`, `::put`, `::get`) | `src/rust_deps/lru.rs` (macro-generated) |
| `:rust::std::io::Stdin` | opaque | `:user::main` arg | `runtime.rs` `Value::io__Stdin` |
| `:rust::std::io::Stdout` | opaque | `:user::main` arg | `Value::io__Stdout` |
| `:rust::std::io::Stderr` | opaque | `:user::main` arg | `Value::io__Stderr` |
| `:rust::crossbeam_channel::Sender<T>` | opaque | queue endpoint | `Value::crossbeam_channel__Sender` |
| `:rust::crossbeam_channel::Receiver<T>` | opaque | queue endpoint | `Value::crossbeam_channel__Receiver` |
| `:rust::std::collections::HashMap<K,V>` | opaque | backing for `:wat::std::HashMap` | `Value::wat__std__HashMap` |
| `:rust::std::collections::HashSet<T>` | opaque | backing for `:wat::std::HashSet` | `Value::wat__std__HashSet` |
| `:wat::kernel::ProgramHandle<R>` | opaque | spawn result | `Value::wat__kernel__ProgramHandle` |
| `:wat::kernel::HandlePool<T>` | opaque | claim-or-panic pool | `Value::wat__kernel__HandlePool` |

---

## Reserved prefixes

From `src/resolve.rs::RESERVED_PREFIXES`:

- `:wat::core::`
- `:wat::kernel::`
- `:wat::holon::`
- `:wat::std::`
- `:wat::config::`
- `:wat::load::`
- `:wat::verify::`
- `:wat::eval::`
- `:wat::io::`
- `:rust::`

User source may not `define` / `defmacro` / declare types under
any of these. The stdlib path (wat/std/*.wat) goes through
privileged registration (`register_stdlib_defmacros`,
`register_stdlib_types`, `register_stdlib_defines`) that
bypasses the gate.

---

## Deferred — paths referenced but not yet shipped

Each ships when a concrete caller demands it. Until then, the
surface stays small per stdlib-as-blueprint discipline
(`CONVENTIONS.md`).

### Core + std

| Path | Status | Source of reference |
|---|---|---|
| `:wat::core::cons` | seed-doc reference; no caller | early notes |
| `:wat::core::when` | FOUNDATION-listed host-inherited Lisp form; body will be tail-carrying when it ships | FOUNDATION + arc 003 DESIGN |
| `:wat::std::cached-encode` | design-deferred; users wrap encode with `LocalCache::get/put` explicitly | arc 001 DESIGN |
| `:wat::std::list::pairwise-map` | referenced by `Ngram.wat`; verify whether `window` + `map` covers the use | `wat/holon/Ngram.wat` |
| `:wat::config::capacity-mode` accessor | mode is read internally by `Bundle`; expose only if user code needs to observe it | FOUNDATION |

### Stream combinators (arc 004 deferred set)

The arc 004 INSCRIPTION shipped the core set (map, filter, chunks,
for-each, collect, fold, spawn-producer). These were sketched in
the DESIGN but deferred:

| Path | Shape | Status |
|---|---|---|
| `:wat::std::stream::chunks-by` | N:1, key-change boundary | **resolved** — ships as library on `with-state`, blocked on with-state |
| `:wat::std::stream::window` | N:1, sliding window | **resolved** — ships as library on `with-state`, blocked on with-state |
| `:wat::std::stream::with-state` | Mealy-machine substrate primitive `:Stream<T> × Acc × step × flush -> :Stream<U>` | pending implementation (arc 007 or later) |
| `:wat::std::stream::time-window` | N:1, time-bucket boundary | substrate-blocked (clock primitive) |
| `:wat::std::stream::first` (terminal form) | terminal, take-N returning Vec | **rejected** — reframed as `take` stage (arc 006 BACKLOG "What wat deliberately does NOT have") |
| `:wat::std::stream::from-iterator` | alternate constructor | substrate-blocked (Iterator surfacing) |
| `:wat::std::stream::from-fn` | alternate constructor | deferred (near-redundant with spawn-producer) |
| `:rust::std::iter::Iterator<T>` surfacing | in-process lazy adapter chain via `#[wat_dispatch]` | substrate-blocked (own arc) |

---

## Rejected — paths with audit trail

| Path | Why rejected | Record |
|---|---|---|
| `:wat::std::stream::pipeline` | `let*` already IS the pipeline. The "boilerplate" the composer would eliminate was per-stage type annotations — information, not ceremony. Captured as `feedback_verbose_is_honest` | arc 004 INSCRIPTION, `BACKLOG.md` pipeline-rejection section |
| `:wat::core::presence` | Lives at `:wat::holon::presence?` — an algebra measurement, not a core form. Old USER-GUIDE and README referenced the wrong namespace; fixed during this audit | arc 005 Pass 3 commit `f955cf2` |

---

*Pass 1 + Pass 5 complete. 100+ primitives inventoried,
deferred and rejected paths cataloged with their audit
trails. FOUNDATION names the why; this file names the what
and where; the INSCRIPTION is the shipped contract.*
