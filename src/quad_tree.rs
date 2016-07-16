use std::iter;
use std::collections::linked_list;
use std::collections::LinkedList;

struct QuadTree<T> {
  value: T,
  children: Option<Box<[QuadTree<T>; 4]>>,
}

struct QuadTreeIterEntry<'a, T: 'a> {
  node: &'a QuadTree<T>,
  childIndex: usize,
}

struct QuadTreeIter<'a, T: 'a> {
  visiting: LinkedList<QuadTreeIterEntry<'a, T>>,
  visiting_back: LinkedList<QuadTreeIterEntry<'a, T>>,
}

struct QuadTreeIterMutEntry<'a, T: 'a> {
  node: &'a mut QuadTree<T>,
  childIndex: usize,
}

struct QuadTreeIterMut<'a, T: 'a> {
  // visit_push: &'a Fn(&'a mut QuadTree<T>),
  // visit_pop: &'a Fn(),
  // visit_push_back: &'a Fn(&'a mut QuadTree<T>),
  // visit_pop_back: &'a Fn(),
  visiting: LinkedList<QuadTreeIterMutEntry<'a, T>>,
  visiting_back: LinkedList<QuadTreeIterMutEntry<'a, T>>,
}

struct QuadTreeRefStack<'a, T: 'a> {
  list: LinkedList<&'a mut T>,
}

// struct QuadTreeRefStackIter<'a, T: 'a> {
//   iter: LinkedList::Iter<&'a mut T>,
// }
//
// struct QuadTreeRefStackIterMut<'a, T: 'a> {
//   iter: LinkedList::IterMut<&'a mut T>,
// }

struct QuadTreeIterMutStack<'a, T: 'a> {
  // bound_visit_push: Box<Fn(&'a mut QuadTree<T>)>,
  // bound_visit_pop: Box<Fn()>,
  // bound_visit_push_back: Box<Fn(&'a mut QuadTree<T>)>,
  // bound_visit_pop_back: Box<Fn()>,
  first: bool,
  first_back: bool,
  visiting_stack: QuadTreeRefStack<'a, T>,
  visiting_back_stack: QuadTreeRefStack<'a, T>,
  visiting: LinkedList<QuadTreeIterMutEntry<'a, T>>,
  visiting_back: LinkedList<QuadTreeIterMutEntry<'a, T>>,
  // iter: QuadTreeIterMut<'a, T>,
}

impl<T> QuadTree<T> {
  fn new(value: T) -> QuadTree<T> {
    QuadTree::<T> {
      value: value,
      children: None,
    }
  }

  fn split(&mut self) {}

  fn join(&mut self) {}

  fn iter(&self) -> QuadTreeIter<T> {
    QuadTreeIter::<T>::new(self)
  }

  fn iter_mut(&mut self) -> QuadTreeIterMut<T> {
    QuadTreeIterMut::<T>::new(self)
  }

  fn iter_stack_mut(&mut self) -> QuadTreeIterMutStack<T> {
    QuadTreeIterMutStack::<T>::new(self)
  }
}

impl<'a, T> QuadTreeIter<'a, T> {
  fn new(node: &'a QuadTree<T>) -> QuadTreeIter<'a, T> {
    let mut iter = QuadTreeIter::<T> {
      visiting: LinkedList::<QuadTreeIterEntry<T>>::new(),
      visiting_back: LinkedList::<QuadTreeIterEntry<T>>::new(),
    };
    iter.push_node(node);
    iter.push_node_back(node);
    iter
  }

  fn push_node(&mut self, node: &'a QuadTree<T>) {
    let childIndex = match node.children {
      Some(_) => {0},
      _ => {4},
    };
    self.visiting.push_back(QuadTreeIterEntry::<T> {
      node: node,
      childIndex: childIndex,
    });
    self.step_down();
  }

  fn push_node_back(&mut self, node: &'a QuadTree<T>) {
    let childIndex = match node.children {
      Some(_) => {0},
      _ => {4},
    };
    self.visiting_back.push_back(QuadTreeIterEntry::<T> {
      node: node,
      childIndex: childIndex,
    });
  }

  fn step_down(&mut self) {
    let maybeNode = match self.visiting.back_mut() {
      Some(ref mut entry) => {
        if entry.childIndex < 4 {
          match entry.node.children {
            Some(ref children) => {
              let result = Some(&children[entry.childIndex]);
              entry.childIndex += 1;
              result
            },
            _ => {
              entry.childIndex = 4;
              None
            },
          }
        }
        else {
          None
        }
      },
      _ => {None},
    };
    match maybeNode {
      Some(node) => {
        self.push_node(node);
      },
      _ => {},
    }
  }

  fn step_back_down(&mut self) {
    let maybeNode = match self.visiting_back.back_mut() {
      Some(ref mut entry) => {
        if entry.childIndex < 4 {
          match entry.node.children {
            Some(ref children) => {
              let result = Some(&children[3 - entry.childIndex]);
              entry.childIndex += 1;
              result
            },
            _ => {
              entry.childIndex = 4;
              None
            },
          }
        }
        else {
          None
        }
      },
      _ => {None},
    };
    match maybeNode {
      Some(node) => {
        self.push_node_back(node);
      },
      _ => {
        self.step_back_up();
      },
    }
  }

  fn step_up(&mut self) {
    self.visiting.pop_back();
    if self.visiting.len() > 0 {
      self.step_down();
    }
  }

  fn step_back_up(&mut self) {
    loop {
      self.visiting_back.pop_back();
      match self.visiting_back.back() {
        Some(ref entry) => {
          if entry.childIndex < 4 {
            break;
          }
        },
        _ => {break;},
      }
    }
    if self.visiting_back.len() > 0 {
      self.step_back_down();
    }
  }

  fn check_end(&mut self) {
    let is_end = match self.visiting.back() {
      Some(ref front_entry) => {
        match self.visiting_back.back() {
          Some(ref back_entry) => {
            unsafe {
              let front_ptr = &*front_entry.node as *const QuadTree<T>;
              let back_ptr = &*back_entry.node as *const QuadTree<T>;
              front_ptr == back_ptr
            }
          },
          _ => {false},
        }
      },
      _ => {false},
    };
    if is_end {
      while self.visiting.len() > 0 {
        self.visiting.pop_back();
      }
      while self.visiting_back.len() > 0 {
        self.visiting_back.pop_back();
      }
    }
  }
}

impl<'a, T> Iterator for QuadTreeIter<'a, T> {
  type Item = &'a QuadTree<T>;

  fn next(&mut self) -> Option<Self::Item> {
    let result = match self.visiting.back() {
      Some(ref entry) => {
        Some(entry.node)
      },
      _ => {None},
    };
    self.check_end();
    self.step_up();
    result
  }
}

impl<'a, T> DoubleEndedIterator for QuadTreeIter<'a, T> {
  fn next_back(&mut self) -> Option<Self::Item> {
    let result = match self.visiting_back.back() {
      Some(ref entry) => {
        Some(entry.node)
      },
      _ => {None},
    };
    self.check_end();
    self.step_back_down();
    result
  }
}

#[test]
fn test_quad_tree_iter() {
  let tree = QuadTree::<i32> {
    value: 0,
    children: Some(Box::new([
      QuadTree::<i32>::new(1),
      QuadTree::<i32>::new(2),
      QuadTree::<i32>::new(3),
      QuadTree::<i32>::new(4),
    ])),
  };
  {
    let mut iter = tree.iter();
    assert_eq!(iter.next().unwrap().value, 1);
    assert_eq!(iter.next().unwrap().value, 2);
    assert_eq!(iter.next().unwrap().value, 3);
    assert_eq!(iter.next().unwrap().value, 4);
    assert_eq!(iter.next().unwrap().value, 0);
    assert_eq!(iter.next().is_none(), true);
  }
}

#[test]
fn test_deeper_quad_tree_iter() {
  let tree = QuadTree::<i32> {
    value: 0,
    children: Some(Box::new([
      QuadTree::<i32>::new(1),
      QuadTree::<i32> {
        value: 2,
        children: Some(Box::new([
          QuadTree::<i32>::new(5),
          QuadTree::<i32>::new(6),
          QuadTree::<i32>::new(7),
          QuadTree::<i32>::new(8),
        ])),
      },
      QuadTree::<i32>::new(3),
      QuadTree::<i32>::new(4),
    ])),
  };
  {
    let mut iter = tree.iter();
    assert_eq!(iter.next().unwrap().value, 1);
    assert_eq!(iter.next().unwrap().value, 5);
    assert_eq!(iter.next().unwrap().value, 6);
    assert_eq!(iter.next().unwrap().value, 7);
    assert_eq!(iter.next().unwrap().value, 8);
    assert_eq!(iter.next().unwrap().value, 2);
    assert_eq!(iter.next().unwrap().value, 3);
    assert_eq!(iter.next().unwrap().value, 4);
    assert_eq!(iter.next().unwrap().value, 0);
    assert_eq!(iter.next().is_none(), true);
  }
}

#[test]
fn test_quad_tree_next_back() {
  let tree = QuadTree::<i32> {
    value: 0,
    children: Some(Box::new([
      QuadTree::<i32>::new(1),
      QuadTree::<i32>::new(2),
      QuadTree::<i32>::new(3),
      QuadTree::<i32>::new(4),
    ])),
  };
  {
    let mut iter = tree.iter();
    assert_eq!(iter.next_back().unwrap().value, 0);
    assert_eq!(iter.next_back().unwrap().value, 4);
    assert_eq!(iter.next_back().unwrap().value, 3);
    assert_eq!(iter.next_back().unwrap().value, 2);
    assert_eq!(iter.next_back().unwrap().value, 1);
    assert_eq!(iter.next_back().is_none(), true);
  }
}

#[test]
fn test_deeper_quad_tree_next_back() {
  let tree = QuadTree::<i32> {
    value: 0,
    children: Some(Box::new([
      QuadTree::<i32>::new(1),
      QuadTree::<i32> {
        value: 2,
        children: Some(Box::new([
          QuadTree::<i32>::new(5),
          QuadTree::<i32>::new(6),
          QuadTree::<i32>::new(7),
          QuadTree::<i32>::new(8),
        ])),
      },
      QuadTree::<i32>::new(3),
      QuadTree::<i32>::new(4),
    ])),
  };
  {
    let mut iter = tree.iter();
    assert_eq!(iter.next_back().unwrap().value, 0);
    assert_eq!(iter.next_back().unwrap().value, 4);
    assert_eq!(iter.next_back().unwrap().value, 3);
    assert_eq!(iter.next_back().unwrap().value, 2);
    assert_eq!(iter.next_back().unwrap().value, 8);
    assert_eq!(iter.next_back().unwrap().value, 7);
    assert_eq!(iter.next_back().unwrap().value, 6);
    assert_eq!(iter.next_back().unwrap().value, 5);
    assert_eq!(iter.next_back().unwrap().value, 1);
    assert_eq!(iter.next_back().is_none(), true);
  }
}

// const push_noop : Fn(&'a mut QuadTree<T>) = |x| {};
// const pop_noop = || {};

fn push_noop<T>(x: &mut QuadTree<T>) {}
fn pop_noop() {}

impl<'a, T> QuadTreeIterMut<'a, T> {
  fn new(node: &'a mut QuadTree<T>) -> QuadTreeIterMut<'a, T> {
    let mut iter = QuadTreeIterMut::<T> {
      // visit_push: &push_noop,
      // visit_pop: &pop_noop,
      // visit_push_back: &push_noop,
      // visit_pop_back: &pop_noop,
      visiting: LinkedList::<QuadTreeIterMutEntry<T>>::new(),
      visiting_back: LinkedList::<QuadTreeIterMutEntry<T>>::new(),
    };
    unsafe {
      let ptr = &mut *node as *mut QuadTree<T>;
      iter.push_node(&mut *ptr as &'a mut QuadTree<T>);
      iter.push_node_back(&mut *ptr as &'a mut QuadTree<T>);
    }
    iter
  }

  fn push_node(&mut self, node: &'a mut QuadTree<T>) {
    let childIndex = match node.children {
      Some(_) => {0},
      _ => {4},
    };
    // (*self.visit_push)(&mut node);
    self.visiting.push_back(QuadTreeIterMutEntry::<T> {
      node: node,
      childIndex: childIndex,
    });
    self.step_down();
  }

  fn push_node_back(&mut self, node: &'a mut QuadTree<T>) {
    let childIndex = match node.children {
      Some(_) => {0},
      _ => {4},
    };
    // (*self.visit_push_back)(&mut node);
    self.visiting_back.push_back(QuadTreeIterMutEntry::<T> {
      node: node,
      childIndex: childIndex,
    });
  }

  fn step_down(&mut self) {
    let maybeNode = match self.visiting.back_mut() {
      Some(ref mut entry) => {
        if entry.childIndex < 4 {
          match entry.node.children {
            Some(ref mut children) => {
              unsafe {
                let ptr = &mut children[entry.childIndex] as *mut QuadTree<T>;
                let result = Some(&mut *ptr as &'a mut QuadTree<T>);
                entry.childIndex += 1;
                result
              }
            },
            _ => {
              entry.childIndex = 4;
              None
            },
          }
        }
        else {
          None
        }
      },
      _ => {None},
    };
    match maybeNode {
      Some(node) => {
        self.push_node(node);
      },
      _ => {},
    }
  }

  fn step_back_down(&mut self) {
    let maybeNode = match self.visiting_back.back_mut() {
      Some(ref mut entry) => {
        if entry.childIndex < 4 {
          match entry.node.children {
            Some(ref mut children) => {
              unsafe {
                let ptr = &mut children[3 - entry.childIndex] as *mut QuadTree<T>;
                let result = Some(&mut *ptr as &'a mut QuadTree<T>);
                entry.childIndex += 1;
                result
              }
            },
            _ => {
              entry.childIndex = 4;
              None
            },
          }
        }
        else {
          None
        }
      },
      _ => {None},
    };
    match maybeNode {
      Some(node) => {
        self.push_node_back(node);
      },
      _ => {
        self.step_back_up();
      },
    }
  }

  fn step_up(&mut self) {
    // (*self.visit_pop)();
    self.visiting.pop_back();
    if self.visiting.len() > 0 {
      self.step_down();
    }
  }

  fn step_back_up(&mut self) {
    loop {
      // (*self.visit_pop_back)();
      self.visiting_back.pop_back();
      match self.visiting_back.back() {
        Some(ref entry) => {
          if entry.childIndex < 4 {
            break;
          }
        },
        _ => {break;},
      }
    }
    if self.visiting_back.len() > 0 {
      self.step_back_down();
    }
  }

  fn check_end(&mut self) {
    let is_end = match self.visiting.back() {
      Some(ref front_entry) => {
        match self.visiting_back.back() {
          Some(ref back_entry) => {
            unsafe {
              let front_ptr = &*front_entry.node as *const QuadTree<T>;
              let back_ptr = &*back_entry.node as *const QuadTree<T>;
              front_ptr == back_ptr
            }
          },
          _ => {false},
        }
      },
      _ => {false},
    };
    if is_end {
      while self.visiting.len() > 0 {
        self.visiting.pop_back();
      }
      while self.visiting_back.len() > 0 {
        self.visiting_back.pop_back();
      }
    }
  }
}

impl<'a, T> Iterator for QuadTreeIterMut<'a, T> {
  type Item = &'a mut QuadTree<T>;

  fn next(&mut self) -> Option<Self::Item> {
    let result : Option<&'a mut QuadTree<T>> = match self.visiting.back_mut() {
      Some(ref mut entry) => {
        unsafe {
          let ptr = entry.node as *mut QuadTree<T>; 
          Some(&mut *ptr as &'a mut QuadTree<T>)
        }
      },
      _ => {None},
    };
    self.step_up();
    result
  }
}

impl<'a, T> DoubleEndedIterator for QuadTreeIterMut<'a, T> {
  fn next_back(&mut self) -> Option<Self::Item> {
    let result = match self.visiting_back.back_mut() {
      Some(ref mut entry) => {
        unsafe {
          let ptr = entry.node as *mut QuadTree<T>; 
          Some(&mut *ptr as &'a mut QuadTree<T>)
        }
      },
      _ => {None},
    };
    self.check_end();
    self.step_back_down();
    result
  }
}

impl<'a, T> QuadTreeRefStack<'a, T> {
  fn new() -> QuadTreeRefStack<'a, T> {
    QuadTreeRefStack::<T> {
      list: LinkedList::<&'a mut T>::new(),
    }
  }

  fn push(&mut self, t: &'a mut T) {
    self.list.push_back(t);
  }

  fn pop(&mut self) {
    self.list.pop_back();
  }

  fn len(&self) -> usize {
    self.list.len()
  }

  fn iter(&self) -> iter::Map<linked_list::Iter<&'a mut T>, fn(& &'a mut T) -> &'a T> {
    fn deref<'a, T>(t: &&'a mut T) -> &'a T {
      unsafe {& *(& **t as *const T) as &'a T}
    }
    self.list.iter().map(deref)
  }

  fn iter_mut(&mut self) -> iter::Map<linked_list::IterMut<&'a mut T>, fn(&mut &'a mut T) -> &'a mut T> {
    fn deref<'a, T>(t: &mut &'a mut T) -> &'a mut T {
      unsafe {&mut *(&mut **t as *mut T) as &'a mut T}
    }
    self.list.iter_mut().map(deref)
  }
}

// impl<'a, T> QuadTreeRefStackIter<'a, T> {
//   fn new(stack: &'a QuadTreeRefStack<T>) -> QuadTreeRefStackIter<'a, T> {
//     QuadTreeRefStackIter::<T> {
//       iter: stack.list.iter(),
//     }
//   }
// }
//
// impl<'a, T> Iterator for QuadTreeRefStackIter<'a, T> {
//   type Item = &'a T;
//
//   fn next() -> {
//     match self.iter.next() {
//
//     }
//   }
// }

impl<'a, T> QuadTreeIterMutStack<'a, T> {
  fn new(node: &'a mut QuadTree<T>) -> QuadTreeIterMutStack<'a, T> {
    let mut iter = QuadTreeIterMutStack::<T> {
      // visit_push: &push_noop,
      // visit_pop: &pop_noop,
      // visit_push_back: &push_noop,
      // visit_pop_back: &pop_noop,
      first: true,
      first_back: true,
      visiting_stack: QuadTreeRefStack::<'a, T>::new(),
      visiting_back_stack: QuadTreeRefStack::<'a, T>::new(),
      visiting: LinkedList::<QuadTreeIterMutEntry<T>>::new(),
      visiting_back: LinkedList::<QuadTreeIterMutEntry<T>>::new(),
    };
    unsafe {
      let ptr = &mut *node as *mut QuadTree<T>;
      iter.push_node(&mut *ptr as &'a mut QuadTree<T>);
      iter.push_node_back(&mut *ptr as &'a mut QuadTree<T>);
    }
    iter
  }

  fn push_node(&mut self, node: &'a mut QuadTree<T>) {
    let childIndex = match node.children {
      Some(_) => {0},
      _ => {4},
    };
    unsafe {
      let ptr = &mut (*node).value as *mut T;
      self.visiting_stack.push(&mut *ptr as &'a mut T);
    }
    // (*self.visit_push)(&mut node);
    self.visiting.push_back(QuadTreeIterMutEntry::<T> {
      node: node,
      childIndex: childIndex,
    });
    self.step_down();
  }

  fn push_node_back(&mut self, node: &'a mut QuadTree<T>) {
    let childIndex = match node.children {
      Some(_) => {0},
      _ => {4},
    };
    unsafe {
      let ptr = &mut (*node).value as *mut T;
      self.visiting_back_stack.push(&mut *ptr as &'a mut T);
    }
    // (*self.visit_push_back)(&mut node);
    self.visiting_back.push_back(QuadTreeIterMutEntry::<T> {
      node: node,
      childIndex: childIndex,
    });
  }

  fn step_down(&mut self) {
    let maybeNode = match self.visiting.back_mut() {
      Some(ref mut entry) => {
        if entry.childIndex < 4 {
          match entry.node.children {
            Some(ref mut children) => {
              unsafe {
                let ptr = &mut children[entry.childIndex] as *mut QuadTree<T>;
                let result = Some(&mut *ptr as &'a mut QuadTree<T>);
                entry.childIndex += 1;
                result
              }
            },
            _ => {
              entry.childIndex = 4;
              None
            },
          }
        }
        else {
          None
        }
      },
      _ => {None},
    };
    match maybeNode {
      Some(node) => {
        self.push_node(node);
      },
      _ => {},
    }
  }

  fn step_back_down(&mut self) {
    if self.first_back {
      self.first_back = false;
      return;
    }
    let maybeNode = match self.visiting_back.back_mut() {
      Some(ref mut entry) => {
        if entry.childIndex < 4 {
          match entry.node.children {
            Some(ref mut children) => {
              unsafe {
                let ptr = &mut children[3 - entry.childIndex] as *mut QuadTree<T>;
                let result = Some(&mut *ptr as &'a mut QuadTree<T>);
                entry.childIndex += 1;
                result
              }
            },
            _ => {
              entry.childIndex = 4;
              None
            },
          }
        }
        else {
          None
        }
      },
      _ => {None},
    };
    match maybeNode {
      Some(node) => {
        self.push_node_back(node);
      },
      _ => {
        self.step_back_up();
      },
    }
  }

  fn step_up(&mut self) {
    if self.first {
      self.first = false;
      return;
    }
    self.visiting_stack.pop();
    // (*self.visit_pop)();
    self.visiting.pop_back();
    if self.visiting.len() > 0 {
      self.step_down();
    }
  }

  fn step_back_up(&mut self) {
    loop {
      self.visiting_back_stack.pop();
      // (*self.visit_pop_back)();
      self.visiting_back.pop_back();
      match self.visiting_back.back() {
        Some(ref entry) => {
          if entry.childIndex < 4 {
            break;
          }
        },
        _ => {break;},
      }
    }
    if self.visiting_back.len() > 0 {
      self.step_back_down();
    }
  }

  fn check_end(&mut self) {
    let is_end = match self.visiting.back() {
      Some(ref front_entry) => {
        match self.visiting_back.back() {
          Some(ref back_entry) => {
            unsafe {
              let front_ptr = &*front_entry.node as *const QuadTree<T>;
              let back_ptr = &*back_entry.node as *const QuadTree<T>;
              front_ptr == back_ptr
            }
          },
          _ => {false},
        }
      },
      _ => {false},
    };
    if is_end {
      while self.visiting.len() > 0 {
        self.visiting.pop_back();
      }
      while self.visiting_back.len() > 0 {
        self.visiting_back.pop_back();
      }
      while self.visiting_stack.len() > 0 {
        self.visiting_stack.pop();
      }
      while self.visiting_back_stack.len() > 0 {
        self.visiting_back_stack.pop();
      }
    }
  }
}

impl<'a, T> Iterator for QuadTreeIterMutStack<'a, T> {
  type Item = &'a mut QuadTreeRefStack<'a, T>;

  fn next(&mut self) -> Option<Self::Item> {
    self.step_up();
    let result = if self.visiting_stack.len() > 0 {
      unsafe {
        let ptr = &mut self.visiting_stack as *mut QuadTreeRefStack<'a, T>;
        Some(&mut *ptr as &'a mut QuadTreeRefStack<'a, T>)
      }
    }
    else {
      None
    };
    result
  }
}

impl<'a, T> DoubleEndedIterator for QuadTreeIterMutStack<'a, T> {
  fn next_back(&mut self) -> Option<Self::Item> {
    self.check_end();
    self.step_back_down();
    let result = if self.visiting_back_stack.len() > 0 {
      unsafe {
        let ptr = &mut self.visiting_back_stack as *mut QuadTreeRefStack<'a, T>;
        Some(&mut *ptr as &'a mut QuadTreeRefStack<'a, T>)
      }
    }
    else {
      None
    };
    result
  }
}

// impl<'a, T> QuadTreeIterMutStack<'a, T> {
//   fn new(node: &'a mut QuadTree<T>) -> QuadTreeIterMutStack<'a, T> {
//     unsafe {
//       let node_ptr = &mut *node as *mut QuadTree<T>;
//       let mut ptr : *mut QuadTreeIterMutStack<T>;
//       let mut iter = QuadTreeIterMutStack::<T> {
//         bound_visit_push: Box::new(|x| (*ptr).handle_visit_push(x)),
//         bound_visit_pop: Box::new(|| (*ptr).handle_visit_pop()),
//         bound_visit_push_back: Box::new(|x| (*ptr).handle_visit_push_back(x)),
//         bound_visit_pop_back: Box::new(|| (*ptr).handle_visit_pop_back()),
//         visiting_stack: LinkedList::new(),
//         visiting_back_stack: LinkedList::new(),
//         // iter: QuadTreeIterMut::new(&mut *node_ptr as &'a mut QuadTree<T>),
//         iter: (*node_ptr).iter_mut(),
//       };
//       ptr = &mut iter as *mut QuadTreeIterMutStack<T>;
//       iter.iter.visit_push = &*iter.bound_visit_push;
//       iter.iter.visit_pop = &*iter.bound_visit_pop;
//       iter.iter.visit_push_back = &*iter.bound_visit_push_back;
//       iter.iter.visit_pop_back = &*iter.bound_visit_pop_back;
//       iter
//     }
//   }
//
//   fn handle_visit_push(&mut self, node: &'a mut QuadTree<T>) {
//     self.visiting_stack.push_back(&mut node.value);
//   }
//
//   fn handle_visit_pop(&mut self) {
//     self.visiting_stack.pop_back();
//   }
//
//   fn handle_visit_push_back(&mut self, node: &'a mut QuadTree<T>) {
//     self.visiting_back_stack.push_back(&mut node.value);
//   }
//
//   fn handle_visit_pop_back(&mut self) {
//     self.visiting_back_stack.pop_back();
//   }
// }
//
// impl<'a, T> Iterator for QuadTreeIterMutStack<'a, T> {
//   type Item = &'a mut LinkedList<&'a mut T>;
//
//   fn next(&mut self) -> Option<Self::Item> {
//     match self.iter.next() {
//       Some(_) => {
//         unsafe {
//           let ptr = &mut self.visiting_stack as *mut LinkedList<&'a mut T>;
//           Some(&mut *ptr as &'a mut LinkedList<&'a mut T>)
//         }
//       },
//       _ => {None},
//     }
//   }
// }
//
// impl<'a, T> DoubleEndedIterator for QuadTreeIterMutStack<'a, T> {
//   fn next_back(&mut self) -> Option<Self::Item> {
//     match self.iter.next_back() {
//       Some(_) => {
//         unsafe {
//           let ptr = &mut self.visiting_back_stack as *mut LinkedList<&'a mut T>;
//           Some(&mut *ptr as &'a mut LinkedList<&'a mut T>)
//         }
//       },
//       _ => {None},
//     }
//   }
// }

#[test]
fn test_quad_tree_iter_mut() {
  let mut tree = QuadTree::<i32> {
    value: 0,
    children: Some(Box::new([
      QuadTree::<i32>::new(1),
      QuadTree::<i32>::new(2),
      QuadTree::<i32>::new(3),
      QuadTree::<i32>::new(4),
    ])),
  };
  for node in tree.iter_mut() {
    node.value += 1;
  }
  {
    let mut iter = tree.iter();
    assert_eq!(iter.next().unwrap().value, 2);
    assert_eq!(iter.next().unwrap().value, 3);
    assert_eq!(iter.next().unwrap().value, 4);
    assert_eq!(iter.next().unwrap().value, 5);
    assert_eq!(iter.next().unwrap().value, 1);
    assert_eq!(iter.next().is_none(), true);
  }
}

#[test]
fn test_deeper_quad_tree_iter_mut() {
  let mut tree = QuadTree::<i32> {
    value: 0,
    children: Some(Box::new([
      QuadTree::<i32>::new(1),
      QuadTree::<i32> {
        value: 2,
        children: Some(Box::new([
          QuadTree::<i32>::new(5),
          QuadTree::<i32>::new(6),
          QuadTree::<i32>::new(7),
          QuadTree::<i32>::new(8),
        ])),
      },
      QuadTree::<i32>::new(3),
      QuadTree::<i32>::new(4),
    ])),
  };
  for node in tree.iter_mut() {
    node.value += 1;
  }
  {
    let mut iter = tree.iter();
    assert_eq!(iter.next().unwrap().value, 2);
    assert_eq!(iter.next().unwrap().value, 6);
    assert_eq!(iter.next().unwrap().value, 7);
    assert_eq!(iter.next().unwrap().value, 8);
    assert_eq!(iter.next().unwrap().value, 9);
    assert_eq!(iter.next().unwrap().value, 3);
    assert_eq!(iter.next().unwrap().value, 4);
    assert_eq!(iter.next().unwrap().value, 5);
    assert_eq!(iter.next().unwrap().value, 1);
    assert_eq!(iter.next().is_none(), true);
  }
}

#[test]
fn test_quad_tree_iter_mut_back() {
  let mut tree = QuadTree::<i32> {
    value: 0,
    children: Some(Box::new([
      QuadTree::<i32>::new(1),
      QuadTree::<i32>::new(2),
      QuadTree::<i32>::new(3),
      QuadTree::<i32>::new(4),
    ])),
  };
  for node in tree.iter_mut() {
    node.value += 1;
  }
  {
    let mut iter = tree.iter();
    assert_eq!(iter.next_back().unwrap().value, 1);
    assert_eq!(iter.next_back().unwrap().value, 5);
    assert_eq!(iter.next_back().unwrap().value, 4);
    assert_eq!(iter.next_back().unwrap().value, 3);
    assert_eq!(iter.next_back().unwrap().value, 2);
    assert_eq!(iter.next_back().is_none(), true);
  }
}

#[test]
fn test_deeper_quad_tree_iter_mut_back() {
  let mut tree = QuadTree::<i32> {
    value: 0,
    children: Some(Box::new([
      QuadTree::<i32>::new(1),
      QuadTree::<i32> {
        value: 2,
        children: Some(Box::new([
          QuadTree::<i32>::new(5),
          QuadTree::<i32>::new(6),
          QuadTree::<i32>::new(7),
          QuadTree::<i32>::new(8),
        ])),
      },
      QuadTree::<i32>::new(3),
      QuadTree::<i32>::new(4),
    ])),
  };
  for node in tree.iter_mut() {
    node.value += 1;
  }
  {
    let mut iter = tree.iter();
    assert_eq!(iter.next_back().unwrap().value, 1);
    assert_eq!(iter.next_back().unwrap().value, 5);
    assert_eq!(iter.next_back().unwrap().value, 4);
    assert_eq!(iter.next_back().unwrap().value, 3);
    assert_eq!(iter.next_back().unwrap().value, 9);
    assert_eq!(iter.next_back().unwrap().value, 8);
    assert_eq!(iter.next_back().unwrap().value, 7);
    assert_eq!(iter.next_back().unwrap().value, 6);
    assert_eq!(iter.next_back().unwrap().value, 2);
    assert_eq!(iter.next_back().is_none(), true);
  }
}

#[test]
fn test_quad_tree_iter_stack_mut() {
  let mut tree = QuadTree::<i32> {
    value: 0,
    children: Some(Box::new([
      QuadTree::<i32>::new(1),
      QuadTree::<i32>::new(2),
      QuadTree::<i32>::new(3),
      QuadTree::<i32>::new(4),
    ])),
  };
  for value_stack in tree.iter_stack_mut() {
    for value in value_stack.iter_mut() {
      *value += 1;
    }
  }
  {
    let mut iter = tree.iter_stack_mut();
    assert_eq!(iter.next().unwrap().len(), 2);
    assert_eq!(iter.next().unwrap().len(), 2);
    assert_eq!(iter.next().unwrap().len(), 2);
    assert_eq!(iter.next().unwrap().len(), 2);
    assert_eq!(iter.next().unwrap().len(), 1);
    assert_eq!(iter.next().is_none(), true);
  }
  {
    let mut iter = tree.iter();
    assert_eq!(iter.next().unwrap().value, 2);
    assert_eq!(iter.next().unwrap().value, 3);
    assert_eq!(iter.next().unwrap().value, 4);
    assert_eq!(iter.next().unwrap().value, 5);
    assert_eq!(iter.next().unwrap().value, 5);
    assert_eq!(iter.next().is_none(), true);
  }
}

#[test]
fn test_quad_tree_iter_stack_mut_rev() {
  let mut tree = QuadTree::<i32> {
    value: 0,
    children: Some(Box::new([
      QuadTree::<i32>::new(1),
      QuadTree::<i32>::new(2),
      QuadTree::<i32>::new(3),
      QuadTree::<i32>::new(4),
    ])),
  };
  for value_stack in tree.iter_stack_mut().rev() {
    for value in value_stack.iter_mut() {
      *value += 1;
    }
  }
  {
    let mut iter = tree.iter_stack_mut().rev();
    assert_eq!(iter.next().unwrap().len(), 1);
    assert_eq!(iter.next().unwrap().len(), 2);
    assert_eq!(iter.next().unwrap().len(), 2);
    assert_eq!(iter.next().unwrap().len(), 2);
    assert_eq!(iter.next().unwrap().len(), 2);
    assert_eq!(iter.next().is_none(), true);
  }
  {
    let mut iter = tree.iter().rev();
    assert_eq!(iter.next().unwrap().value, 5);
    assert_eq!(iter.next().unwrap().value, 5);
    assert_eq!(iter.next().unwrap().value, 4);
    assert_eq!(iter.next().unwrap().value, 3);
    assert_eq!(iter.next().unwrap().value, 2);
    assert_eq!(iter.next().is_none(), true);
  }
}
