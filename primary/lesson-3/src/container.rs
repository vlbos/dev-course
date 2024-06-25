use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};
use std::rc::Rc;

use std::sync::{atomic, Arc};

pub fn container() {
    // vector
    let v = vec![1, 2, 3];
    // slice
    let r: &[i32] = &v[0..1];
    println!("slice as {:?}", r);

    // vector queue double end
    let mut v = VecDeque::new();
    v.push_back(0);
    v.push_front(0);

    // double linked list
    let mut l = LinkedList::new();
    l.push_back(0);

    // hash map
    let mut map = HashMap::<i32, i32>::new();
    map.insert(1, 2);
   let value =  map.get(&1).expect("the key not exist");

    // hash set
    let mut set = HashSet::new();
    set.insert(0);

    // binary map
    let mut b = BTreeMap::new();
    b.insert(0, 0);
    b.iter().for_each(|item|  println!("item is {:?}", item));

    // binary set
    let mut b = BTreeSet::new();
    b.insert(0);

    // binary heap
    let mut b = BinaryHeap::new();
    b.push(0);
    b.pop();

    // box in heap 1. big object, 2. unknown size
    let b = Box::new(0_i32);
    let c = b;

    // Rc contain reference count
    let r = Rc::new(0_i32);
    let s = r;

    // Cell contain a mutable value
    let c = Cell::new(0_i32);
    c.set(10);
    println!("cell is {:?}", c.get());
}
