---
layout: post
title:  "Control Flow for Visitor Callbacks"
date:   2016-10-17 20:01:00 +0200
categories: rust
---

For my graph library [**petgraph**][1] I found a new pattern
for visitor callbacks that hits the sweet spot for both convenience and the
“pay for what you use” principle.

[1]: https://docs.rs/petgraph/

It’s a visitor callback based graph traversal function, and of course it
should support breaking the traversal early.

One common solution is to use a boolean return value for the visitor closure.
`true` to continue, and `false` to break — or is it the other way around? At
this point in my Rust experience much prefer enums to booleans, they make the
code easier to both read and write.

Enums are one of Rust’s absolute best features, and we can quickly express
what we want. We allow the user to break with a value:

{% highlight rust %}
pub enum Control<B> {
    Continue,
    Break(B),
}
{% endhighlight %}

At the same time we don't want to burden the user with verbose enum imports
for the common case of never breaking at all. Traits to the rescue!

The default return type of a closure is `()`; this is returned if you just leave
off with a semicolon. Uniting both `Control<B>` and `()` with a common trait
will be really useful!

{% highlight rust %}
pub trait ControlFlow {
    fn continuing() -> Self;
    fn should_break(&self) -> bool;
}

impl ControlFlow for () {
    fn continuing() { }
    fn should_break(&self) -> bool { false }
}

impl<B> ControlFlow for Control<B> {
    fn continuing() -> Self { Control::Continue }
    fn should_break(&self) -> bool {
	if let Control::Break(_) = *self { true } else { false }
    }
}
{% endhighlight %}


The result is an API where the default case (no break) is seamlessly
available. Breaking with a value is opt-in, and what’s even better is
that the compiler knows statically whether it needs to check for
early return or not, which is the “pay for what you use” factor.

The finished graph traversal function is [`depth_first_search`][dfs].

[dfs]: https://docs.rs/petgraph/^0.4/petgraph/visit/fn.depth_first_search.html

As a bonus, we let errors break through as well (which makes some
algorithms [beautiful][bea]):

{% highlight rust %}
impl<E> ControlFlow for Result<(), E> {
    fn continuing() -> Self { Ok(()) }
    fn should_break(&self) -> bool {
        if let Err(_) = *self { true } else { false }
    }
}
{% endhighlight %}

[bea]: https://docs.rs/petgraph/0.4.0/src/petgraph/.cargo/registry/src/github.com-1ecc6299db9ec823/petgraph-0.4.0/src/algo.rs.html#154-165

In the usage site we use a macro similar to `try!()` to simplify the usage of
the `ControlFlow` trait.

{% highlight rust %}
/// Return if the expression is a break value.
macro_rules! try_control {
    ($e:expr) => {
        match $e {
            x => if x.should_break() {
                return x;
            }
        }
    }
}
{% endhighlight %}
