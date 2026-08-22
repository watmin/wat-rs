# TABLE — `defservice`'s type-name sites (arc 109, identity stone 2, classification pass)

**Deliverable of** `BRIEF-STONE-identity-2-classification-pass.md`. Measurement only — no source
changed.

## Population — derivation, command, and what it cannot see

Every built name in `defservice` is minted the same mechanical way: a string is assembled
(`string::interpolate`/`string::concat`) and turned into a node with
`(:wat::core::keyword/from-string ...)`. That call is the one anchor point common to every built
name in the file, so the population is defined as: **every `let`-local binding in `wat/service.wat`
whose right-hand side is a call to `:wat::core::keyword/from-string`.**

Command:

```
grep -noE '^\s*\[?[A-Za-z_][A-Za-z0-9_-]*\s+\(:wat::core::keyword/from-string' wat/service.wat
```

(the `\[?` also catches the two bindings written inside a `let`-vector destructure, `[sf-kw (...)]`
and `[req-ty (...)]`, which a plain leading-whitespace match misses).

**Count: 94** distinct binding sites (94 lines matched; 5 identifiers — `req-ty`, `op-variant-kw`,
`reply-variant-kw`, `cap-const-kw`, `rtl-ctor-kw` — are each bound **twice**, at two unrelated
scopes inside two different `foldl` loop bodies; a shared name, not a shared binding, so each
occurrence is its own row below).

Sanity check: `grep -c ':wat::core::keyword/from-string' wat/service.wat` → **126** total
occurrences. 126 − 94 = 32 lines are not bindings; all 32 were read by hand (none were comments —
a stricter token match than my first attempts, `:wat::core::keyword/from-string` in full, excludes
the 3 comment-only hits that a bare `keyword/from-string` substring match picks up). All 32 are
`(:wat::core::keyword/from-string ~some-str-binding)` calls **spliced into the generated program
itself** — i.e. the macro emits a call to `keyword/from-string` that runs at the SERVICE's own
runtime, not at macro-expand time (`wat/service.wat:884-886` explains why: a spliced literal
keyword would resolve to the already-existing Fn/macro, not a keyword, so the emitted code has to
mint the keyword by name at runtime instead). Every one of the 32 consumes an already-excluded
**function-name** string (`dispatch-admin-name-str`, `serve-name-str`, `extract-addr-name-str`,
`status-started-str`, `fqdn-base`) — see "Runtime-emission sites" below. None of the 32 introduces a
new type-name binding, so they are not additional population, but I did not want to make that
finding silently — a command that clips itself to left-hand-side bindings is blind to a
keyword/from-string call used as a bare *expression*, and this file has 32 of them.

**What this command cannot see, beyond the 32 above:**
- A type name minted by some OTHER constructor (not `keyword/from-string`) — I checked for this by
  reading the whole file (all 2823 lines) rather than trusting the grep alone; I found none. Every
  angle-bracket type keyword in this macro is built through this one call.
- A binding whose name and call are split across a line boundary in a way my regex's single-line
  anchor doesn't catch. I do not believe this happens (the file's style always keeps `name (` on one
  line even when the call's arguments wrap), but I did not build an independent AST-level check to
  refute it — a residual risk, not a claim I can rule to zero.
- Two bindings that happen to share an identifier because they live in different `let` scopes
  (noted above, 5 instances) look identical to a plain grep; only reading the surrounding `foldl` told
  them apart. A pure-text tool that globs by name (rather than by scope) will silently merge them.

**Population vs. the brief's ~110 anchor:** 94 is within a factor of 1.17 of ~110 — inside the
STOP-2 tolerance (factor of 2), so I did not stop.

## Scope

In scope: a binding whose value is or becomes a **TYPE name** — something that would need to change
spelling under the `Head<A,B>` → `(Head :- [A B])` migration, i.e. it either carries (or could carry)
a bracketed parameter list, or sits in a position the checker reads as a type.

Out of scope, per the brief's own examples (`init-name`, `serve-name`, `handle-new-kw`) generalized
to the two patterns that recur ~60 times in this file:
1. **FUNCTION names** — a binding whose value names an emitted `defn`/`defmacro`, called by name
   elsewhere. These never carry type params; the macro's own comment block at `wat/service.wat:213-216`
   states the rule directly: construction and by-name runtime resolution both key on the BASE name.
2. **VALUE / ACCESSOR / CONSTRUCTOR (variant) names** — a binding whose value names a constructor,
   accessor, or enum variant (`::State'`, `::State/durable`, `::Admin::Init`, a numeric constant
   keyword). The SAME comment block names `::Admin::Init` explicitly as an example of this class:
   "CONSTRUCTOR / ACCESSOR / VARIANT / runtime-name-string keywords → BASE, no params." These never
   carry `<…>` either, so the migration has nothing to do to them.

Of the 94, **36 are in scope** (type names) and **58 are excluded** (32 function names, 26
value/accessor/constructor names) — both counted and listed below, not merely asserted.

---

## PART 1 — IN-SCOPE (TYPE name) sites — 73 consumption rows over 36 bindings

One row per consumption site. Where the exact same mechanical form is repeated verbatim across
near-duplicate thread/process/wasm-tier code (a recurring feature of this file), the lines are
grouped into one row and every line is still cited — never silently dropped.

| binding | built at | consumed at | consumer form (verbatim, trimmed) | role | notes |
|---|---|---|---|---|---|
| `service-op-decl-kw` | 1165 | 1248 | `` `(:wat::core::defenum ~service-op-decl-kw :wat::enum::Pure ~@service-op-variant-items)` `` | DECL-NAME | **THE KNOWN EXAMPLE** from the brief. |
| `service-op-decl-kw` | 1165 | 1799 | `(:wat::kernel::retag-op op ~proto-op-ty-kw ~service-op-decl-kw)` | RUNTIME-ARG | Same binding as above — the brief's proof that one conversion breaks one of the two. |
| `state-ty` | 591 | 617 | `` `(:wat::core::fn [~d-sym <- ~record-ty] -> ~state-ty (~state-new-kw ~d-sym))))` `` | ANNOTATION | return-type slot of the default `:init` fn |
| `state-ty` | 591 | 648 | `` init-def `(:wat::core::defn ~init-name ~init-params-vec -> ~state-ty ~init-body)` `` | ANNOTATION | return type of the emitted `init` defn |
| `state-ty` | 591 | 660 | `` `(:wat::core::fn [~s-sym <- ~state-ty] -> ~record-ty (~state-durable-kw ~s-sym)))` `` | ANNOTATION | param type, default `:stop` fn |
| `state-ty` | 591 | 681 | `` `(:wat::core::fn [~s-sym <- ~state-ty] -> ~record-ty (~state-durable-kw ~s-sym)))` `` | ANNOTATION | param type, default `:hibernate` fn |
| `state-ty` | 591 | 731 | `` state-def `(:wat::core::defstruct ~state-ty ~state-field-vec)` `` | DECL-NAME | the `::State` defstruct's own name slot |
| `state-ty` | 591 | 1690 | `state       <- ~state-ty]` | ANNOTATION | `serve-params` field type |
| `record-ty` | 592 | 617 | `` `(:wat::core::fn [~d-sym <- ~record-ty] -> ~state-ty …)` `` | ANNOTATION | param type, default `:init` fn |
| `record-ty` | 592 | 660 | `` `(:wat::core::fn [~s-sym <- ~state-ty] -> ~record-ty …)` `` | ANNOTATION | return type, default `:stop` fn |
| `record-ty` | 592 | 681 | `` `(:wat::core::fn [~s-sym <- ~state-ty] -> ~record-ty …)` `` | ANNOTATION | return type, default `:hibernate` fn |
| `record-ty` | 592 | 697 | `` hibernate-project-def `(:wat::core::defn ~hibernate-project-name ~hibernate-params-vec -> ~record-ty ~hibernate-body)` `` | ANNOTATION | return type of the emitted hibernate-projection defn |
| `record-ty` | 592 | 706/707 | `` `(:wat::holon::defrecord ~record-ty ~durable-fields)` / `(:wat::core::defrecord ~record-ty ~durable-fields))` `` | DECL-NAME | two mutually-exclusive branches (`:durable-parent` = holon or not), same slot |
| `record-ty` | 592 | 712 | `` durable-prefix-vec `[durable <- ~record-ty]` `` | ANNOTATION | prefix field of `::State`'s own field vector |
| `record-ty` | 592 | 1089 | `:Hibernated [snapshot <- ~record-ty]` | ANNOTATION | field type inside the Status enum's own variant |
| `record-ty` | 592 | 2095 | `` hibernate-method `(:wat::core::defn ~hibernate-method-name ~hibernate-method-params -> ~record-ty ~hibernate-method-body)` `` | ANNOTATION | return type of the owner-only `/hibernate` method |
| `enum-name` | 866 | 1268 | `` `(:wat::core::derive ~enum-name ~service-op-kw)))` `` | RUNTIME-ARG | `derive` — one of the two named RUNTIME-ARG exemplars in the brief |
| `reply-name` | 868 | — | — | **OTHER (dead)** | Built, **never consumed anywhere** — `grep -n '\breply-name\b'` finds only its own definition line. Comment at :861 calls it a sibling of `enum-name` ("stay at the BASE… registers a subtype edge"), but no such edge or any other use exists for Reply. STOP-1: recorded rather than guessed at. |
| `proto-op-ty-kw` | 874 | 1799 | `(:wat::kernel::retag-op op ~proto-op-ty-kw ~service-op-decl-kw)` | RUNTIME-ARG | |
| `proto-op-ty-kw` | 874 | 2303 | `~proto-op-ty-kw ~proto-reply-ty-kw ~max-frame-bytes-node)` (arg to `:wat::kernel::listener`) | RUNTIME-ARG | |
| `proto-reply-ty-kw` | 875 | 2303 | same line as above | RUNTIME-ARG | |
| `peer-ty` | 936 | — | — | **OTHER (dead)** | Built (`Peer<Reply,Op>`), **never spliced anywhere** — `grep -n '~peer-ty\b'` returns nothing. Only mentioned in two comments (:954, :1006) contrasting it with `client-peer-ty`. STOP-1. |
| `listener-ty` | 940 | 1687 | `l           <- ~listener-ty` | ANNOTATION | `serve-params` field type |
| `vector-ty` | 944 | — | — | **OTHER (dead)** | Built (`Vector<Peer<Reply,Op>>`), **never consumed** — no `~vector-ty` anywhere in the file. STOP-1. |
| `addr-ty` | 948 | 1087 | `:Started   [addr     <- ~addr-ty]` | ANNOTATION | Status enum variant field |
| `addr-ty` | 948 | 1137 | `[lu <- ~status-ty] -> ~addr-ty` | ANNOTATION | return type of `extract-addr` |
| `addr-ty` | 948 | 2743 | `` handle-fields `[handle <- ~handle-peer-ty addr <- ~addr-ty]` `` | ANNOTATION | Handle struct field |
| `client-peer-ty` | 955 | 1903 | `` method-params   `[c <- ~client-peer-ty req <- ~req-ty]` `` | ANNOTATION | per-op client method's `c` param |
| `admin-ty` | 1000 | 1076 | `` admin-enum-def `(:wat::core::defenum ~admin-ty :wat::enum::Pure` `` | DECL-NAME | Admin enum's own name slot |
| `admin-ty` | 1000 | 1105 | `` dispatch-admin-def `(:wat::core::defn ~dispatch-admin-name [ai <- ~admin-ty] -> ~state-ty` `` | ANNOTATION | param type |
| `admin-ty` | 1000 | 2304 | `~cm-self-sym (:wat::program::self-peer ~status-ty ~admin-ty)` | RUNTIME-ARG | `self-peer` intrinsic, evaluated in the generated child-main body |
| `status-ty` | 1004 | 1086 | `` status-enum-def `(:wat::core::defenum ~status-ty :wat::enum::Pure` `` | DECL-NAME | Status enum's own name slot |
| `status-ty` | 1004 | 1137 | `[lu <- ~status-ty] -> ~addr-ty` | ANNOTATION | param type of `extract-addr` |
| `status-ty` | 1004 | 2304 | `(:wat::program::self-peer ~status-ty ~admin-ty)` | RUNTIME-ARG | same `self-peer` call as `admin-ty` above |
| `lineage-peer-ty` | 1011 | 1686 | `` serve-params `[self        <- ~lineage-peer-ty` `` | ANNOTATION | `serve`'s own `self` param type |
| `service-op-kw` | 1158 | 1268 | `` `(:wat::core::derive ~enum-name ~service-op-kw)))` `` | RUNTIME-ARG | `derive` |
| `selectable-peer-ty` | 1169 | 1206 | `` peers-only-expr `(:wat::core::foldl ~peers-fold-fn (:wat::core::Vector ~selectable-peer-ty) selectables)` `` | CTOR-ARG | `Vector`'s element-type argument |
| `selectable-peer-ty` | 1169 | 1461 | `(:wat::core::conj (:wat::core::Vector ~selectable-peer-ty) …)` | CTOR-ARG | same — the one-element widening detour (arm-fn) |
| `selectable-vec-ty` | 1175 | 1203 | `` peers-fold-fn `(:wat::core::fn [~peers-acc-sym <- ~selectable-vec-ty  ~peers-t-sym <- ~selectable-entry-ty]` `` | ANNOTATION | fold accumulator param type |
| `selectable-vec-ty` | 1175 | 1204 | `-> ~selectable-vec-ty` | ANNOTATION | same fn's return type |
| `selectable-entry-ty` | 1189 | 1203 | `~peers-t-sym <- ~selectable-entry-ty]` (same form as above) | ANNOTATION | fold element param type |
| `selectable-entry-ty` | 1189 | 2342 | `(:wat::core::Vector ~selectable-entry-ty)` | CTOR-ARG | element type of the process-tier child's empty `selectables` seed |
| `selectable-entry-vec-ty` | 1192 | 1456 | `` arm-fn        `(:wat::core::fn [~arm-acc-sym <- ~selectable-entry-vec-ty  ~arm-alarm-sym <- ~alarm-o-ty]` `` | ANNOTATION | param type |
| `selectable-entry-vec-ty` | 1192 | 1457 | `-> ~selectable-entry-vec-ty` | ANNOTATION | return type, same fn |
| `selectable-entry-vec-ty` | 1192 | 1688 | `selectables <- ~selectable-entry-vec-ty` | ANNOTATION | `serve-params`' `selectables` field — the REAL declared type (comment at :1684 says so explicitly) |
| `alarm-o-ty` | 1208 | 1456 | `~arm-alarm-sym <- ~alarm-o-ty]` (same form as above) | ANNOTATION | arm-fold's alarm param type |
| `req-ty` (server-arm scope) | 1241 | 1244 | `` `[req <- ~req-ty]))]` `` | ANNOTATION | the superset Op variant's own field type |
| `req-ty-kw` | 1637 | 1645 | `` shape-guarded `(:wat::core::match (:wat::edn::validate ~req-binder ~req-ty-kw)` `` | RUNTIME-ARG | `:wat::edn::validate` — comment at :1623-1631 states outright it is "a RUNTIME argument… not a type position the checker reads" |
| `req-ty` (client-method scope) | 1884 | 1903 | `` `[c <- ~client-peer-ty req <- ~req-ty]` `` | ANNOTATION | client method's `req` param type |
| `resp-ty` (client-method scope) | 1889 | — | — | **OTHER (dead)** | Built (`<proto>::<op>/Response`), **never consumed** — only `resp-ty-str`, its string precursor, is used (to build `recv-ret-ty`, via string concat). `awk` scoped to lines 1860-2010 confirms `~resp-ty` appears nowhere. A DIFFERENT, unrelated `resp-ty` (built at :664, not via `keyword/from-string` — extracted from the user's `:stop` fn AST via `nth`) IS used (:670, :1088, :2054); that one is out of THIS population by construction (see command above) but is worth a flag so nobody conflates the two same-named, differently-built, differently-fated bindings. STOP-1 on the :1889 one. |
| `recv-ret-ty` | 1894 | 2008 | `` `(:wat::core::defn ~method-name ~method-params -> ~recv-ret-ty ~method-body)))))` `` | ANNOTATION | client method's return type |
| `handle-name` | 921 | 2518 | `` start-impl-fn `(:wat::core::defn ~start-impl-name ~start-impl-params -> ~handle-name ~start-body)` `` | ANNOTATION | abstract-locus `/start$impl` return type |
| `handle-name` | 921 | 2644 | `` resume-impl-fn `(:wat::core::defn ~resume-impl-name ~start-impl-params -> ~handle-name ~resume-body)` `` | ANNOTATION | abstract-locus `/resume$impl` return type |
| `handle-name` | 921 | 2744 | `` handle-record `(:wat::core::defstruct ~handle-name ~handle-fields)` `` | DECL-NAME | the Handle struct's own name slot |
| `handle-shared-name` | 923 | 2502 | `(:wat::core::ann-form ~start-handle-expr ~handle-shared-name))` | ANNOTATION | thread-tier `ann-form` ascription |
| `handle-shared-name` | 923 | 2519 | `-> ~handle-shared-name ~start-body-thread)` | ANNOTATION | thread-tier `/start$impl-thread` return type |
| `handle-shared-name` | 923 | 2628 | `(:wat::core::ann-form ~start-handle-expr ~handle-shared-name))` | ANNOTATION | thread-tier resume `ann-form` |
| `handle-shared-name` | 923 | 2645 | `-> ~handle-shared-name ~resume-body-thread)` | ANNOTATION | thread-tier `/resume$impl-thread` return type |
| `handle-wire-name` | 925 | 2517 | `(:wat::core::ann-form ~start-handle-expr ~handle-wire-name))` | ANNOTATION | process-tier `ann-form` |
| `handle-wire-name` | 925 | 2520 | `-> ~handle-wire-name ~start-body-process)` | ANNOTATION | process-tier `/start$impl-process` return type |
| `handle-wire-name` | 925 | 2643 | `(:wat::core::ann-form ~start-handle-expr ~handle-wire-name))` | ANNOTATION | process-tier resume `ann-form` |
| `handle-wire-name` | 925 | 2646 | `-> ~handle-wire-name ~resume-body-process)` | ANNOTATION | process-tier `/resume$impl-process` return type |
| `handle-bare-name` | 919 | 2023, 2067, 2114, 2165 | `` `[h <- ~handle-bare-name]` `` (identical pattern, 4 sites) | ANNOTATION | `h` param of `/stop`, `/hibernate`, `/grant`, `/revoke` |
| `handle-bare-name` | 919 | 2759 | `` grantable-extend `(:wat::core::extend-type ~handle-bare-name :wat::capability::Capability` `` | ANNOTATION | `extend-type`'s TARGET-type argument — a type-reference slot, not a field/return annotation; flagged as its own sub-case below |
| `handle-bare-name` | 919 | 2776 | `` dialable-extend `(:wat::core::extend-type ~handle-bare-name ~dialable-ty` `` | ANNOTATION | same sub-case |
| `handle-bare-name` | 919 | 2790 | `` typedcap-extend `(:wat::core::extend-type ~handle-bare-name ~typedcap-ty)]` `` | ANNOTATION | same sub-case |
| `handle-peer-ty` | 2738 | 2743 | `` handle-fields `[handle <- ~handle-peer-ty addr <- ~addr-ty]` `` | ANNOTATION | Handle struct field |
| `dialable-ty` | 2773 | 2776 | `` `(:wat::core::extend-type ~handle-bare-name ~dialable-ty` `` | ANNOTATION | `extend-type`'s SATISFIED-SURFACE argument — same sub-case as above, other position |
| `typedcap-ty` | 2787 | 2790 | `` `(:wat::core::extend-type ~handle-bare-name ~typedcap-ty)]` `` | ANNOTATION | same sub-case |
| `fqdn-kw` | 189 | 1403 | `:namespace      ~fqdn-kw` | RUNTIME-ARG | `SelfInvocation`'s `:namespace` field value |
| `fqdn-kw` | 189 | 1409 | `:namespace      ~fqdn-kw` | RUNTIME-ARG | `Invocation`'s `:namespace` field value (twin ctor, public-arm branch) |
| `surface-kw` | 471 | 1225, 1227, 1296, 1382, 1384, 1871 | `(:wat::core::string::kebab->pascal-in surface-kw op-str)` (repeated, 6 call sites across 4 different `foldl` bodies) | **OTHER** | Consumed ONLY as a macro-expand-time argument to a naming helper — never `~`-spliced into the emitted program at all, so it never reaches a runtime call OR a type position. None of the four named roles fit; whether/how it interacts with the `:-` migration depends on what `kebab->pascal-in` itself expects, which is outside this file. |
| `launch-head-kw` | 2238 | 2464, 2490, 2505, 2599, 2616, 2631 | `` ~lr-sym (~launch-head-kw …) `` (identical call-head splice, 6 sites: start × 3 tiers, resume × 3 tiers) | **OTHER** | This binding IS a generic function reference with its type args baked directly into the string (`wat::spawn::Locus/launch<Op,Reply,State,Admin,Status>`), spliced as the OPERATOR position of a call form. It is not a value in a type slot (not ANNOTATION), not a declaration name (not DECL-NAME), not an argument to something else (not CTOR-ARG/RUNTIME-ARG — it IS the callee). The `:-` migration has to decide what a "generic call head" becomes, which none of the four roles anticipate. |

### Multi-role bindings — every one found (7, plus the known example)

| binding | roles | evidence |
|---|---|---|
| `service-op-decl-kw` | DECL-NAME + RUNTIME-ARG | the brief's own worked example (:1248, :1799) |
| `state-ty` | DECL-NAME + ANNOTATION | :731 (defstruct name) vs. :617/:648/:660/:681/:1690 (param/return/field types) |
| `record-ty` | DECL-NAME + ANNOTATION | :706/707 (defrecord name) vs. six ANNOTATION sites |
| `admin-ty` | DECL-NAME + ANNOTATION + RUNTIME-ARG | :1076 (defenum name), :1105 (param type), :2304 (`self-peer` runtime arg) |
| `status-ty` | DECL-NAME + ANNOTATION + RUNTIME-ARG | :1086, :1137, :2304 — same three-way split as `admin-ty` |
| `handle-name` | DECL-NAME + ANNOTATION | :2744 (defstruct name) vs. :2518/:2644 (return types) |
| `selectable-entry-ty` | ANNOTATION + CTOR-ARG | :1203 (fn param type) vs. :2342 (`Vector`'s element-type arg) |

`admin-ty` and `status-ty` are the most consequential pair here: they are the ONLY two bindings that
hit THREE of the four roles, and they hit them **in the same call** (`self-peer` at :2304 takes both
as RUNTIME-ARGs while each is separately a DECL-NAME and an ANNOTATION elsewhere) — the same failure
mode the brief's `service-op-decl-kw` example warns about, doubled.

### A new sub-case the role table doesn't name: `extend-type`'s target/trait arguments

`handle-bare-name` (as `extend-type`'s first, TARGET-type argument, :2759/:2776/:2790),
`dialable-ty`, and `typedcap-ty` (as `extend-type`'s second, SATISFIED-SURFACE argument, :2776/:2790)
are all type-reference positions but neither a field/param/return annotation nor a declaration name —
`extend-type` doesn't declare a new type, it registers an impl between two existing ones. I filed
these under ANNOTATION (the migration's ANNOTATION destination — "a reference FORM `(Head :- [args])`"
— is the right shape for both slots) but they are a distinguishable sub-flavor an implementation brief
should probably name explicitly rather than silently fold into ordinary field-type annotations.

---

## PART 2 — EXCLUDED bindings (58) — named, not merely counted

### Class 1 — FUNCTION names (32): a `defn`/`defmacro` name, called by name elsewhere; never carries `<…>`

| binding | built at | mints |
|---|---|---|
| `init-name` | 646 | `<fqdn>::init` (given in the brief) |
| `serve-name` | 882 | `<fqdn>::serve` (given in the brief) |
| `start-name` | 888 | `<fqdn>/start` — **also dead**: `grep -n start-name` finds only its own definition line. Superseded by `start-macro-name`/`start-impl-*` (host-parity-4a); the comment at :884-886 explains the successor mechanism but the binding itself was never deleted. |
| `resume-name` | 2594 | `<fqdn>/resume` — **also dead**, same shape as `start-name` (superseded by `resume-macro-name`/`resume-impl-*`). |
| `dispatch-admin-name` | 1062 | `<fqdn>::dispatch-admin` |
| `extract-addr-name` | 1064 | `<fqdn>::extract-addr` |
| `stop-project-name` | 668 | `<fqdn>::stop-project` |
| `hibernate-project-name` | 696 | `<fqdn>::hibernate-project` |
| `stop-method-name` | 2019 | `<fqdn>/stop` |
| `hibernate-method-name` | 2065 | `<fqdn>/hibernate` |
| `grant-method-name` | 2108 | `<fqdn>/grant` |
| `grant-call-name` | 2112 | `<fqdn>/grant` (same string, "the BASE call name" — comment at :2110-2111) |
| `revoke-method-name` | 2161 | `<fqdn>/revoke` |
| `revoke-call-name` | 2163 | `<fqdn>/revoke` |
| `method-name` | 1877 | `<fqdn>/<op>` (per-op client method) |
| `service-forms-kw` | 2253 | `<fqdn>::service-forms` |
| `surface-forms-kw` | 2385 | `<surface>::surface-forms` |
| `sf-kw` | 852 | `<peer-surface>::surface-forms` (called at :854, `` `(~sf-kw)` ``) |
| `start-impl-name` / `start-impl-call` | 2420 / 2422 | `<fqdn>/start$impl` |
| `start-impl-thread-name` / `start-impl-thread-call` | 2424 / 2426 | `<fqdn>/start$impl-thread` |
| `start-impl-process-name` / `start-impl-process-call` | 2428 / 2430 | `<fqdn>/start$impl-process` |
| `start-macro-name` | 2432 | `<fqdn>/start` |
| `resume-impl-name` / `resume-impl-call` | 2434 / 2436 | `<fqdn>/resume$impl` |
| `resume-impl-thread-name` / `resume-impl-thread-call` | 2438 / 2440 | `<fqdn>/resume$impl-thread` |
| `resume-impl-process-name` / `resume-impl-process-call` | 2442 / 2444 | `<fqdn>/resume$impl-process` |
| `resume-macro-name` | 2446 | `<fqdn>/resume` |

### Class 2 — VALUE / ACCESSOR / CONSTRUCTOR (variant) names (26): base-only, per `wat/service.wat:213-216`

| binding | built at | mints |
|---|---|---|
| `state-new-kw` | 605 | `<fqdn>::State'` — prime positional ctor (mirrors the brief's `handle-new-kw`) |
| `state-durable-kw` | 653 | `<fqdn>::State/durable` — accessor |
| `handle-new-kw` | 930 | `<fqdn>::Handle'` — prime positional ctor (given in the brief) |
| `handle-handle-acc` | 2021 | `<fqdn>::Handle/handle` — accessor |
| `handle-addr-name` | 2757 | `<fqdn>::Handle/addr` — accessor |
| `admin-init-kw` | 1016 | `<fqdn>::Admin::Init` — variant ctor |
| `admin-stop-kw` | 1018 | `<fqdn>::Admin::Stop` — variant ctor |
| `admin-hibernate-kw` | 1021 | `<fqdn>::Admin::Hibernate` — variant ctor |
| `admin-resume-kw` | 1023 | `<fqdn>::Admin::Resume` — variant ctor |
| `status-started-kw` | 1025 | `<fqdn>::Status::Started` — variant ctor |
| `status-stopped-kw` | 1033 | `<fqdn>::Status::Stopped` — variant ctor |
| `status-hibernated-kw` | 1036 | `<fqdn>::Status::Hibernated` — variant ctor |
| `admin-allow-peer-kw` | 1042 | `<fqdn>::Admin::AllowPeer` — variant ctor |
| `status-peers-allowed-kw` | 1044 | `<fqdn>::Status::PeersAllowed` — variant ctor |
| `admin-deny-peer-kw` | 1053 | `<fqdn>::Admin::DenyPeer` — variant ctor |
| `status-peers-denied-kw` | 1055 | `<fqdn>::Status::PeersDenied` — variant ctor |
| `reply-failed-kw` | 880 | `<proto>::Reply::Failed` — variant ctor |
| `op-variant-kw` (×2, server-arm :1387 / client-method :1897) | 1387, 1897 | `<service>::Op::<Variant>` / `<proto>::Op::<Variant>` — match-pattern head / send-call ctor |
| `reply-variant-kw` (×2, :1391 / :1900) | 1391, 1900 | `<proto>::Reply::<Variant>` — variant ctor |
| `rtl-ctor-kw` (×2, :1529 / :1924) | 1529, 1924 | `<Op>Response::RequestTooLarge` — variant ctor |
| `rm-ctor-kw` | 1532 | `<Op>Response::RequestMalformed` — variant ctor |
| `cap-const-kw` (×2, :1516 / :1921) | 1516, 1921 | `<proto>::<OP>-MAX-REQUEST-BYTES` — **not even a variant**: a numeric CONSTANT keyword, resolved and compared with `i64::>` (:1664, :2000). The furthest from a type name of anything in this file. |

**32 + 26 = 58 excluded, 36 in scope, 58 + 36 = 94 = the full population.**

---

## Dead bindings found (4) — built, never consumed

All four are genuine STOP-1 findings — I looked, and confirmed absence with a plain-text,
scope-unaware grep for the bare identifier across the whole file (so even a comment mention would
have shown up; none did beyond the ones cited):

| binding | built at | would-be role | why it matters |
|---|---|---|---|
| `reply-name` | 868 | (would have been RUNTIME-ARG, mirroring `enum-name`) | the Reply-side twin of `enum-name` was apparently meant to register a subtype edge too, and doesn't |
| `peer-ty` | 936 | (would have been ANNOTATION) | `Peer<Reply,Op>` — superseded by `lineage-peer-ty`/`client-peer-ty`/`selectable-peer-ty` without being deleted |
| `vector-ty` | 944 | (would have been CTOR-ARG) | `Vector<Peer<Reply,Op>>` — superseded by `selectable-vec-ty` |
| `resp-ty` (client-method scope) | 1889 | (would have been ANNOTATION) | only its string precursor `resp-ty-str` is used; the keyword itself is discarded |

None of these affects the migration's CORRECTNESS (a dead binding has no consumer to get the wrong
spelling), but a codemod that walks every `keyword/from-string` call site and tries to classify a
destination for each will hit these four and find nothing to attach a role to — worth excluding
explicitly rather than leaving them to look like tool failure.

---

## Report summary (see also the narrative sections above)

- **Command / count**: `grep -noE '^\s*\[?[A-Za-z_][A-Za-z0-9_-]*\s+\(:wat::core::keyword/from-string' wat/service.wat` → **94** bindings (verified against the raw `keyword/from-string` occurrence count of 126; the 32-line gap is fully accounted for as runtime-emission call sites, not additional bindings).
- **What the command cannot see**: a type name built through any OTHER constructor (checked by full-file read, found none); a name/call split across a line boundary (believed absent, not proven); two same-named bindings in different scopes look identical to a name-only tool (5 such pairs found by hand).
- **Role distribution** (36 in-scope bindings, 73 consumption rows): ANNOTATION dominates (14 single-role + shares of the 7 multi-role bindings); RUNTIME-ARG 6 single-role + shares of 3 multi-role; DECL-NAME never appears alone — always paired with at least ANNOTATION; CTOR-ARG appears in 2 bindings (1 single-role, 1 shared with ANNOTATION); OTHER: 6 (2 structurally novel — `surface-kw`, `launch-head-kw` — and 4 dead).
- **Every OTHER row**: `reply-name`, `peer-ty`, `vector-ty`, `resp-ty`(:1889) — all dead (STOP-1); `surface-kw` — macro-expand-time-only helper argument; `launch-head-kw` — generic call-head with type args baked into the string.
- **Every binding with more than one role**: `service-op-decl-kw` (the known example) plus 6 new ones — `state-ty`, `record-ty`, `admin-ty`, `status-ty`, `handle-name`, `selectable-entry-ty`. `admin-ty` and `status-ty` are the sharpest: DECL-NAME + ANNOTATION + RUNTIME-ARG, and the RUNTIME-ARG site is the SAME call (`self-peer`) for both.
- **What surprised me**: (1) four dead in-scope bindings, a whole quarter of the "unique role" bindings I initially expected to classify turned out to have no consumer at all; (2) `extend-type`'s two type-reference argument positions (target type, satisfied-surface) don't map cleanly onto any of the four named roles — I filed them under ANNOTATION but flagged them as a distinguishable sub-case; (3) `launch-head-kw` is a role the brief's four-role table has no slot for at all — a generic function CALL-HEAD with its type arguments baked directly into the built name, spliced as the callee rather than as an argument to anything.
