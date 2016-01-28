fasthashmap
===========

Rust hashmap based on a faster hash function and Python dictionaries.


WARNING
-------

The hash function used by this map is not cryptographically
strong. This map is open to
[denial of service attacks](http://events.ccc.de/congress/2011/Fahrplan/events/4680.en.html).

CAUTION IS ADVISED.

HashMap Use
-----------

The implementation in this module currently supports many of the same methods as
std::collections::HashMap, but not all.

### Constructing a HashMap ###

    let map = HashMap::new();

Constructs a map with a default capacity, currently 8. The map will
expand as necessary.

    let map = HashMap::with_capacity( 30 );

Constructs a map with a capacity of at least 30.


HashSet Use
-----------

The implementation in this module currently supports many of the same methods as
std::collections::HashSet, but not all.

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

The hashtable code is loosely based on Python's dictionaries. Since the HashMap
implementation in the Rust standard library has been updated to be Robin Hood
Hashing (i.e. bucket stealing), this implementation may not be faster than the
standard libraries', although using the DJB2 hash function makes it likely.

It should be possible to mix-n-match hashing function implementations, though,
so it might be worth experimenting further. Soon.

For more information on this implementation, see

* [dictobject.c](http://hg.python.org/cpython/file/tip/Objects/dictobject.c)

* [dictobject.h](http://hg.python.org/cpython/file/tip/Include/dictobject.h)

* [dictnotes.txt](http://hg.python.org/cpython/file/tip/Objects/dictnotes.txt)

* [How are Python's Built In Dictionaries Implemented](http://stackoverflow.com/questions/327311/how-are-pythons-built-in-dictionaries-implemented). [StackOverflow]

* [Python dictionary implementation](http://www.laurentluce.com/posts/python-dictionary-implementation/)

* [Pure Python Dictionary Implementation](http://pybites.blogspot.com/2008/10/pure-python-dictionary-implementation.html)

For information on Robin Hood hashing, see the papers/articles mentioned in the
Rust docs:

* Pedro Celis. ["Robin Hood Hashing"](https://cs.uwaterloo.ca/research/tr/1986/CS-86-14.pdf).
* Emmanuel Goossaert. ["Robin Hood hashing"](http://codecapsule.com/2013/11/11/robin-hood-hashing/).
* Emmanuel Goossaert. ["Robin Hood hashing: backward shift deletion"](http://codecapsule.com/2013/11/17/robin-hood-hashing-backward-shift-deletion/).

Name
----

Yes, I had to change the repo name from 'fashhashmap'. This is because I can neither type nor see, apparently. Sorry.
