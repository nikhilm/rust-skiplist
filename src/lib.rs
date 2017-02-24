extern crate rand;

use std::ptr;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt::Debug;

pub struct List<T> {
    head: Links<T>,
    level: usize,
    // TODO: Not really required.
    // tail: *mut Node<T>,
}

// A SkipList Links has levels instead of just one next pointer. Hence the Vec. If there is no Link
// at a certain level, it is set to None. Links always start with [0] set to None instead of having
// an empty vector. This means the lowest level is always initialized, which we need any way, and
// it helps avoid another level of checks.
// TODO: Measure perf of this pre-alloc vs starting from empty.
// Since multiple links can point to the same node, we use Rc<RefCell<>>. The RefCell is needed
// because we intend to mutate the underlying node (to change its Links for example), while
// multiple Rcs are referencing it.
// Due to our Node/Link distinction, we have no ability, given a Link(s), to see the value of the
// Node containing that Link(s), i.e. the value of the "current" node.
type Link<T> = Option<Rc<RefCell<Node<T>>>>;
type Links<T> = Vec<Link<T>>;

struct Node<T> {
    elem: T,
    next: Links<T>,
}

pub struct IntoIter<T>(List<T>);

pub struct Iter<'a, T:'a> {
    next: Option<&'a Node<T>>,
}

pub struct IterMut<'a, T: 'a> {
    next: Option<&'a mut Node<T>>,
}

fn pickLevel(compLevel: usize) -> usize {
    let mut level = 0;
    // Probability 1/2 is captured by boolean random.
    // We don't have an upper limit on level, something to do later.
    while rand::random() {
        level += 1;
        if level > compLevel {
            break;
        }
    }

    level
}

// In a SkipList, level represents the last safely indexable position, not the one just beyond it.
// So it is len(Links<>) - 1.
impl<T> List<T> where T: PartialOrd {
    pub fn new() -> Self {
        List { head: vec![None], level: 0/*, tail: ptr::null_mut() */ }
    }

    pub fn insert(&mut self, elem: T) -> bool where T: Debug + Copy {
        // Oh god! Because of "update is a vec of links we need to go back and rewire",
        // we may need Links to be Rc<RefCell<>> too. Or update could be a list of Nodes since we
        // know they are always valid.
        println!("Inserting {:?}. Current level {}", elem, self.level);
        let mut newNode = Rc::new(RefCell::new(Node{ elem: elem, next: vec![None] }));
        // What we want to say is, at level i, we are going to change a certain Links[i] to point
        // to something else. We need to store a ref to a mutable Links here, while at the same
        // time, a node somewhere (or the list in the case of head) owns the Link.
        let mut update: Vec<&mut Links<T>> = Vec::new();

        // x is a reference to the Links we want to "insert after", so we keep changing it to point
        // to the right Links. Not sure if there is a more idiomatic way to do this.
        let mut x: &mut Links<T> = &mut self.head;
        for i in (0..(self.level+1)).rev() {
            println!("i = {}", i);
            // TODO: May be a more idiomatic way to do this.
            while x[i].is_some() {
                {
                    let val = x[i].as_ref().unwrap().borrow();
                    if val.elem >= elem {
                        break;
                    }
                }
                // Move to the next Links.
                x = &mut x[i].clone().unwrap().borrow_mut().next;
                // Gah! How do we tell the compiler that all these links actually stay alive for
                // the lifetime of this function?
                // Now x outlives the lifetime of x.next, or so Rust thinks. What we want to say
                // is, the lifetimes are actually the lifetime of the list.
            }

            // TODO: Backfill update[];
            update.push(&mut x);
        }

        if x[0].is_some() && x[0].as_ref().unwrap().borrow().elem == elem {
            // No dupes.
            return false;
        }

        let level = pickLevel(self.level);
        println!("New level {}", self.level);
        if level > self.level {
            // All the updates for higher levels are the head node, with pointers to the new node
            // later
            // TODO: Backfill update[]
            
            // We need a little utility method that takes a Links and resizes it to accomodate upto
            // level.
            
            self.level = level;
        }

        for i in 0..(self.level+1) {
            // We may need to mem::swap/mem::replace.
            newNode.borrow_mut().next[i] = update[i][i].clone();
            update[i][i] = Some(newNode.clone());
        }

        return true;
    }

    // TODO Get rid of this.
    /*pub fn push(&mut self, elem: T) {
        let mut new_tail = Box::new(Node {
            elem: elem,
            next: vec![None],
        });

        let raw_tail: *mut _ = &mut *new_tail;

        if !self.tail.is_null() {
            unsafe {
                (*self.tail).next = Some(new_tail);
            }
        } else {
            self.head = Some(new_tail);
        }

        self.tail = raw_tail;
    }

    // TODO Get rid of this.
    pub fn pop(&mut self) -> Option<T> {
        self.head.take().map(|head| {
            let head = *head;
            self.head = head.next;

            if self.head.is_none() {
                self.tail = ptr::null_mut();
            }

            head.elem
        })
    }

    // TODO Get rid of this.
    pub fn peek(&self) -> Option<&T> {
        self.head.as_ref().map(|node| {
            &node.elem
        })
    }

    // TODO Get rid of this.
    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.head.as_mut().map(|node| {
            &mut node.elem
        })
    }*/

    //pub fn into_iter(self) -> IntoIter<T> {
    //    IntoIter(self)
    //}

    //pub fn iter(&self) -> Iter<T> {
    //    Iter { next: self.head[0].as_ref().map(|node| &**node) }
    //}

    //pub fn iter_mut(&mut self) -> IterMut<T> {
    //    IterMut { next: self.head[0].as_mut().map(|node| &mut **node) }
    //}
}


//impl<T> Drop for List<T> {
//    fn drop(&mut self) {
//        // TODO
//        let mut cur_link = self.head[0].take();
//        while let Some(mut boxed_node) = cur_link {
//            cur_link = boxed_node.next[0].take();
//        }
//    }
//}


/*impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}*/

//impl<'a, T> Iterator for Iter<'a, T> {
//    type Item = &'a T;
//
//    fn next(&mut self) -> Option<Self::Item> {
//        self.next.map(|node| {
//            self.next = node.next[0].as_ref().map(|node| &**node);
//            &node.elem
//        })
//    }
//}
//
//impl<'a, T> Iterator for IterMut<'a, T> {
//    type Item = &'a mut T;
//
//    fn next(&mut self) -> Option<Self::Item> {
//        self.next.take().map(|node| {
//            self.next = node.next[0].as_mut().map(|node| &mut **node);
//            &mut node.elem
//        })
//    }
//}



#[cfg(test)]
mod test {
    use super::List;
    /*
    #[test]
    fn basics() {
        let mut list = List::new();

        // Check empty list behaves right
        assert_eq!(list.pop(), None);

        // Populate list
        list.push(1);
        list.push(2);
        list.push(3);

        // Check normal removal
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), Some(2));

        // Push some more just to make sure nothing's corrupted
        list.push(4);
        list.push(5);

        // Check normal removal
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), None);

        // Check the exhaustion case fixed the pointer right
        list.push(6);
        list.push(7);

        // Check normal removal
        assert_eq!(list.pop(), Some(6));
        assert_eq!(list.pop(), Some(7));
        assert_eq!(list.pop(), None);
    }

    #[test]
    fn into_iter() {
        let mut list = List::new();
        list.push(1); list.push(2); list.push(3);

        let mut iter = list.into_iter();
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter() {
        let mut list = List::new();
        list.push(1); list.push(2); list.push(3);

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_mut() {
        let mut list = List::new();
        list.push(1); list.push(2); list.push(3);

        let mut iter = list.iter_mut();
        assert_eq!(iter.next(), Some(&mut 1));
        assert_eq!(iter.next(), Some(&mut 2));
        assert_eq!(iter.next(), Some(&mut 3));
        assert_eq!(iter.next(), None);
    }*/
}

