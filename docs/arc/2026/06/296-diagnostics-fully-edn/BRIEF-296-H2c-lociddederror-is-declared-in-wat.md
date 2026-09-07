# BRIEF — 296 H-2c: LociDiedError is DECLARED IN WAT, and Rust sources from it

> H-2's writer is correct and H-2b finished the corpus. **The floor is RED at 24, and every one of
> them is one enum whose identity is retyped as a Rust string literal in three places.**

## THE DEFECT — one class, three sites, and two of them are now PERMANENTLY FALSE

Under H-2 a variant renders `#wat.kernel/LociDiedError.Panic` — namespace `wat.kernel`, name
`LociDiedError.Panic`. These were written against the old shape:

```
src/kernel/error.rs:367    tag.namespace() == "wat.kernel.LociDiedError"      ⛔ now always false
src/kernel/error.rs:508    tag.namespace() == "wat.kernel.LociDiedError"      ⛔ now always false
src/process/verbs.rs:81    Tag::ns("wat.kernel.LociDiedError","StartupError") + Vector body
                           ⛔ a PRODUCER of the retired wire; the new reader correctly refuses it
                              ("StartupError vector body retired (arc 296 H-2)")
```

The consumers never recognise a chain, so `loci_died_error_from_reason` falls to its fabricating
fallback and the caller gets the whole envelope as a STRING. That is every remaining failure:
`:actual "[#wat.kernel/LociDiedError.Panic {…}]"` where `:expected "assert-eq failed"`.

★ This is the recurring class named in `CLAUDE.md`: *"suspect a **string comparison with one side
normalized and the other not** before suspecting the type system."* Retyping the three literals
would go green and leave the class alive — the NEXT wire change unhooks them again, silently.

## THE RULING — builder, 2026-09-06

> *"get them declared in `wat/kernel/*.wat` — have the rust code source from wat. we have several
> examples for how to do this."*

`LociDiedError` is a **Rust builtin today** — no `defenum` for it exists in `wat/`; its variant field
names come from `TypeEnv::with_builtins` via `builtin_enum_variant_names` (`src/runtime.rs:10047`),
an 11-arm hand table that falls through to a registry lookup and then a panic. Declare the enum in
wat; generate the Rust from it.

## THE EXAMPLES TO COPY — eight live consumers, and their prose is the argument

```
src/intrinsic/mod.rs:69·75·81      Kind · DefinedIn · Layer   <- "wat/runtime-meta.wat"
crates/wat-doc/src/lib.rs:60·66·107·2436·2473                 Category · Purity · Determinism · …
```

`src/intrinsic/mod.rs:54`, verbatim, is why this stone is the right shape and not a patch:

> *"each variant's prose live in the `.wat` file's `defenum` forms; `wat_enum_from!` reads them at
> compile time. **Add a variant there and these types follow — there is no Rust list to keep in
> step, which is why the drift gate that used to guard them is gone rather than merely passing.**"*

The macros: `wat_enum_from!` (a `defenum` → a Rust enum), `wat_enum_field_names_from!`,
`wat_field_names_from!` — all in `crates/wat-source-derive/src/lib.rs` (`:160`, `:671`, `:547`).

## READ IN ORDER

```
src/intrinsic/mod.rs:54-90              THE PATTERN — read the prose above the three macros first
crates/wat-source-derive/src/lib.rs:160 wat_enum_from!'s contract (what a defenum must look like)
wat/kernel/diagnostics.wat:124-133      LociDiedError is ALREADY REFERENCED here
                                        (`upstream-chain <- (Vector :- [:wat::kernel::LociDiedError])`)
                                        — this file is its home, and the reference proves it
src/types.rs                            the builtin registration the wat declaration replaces
src/runtime.rs:10047                    builtin_enum_variant_names — the hand table
src/kernel/error.rs:364·493             the two consumers
src/process/verbs.rs:75-88              the producer
```

## THE WORK

1. **Declare `:wat::kernel::LociDiedError` as a `defenum` in `wat/kernel/diagnostics.wat`**, with
   every variant and payload the Rust builtin registers today. **Derive the variant list from the
   registration, not from this brief** — I have not enumerated it here on purpose.
2. **Generate the Rust from it** with `wat_enum_from!`, mirroring `src/intrinsic/mod.rs`.
3. **Retire the hand-typed identity at all three sites.** The two consumers must ask a generated
   question, not compare a typed string. The producer must go through the same writer every other
   variant uses.

## STOP TRIGGERS

- **STOP-1 — you are about to retype the three string literals in the new shape.** That is the
  patch, not the stone: it goes green and leaves the class alive. If wat-sourcing turns out to be
  blocked, STOP and report what blocks it.
- **STOP-2 — `wat/kernel/diagnostics.wat` is FROZEN into the release binary.** Read `wat/fix.wat`'s
  header (lines 22-53) BEFORE running anything that needs the new declaration visible. If a stash
  dance is required, that is the supported path.
- **STOP-3 — the declaration and the builtin registration disagree.** `builtin_enum_variant_names`
  already panics with *"the constructor and the registry disagree."* If that fires, the wat
  declaration is wrong — fix it, do not widen the panic.
- **STOP-4 — H-2's probe goes red.** `tests/types/probe_arc296_h2_variant_tag_and_body.rs` is 3/3
  green. This stone must not move the writer.
- **STOP-5 — another builtin enum needs the same treatment to make this compile.** ServiceEvent (7
  arms) and sqlite::Cell (2) share that hand table. If they must move too, STOP and report — that
  is a bigger stone and the builder decides its scope.

## OUT OF SCOPE, AFFIRMATIVELY

H-3 (Option/Result into wat declarations) · the match arm · the constructor form · the
record-steals-a-variant's-constructor NOTE.
