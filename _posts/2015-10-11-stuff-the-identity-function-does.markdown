---
layout: post
title:  "Stuff the Identity Function Does (in Rust)"
date:   2015-10-11 14:00:00
categories: rust fun
---

The identity function looks like this in Rust:

{% highlight rust %}
/// The identity function.
fn id<T>(x: T) -> T { x }
{% endhighlight %}

`id` returns the same value that is passed in:
{% highlight rust %}
assert_eq!(1, id(1));
{% endhighlight %}


Beyond the obvious, it does some curious and fun things!

You can test [this blog post's code in the Rust Playground][gist].

[gist]: https://play.rust-lang.org/?gist=724e8c931a8e7515ef31&version=stable


## id Type Hints or Coerces

{% highlight rust %}
let string = "hi".to_string();
// Coerce a &String to &str, with id:
match id::<&str>(&string) {
    "hi" => {}
    _ => panic!("at the disco"),
}
{% endhighlight %}

No magic, it's just that you can specify with an explicit type *which* identity
function you are calling. If the expression can coerce to that type, then it compiles.

## id Forces References To Move

Let's say we have a simple recursive datastructure:
{% highlight rust %}
struct List {
    next: Option<Box<List>>,
}
{% endhighlight %}

And we want to walk it, with a mutable reference, through a loop.
{% highlight rust %}
impl List {
    fn walk_the_list(&mut self) {
        let mut current = self;
        loop {
            match current.next {
                None => return,
                Some(ref mut inner) => current = inner,
            }
        }
    }
}
{% endhighlight %}

Looks good? Rustc disagrees! [(compile in the playground)][gisterr]

<pre>
error: cannot borrow `current.next.0` as mutable more than once at a time [E0499]
          Some(ref mut inner) => current = inner,
               ^~~~~~~~~~~~~
</pre>

[gisterr]: https://play.rust-lang.org/?gist=613e13fd515bfca647ca&version=stable

It turns out Rust's mutable references do something interesting, and most of
the time it's very useful: when they are passed, they reborrow the local variable
rather than move it. The explicit equivalent of the reborrow would be `&mut *current`.

`id` tells a mutable reference to *move* instead of reborrow! This way it compiles:

{% highlight rust %}
impl List {
    fn walk_the_list(&mut self) {
        let mut current = self;
        loop {
            match id(current).next {
                None => return,
                Some(ref mut inner) => current = inner,
            }
        }
    }
}
{% endhighlight %}

This is a point where Rust could improve by learning to infer whether to
reborrow or move mutable references. Until then, we have `id`.


## id Makes Immutable Locals Mutable

`id` returns just the same thing as you pass in. Except it's now an rvalue, and
implicitly mutable.

{% highlight rust %}
impl List {
    fn consume_the_list(self) {
        // error: cannot borrow immutable local variable `self` as mutable
        // self.walk_the_list();
        
        id(self).walk_the_list();
    }
}
{% endhighlight %}

This is no violation of Rust philosophy. Using `mut` on locals is simple and
pragmatic, and mutability radiates from the owner. If your value is now
a temporary, it's not owned by an immutable binding anymore (or any other
variable binding).

## Rust has Dedicated Syntax for This!

If you thought that was cryptic, here's one better. The secret syntax is just
`{` and its companion `}`, and it allows you to manipulate move semantics just
the same way `id` does:

{% highlight rust %}
impl List {
    fn walk_the_list_with_braces(&mut self) {
        let mut current = self;
        loop {
            match {current}.next {
                None => return,
                Some(ref mut inner) => current = inner,
            }
        }
    }
    
    fn consume_the_list_with_braces(self) {
        {self}.walk_the_list_with_braces();
    }
}
{% endhighlight %}


## Force a Move Today

If you actually use this, I think `moving` is actually a pretty good name
(`move` is taken, it's a keyword). Save the `{}` for obfuscation contests.

{% highlight rust %}
/// The identity function.
///
/// Also forces the argument to move.
fn moving<T>(x: T) -> T { x }
{% endhighlight %}


## Epilogue

It's February 2017 and this is hilarious:

Calling `id::<&T>()`, since it's a passing a shared reference, inserts
a `noalias` annotation for the pointer, which might otherwise not have been
there! As soon as llvm's metadata propagation improves, it might even have
actual use.

