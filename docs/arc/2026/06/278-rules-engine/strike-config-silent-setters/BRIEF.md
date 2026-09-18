# BRIEF — Ω4: silent config setters

Cure both defects **and** land their gates in one strike. **Floor GREEN when you are done.**

## Read in order

1. **`DESIGN.md` beside this file** — it pins the discriminator (`ends_with('!')`) and records that
   the ward reported one defect where driving found two.
2. **`src/config.rs:455-485`** — the setter-match arm, the `_ =>` that sets `remainder_start` and
   `break`s, and the guard at `:477` that can never fire.
3. **`src/config.rs:220-232`** — `ConfigErrorKind::SetterAfterNonSetter { setter_head }`, declared
   and never constructed.
4. **`src/config.rs:16` and `:30-36`** — the module doc, which contradicts itself on whether fields
   are required. **Do not fix that here** (see the DESIGN's cut), but read it so you do not make it
   worse.
5. **`src/freeze.rs:1221`** — `collect_entry_file(entry_forms)`, the one caller on the program path.

## Implementation sketch

The section still ends at the first non-setter — keep the `break`. Then scan the remainder:

```rust
// After the loop, `remainder_start` marks where the body begins.
if let Some(start) = remainder_start {
    for (j, form) in forms[start..].iter().enumerate() {
        let Some(head) = head_keyword_of(form) else { continue };
        if head.starts_with(":wat::config::") && head.ends_with('!') {
            // setter-shaped, in the body: SetterAfterNonSetter if the leaf is a
            // valid `set-`, else a malformed config setter. Both LOCATED.
            return Err(ConfigError { span: …, kind: … });
        }
    }
}
```

Reuse the file's existing head-extraction helper rather than hand-rolling one, and reuse
`wat_reader::identifier::leaf` for the `set-` test — `config.rs:457-461` records that a hand-rolled
`rsplit("::")` here was caught by `tests/lint/one_name_grammar.rs`, and that a name has ONE grammar
door. **Do not add a second parser of a name.**

## The gates — `tests/` , adjacent fixtures

Four rows, three red at HEAD and one control:

| fixture | at HEAD | proves |
|---|---|---|
| typo'd setter, then a valid one | `10000` rc=0 | Ω4a |
| typo'd setter alone | `10000` rc=0 | Ω4a, no valid setter to mask it |
| body form, then a valid setter | `10000` rc=0 | Ω4b — `SetterAfterNonSetter` reachable |
| valid setter alone | `4096` rc=0 | **control** — must stay green, and a body form calling `(:wat::config::dim-count)` must stay legal |

The control is load-bearing: it is what proves the cure did not outlaw accessors in the body.

## Blast radius

`src/config.rs` + tests. No change to `resolve/` or `check.rs` — closing the `:wat::` vocabulary is
a different strike.

## STOP triggers

1. **If the cure makes any existing `.wat` in the corpus fail to load, STOP and report** with the
   file and the verbatim error. Some entry file may legitimately carry a `:wat::config::…!` form in
   its body today, and that is a finding about the corpus, not a nuisance.
2. **If you need a second name parser to tell a setter from an accessor, STOP** — see the
   one-name-grammar note above.
3. **If `RequiredFieldMissing` becomes reachable as a side effect, STOP** — that is the cut item and
   it changes whether an empty entry file is legal.
4. **On any RED elsewhere: DO NOT RE-RUN.** Capture whole, name the arm, surface it.

## Prior result to copy for shape

`../strike-left-idx-latch/` — cure and gate in one strike, fixture byte-identical to the probe that
found the defect, floor green, and the unrepresentability claim proven by a compiler error.
