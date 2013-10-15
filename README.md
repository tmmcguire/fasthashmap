fashhashmap
===========

Rust hashmap based on a faster hash function and Python dictionaries


HashMap Use
-----------

The implementation in this module currently supports the
[Container](http://static.rust-lang.org/doc/0.8/std/container/trait.Container.html),
[Mutable](http://static.rust-lang.org/doc/0.8/std/container/trait.Mutable.html),
[Map](http://static.rust-lang.org/doc/0.8/std/container/trait.Map.html),
and
[MutableMap](http://static.rust-lang.org/doc/0.8/std/container/trait.MutableMap.html)
traits.

### Constructing a HashMap ###

    let map = HashMap::new();

Constructs a map with a default capacity, currently 8. The map will
expand as necessary.

    let map = HashMap::with_capacity( 30 );

Constructs a map with a capacity of at least 30.


HashSet Use
-----------

The implementation in this module currently supports the
[Container](http://static.rust-lang.org/doc/0.8/std/container/trait.Container.html),
[Mutable](http://static.rust-lang.org/doc/0.8/std/container/trait.Mutable.html),
[Set](http://static.rust-lang.org/doc/0.8/std/container/trait.Set.html),
and
[MutableSet](http://static.rust-lang.org/doc/0.8/std/container/trait.MutableSet.html)
traits.

### Constructing a HashSet ###

    let set = HashSet::new();

Constructs a set with a default capacity, currently 8. The set will
expand as necessary.

Hashing Function
----------------

The hashing function currently used is DJB2,

    unsigned long
    hash(unsigned char *str)
    {
        unsigned long hash = 5381;
        int c;
        while (c = *str++)
            hash = (hash * 33) ^ c;
        return hash;
    }

It may not be the best, but it is short.

For more information, see http://cr.yp.to/cdb/cdb.txt and
http://www.cse.yorku.ca/~oz/hash.html.

Hashtable
---------

The hashtable code is loosely based on Python's dictionaries.

For more information on them, see

* [dictobject.c](http://hg.python.org/cpython/file/tip/Objects/dictobject.c)

* [dictobject.h](http://hg.python.org/cpython/file/tip/Include/dictobject.h)

* [dictnotes.txt](http://hg.python.org/cpython/file/tip/Objects/dictnotes.txt)

* [How are Python's Built In Dictionaries Implemented](http://stackoverflow.com/questions/327311/how-are-pythons-built-in-dictionaries-implemented). [StackOverflow]

* [Python dictionary implementation](http://www.laurentluce.com/posts/python-dictionary-implementation/)

* [Pure Python Dictionary Implementation](http://pybites.blogspot.com/2008/10/pure-python-dictionary-implementation.html)

