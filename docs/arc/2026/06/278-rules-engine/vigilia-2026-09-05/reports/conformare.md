# ward `conformare` — vigilia 2026-09-05

> Cast at HEAD `21530efab`, branch `grok-rete`. Read-only unless the ward says otherwise.
> **Verbatim ward return, preserved unedited.** The 2026-08-30 cast lost its 19 returns
> because they lived only as subagent messages; this file exists so that cannot recur.
> Nothing here is adjudicated — see `../WORK-LIST.md` for what the orchestrator credited.

---

Report complete. Findings below.

---

# `conformare` — wat-rs @ `21530efab` (`grok-rete`)

**Substrate manifest.** 26 error types. Pattern A is the settled shape and it has landed broadly. Two sub-shapes exist and nobody has named the split: **sealed** Pattern A (`TypeError`, `RuntimeError`, `LoadError`, `StdlibError` — private `span`, a `new()` "ONE door") vs **open** Pattern A (`CheckError`, `MacroError`, `ConfigError`, `LowerError`, `ExtractionError`, `ClauseGrammarError`, `EdnReadError`, `ReteCheckError`, `ArgSpecError` — `pub span`, struct-literal constructible). Every finding below lives in the seams the gates key on `&Span` params and therefore cannot see.

---

## L1 — defects

### L1-1 · `StartupError::SigmaFn(String)` is the one-`String` carrier this repo already documented as a defect class, with four live producers re-creating the mask

`src/freeze.rs:715` — `SigmaFn(String)`. Producers at `src/freeze.rs:560`, `:569`, `:594`, `:603`:

```rust
let v = crate::runtime::eval(&sigma_ast, &env, &symbols)
    .map_err(|e| {
        StartupError::SigmaFn(format!(
            "set-presence-sigma! body failed to evaluate: {}", e))
    })?
```

`crate::runtime::eval` returns `Result<TrackedValue, RuntimeError>` (`src/runtime.rs:5113-5117`). `RuntimeError` is sealed Pattern A with a real span, and its `Display` is `to_wire_edn(self)` (`src/value/signal.rs`, via `runtime_error_edn.rs`). So `e`'s **entire `error_edn()` floor record — tag, `:message`, `:location`, variant fields — is `wat_edn::write`n and interpolated into a String slot.**

Downstream (`src/macros/error_edn.rs:179`, `:196`) the wrapper then reports `location() → Nil` and `causes() → []`.

The repo has already ruled on exactly this. `check_failed_cause` (`src/runtime.rs:29098-29162`) carries the argument verbatim:

> *"The FIRST draft of this function did exactly the wrong thing — `e.to_string()` into `:wat::kernel::StartupError`, whose registered shape is a single `message <- String` — which re-created the mask `startup_error_chain_edn` had already been fixed to remove … The root was never the call site — **it is that a one-`String` carrier leaves a producer NO honest option, so every new producer re-creates the mask.**"*

`SigmaFn(String)` is that carrier and these are those producers. And they route through `check_failed_cause` itself: a REPL/MCP turn that trips a sigma failure emits a `#wat.core/Fault` whose `:causes` holds a `#wat.macro/SigmaFnError {:location nil :causes [] :detail "<a whole EDN record as prose>"}`.

**What a caller cannot learn:** the source location of the failing sigma body (it exists, in the inner `RuntimeError.span`, present only as characters); the inner error's kind/variant as data; anything at all by walking `:causes`. `:location nil` is a false statement — a location was in hand and was spent on prose.

**Fix:** `SigmaFn { span: Span, cause: Box<RuntimeError> }` for the eval arm (the `sigma_ast` at `freeze.rs:556`/`:590` supplies the span); `causes()` returns the cause's `to_edn()`. Precedent is `RuntimeErrorKind::MacroExpansionFailed { cause: Box<MacroError> }` (`src/value/signal.rs:495`) and `StdlibError::ParseFailed`.

### L1-2 · `WatError::causes()` returns `[]` in **every** implementation in the crate — the floor's third key is structurally dead

`src/to_edn.rs:90-93` states the contract: *"include the inner error's `ToEdn::to_edn` output for errors that wrap a typed cause."*

Every impl in `src/`: `parser.rs:24`, `config.rs:340`, `load.rs:405`, `types/error.rs:448`, `runtime_error_edn.rs:95`, `stdlib.rs:657`, `check/error_edn.rs:58` and `:95`, `resolve/error.rs:66`, `rete/validate/error.rs:505` and `:529`, `macros/error_edn.rs:120`, `to_edn.rs:378`. All literal `Vector(vec![])`. The only non-literal arms — `macros/error_edn.rs:183-198` — delegate to those, and bottom out empty.

The substrate demonstrably **has** wrapped typed causes: `LoadErrorKind::Fetch(LoadFetchError)` (`load.rs:315`), `LoadErrorKind::Parse { err: ParseError }` (`load.rs:~316`), `MacroErrorKind` at `macros/error.rs:120` and `:132`, `RuntimeErrorKind::MacroExpansionFailed` (`signal.rs:495`), `ArgSpecErrorKind::MalformedTypeKeyword { inner }` (`argspec/error.rs:37`). They are all nested under a **variant-specific `:cause` key** while the floor's `:causes` says `[]`.

**What a caller cannot learn:** whether an error has a nested cause, without family-specific knowledge of which variant uses which key. That is precisely the decomplection the floor exists to provide — `to_edn.rs:47-50`: *"the three fields that tooling and the runtime always expect to navigate, **regardless of the specific error family**."* A generic consumer walking `:causes` traverses zero levels, forever, and the empty vector reads as an affirmative "no nested cause," which is false.

**Fix:** make the wrapping variants populate `causes()` from their `:cause`/`:err`/`:inner` field (`LoadError`, `MacroError`, `RuntimeError` first). Then a gate: for each error type, a probe asserting a constructed wrapping variant's `:causes` is non-empty. Note `#wat.core/Fault` from `check_failed_cause` **does** populate `:causes` — so the wire shape is right and only the Rust side never fills it.

### L1-3 · A `load!` of a missing file points the user at `src/load.rs`, while the cycle error two lines away points at their `.wat`

`src/load.rs:435-438`:
```rust
impl From<LoadFetchError> for LoadError {
    fn from(e: LoadFetchError) -> Self {
        LoadError::new(crate::rust_caller_span!(), LoadErrorKind::Fetch(e))
```

Reached from `src/load.rs:489-497` `process_single_load(spec, form_span: Span, …)`:
- `:498` `let fetched = fetch_source(&spec.source, base_canonical, loader)?;` — `form_span` **not passed**; `fetch_source` (`:558-568`) takes no span; the `?` fires the `From` above.
- `:503` `LoadError::new(form_span.clone(), LoadErrorKind::CycleDetected { … })` — the same fn, the real wat span, used correctly.
- `:524` `crate::rust_caller_span!()` for `LoadErrorKind::Parse`, un-runed, with `form_span` alive in scope.

**Why no gate sees it.** `tests/lint/span_substitution_justified.rs:113` — `SPAN_PARAM_TAIL = r"span: ?&Span\b"`. `form_span: Span` is **by value**, so `has_used_span_param` is false and the fn is skipped entirely. This is the same class the lint's own header names as its measured proxy failure (`refuse_export_without_arm`), recurring through a narrower hole: not "no span param," but "a span param that isn't a reference."

I swept `src/` for the shape (by-value non-`_` `…span: Span` param, no `&Span` param, `rust_caller_span!()` in body, no rune), anchored on this known positive: **exactly one site, `src/load.rs:524`.**

**What a caller cannot learn:** which `load!` form in their program failed. A multi-load file gives them `src/load.rs:437`.

**Fix:** thread `form_span` into `fetch_source`/`fetch_payload` and construct `LoadErrorKind::Fetch` at the call site; delete the `From` impl or rune it. Then widen `SPAN_PARAM_TAIL` to `span: ?&?Span\b` — the by-value form is the same claim.

### L1-4 · `src/distribution/mcp.rs` hand-formats EDN onto a live wire, unescaped, under an unregistered tag

`src/distribution/mcp.rs:273`:
```rust
format!("#wat.core/Fault {{:message \"session survived a panic: {msg}\"}}")
```
`msg` is `panic_text(&payload)` → `format_panic_payload` (`src/runtime.rs:27639-27650`), which returns a panic `String`, an `&'static str`, or `AssertionPayload.message`. An `assert-eq!` message carries rendered values; a `panic!("{:?}", s)` carries quotes. **Any `"` or `\` in `msg` emits malformed EDN.** No escaping anywhere on this path.

Two further problems on the same line and at `:319-320`:
- `#wat.core/Fault` here carries `:message` only — no `:location`, no `:causes`. Everywhere else in the tree a `#wat.core/Fault` is the three-field floor record (`wat/core.wat:2101`, `tests/cli/wat_repl__bad_then_good_fault.edn`). A strict decoder will reject it; a lenient one gets a `Fault` missing mandatory fields.
- `:319-320` emit `#wat.mcp/Fault` — **`wat.mcp` is not in `src/error_ns.rs`**, whose header claims to be *"THE single source of truth for error tag namespaces. Rename HERE → every production emission site follows (one edit)."* Two hand-typed literals falsify that claim.

**Why no gate sees it.** `tests/lint/no_inlined_edn.rs:75-77` scans root `tests/` only, justified as *"`src/`/`crates/*/src/` EDN is edn-WRITING machinery."* True of `crates/wat-edn/src/json.rs`. False of these three sites, which are hand-written literals on the MCP wire.

**Fix:** build these through `OwnedValue` + `wat_edn::write` like every other emission; add `MCP` to `error_ns.rs` or use `CORE` with the full floor.

### L1-5 · `UnknownEnumVariant` — D1's cure landed the name and dropped the caret, contradicting the doctrine 20 lines below it

`src/rete/validate/typing.rs:46` — `if let WatAST::Keyword(k, _) = operand` discards the keyword's own span.
`src/rete/validate/typing.rs:60-69` — the new `UnknownEnumVariant` is built with `span: clause.span().clone()`.

Two lines below, `:76-77`, the sibling arm carries the opposite instruction:
> *"The OPERAND NODE, not `clause.span()`: the keyword IS the field reference, so its own span is the only one this producer can be handed."*

And `check_field_kw`'s doc (`:82-97`) is a full argued ruling against exactly this:
> *"⛔ **It does not take a `Span`, and that is the whole point.** … BOTH its callers passed `clause.span()`, while two more sites open-coded the same error against an enclosing form's span … **Taking the NODE makes the wrong span unwritable at the call.**"*

D1 cured the *message* (a variant typo no longer says "has no field") but reintroduced the *span* half of the same defect, in the same file, via the escape hatch that doctrine did not close: `ReteCheckError`'s fields are `pub`, so the raw struct literal accepts any span. `check_field_kw`'s node-taking discipline binds one producer; **25 open-coded `ReteCheckError { span, kind }` literals** exist in `src/`, of which 4 pass `clause.span()`, 2 `cond.span()`, 2 `fact_span` — all enclosing forms.

**What a caller cannot learn:** which keyword in a multi-operand `:when` clause is the misspelling. The message names the variant; the caret spans the whole clause.

**Fix:** pass `operand` (bind the span at `:46`) — one-line. Structurally: seal `ReteCheckError` behind a constructor that takes `&WatAST` for the offending node, mirroring `check_field_kw`'s own argument, which makes the enclosing-form span unwritable for all 25 sites rather than one.

---

## L2 — weaknesses

**L2-1 · `validate_user_main_not_useless` throws away the span it is standing on.** `src/freeze.rs:1682` — `if matches!(&**ast, WatAST::NilLit(_))`. `WatAST::NilLit(span)` carries a `Span` (bound at `src/lower.rs:166`). The fn returns `Result<(), String>` (`:1676`), so the type cannot carry it. `StartupError::MainSignature(String)` at `freeze.rs:720` reports `:location nil` (`macros/error_edn.rs:180`). *Remedy:* return `Result<(), (Span, String)>` or a kind, bind the `NilLit` span. Its sibling `validate_user_main_signature` (`:1623`) is genuinely spanless — `Function` (`src/value/environment.rs:48-130`) has no span field — and earns a `rune:conformare(spanless-by-domain)` naming that.

**L2-2 · `check_sigma_fn_contract` is an API discarding caller span context.** `src/freeze.rs:752` takes `(setter, func, sym)`; call sites `:575`/`:609` hold `sigma_ast` — a `WatAST` with a span — and don't pass it. Four spanless `SigmaFn` errors at `:758`, `:767`, `:774`, `:808`. *Remedy:* add `span: &Span`; that also brings the fn inside `span_substitution_justified`'s view.

**L2-3 · `_ =>` over `StartupError` decides a process exit code.** `src/distribution/mod.rs:434-438`: `MainSignature(_) => EXIT_MAIN_SIGNATURE, _ => EXIT_STARTUP_ERROR`. `StartupError` has 11 variants (`freeze.rs:680-721`); the catch-all holds "not a main-signature failure" *and* "a variant added after this was written." Exit codes are the only channel a shell caller has, and a new variant needing its own code gets 3 silently. *Remedy:* name all 11 arms.

**L2-4 · `HarnessError` is the one public error type outside the floor.** `src/harness.rs:68-86`: no `ToEdn`, no `WatError`. Its `Display` at `:77`/`:79` does `write!(f, "startup: {}", e)` where `e`'s `Display` **is** `to_wire_edn` — producing `startup: #wat.kernel/…{…}`, neither EDN nor prose. It is the return type of `compose_and_run` / `compose_and_run_with_loader` (`src/compose.rs:139`, `:164`) — what `wat::main!` expands to, i.e. the outermost error surface of a real wat binary. `MainSignature(String)` and `StdioSnapshot(String)` are flat carriers with no rune. *Remedy:* implement `WatError` delegating like `StartupError` does; drop the prose prefixes.

**L2-5 · `rune:conformare` — this ward's own exemption mechanism — is ungated.** `tests/lint/no_unknown_ward_rune.rs:52-66` holds two rows: `perspicere`, `purgare`. A census of the tree finds **14 rune owners**: `lint` 469, `perspicere` 47, `sequi` 39, `struere` 26, `vocare` 24, `purgare` 11, `exigere` 10, `temperare` 9, `complectens` 7, `solvere` 6, `excusare` 5, `coverage` 3, `intueri` 2, `conformare` 2, `circumspicere` 2. Only `perspicere`/`purgare` (this gate) and `sequi` (`no_unknown_sequi_rune.rs`) are validated. `rune:conformare(anything-at-all)` passes today. The two live sites (`src/collection/eval.rs:22`, `src/capability/registry.rs:93`) are both sound and both `spanless-by-domain` — but nothing holds them to it. *Remedy:* add the remaining rows, `("conformare", &["spanless-by-domain", "attested-arc"])` first.

**L2-6 · `coerce_variant_single`'s `_ =>` conflates two failures and names neither.** `src/edn_shim.rs:2108-2118`:
```rust
Edn::Vector(items) | Edn::List(items) if items.len() == 1 => Ok(&items[0]),
_ => Err(mismatch(target, edn)),
```
`mismatch(target, edn)` (`:2098-2104`) reports `got: edn_shape_name(edn)` — of the **outer tagged value**. So `#tag [1 2]` (right tag, wrong arity) reports `expected: <type>, got: "Tagged"`. The doc at `:2104-2106` claims it *"Enforces vector body + arity-1 so a malformed body fails loudly"* — the failure names neither the body's shape nor its arity. *What a caller cannot learn:* whether the body was the wrong shape or the right shape with the wrong arity. *Remedy:* split the arms; carry `expected_arity`/`got_arity`.

**L2-7 · The four `From<ArgSpecError>` impls flatten a typed cause to prose.** `src/argspec/error.rs:78-107`. All four call `e.kind.reason()` → `String`. `ArgSpecErrorKind::MalformedTypeKeyword { inner: Box<TypeErrorKind> }` (`:37`) is a typed cause; `reason()` (`:62-63`) does `format!("invalid type keyword: {}", inner)`. Span survives (good) — structure does not, and the destination's `causes()` is `[]` (L1-2). *Remedy:* route `inner` into the destination's cause field.

**L2-8 · `LoadFetchError::Other` collapses every non-`NotFound` io kind.** `src/load.rs:1110-1114`, `:1133-1137`, `:1222-1226` — `match e.kind() { NotFound => …, _ => Other { path, reason } }`. `PermissionDenied` (fix: chmod), `IsADirectory` (fix: a different path), and a transient EIO become one variant distinguishable only by parsing `reason`. *Remedy:* name `PermissionDenied` and `IsADirectory`; the rest is an honest `Other`.

**L2-9 · A rotted doc line on the flat-arm inventory.** `src/macros/error_edn.rs:142`: *"The only genuinely flat arm is `SigmaFn`."* There are two — `MainSignature` sits beside it in every one of the four matches (`:160`, `:180`, `:197`, `:224`). The claim was true when written and was not re-derived when the second arm landed. *Remedy:* one word, or name both.

---

## L3 — judgement

**L3-1 · The substrate's answer is sealed Pattern A, and half the catalogue hasn't taken it.** Four types (`TypeError`, `RuntimeError`, `LoadError`, `StdlibError`) already run private-`span` + `new()`-as-the-ONE-door and each carries a paragraph explaining why. Nine run `pub span` and are struct-literal constructible from anywhere. On the four-questions, the split is decisive on **Honest** — an open type cannot refuse a wrong span, and L1-5 is that refusal failing in the newest code in the tree. Pattern A (sealed) over Pattern B/C: B leaks `Spanned<E>` into every signature; C introduces two authorship patterns per type. Sealing is a mechanical retrofit and the precedent is already written four times. **Retrofit order by cascade × depth:** `ReteCheckError` (25 open literals, a live wrong-caret defect, newest code) → `CheckError` (widest consumer set) → `MacroError` → the rest.

**L3-2 · `causes()` is the honest place to put the whole class.** L1-1, L1-2, and L2-7 are one defect wearing three hats: a typed inner error exists, and the floor key built to carry it is empty while the payload rides as prose or under a private key. Fixing `causes()` for the wrapping variants closes all three and gives L1-1's cure somewhere to land.

**L3-3 · Both span lints key on `&Span` and that is now the shape of the remaining holes.** `span_substitution_justified` skips by-value `Span` (L1-3); neither lint can see a fn returning `Result<_, String>` (L2-1, L2-2), a `map_err` closure stringifying a spanned error (L1-1), or a span discarded in a *pattern* rather than a param (`WatAST::Keyword(k, _)` — L1-5, `NilLit(_)` — L2-1). The lints' own headers are unusually honest about being proxies; this is the next iteration of that honesty. A cheap third gate with real reach: **no substrate error type may be constructed from a `format!` whose argument implements `WatError`** — that single rule catches L1-1 and is checkable by regex.

---

## What I could not check, and why

- **I did not build, run, or drive anything.** Read-only cast. Every claim is a reading of source at `21530efab`. L1-4's escaping defect is inferred from the absence of any escape on the path (`format_panic_payload` returns the raw String; `format!` interpolates it into a quoted EDN literal) — **I did not produce a panic with a `"` in it and observe malformed EDN.** That is one MCP `eval` turn away and should be the first thing driven.
- **I could not mutation-prove any gate hole.** L1-3's "the lint cannot see this" rests on reading `SPAN_PARAM_TAIL` and matching it against `form_span: Span` by hand, plus my own reimplementation of the walk. Reimplementing a gate to prove it blind is exactly the instrument this repo distrusts — I anchored on a known positive (`process_single_load` must appear, and did), but the correct proof is renaming `form_span` to `form_span: &Span` and watching the lint go red. Same for L1-4 vs `no_inlined_edn` (whose `src/`-exclusion is stated in its own header, so that one is documentary rather than inferred).
- **`crates/` is unexamined.** `crates/wat-edn`, `crates/wat-reader` and the other workspace members define `ParseError` and the `ToEdn` derive, both load-bearing for every finding above. The brief scoped me to `src/`, `tests/`, `wat/`; `ParseError`'s own shape is unaudited and it is the leaf of the `LoadErrorKind::Parse` cause chain in L1-2.
- **`wat/` was touched only where a Rust finding pointed into it** (`wat/core.wat:2101-2122`, `wat/repl.wat:90-112`). I did **not** audit wat-level error construction — `:wat::core::fault`, `raise`, `assertion-failed!` — as its own error surface. If wat-side producers build `Fault` values with empty `:causes` the way `test_runner.rs:1046` does, L1-2 has a second half I have not measured.
- **L1-2's universality claim is a reading of 14 impl bodies, not an exhaustive type-level proof.** I grepped `fn causes(&self)` across `src/` and read every body; a `causes()` reached through a blanket impl or a macro expansion would not appear in that grep. I judge this unlikely (`WatError` impls here are all hand-written) but I did not rule it out.
- **I did not count consumers** of the nine open Pattern A types, so L3-1's retrofit ordering is ranked on open-literal counts and code age, not on measured cascade. Do not treat that ordering as measured.
