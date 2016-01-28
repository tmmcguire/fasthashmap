// Copyright 2013 Tommy M. McGuire
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::borrow::Borrow;
use std::hash::{Hasher,Hash};
use std::slice::Iter;

// Simple implementation of the DJB hash
// (See http://cr.yp.to/cdb/cdb.txt and http://www.cse.yorku.ca/~oz/hash.html)
pub struct DJBHasher {
    hash: u64
}

impl DJBHasher {
    #[inline]
    pub fn new() -> DJBHasher {
        DJBHasher { hash: 5381u64 }
    }
}

impl Hasher for DJBHasher {
    fn finish(&self) -> u64 {
        self.hash
    }

    fn write(&mut self, bytes: &[u8]) {
        for i in 0..bytes.len() {
            self.hash = (33u64 * self.hash) ^ bytes[i] as u64
        }
    }
}

/* ----------------------------------------------- */

// This is an implementation of a Rust HashMap based on the DJB hash and
// Python's dictionaries. See:
//
// * http://stackoverflow.com/questions/327311/how-are-pythons-built-in-dictionaries-implemented
//   (especially the links in the first answer),
//
// * http://www.laurentluce.com/posts/python-dictionary-implementation/
//
// * http://pybites.blogspot.com/2008/10/pure-python-dictionary-implementation.html

static PERTURB_SHIFT: usize = 5;

#[derive(Clone)]
pub enum Entry<K,V> {
    Empty,                      // This slot is empty
    Full(K,V,u64),              // This slot is holding a key and value
    Ghost(K,u64),               // This slot once held key k
}

impl<K, V> Entry<K,V> {
    #[inline]
    #[allow(dead_code)]         // For completeness; this fn isn't used here
    pub fn is_empty(&self) -> bool {
        match *self {
            Entry::Empty => true,
            _ => false
        }
    }

    #[inline]
    pub fn is_full(&self)  -> bool {
        match *self {
            Entry::Full(..) => true,
            _ => false
        }
    }

    #[inline]
    pub fn is_ghost(&self) -> bool {
        match *self {
            Entry::Ghost(..) => true,
            _ => false
        }
    }

    #[inline]
    pub fn matches<Q: PartialEq<K>>(&self, key: &Q, hash: u64) -> bool {
        match *self {
            Entry::Empty                                      => true,
            Entry::Full(ref k, _, h) | Entry::Ghost(ref k, h) => hash == h && key == k,
        }
    }

    #[inline]
    pub fn key(&self) -> Option<&K> {
        match *self {
            Entry::Full(ref k,_,_) => Some(k),
            _ => None
        }
    }

    #[inline]
    pub fn value<'l>(&'l self) -> Option<&'l V> {
        match *self {
            Entry::Full(_,ref v,_) => Some(v),
            _ => None
        }
    }
}

pub struct HashMap<K,V> {
    table     : Vec<Entry<K,V>>,
    capacity  : usize,
    mask      : u64,
    length    : usize,
    ghosts    : usize,
}

impl<K,V> HashMap<K,V> where K: Hash + Eq {

    #[inline]
    pub fn new() -> HashMap<K,V> {
        HashMap::with_capacity(8)
    }

    #[inline]
    pub fn with_capacity(sz: usize) -> HashMap<K,V> {
        let capacity = usize::next_power_of_two(sz);
        let mut table = Vec::with_capacity(capacity);
        for _ in 0..capacity { table.push(Entry::Empty); }
        HashMap {
            table   : table,
            capacity: capacity,
            mask    : (capacity as u64) - 1,
            length  : 0,
            ghosts  : 0,
        }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    // This algorithm gleefully stolen from Python
    #[inline]
    fn probe(&self, key: &K, hash: u64) -> usize {
        let mut shifted_hash = hash;
        let mut free         = None;
        let mut i            = (shifted_hash & self.mask) as usize;
        while !self.table[i].matches(key,hash) {
            if free.is_none() && self.table[i].is_ghost() { free = Some(i); }
            i = (((5 * i as u64) + 1 + shifted_hash) & self.mask) as usize;
            shifted_hash = shifted_hash >> PERTURB_SHIFT;
        }
        if self.table[i].is_full() || free.is_none() {
            i
        } else {
            free.unwrap()
        }
    }

    #[inline]
    fn probe_equiv<Q:PartialEq<K>>(&self, key: &Q, hash: u64) -> usize {
        let mut shifted_hash = hash;
        let mut free         = None;
        let mut i            = (shifted_hash & self.mask) as usize;
        while !self.table[i].matches(key,hash) {
            if free.is_none() && self.table[i].is_ghost() { free = Some(i); }
            i = (((5 * i as u64) + 1 + shifted_hash) & self.mask) as usize;
            shifted_hash = shifted_hash >> PERTURB_SHIFT;
        }
        if self.table[i].is_full() || free.is_none() {
            i
        } else {
            free.unwrap()
        }
    }

    // Precondition: this is used by expand, so there must be enough space in the table.
    #[inline]
    fn swap_with_hash(&mut self, key: K, hash: u64, value: V) -> Option<V> {
        let i = self.probe(&key, hash);
        let mut elt = &mut self.table[i];
        match elt {
            &mut Entry::Empty => {
                let f = Entry::Full(key,value,hash);
                std::mem::replace(elt, f);
                self.length += 1;
                None
            },
            &mut Entry::Ghost(..) => {
                let f = Entry::Full(key,value,hash);
                std::mem::replace(elt, f);
                self.length += 1;
                self.ghosts -= 1;
                None
            },
            &mut Entry::Full(_,ref mut v, _) => {
                Some( std::mem::replace(v, value) )
            },
        }
    }

    #[inline]
    fn do_expand(&mut self, new_capacity: usize) {
        let mut new_tbl = HashMap::with_capacity( new_capacity );
        for i in 0..self.table.len() {
            match std::mem::replace(&mut self.table[i], Entry::Empty) {
                Entry::Full(k,v,h)               => { new_tbl.swap_with_hash(k,h,v); }
                Entry::Empty | Entry::Ghost(..)  => { }
            }
        }
        // Copy new table's elements into self.  Note: attempting
        // to do this more directly causes: "use of partially moved
        // value"
        let cap    = new_tbl.capacity;
        let mask   = new_tbl.mask;
        let len    = new_tbl.length;
        let ghosts = new_tbl.ghosts;
        self.table    = new_tbl.table;
        self.capacity = cap;
        self.mask     = mask;
        self.length   = len;
        self.ghosts   = ghosts;
    }

    #[inline]
    fn expand(&mut self) {
        let capacity = self.capacity;
        if self.length * 3 > capacity * 2 {
            // Expand table if live entries nearing capacity
            self.do_expand( capacity * 2 );
        } else if (self.length + self.ghosts) * 3 >= capacity * 2 {
            // Rehash to flush out excess ghosts
            self.do_expand( capacity );
        }
    }

    #[inline]
    pub fn find_equiv<'a, Q>(&'a self, k: &Q) -> Option<&'a V> where Q: Hash + PartialEq<K> {
        let mut hasher = DJBHasher::new();
        k.hash(&mut hasher);
        let hash = hasher.finish();
        let i = self.probe_equiv(k, hash);
        match &self.table[i] {
            &Entry::Empty | &Entry::Ghost(..) => None,
            &Entry::Full(_, ref val, _)       => Some(val),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.length
    }

    #[inline]
    pub fn clear(&mut self) {
        for i in 0..self.table.len() {
            self.table[i] = Entry::Empty;
        }
        self.length = 0;
        self.ghosts = 0;
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        self.expand();
        let mut hasher = DJBHasher::new();
        k.hash(&mut hasher);
        let hash = hasher.finish();
        self.swap_with_hash(k, hash, v)
    }

    pub fn get<Q>(&self, k: &Q) -> Option<&V> where K: Borrow<Q>, Q: Hash + PartialEq<K> {
        self.find_equiv(k)
    }

    pub fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut V> where K: Borrow<Q>, Q: Hash + PartialEq<K> {
        let mut hasher = DJBHasher::new();
        k.hash(&mut hasher);
        let hash = hasher.finish();
        let i = self.probe_equiv(k, hash);
        let mut elt = &mut self.table[i];
        match elt {
            &mut Entry::Empty | &mut Entry::Ghost(..) => None,
            &mut Entry::Full(_,ref mut v, _) => Some(v),
        }
    }

    fn contains_key<Q>(&self, k: &Q) -> bool where K: Borrow<Q>, Q: Hash + PartialEq<K> {
        self.get(k).is_some()
    }

    pub fn iter(&self) -> HashMapIter<K,V> {
        HashMapIter { inner: self.table.iter() }
    }

    pub fn keys(&self) -> HashMapKeys<K,V> {
        HashMapKeys { inner: self.iter() }
    }
}

// ----------------------------------------

pub struct HashMapIter<'l,K: 'l,V: 'l> {
    inner: Iter<'l,Entry<K,V>>,
}

impl<'l,K,V> Iterator for HashMapIter<'l,K,V> {
    type Item = (&'l K, &'l V);
    fn next(&mut self) -> Option<(&'l K, &'l V)> {
        let mut n = self.inner.next();
        loop {
            match n {
                Some(entry) if entry.is_full() => {
                    return Some((entry.key().unwrap(), entry.value().unwrap()))
                }
                Some(..) => {
                    n = self.inner.next();
                }
                None => {
                    return None;
                }
            }
        }
    }
}

pub struct HashMapKeys<'l,K: 'l,V: 'l> {
    inner: HashMapIter<'l,K,V>,
}

impl<'l,K,V> Iterator for HashMapKeys<'l,K,V> {
    type Item = &'l K;
    fn next(&mut self) -> Option<&'l K> {
        match self.inner.next() {
            Some((ref k, _)) => Some(k),
            None => None
        }
    }
}

/* ----------------------------------------------- */

pub struct HashSet<T> {
    map: HashMap<T,()>
}

impl<T> HashSet<T> where T: Hash + Eq {

    #[inline]
    pub fn new() -> HashSet<T> {
        HashSet { map: HashMap::new() }
    }

    #[inline]
    pub fn insert(&mut self, v: T) -> bool {
        match self.map.insert(v, ()) {
            Some(_) => false,
            None    => true,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    #[inline]
    pub fn iter(&self) -> HashMapKeys<T,()> {
        self.map.keys()
    }

    #[inline]
    pub fn contains<Q>(&self, v: &Q) -> bool where T: Borrow<Q>, Q: Hash + PartialEq<T> {
        self.map.contains_key(v)
    }

    #[inline]
    pub fn is_disjoint(&self, other: &HashSet<T>) -> bool {
        for elt in self.map.table.iter() {
            match *elt {
                Entry::Full(ref k,_,_) => { if other.contains(k) { return false; } },
                _ => { },
            }
        }
        return true;
    }

    #[inline]
    pub fn is_subset(&self, other: &HashSet<T>) -> bool {
        for elt in self.map.table.iter() {
            match *elt {
                Entry::Full(ref k,_,_) => { if !other.contains(k) { return false; } },
                _ => { },
            }
        }
        return true;
    }

    #[inline]
    pub fn is_superset(&self, other: &HashSet<T>) -> bool { other.is_subset(self) }
}

/* ----------------------------------------------- */

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::{BufRead,BufReader};

    use super::{HashMap,HashSet};

    #[allow(dead_code)]         // Used by benchmarking code.
    fn get_words() -> Vec<String> {
        let file = File::open("/usr/share/dict/words").expect("cannot read words");
        let buffered_file = BufReader::new(file);
        buffered_file.lines().map(|l| l.unwrap()).collect()
    }

    #[test]
    fn test_empty() {
        let m: HashMap<usize,usize> = HashMap::new();
        assert_eq!(m.len(), 0);
        assert_eq!(m.capacity(), 8);
        assert_eq!(m.get(&1), None);

        let mut count = 0;
        for (_,_) in m.iter() { count += 1; }
        assert_eq!(count, 0);
    }

    #[test]
    fn test_one() {
        let mut m: HashMap<usize,usize> = HashMap::new();
        assert_eq!(m.len(), 0);
        assert_eq!(m.insert(1,400), None);
        assert_eq!(m.len(), 1);
        assert_eq!(m.capacity(), 8);
        match m.get(&1) {
            Some(y) => assert_eq!(*y,400),
            None => panic!("panicure!")
        }
        match m.get_mut(&1) {
            Some(y) => *y = 500,
            _ => panic!("paniced!")
        }
        match m.get(&1) {
            Some(y) => assert_eq!(*y,500),
            None => panic!("panicure again!")
        }

        let mut count = 0;
        for (_,_) in m.iter() { count += 1; }
        assert_eq!(count, 1);
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn test_eight() {
        let mut m: HashMap<usize,usize> = HashMap::new();
        let v = [1,3,5,7,9,11,13,15];
        for i in v.iter() {
            assert_eq!(m.insert(*i,100 * *i), None);
        }
        assert_eq!(m.len(),8);
        assert_eq!(m.capacity(),16);
        assert_eq!(m.insert(3, 12000), Some(300));

        let mut count = 0;
        for (_,_) in m.iter() { count += 1; }
        assert_eq!(count, 8);
    }

    #[test]
    fn test_set_empty() {
        let s: HashSet<usize> = HashSet::new();
        assert_eq!(s.len(), 0);
        assert!(!s.contains(&3));
        let mut count = 0;
        for _ in s.iter() { count += 1; }
        assert_eq!(count, 0);
    }

    #[test]
    fn test_set_nonempty() {
        let mut s: HashSet<usize> = HashSet::new();
        let v = [1,3,5,7,9,11,13,15];
        for i in v.iter() {
            assert!(s.insert(*i));
        }
        assert_eq!(s.len(), 8);
        assert!(s.contains(&3));
        let mut count = 0;
        for _ in s.iter() { count += 1; }
        assert_eq!(count, 8);
        let empty: HashSet<usize> = HashSet::new();
        assert!( s.is_disjoint(&empty) );
        assert!( empty.is_subset(&s) );
        assert!( s.is_superset(&empty) );
    }

    // #[bench]
    // fn hash_bench_siphash(b: &mut test::Bencher) {
    //     let s = "abcdefghijklmnopqrstuvwxyz";
    //     b.iter( || { s.hash(); } );
    // }

    // #[bench]
    // fn hashmap_bench_stdlib(b: &mut extra::test::BenchHarness) {
    //     let list = ["abashed", "acrid", "dachshund's", "hackle", "zigzagging"];
    //     b.iter( || {
    //         let mut m = std::hashmap::HashMap::new();
    //         for w in list.iter() { m.insert(w, 27); }
    //     } );
    // }

    // #[bench]
    // fn hash_bench_djbhash(b: &mut extra::test::BenchHarness) {
    //     let s = "abcdefghijklmnopqrstuvwxyz";
    //     b.iter( || { DJBState::djbhash(&s); } );
    // }

    // #[bench]
    // fn hashmap_bench_fasthashmap(b: &mut extra::test::BenchHarness) {
    //     let list = ["abashed", "acrid", "dachshund's", "hackle", "zigzagging"];
    //     b.iter( || {
    //         let mut m = HashMap::new();
    //         for w in list.iter() { m.insert(w, 27); }
    //     } );
    // }

    // #[bench]
    // fn big_hashmap_bench_fasthashmap(b: &mut extra::test::BenchHarness) {
    //     let words = get_words();
    //     b.iter( || {
    //         let mut m = HashMap::new(); // with_capacity(2 * words.len());
    //         for w in words.iter() { m.insert(w, 27); }
    //     } );
    // }

    // #[bench]
    // fn big_hashmap_bench_siphashmap(b: &mut extra::test::BenchHarness) {
    //     let words = get_words();
    //     b.iter( || {
    //         let mut m = std::hashmap::HashMap::new(); // with_capacity(2 * words.len());
    //         for w in words.iter() { m.insert(w, 27); }
    //     } );
    // }

}
