# SCORE — STONE 255.45: measure declarations and drops

Measurement only. Nothing in `src/` or `wat/` changed. Struck against draw
`e31e6a875` (main at the strike is that draw, parent `3d6f079d2`). The
registry numbers come from running
`probe_can_doc_types_reconstruct_the_checker_scheme` on the release lib
(`SCHEME_RC=0`). The class of each hand arm is from reading that arm in
`src/check.rs`. The drop census is a structural walk, and it says so.

## Part A — one clause list per intrinsic

The live registry is 582 rows. 449 have a `TypeScheme`. 90 of those are
generic, and every type variable is named in the row's `@arg`/`@ret` doc.
51 are `Kind::SpecialForm` with no scheme. 82 are intrinsics with no
scheme. 449 + 51 + 82 = 582.

Of the 82, 70 have a `match k.as_str()` arm before scheme lookup
(`src/check.rs` 2725–5799). 12 do not. Those 12 pass the arity door and
return a fresh type variable (`src/check.rs` 6010–6048). The finding's
72 / 10 split was at `4286205ea`. This tree's split of the same 82 is
70 / 12.

`defclause` supports `& rest`. `wat/core.wat` writes it, and eval checks
each extra argument against the element type (`src/function/eval.rs` 227).
Check-time dispatch records only `has_rest` (`src/check/env.rs` 181) and
then zips the fixed parameters (`src/check.rs` 5641 and 5665). Extra
arguments are not typechecked there. A rest clause can be written. The
check dispatch in force today would not enforce the element type a
`TypeScheme.rest_param_type` enforces (`src/check.rs` 6116–6166).

### Counts

| class | rows | what |
|---|---|---|
| (i) one clause | 446 schemes + 28 hand arms | the scheme is already one clause, or the arm checks one signature |
| (ii) several clauses | 11 hand arms | finite overloads |
| (iii) rest clause | 3 schemes + 3 constructors | variadic |
| (iv) a clause cannot say the current check | 28 hand arms + 12 fresh-return rows | listed below |
| special forms | 51 | no rank-1 scheme |

446 + 3 = 449 schemes. 28 + 11 + 3 + 28 = 70 hand arms. 12 fresh-return
rows sit in (iv) and are the rest of the 82.

### (i) — one clause

The 446 schemes with `rest_param_type: None`, including the 90 generic
ones. A generic clause is still one clause (`src/check.rs` 5645
instantiates the clause's free type variables).

Hand arms whose check is one signature:

`variant-name`, `Result/try`, `Option/try`, `Option/expect`,
`Result/expect`, `linkedlist/conj`, `cosine`, `dot`, `coincident?`,
`coincident-explain`, `simhash`, `spawn-thread`, `spawn-process`,
`fn-forms`, `after`, `close` (one clause on `Spawned`), `signal` (one
clause on `Process`), `peer-process`, `peer-wire?`, `address-wire?`,
`require-wire-address`, `allow`, `deny`, `connect`, `accept`,
`serve-dispatch-op` (the body type is the result), `=` and `not=`
(one clause `(T, T) -> bool`; the arm is structural equality).

`close`'s own comment says a defclause cannot enumerate every `(I, O)`
(`src/check.rs` 11665). The check dispatch that exists now instantiates
type variables, so `(Spawned :- [S R]) -> CloseOutcome` is one clause.
The comment describes a monomorphic reading. The dispatch code is the
current mechanism.

### (ii) — several clauses

| rows | why |
|---|---|
| `:wat::time::+`, `:wat::time::-` | the result depends on the pair: Instant+Duration, Instant−Duration, Instant−Instant (`src/check.rs` 4113–4118). Three clauses, not one |
| `send`, `try-send`, `recv` | `project_peer_io` accepts an owner (`Spawned`) or a `Peer` (`src/check.rs` 11274–11280). Those are two heads. Both return the same outcome type today |
| `reverse`, `take`, `drop`, `zip`, `window`, `remove-at` | the result keeps the container constructor (`infer_reverse` is `C<T> → C<T>`). One clause per container in that finite set |

### (iii) — rest

Schemes: `:wat::intrinsic::variadic-args-measurement` (`src/check.rs`
20881), `:wat::f64::max-of`, `:wat::f64::min-of` (23652). Hand arms:
`:wat::core::List`, `:wat::core::PersistentMap`,
`:wat::core::PersistentVector`. Same rest-element gap as above: eval
checks the element, check-time clause dispatch does not.

### (iv) — a clause cannot say what the arm checks

| row | reason | where |
|---|---|---|
| `:wat::core::<` `>` `<=` `>=` | after unifying the two arguments, the arm gates on an orderable class. A finite clause list does not state that class. The arm's own comment says a clause list cannot express `(Vector :- [T]) < (Vector :- [T])` | `src/check.rs` 4093–4104 |
| `:wat::string::declare-acronyms` | parses the form as a declaration and returns unit. The arguments are syntax | 2817–2835 |
| `:wat::edn::validate` | the second argument is a type keyword or a type form, deliberately not inferred as a value | 3088–3115 |
| `:wat::string::interpolate` | the tail is keyword/value pairs, not one rest element type | 3412–3413 |
| `:wat::runtime::type-of` | a literal keyword is type-position and is not inferred; the return is `TypeInfo` | 3751–3766 |
| `:wat::runtime::field-names-of`, `field-types-of` | the argument is not constrained, because a struct name is also a constructor. Inferring it would see the constructor's `Fn` type | 3783–3791 |
| `:wat::core::aggregate-new`, `kwargs-construct` | the result type is the type named by a keyword argument, and kwargs are not a positional clause | 4896–4918 |
| `:wat::core::to-record`, `:wat::holon::to-record` | the second argument is a surface keyword literal; the result is that type | 4922–4924 |
| `:wat::core::variant` | the first arguments are name literals, not values | 4942–4946 |
| `:wat::kernel::retag-op` | the result type is the type keyword in argument 2 | 4395–4401 |
| `:wat::kernel::select`, `:wat::kernel::poll` | the peers argument is a `Vector` whose element is an owner or a `Peer`. Same-head parametric arguments are invariant (`src/check.rs` 17549–17589), with a `Peer` received-op exception that does not cover `Thread`/`Process`/`Spawned`. A clause `(Vector :- [(Spawned :- [S R])])` does not accept `(Vector :- [(Thread :- [S R])])`. `poll`'s listener is deliberately unconstrained (12323); that one parameter a clause can say | 4410–4424, 12233–12280 |
| `:wat::kernel::listener` | arity and result depend on `ThreadOpts` versus `ProcessOpts`, and `:S`/`:R` are type arguments, not values | 10419–10436 |
| `:wat::core::Ok`, `Err`, `Some` | the arm refuses the name (retired bare variant). A clause would give it a type. The current check is a refusal | 5172–5186 |
| `:wat::core::struct-new` | the arm does not return. It appends a nature error and falls through. There is no scheme, so the result is a fresh variable plus that wall | 5188–5203 |
| `:wat::core::first`, `second`, `third`, `nth` | the element type comes from an open container registry, and a tuple's slot type depends on the index. A fixed clause list does not cover every tuple arity | 10066–10073 |

The 12 fresh-return rows have no type to transcribe. A clause would invent
one. They are `:wat::core::fresh-symbol`, `macro-error`, `str`,
`struct-field`, `type-equal?`, `type-params-used-in`,
`:wat::kernel::peer-pid`, `:wat::linkedlist::contains?`, `empty?`, `get`,
`length`, `:wat::runtime::metadata-of`.

### Special forms

The probe counts 51 `Kind::SpecialForm` rows with no scheme. Source has
34 `#[wat_special_form]` language rows under `src/intrinsic/special/`
(excluding `rete_alias.rs`) and 52 rete alias rows in `rete_alias.rs`.
34 + 52 = 86 attributes. The probe's 51 is not 86. This score does not
invent which 35 attributes are absent from that 51. Both numbers were
measured: the probe by running it, the 86 by counting attributes.

The language rows that are forms: `if`, `let`, `do`, `match`, `and`,
`or`, `fn`, `quote`, `quasiquote`, `ann-form`, `forms`, `def`,
`defalias`, `defclause`, `defenum`, `defmacro`, `defsurface`, `derive`,
`extend-type`, `newtype`, `structtype`, `typealias`, `use!`,
`load-file!`, `digest-load!`, `signed-load!`, `stream/lazy`,
`holon/literal`. Their arguments are syntax, bindings, or unevaluated
branches.

`self-peer` is a form because `:S` and `:R` are type keywords, not values
(`src/intrinsic/special/program_self_peer.rs` 12–16). `macroexpand` and
`macroexpand-1` see a quoted form. `struct->form` and the two
`set-redef!` rows are the ones closest to ordinary functions: `set-redef!`
is one bool and returns unit, and it is a form because it updates checker
state as a declaration.

The rete rows are aliases. A `Form` alias (`if`, `let`, `match`, `fn`,
`and`, `or`) is still a form. An `Alias` row that re-dispatches to a
function has that function's signature; the alias entry itself is not a
second signature.

### The two gaps

**Vector covariance (p10).** Two rows: `select` and `poll`. Measured at
the assignable rule cited above. No `TypeScheme` in `register_builtins`
types a `Vector` of `Spawned` or `Peer`.

**Unresolved receiver (p11).** `project_peer_io` requires a parametric
owner or `Peer` and otherwise emits `TypeMismatch` (`src/check.rs`
11282–11292). A fresh variable is not that parametric. Defclause dispatch
treats a missing argument type as a successful position (`None => true`,
5668) and unifies a fresh variable into the first clause. Moving these
rows onto clauses would accept an unresolved receiver that the arm
refuses today:

`send`, `try-send`, `recv`, `peer-process`, `peer-wire?` (the five
`project_peer_io` callers), plus `close`, `signal`, `select`, `poll`,
and `listener`. Ten rows.

## Part B — the 101 drops

The saved list is
`scratchpad/f5/classified_sites.tsv` from the F5 session. It has 99
class-(b) rows, not 101. The finding's published split was 21 + 50 + 11
+ 6 + 9 + 3 + 1 = 101. This file's split is:

| subclass | rows |
|---|---|
| b3 evaluated for a type or a raise | 49 |
| b1 write/print returns a count | 21 |
| b4 `mapv`/`foldl` for effect | 11 |
| b6 incidental return | 8 |
| b5 `stop` returns the final record | 6 |
| b2 immutability test | 3 |
| b7 returns its input | 1 |

99. Every one of those 99 still parses, and the callee at that point was
read off the form. The two-row gap against the finding is in this file,
not a line that moved: 0 of the 99 files were missing.

The reader/dropper walk covered 2283 tracked `.wat` files, 0 parse
failures. A call is a drop when it is a non-final `do` child, or a `let`
init whose binder starts with `_`. Anything else is a use. A named
binding that is never read counts as a use. That is the heuristic.

### (α) nobody reads it

The six `stop` sites. Each concrete `stop` (`my::svc/stop`,
`hologram-svc/stop`, `lru-svc/stop`, `barebox-svc/stop`,
`pcache-svc/stop`, `mal-bag/stop`) appears once, as a drop. The two
stdlib names in that set, `hologram-svc/stop` and `lru-svc/stop`, have 0
uses and 1 drop in the walk. Returning `nil` from `stop` removes these
6 sites.

### (β) some callers read it

Every other stdlib producer at a class-(b) site has at least one reader.
Making it return `nil` does not remove the drop sites, and it breaks the
readers. The counts:

| function | uses | drops | note |
|---|---|---|---|
| `IOWriter/write-string` | 4 | 6 | returns `i64` (`src/check.rs` 18503). `print` already returns unit (18512) |
| `IOWriter/writeln` | 3 | 6 | returns `i64` (18530). `println` returns unit (18521) |
| `core/mapv` | 63 | 7 | |
| `core/foldl` | 720 | 13 | |
| `cache/Lru/put` | 6 | 7 | the evicted entry |
| `IOReader/read-line` | 10 | 1 | |
| `IOReader/read` | 3 | 1 | |
| `IOReader/read-all` | 2 | 1 | |
| `core/conj` | 185 | 1 | |
| `vec/conj` | 7 | 1 | |
| `core/assoc` | 20 | 1 | |
| `holon/to-holon` | 1967 | 1 | |
| `holon/leaf` | 244 | 2 | |
| `Bundle/first` | 5 | 2 | |
| `Bundle/children` | 6 | 1 | |
| `Hologram/remove` | 2 | 1 | |
| `HolographicLru/get` | 8 | 1 | |
| `spawn/thread` | 232 | 1 | |
| `spawn/process` | 274 | 1 | |
| `Result/expect` | 31 | 5 | |
| `runtime/argv` | 25 | 4 | |

`map` (131/1), `filter` (42/1), `fix-text-apply` (90/1), `bracket/map`
(23/1), `regex/matches?` (12/1), `i64/quot` (8/1), and the `Env/*` and
reflection verbs in the same walk are the same shape: one drop, many
readers.

### (γ) the drop is the test

The non-`:wat::` callees at these sites (`:my::identity` 6,
`:user::probe-one` 5, `:probe::whoami-id` 3, `:u::wants-holon` 3,
`:u::wants-record` 3, and the single-site probes) are the test's subject.
They are not an API returning a count. The finding's b3 (49) is this
population plus a few stdlib calls made so the typechecker will reject
or accept them.

### What would disappear

If every (α) function returned `nil`, 6 of the 99 sites disappear: the
`stop` record. The other 93 stay. The write-count sites (21) and the
`mapv`/`foldl` sites (11) are (β): the nil-returning twin already exists
for `write-string` (`print`) and for `writeln` (`println`), and the
readers of the `i64` still need the count. Those drop sites need the
caller to pick the twin, or an explicit drop. They do not vanish by
changing the function that readers use.
