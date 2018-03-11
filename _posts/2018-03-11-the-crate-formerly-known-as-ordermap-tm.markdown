---
layout: post
title:  "indexmap 1.0"
date:   2018-03-11 13:20:00 +0100
categories: rust
---

[**indexmap**][1] is a data structure library for Rust that implements a
regular hash table with some extra features.

It has an interface that is drop-in compatible with the map and set types from
the Rust standard library. The extra features it implements are foremost:

- The iteration order is deterministic and independent of the hash function used
- Limited element order stability. Stable over insertion, extension,
  de/serialization, through `retain` and many other operations.
- Look up elements by either their key or their numerical index
- Sort the key-value pairs in a map or the elements in a set by arbitrary
  comparison function. The key lookup is preserved of course, while changing
  the order of the elements in the data structure.

The crate is implemented in entirely safe Rust code and the 1.0 version
requires Rust 1.18 as of this writing.

[1]: https://docs.rs/indexmap/1/


## Why the Data Structure Indexmap?

It started as a fun experiment, after reading about Python 3.6's dict implementation;
they made their hash tables preserve full insertion order by default, using
a simple structure. (They released Python 3.6 with that improvement but with no
comittment to the insertion order as a guaranteed feature).

Indexmap turned out to be something a bit different than its inspiration. Its
compactly indexed space was very useful for many applications, and its
deterministic order behaviour was delightful in almost every situation a hash
table was needed.

## Why Safe Rust?

Safe Rust is a good place to start, and if you are lucky, you never have
to leave safety.  We are working with a data structure that was easy to build
on top of a simpler one (the `Vec`), so we didn't need to break out the `unsafe
{ ... }` blocks.

Safe-only isn't that important, but it feels good to be able
to demonstrate the power of safe rust in a small but useful, and fast, library.

## Who Is Indexmap?

I was the initial author and indexmap is maintained by me ([bluss on github][bluss]),
and Josh Stone ([cuviper on github][cuviper]). We have also had some great contributions
during the relatively small development of the crate (small means 36 merged pull requests,
out of which 17 are not by me).

Thanks to [all contributors][contrib] of indexmap which are:

bluss, cuviper, ignatenkobrain, xfix, overvenus, stevej, pczarn, lucab

[bluss]: https://github.com/bluss
[cuviper]: https://github.com/cuviper/
[contrib]: https://github.com/bluss/indexmap/graphs/contributors

(Thank  you @pczarn who implemented a good performance improvement for insertions!)

On the way, there has been contributions that have not yet been merged (there
might be some awesome features to come).  There has also been other proposals
where we decided they are not in scope for indexmap: a hash table that is
similar but that preserves insertion order acorss removal. It's a useful
project, but it will not be this one. I have encouraged a fork or restart to
work on that.

## Why the Name Indexmap?

The initial name for the crate was `orderedmap`. For a short while. That was
a bit misleading, so let's be clever: I used the name `ordermap` (It has
*an* order.., it has useful features around order).

Clever doesn't help us in the long run: we want to deliver the right
expectations. We have no perfect name, but we wanted to avoid over promising
on order. It has a lot of useful order properties (stable, deterministic, sortable,
indexable), but it has other properties too. One of the features is that it
has just the same interface as a regular hash map, so you can just drop it in.

It now has index in the name, to suggest some of the useful indexing features,
and it has “map” in the name to suggest that it's a hash table, or at least
something like it.


## Other Notes

+ See the [changelog][ch] for more information

[ch]: https://github.com/bluss/indexmap#recent-changes
