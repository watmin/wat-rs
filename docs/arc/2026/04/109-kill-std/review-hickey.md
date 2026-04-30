# Review: Rich Hickey

Verdict: APPROVED

## On the partition

Three tiers, three honest questions: "is this an irreducible primitive?", "is this convenience that hides a runtime decision?", "is this composition over a sequence?" That's three concerns, three names, no overlap. I can answer each question about any op in the substrate without consulting a second one. That's simple in the sense I care about.

The honesty rule for `core` — "if the op dispatches on operand TYPE, it does NOT live here" — is exactly the kind of constraint that keeps a tier from rotting. The minute you let `+` sit in `core` and quietly branch on i64-vs-f64 inside the implementation, you've complected the primitive layer with a dispatch mechanism. Pulling that apart is the right move. `:wat::core::i64::+` says what it does; `:wat::poly::+` admits what it does. Neither pretends.

`list` as its own top-level tier, not `:wat::std::list::*`, is also right. "std" is the dumping-ground name. It tells you nothing about what's inside. Once you've decided `list` is a substrate concern worth exposing at all, hiding it under a generic umbrella is just one more indirection the reader has to walk through. Same for `math` and `stat`. `std` was a place; the new tiers are concerns. Kill the place. Good.

## On `:wat::poly::*`

This is the piece I want to push on, but I'll come down on the side of keeping it.

The temptation would be to say: "you have mono-typed primitives, that's enough; let users compose what they want on top." That's the purist line. But polymorphic `+` over numerics, polymorphic `empty?`/`length`/`get` over collections — these aren't really conveniences in the sloppy sense. They're a recognition that "the same operation across many types" is itself a thing programmers reach for, and naming the tier where that lives is more honest than scattering the dispatch through `core`.

The name `poly` does the work. It tells you the dispatch story before you call the function. It's not "common" or "auto" or "ergo" — those describe motivation. `poly` describes mechanism. That's the distinction that matters.

What I'd watch: don't let `poly` grow into a place where everything-someone-found-handy gets dropped. The tier earns its keep ONLY for ops that genuinely dispatch on operand type. The moment `poly::frob` shows up because it "felt convenient" with no type-driven story, the tier has rotted. The doctrine in the doc — "if it exists ONLY because it makes the surface less verbose, this is its home" — is dangerously close to that failure mode. Tighten it: "if it dispatches on operand type to give one name across many types, this is its home." Otherwise you've reinvented `std`.

## On FQDN-everywhere

Names are values. Stable, comparable, addressable. `:wat::core::Vector` is a value; `Vec` was a place — a slot in some local namespace that depended on what was imported. Eliminating the imported-name machinery and saying "the FQDN IS the name" pushes the substrate from places to values. I'd do the same.

The verbosity complaint is the kind of complaint people make when they haven't yet had to read code six months later. Long names are read more than written. Fine.

## On algebra complection

I confirmed the algebra at `:wat::holon::*` is untouched. Bind, bundle, cosine — they live where they lived. This proposal partitions the *organizing* layer of the substrate, not the algebraic kernel. No complection.

## On `Vec → Vector` and `Type/verb`

`Vec` was Rust's name. `Vector` is the substrate's. Owning the name is consistent with the doctrine. Making the type and the constructor share the name (`Vector` is both) is one less thing to remember; it's the same kind of unification Clojure does with `vector` and the literal reader.

`Type/verb` — `Option/expect`, `Result/try` — is a clear shape: which type owns the verb. Better than `option::expect` because the slash signals "method on type" rather than "thing inside namespace." I'd commit to it.

## What I'd push back on

Just one thing, repeated: keep `poly` honest. Write the rule down so the next person doesn't soften it.

Approved.
