# HALTED — out of scope. Ω4 belongs to main, not to this session.

**Builder's ruling, 2026-09-05:** *"we should only be working on rete here… main is doing a lot of
cleanup across the code base… this session is dedicated to making the rete code an exemplar."*

`src/config.rs` is not rete. This strike is stopped mid-flight and its code changes are reverted.

## The finding is REAL and stays on the record for main

Driven at HEAD `0ee56325f`, orchestrator's own drives:

```
set-dim-count! 4096                                 ->  "4096"   rc=0   control
setmax-fire-rounds! 5 (typo), then set-dim-count!   -> "10000"   rc=0   Ω4a
setmax-fire-rounds! 5 (typo) alone                  -> "10000"   rc=0   Ω4a
defrecord ..., then set-dim-count! 4096             -> "10000"   rc=0   Ω4b
:totally::bogus::head! ...                          -> UnresolvedReferences, startup fails (control)
```

- **Ω4a** — a mistyped config head fails `leaf.starts_with("set-")` (`config.rs:465-469`), takes the
  `_ =>` arm, ends the setter section, and every valid setter behind it is never processed.
- **Ω4b** — `SetterAfterNonSetter` is unreachable: `remainder_start` is assigned at one site
  immediately followed by `break`, so `if remainder_start.is_some()` (`config.rs:477`) can never see
  `Some`. A correctly-spelled setter after any body form is silently ignored.
- **Root** — `cernere` C1, the open `:wat::` vocabulary (`resolve/walk.rs:268`). The control above
  shows the vocabulary is closed for every other namespace.

## The correct cure, already worked out — and the trap in it

**Do NOT use `ends_with('!')` as the discriminator.** That was this strike's contract decision and it
is WRONG: `set-redef!` / `set-eval-redef!` are processed INLINE mid-program on purpose
(`check.rs:722-728`, arc 157 slice 1a-ii — single-pass program-order semantics). That rule reddened
nine tests in `tests/wat_lang/wat_arc157_def.rs` by outlawing a designed capability.

**The correct discriminator already exists:** `special_forms.rs` registers `set-redef!` (`:128`) and
`set-eval-redef!` (`:133`) as legal FORMS; entry-file-only setters (`set-dim-count!`,
`set-capacity-mode!`, `set-global-seed!`, `rete::set-max-fire-rounds!`) are absent.
`lookup_special_form` (`:65`) is the one query door.

> A `:wat::config::…!` head in the remainder that is NOT a registered special form is misplaced.
> Valid `set-` leaf → `SetterAfterNonSetter`; anything else → `UnknownSetter`.

Use the registry, not a new allowlist in `config.rs` — an allowlist beside the checker's own inline
handling is one rule encoded twice with nothing forcing agreement (CLASS A).

**Whoever lands this must add a gate that `set-eval-redef!` in a body stays legal**, or the next
tightening re-breaks arc 157 silently.

Also open and NOT this: `RequiredFieldMissing` is declared and never constructed, while
`config.rs:16` / `:30-36` contradict each other on whether fields are required.
