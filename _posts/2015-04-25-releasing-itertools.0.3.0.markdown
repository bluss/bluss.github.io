---
layout: post
title:  "Releasing itertools 0.3.0"
date:   2015-04-25 18:00:00
categories: rust
---

The [itertools crate][it] ([rust docs][docs]) has been updated to version 0.3.0. This version
is fully Rust 1.0-beta compatible.

Release highlights:

* Use **IntoIterator** where possible
* Revive the `izip!()` macro
* Remove deprecated methods

[it]: https://crates.io/crates/itertools
[docs]: http://bluss.github.io/rust-itertools/

**IntoIterator** makes everything so much more convenient. While you must still
explicitly convert a value to an iterator to call a method from the `Itertools`
trait, method and function arguments are now converted automatically, like the
arguments to `interleave` and to `equal` here:

{% highlight rust %}
use itertools::Itertools;

let it = (0..3).interleave(vec![7, 7]);
assert!(itertools::equal(it, vec![0, 7, 1, 7, 2]));
{% endhighlight %}

As a bonus the new function `itertools::equal`, computing iterator equality,
makes all the documentation examples easier to read.

The **izip!()** macro was revived: it now uses `IntoIterator` as well,
which was harder to do with `Zip::new(...)`. (I prefer to only implement
macros for what can't be solved in natural Rust.) [^1]

[^1]: It turns out it wasn't so hard, and itertools 0.3.1 uses IntoIterator in
      `Zip::new` too.

Version 0.3.0 finally removed **deprecated items**, which have the following
replacements:

- `.to_string_join()` → `.join()`
- `.drain()` → `.count()`
- `.apply()` → `.foreach()`
- `itertools::write` → `.set_from()`

A few features as of version 0.3.0 are available only in the nightly, and are
available under the optional cargo feature `unstable`. Unstable features
include the method `.enumerate_from()` and `ZipTrusted` (which may not be
needed anymore after improvements to regular `.zip()`'s optimization).

If I would give you a **general itertools feature pitch**, I would say it
provides some relatively simple but oft-wanted extensions to iterator
functionality:

- `.join(separator)` to produce a string from an iterator; for example 
  `[1, 2, 3].iter().join(", ")` produces a string like `"1, 2, 3"`.
- `.foreach(closure)` to consume an iterator and call a closure on each of the
  iterator elements. It is similar to what you can do with a `for` loop, but in
  method chaining style instead.
- Nice iterator adaptors like `.intersperse()`, `.interleave()` and `.zip_longest()`.

