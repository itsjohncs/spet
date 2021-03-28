# Spet

A **spet** is a set of spans (aka intervals). Ex: `{1.2–4, 8.1–8.4}`

Set operations behave intuitively. Ex:

* `{1.2–4, 8.1–8.4} ∪ {3–8.2} = {1.2–8.4}`
* `{1.2–4, 8.1–8.4} ∩ {3–8.2} = {3–4, 8.1–8.2}`

# Construction

Use `SimpleSpan::new` to construct a span (`CreatableSpan` trait must be in scope).

```rust
use spet::{SimpleSpan, CreatableSpan};

let my_span: SimpleSpan<f64> = SimpleSpan::new(1.2, 4);
```

Use `VecSpet::from_sorted_iter` to construct a spet.

```rust
use spet::{VecSpet, SimpleSpan, CreatableSpan};

let spans = vec![
    SimpleSpan::new(1.2, 4),
    SimpleSpan::new(8.1, 8.4),
];

let my_spet: VecSpet<SimpleSpan<f64>> =
    VecSpet::from_sorted_iter(spans);
```

# Operations

A `VecSpet` is immutable once constructed.

```rust
let a_new_spet = a.union(b);
```

A tree-based mutable spet implementation could be made: PRs welcome.
