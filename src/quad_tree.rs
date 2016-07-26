use std::any::Any;
use std::iter;
use std::collections::linked_list;
use std::collections::LinkedList;

pub trait TreeSplitable {
  fn tree_split(&mut self, children: &mut [&mut Self]);
}

pub trait TreeJoinable {
  fn tree_join(&mut self, children: & [&Self]);
}

pub trait TreeSplitableWith {
  fn tree_split_with(&mut self, children: &mut [&mut Self], data: &mut Any);
}

pub trait TreeJoinableWith {
  fn tree_join_with(&mut self, children: & [&Self], data: &mut Any);
}

#[derive(Default)]
pub struct QuadTree<T> {
  pub value: T,
  pub children: Option<Box<[QuadTree<T>; 4]>>,
}

struct PointerIterEntry<T> {
  node: *const QuadTree<T>,
  child: *const QuadTree<T>,
  end: *const QuadTree<T>,
}

pub struct Iter<'a, T: 'a> {
  start: Option<&'a QuadTree<T>>,
  visiting: Vec<PointerIterEntry<T>>,
  visiting_back: Vec<PointerIterEntry<T>>,
}

struct QuadTreeIterEntry<'a, T: 'a> {
  node: &'a QuadTree<T>,
  child_index: usize,
}

// pub struct Iter<'a, T: 'a> {
//   first: bool,
//   first_back: bool,
//   visiting: Vec<QuadTreeIterEntry<'a, T>>,
//   visiting_back: Vec<QuadTreeIterEntry<'a, T>>,
// }

struct QuadTreeIterMutEntry<'a, T: 'a> {
  node: &'a mut QuadTree<T>,
  child_index: usize,
}

struct QuadTreeIterMut<'a, T: 'a> {
  // visit_push: &'a Fn(&'a mut QuadTree<T>),
  // visit_pop: &'a Fn(),
  // visit_push_back: &'a Fn(&'a mut QuadTree<T>),
  // visit_pop_back: &'a Fn(),
  first: bool,
  first_back: bool,
  visiting: Vec<QuadTreeIterMutEntry<'a, T>>,
  visiting_back: Vec<QuadTreeIterMutEntry<'a, T>>,
}

struct QuadTreeIterMutFilter<'a, T: 'a> {
  first: bool,
  first_back: bool,
  visiting: Vec<QuadTreeIterMutEntry<'a, T>>,
  visiting_back: Vec<QuadTreeIterMutEntry<'a, T>>,
  filter_tree: &'a Fn(&QuadTree<T>) -> bool,
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

// struct QuadTreeRefWrap<'a, 'b, T: 'a> {
//   visiting: &'b mut LinkedList<QuadTreeIterMut<'a, T>>,
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
  pub fn new(value: T) -> QuadTree<T> {
    QuadTree::<T> {
      value: value,
      children: None,
    }
  }

  pub fn is_leaf(&self) -> bool {
    self.children.is_none()
  }

  // fn children_as_slice<'a>(&'a self) -> [&'a T] {
  //   match self.children {
  //     Some(ref children) => {
  //       [
  //         &children[0].value,
  //         &children[1].value,
  //         &children[2].value,
  //         &children[3].value,
  //       ][..]
  //     },
  //     _ => {[].into_slice()},
  //   }
  // }
  //
  // fn children_as_mut_slice<'a>(&'a self) -> [&'a mut T] {
  //   match self.children {
  //     Some(ref mut children) => {
  //       [
  //         &mut children[0].value,
  //         &mut children[1].value,
  //         &mut children[2].value,
  //         &mut children[3].value,
  //       ].into_slice()
  //     },
  //     _ => {[].into_slice()},
  //   }
  // }

  pub fn walk_mut(&mut self, filter: &Fn(&T) -> bool, handle: &Fn(&mut QuadTree<T>)) {
    if filter(&self.value) {
      if let Some(ref mut children) = self.children {
        let mut i = 0;
        while i < 4 {
          let child = &mut children[i];
          i += 1;
        // for child in children.iter_mut() {
          child.walk_mut(filter, handle);
        }
      }
      handle(self);
    }
  }

  pub fn iter(&self) -> Iter<T> {
    Iter::<T>::new(self)
  }

  pub fn iter_mut(&mut self) -> QuadTreeIterMut<T> {
    QuadTreeIterMut::<T>::new(self)
  }

  pub fn iter_filter_mut<'a>(&'a mut self, filter: &'a Fn(&QuadTree<T>) -> bool) -> QuadTreeIterMutFilter<T> {
    QuadTreeIterMutFilter::<T>::new(self, filter)
  }

  pub fn iter_stack_mut(&mut self) -> QuadTreeIterMutStack<T> {
    QuadTreeIterMutStack::<T>::new(self)
  }
}

impl<T> QuadTree<T> where T: TreeSplitable + TreeJoinable + Default {
  pub fn split(&mut self) {
    self.children = Some(Box::new([
      QuadTree::<T> {
        value: T::default(),
        children: None,
      },
      QuadTree::<T> {
        value: T::default(),
        children: None,
      },
      QuadTree::<T> {
        value: T::default(),
        children: None,
      },
      QuadTree::<T> {
        value: T::default(),
        children: None,
      },
    ]));
    match self.children {
      Some(ref mut children) => {
        let mut children_array = unsafe {
          [
            &mut *(&mut children[0].value as *mut T) as &mut T,
            &mut *(&mut children[1].value as *mut T) as &mut T,
            &mut *(&mut children[2].value as *mut T) as &mut T,
            &mut *(&mut children[3].value as *mut T) as &mut T,
          ]
        };
        self.value.tree_split(children_array.as_mut());
      },
      _ => {},
    }
    // self.value.tree_split(&mut self.children_as_mut_slice());
  }

  pub fn join(&mut self) {
    match self.children {
      Some(ref children) => {
        self.value.tree_join([
          &children[0].value,
          &children[1].value,
          &children[2].value,
          &children[3].value,
        ].as_ref());
      },
      _ => {},
    }
    // self.value.tree_join(& self.children_as_slice());
    self.children = None;
  }
}

impl<T> QuadTree<T> where T: TreeSplitableWith + TreeJoinableWith + Default {
  pub fn split_with(&mut self, data: &mut Any) {
    self.children = Some(Box::new([
      QuadTree::<T> {
        value: T::default(),
        children: None,
      },
      QuadTree::<T> {
        value: T::default(),
        children: None,
      },
      QuadTree::<T> {
        value: T::default(),
        children: None,
      },
      QuadTree::<T> {
        value: T::default(),
        children: None,
      },
    ]));
    match self.children {
      Some(ref mut children) => {
        let mut children_array = unsafe {
          [
            &mut *(&mut children[0].value as *mut T) as &mut T,
            &mut *(&mut children[1].value as *mut T) as &mut T,
            &mut *(&mut children[2].value as *mut T) as &mut T,
            &mut *(&mut children[3].value as *mut T) as &mut T,
          ]
        };
        self.value.tree_split_with(children_array.as_mut(), data);
      },
      _ => {},
    }
    // self.value.tree_split(&mut self.children_as_mut_slice());
  }

  pub fn join_with(&mut self, data: &mut Any) {
    match self.children {
      Some(ref children) => {
        self.value.tree_join_with(
          [
            &children[0].value,
            &children[1].value,
            &children[2].value,
            &children[3].value,
          ].as_ref(),
          data,
        );
      },
      _ => {},
    }
    // self.value.tree_join(& self.children_as_slice());
    self.children = None;
  }
}

impl<'a, T> Iter<'a, T> {
  fn new(node: &'a QuadTree<T>) -> Iter<'a, T> {
    let mut iter = Iter {
      start: Some(node),
      visiting: Vec::<PointerIterEntry<T>>::new(),
      visiting_back: Vec::<PointerIterEntry<T>>::new(),
    };
    let ptr = node as *const QuadTree<T>;
    unsafe {
      // if let Some(ref children) = node.children {
      //   let children_ptr = &**children as *const QuadTree<T>;
      //   iter.visiting.push(PointerIterEntry {
      //     node: ptr, child: children_ptr.offset(-1), end: children_ptr.offset(4)
      //   });
      //   iter.visiting_back.push(PointerIterEntry {
      //     node: ptr, child: ptr.offset(1), end: ptr.offset(-1)
      //   });
      // }
      // else {
      iter.visiting.push(PointerIterEntry {
        node: ptr, child: ptr.offset(-1), end: ptr.offset(0)
      });
      iter.visiting_back.push(PointerIterEntry {
        node: ptr, child: ptr.offset(1), end: ptr.offset(-1)
      });
      // }
    }
    iter
  }
}

impl<'a, T> Iterator for Iter<'a, T> {
  type Item = &'a QuadTree<T>;

  fn next(&mut self) -> Option<Self::Item> {
    let should_pop = {
      if let Some(ref mut entry) = self.visiting.last_mut() {
        if entry.child != entry.end {
          entry.child = unsafe { entry.child.offset(1) };
          if entry.child == entry.node {
            if let Some(ref children) = unsafe { &*entry.node as &QuadTree<T> }.children {
              entry.child = children.as_ptr();
              entry.end = unsafe { entry.child.offset(4) };
            }
          }
          if entry.child == entry.end {
            entry.child = entry.node;
            entry.end = entry.node;
          }
          false
        }
        else {
          true
        }
      }
      else {
        false
      }
    };
    if should_pop {
      self.visiting.pop();
      if let Some(ref mut entry) = self.visiting.last_mut() {
        if entry.child != entry.end {
          entry.child = unsafe { entry.child.offset(1) };
        }
      }
    }
    let maybe_child = {
      let mut maybe = self.visiting.last();
      if let Some(ref mut entry) = maybe {
        if entry.child != entry.end {
          Some(entry.child)
        }
        else {
          None
        }
      }
      else {
        None
      }
    };
    if let Some(mut child) = maybe_child {
      while let Some(ref children) = unsafe { &*child as &QuadTree<T> }.children {
        let ptr = children.as_ptr();
        self.visiting.push(PointerIterEntry {
          node: child, child: ptr, end: unsafe { ptr.offset(4) }
        });
        child = ptr;
      }
    }
    let maybe = self.visiting.last();
    if let Some(ref entry) = maybe {
      if entry.child == entry.end {
        Some(unsafe { &*entry.node as &QuadTree<T> })
      }
      else {
        Some(unsafe { &*entry.child as &QuadTree<T> })
      }
    }
    else {
      None
    }
  }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
  fn next_back(&mut self) -> Option<Self::Item> {
    let should_push = if let Some(ref mut entry) = self.visiting_back.last_mut() {
      if entry.child != entry.end {
        if entry.end == unsafe { entry.node.offset(-1) } {
          entry.child = unsafe { entry.child.offset(-1) };
          if entry.child == unsafe { entry.node.offset(-1) } {
            if let Some(ref children) = unsafe { &*entry.node as &QuadTree<T> }.children {
              entry.end = unsafe { children.as_ptr().offset(-1) };
              entry.child = unsafe { entry.end.offset(4) };
            }
          }
          false
        }
        else {
          if let Some(_) = unsafe { &*entry.child }.children {
            true
          }
          else {
            entry.child = unsafe { entry.child.offset(-1) };
            false
          }
        }
      }
      else {
        false
      }
    }
    else {
      false
    };
    if should_push {
      let child = self.visiting_back.last().unwrap().child;
      if let Some(ref children) = unsafe { &*child }.children {
        let ptr = children.as_ptr() as *const QuadTree<T>;
        self.visiting_back.push(PointerIterEntry {
          node: child,
          child: unsafe { ptr.offset(3) },
          end: unsafe { ptr.offset(-1) },
        });
      }
    }
    loop {
      let should_pop = {
        if let Some(ref entry) = self.visiting_back.last() {
          entry.child == entry.end
        }
        else {
          false
        }
      };
      if should_pop {
        self.visiting_back.pop();
        let should_push = if let Some(ref mut entry) = self.visiting_back.last_mut() {
          if entry.child != entry.end {
            entry.child = unsafe { entry.child.offset(-1) };
          }
          entry.child != entry.end
        }
        else {
          false
        };
        if should_push {
          let child = self.visiting_back.last().unwrap().child;
          if let Some(ref children) = unsafe { &*child }.children {
            let ptr = children.as_ptr() as *const QuadTree<T>;
            self.visiting_back.push(PointerIterEntry {
              node: child,
              child: unsafe { child.offset(0) },
              end: unsafe { child.offset(-1) },
            });
          }
          break;
        }
      }
      else {
        break;
      }
    }
    // let maybe_child = {
    //   let mut maybe = self.visiting.last();
    //   if let Some(ref entry) = maybe {
    //     if entry.child != entry.end {
    //       Some(entry.child)
    //     }
    //     else {
    //       None
    //     }
    //   }
    //   else {
    //     None
    //   }
    // };
    // if let Some(mut child) = maybe_child {
    //   while let Some(ref children) = unsafe { &*child as &QuadTree<T> }.children {
    //     let ptr = children.as_ptr();
    //     self.visiting.push(PointerIterEntry { node: child, child: ptr, end: unsafe { ptr.offset(4) } });
    //     child = ptr;
    //   }
    // }
    let maybe = self.visiting_back.last();
    if let Some(ref entry) = maybe {
      if entry.end == unsafe { entry.node.offset(-1) } {
        Some(unsafe { &*entry.node as &QuadTree<T> })
      }
      else {
        Some(unsafe { &*entry.child as &QuadTree<T> })
      }
    }
    else {
      None
    }
  }
}

#[test]
fn test_quad_tree_pointer_iter() {
  {
    let tree = QuadTree::<i32>::new(0);
    let mut iter = Iter::<i32>::new(&tree);
    assert_eq!(iter.next().unwrap().value, 0);
    assert_eq!(iter.next().is_none(), true);
  }
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
    let mut iter = Iter::<i32>::new(&tree);
    assert_eq!(iter.next().unwrap().value, 1);
    assert_eq!(iter.next().unwrap().value, 2);
    assert_eq!(iter.next().unwrap().value, 3);
    assert_eq!(iter.next().unwrap().value, 4);
    assert_eq!(iter.next().unwrap().value, 0);
    assert_eq!(iter.next().is_none(), true);
  }
}

#[test]
fn test_deeper_quad_tree_pointer_iter() {
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
    let mut iter = Iter::<i32>::new(&tree);
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
fn test_deeper_quad_tree_pointer_next_back() {
  {
    let tree = QuadTree::<i32>::new(0);
    let mut iter = Iter::<i32>::new(&tree);
    assert_eq!(iter.next_back().unwrap().value, 0);
    assert_eq!(iter.next_back().is_none(), true);
  }
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
    let mut iter = Iter::<i32>::new(&tree);
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

// impl<'a, T> Iter<'a, T> {
//   fn new(node: &'a QuadTree<T>) -> Iter<'a, T> {
//     let mut iter = Iter::<T> {
//       first: true,
//       first_back: true,
//       visiting: Vec::<QuadTreeIterEntry<T>>::new(),
//       visiting_back: Vec::<QuadTreeIterEntry<T>>::new(),
//     };
//     iter.push_node(node);
//     iter.push_node_back(node);
//     iter
//   }
//
//   fn push_node(&mut self, node: &'a QuadTree<T>) {
//     self.visiting.push(QuadTreeIterEntry::<T> {
//       node: node,
//       child_index: 0,
//     });
//     self.step_down();
//   }
//
//   fn push_node_back(&mut self, node: &'a QuadTree<T>) {
//     self.visiting_back.push(QuadTreeIterEntry::<T> {
//       node: node,
//       child_index: 0,
//     });
//   }
//
//   fn step_down(&mut self) {
//     let maybe_node = match self.visiting.last_mut() {
//       Some(ref mut entry) => {
//         if entry.child_index < 4 {
//           match entry.node.children {
//             Some(ref children) => {
//               let result = Some(&children[entry.child_index]);
//               entry.child_index += 1;
//               result
//             },
//             _ => {
//               entry.child_index = 4;
//               None
//             },
//           }
//         }
//         else {
//           None
//         }
//       },
//       _ => {None},
//     };
//     match maybe_node {
//       Some(node) => {
//         self.push_node(node);
//       },
//       _ => {},
//     }
//   }
//
//   fn step_back_down(&mut self) {
//     let maybe_node = match self.visiting_back.last_mut() {
//       Some(ref mut entry) => {
//         if entry.child_index < 4 {
//           match entry.node.children {
//             Some(ref children) => {
//               let result = Some(&children[3 - entry.child_index]);
//               entry.child_index += 1;
//               result
//             },
//             _ => {
//               entry.child_index = 4;
//               None
//             },
//           }
//         }
//         else {
//           None
//         }
//       },
//       _ => {None},
//     };
//     match maybe_node {
//       Some(node) => {
//         self.push_node_back(node);
//       },
//       _ => {
//         self.step_back_up();
//       },
//     }
//   }
//
//   fn step_up(&mut self) {
//     self.visiting.pop();
//     if self.visiting.len() > 0 {
//       self.step_down();
//     }
//   }
//
//   fn step_back_up(&mut self) {
//     loop {
//       self.visiting_back.pop();
//       match self.visiting_back.last() {
//         Some(ref entry) => {
//           if entry.child_index < 4 {
//             break;
//           }
//         },
//         _ => {break;},
//       }
//     }
//     if self.visiting_back.len() > 0 {
//       self.step_back_down();
//     }
//   }
//
//   fn test_end(&self, front_entry: &QuadTreeIterEntry<'a, T>, back_entry: &QuadTreeIterEntry<'a, T>) -> bool {
//     let front_ptr = &*front_entry.node as *const QuadTree<T>;
//     let back_ptr = &*back_entry.node as *const QuadTree<T>;
//     front_ptr == back_ptr &&
//       (
//         front_entry.node.children.is_some() &&
//         front_entry.child_index == 4 - back_entry.child_index ||
//         front_entry.node.children.is_none()
//       )
//   }
//
//   fn check_end(&mut self) {
//     let is_end = match self.visiting.last() {
//       Some(ref front_entry) => {
//         match self.visiting_back.last() {
//           Some(ref back_entry) => {
//             self.test_end(front_entry, back_entry)
//           },
//           _ => {false},
//         }
//       },
//       _ => {false},
//     };
//     if is_end {
//       while self.visiting.len() > 0 {
//         self.visiting.pop();
//       }
//       while self.visiting_back.len() > 0 {
//         self.visiting_back.pop();
//       }
//     }
//   }
// }
//
// impl<'a, T> Iterator for Iter<'a, T> {
//   type Item = &'a QuadTree<T>;
//
//   fn next(&mut self) -> Option<Self::Item> {
//     if !self.first {
//       self.check_end();
//       self.step_up();
//     }
//     else {
//       self.first = false;
//     }
//     let result = match self.visiting.last() {
//       Some(ref entry) => {
//         Some(entry.node)
//       },
//       _ => {None},
//     };
//     result
//   }
// }
//
// impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
//   fn next_back(&mut self) -> Option<Self::Item> {
//     if !self.first_back {
//       self.check_end();
//       self.step_back_down();
//     }
//     else {
//       self.first_back = false;
//     }
//     let result = match self.visiting_back.last() {
//       Some(ref entry) => {
//         Some(entry.node)
//       },
//       _ => {None},
//     };
//     result
//   }
// }

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

// fn push_noop<T>(x: &mut QuadTree<T>) {}
// fn pop_noop() {}

impl<'a, T> QuadTreeIterMut<'a, T> {
  fn new(node: &'a mut QuadTree<T>) -> QuadTreeIterMut<'a, T> {
    let mut iter = QuadTreeIterMut::<T> {
      // visit_push: &push_noop,
      // visit_pop: &pop_noop,
      // visit_push_back: &push_noop,
      // visit_pop_back: &pop_noop,
      first: true,
      first_back: true,
      visiting: Vec::<QuadTreeIterMutEntry<T>>::new(),
      visiting_back: Vec::<QuadTreeIterMutEntry<T>>::new(),
    };
    unsafe {
      let ptr = &mut *node as *mut QuadTree<T>;
      iter.push_node(&mut *ptr as &'a mut QuadTree<T>);
      iter.push_node_back(&mut *ptr as &'a mut QuadTree<T>);
    }
    iter
  }

  fn push_node(&mut self, node: &'a mut QuadTree<T>) {
    let child_index = match node.children {
      Some(_) => {0},
      _ => {0},
    };
    // (*self.visit_push)(&mut node);
    self.visiting.push(QuadTreeIterMutEntry::<T> {
      node: node,
      child_index: child_index,
    });
    self.step_down();
  }

  fn push_node_back(&mut self, node: &'a mut QuadTree<T>) {
    // println!("push_node_back");
    let child_index = match node.children {
      Some(_) => {0},
      _ => {0},
    };
    // (*self.visit_push_back)(&mut node);
    self.visiting_back.push(QuadTreeIterMutEntry::<T> {
      node: node,
      child_index: child_index,
    });
  }

  fn step_down(&mut self) {
    let maybe_node = match self.visiting.last_mut() {
      Some(ref mut entry) => {
        if entry.child_index < 4 {
          match entry.node.children {
            Some(ref mut children) => {
              unsafe {
                let ptr = &mut children[entry.child_index] as *mut QuadTree<T>;
                let result = Some(&mut *ptr as &'a mut QuadTree<T>);
                entry.child_index += 1;
                result
              }
            },
            _ => {
              // entry.child_index = 0;
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
    match maybe_node {
      Some(node) => {
        self.push_node(node);
      },
      _ => {},
    }
  }

  fn step_back_down(&mut self) {
    // println!("step_back_down");
    let maybe_node = match self.visiting_back.last_mut() {
      Some(ref mut entry) => {
        // println!("child_index {}", entry.child_index);
        if entry.child_index < 4 {
          // println!("children: {}", entry.node.children.is_some());
          match entry.node.children {
            Some(ref mut children) => {
              unsafe {
                let ptr = &mut children[3 - entry.child_index] as *mut QuadTree<T>;
                let result = Some(&mut *ptr as &'a mut QuadTree<T>);
                entry.child_index += 1;
                result
              }
            },
            _ => {
              // entry.child_index = 4;
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
    match maybe_node {
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
    self.visiting.pop();
    if self.visiting.len() > 0 {
      self.step_down();
    }
  }

  fn step_back_up(&mut self) {
    // println!("step_back_up");
    loop {
      // (*self.visit_pop_back)();
      self.visiting_back.pop();
      match self.visiting_back.last() {
        Some(ref entry) => {
          if entry.child_index < 4 {
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
    // println!("check_end");
    let is_end = match self.visiting.last() {
      Some(ref front_entry) => {
        match self.visiting_back.last() {
          Some(ref back_entry) => {
            let front_ptr = &*front_entry.node as *const QuadTree<T>;
            let back_ptr = &*back_entry.node as *const QuadTree<T>;
            // println!("end? {} {} {} {}", front_ptr == back_ptr, back_entry.node.children.is_some(), front_entry.child_index, back_entry.child_index);
            front_ptr == back_ptr &&
              (
                front_entry.node.children.is_some() &&
                front_entry.child_index == 4 - back_entry.child_index ||
                front_entry.node.children.is_none()
              )
          },
          _ => {false},
        }
      },
      _ => {false},
    };
    if is_end {
      while self.visiting.len() > 0 {
        self.visiting.pop();
      }
      while self.visiting_back.len() > 0 {
        self.visiting_back.pop();
      }
    }
  }
}

impl<'a, T> Iterator for QuadTreeIterMut<'a, T> {
  type Item = &'a mut QuadTree<T>;

  fn next(&mut self) -> Option<Self::Item> {
    if !self.first {
      self.check_end();
      self.step_up();
    }
    else {
      self.first = false;
    }
    let result : Option<&'a mut QuadTree<T>> = match self.visiting.last_mut() {
      Some(ref mut entry) => {
        unsafe {
          let ptr = entry.node as *mut QuadTree<T>; 
          Some(&mut *ptr as &'a mut QuadTree<T>)
        }
      },
      _ => {None},
    };
    result
  }
}

impl<'a, T> DoubleEndedIterator for QuadTreeIterMut<'a, T> {
  fn next_back(&mut self) -> Option<Self::Item> {
    if !self.first_back {
      self.check_end();
      self.step_back_down();
    }
    else {
      self.first_back = false;
    }
    let result = match self.visiting_back.last_mut() {
      Some(ref mut entry) => {
        unsafe {
          let ptr = entry.node as *mut QuadTree<T>; 
          Some(&mut *ptr as &'a mut QuadTree<T>)
        }
      },
      _ => {None},
    };
    result
  }
}

// impl<'a, 'b, T> QuadTreeBackRef<'a, 'b, T> {
//   fn new(iter: &'b mut QuadTreeRefIterMut<'a, T>) -> QuadTreeRef<'a, 'b, T> {
//     QuadTreeRef<'a, 'b, T> {
//       iter: iter,
//     }
//   }
//
//   fn iter_parents_mut(&mut self) -> iter::Skip<iter::Rev<iter::Map<slice::Iter<&'a mut T>, fn(&mut &'a mut T) -> &'a mut T>>> {
//     fn deref<'a, T>(t: &mut &'a mut T) -> &'a mut T {
//       unsafe {&mut *(&mut **t as *mut T) as &'a mut T}
//     }
//     self.iter.visiting_back.iter_mut().map(deref).rev().skip(1)
//   }
// }
//
// impl<'a, 'b, T> Deref for QuadTreeBackRef<'a, 'b, T> {
//   fn deref(&mut self) -> &'a mut T {
//     self.iter.visiting_back.last_mut().unwrap()
//   }
// }
//
// impl<'a, T> QuadTreeRefIterMut<'a, T> {
//   fn new(node: &'a mut QuadTree<T>) -> QuadTreeRefIterMut<'a, T> {
//     let mut iter = QuadTreeRefIterMut::<T> {
//       // visit_push: &push_noop,
//       // visit_pop: &pop_noop,
//       // visit_push_back: &push_noop,
//       // visit_pop_back: &pop_noop,
//       first: true,
//       first_back: true,
//       visiting: Vec::<QuadTreeIterMutEntry<T>>::new(),
//       visiting_back: Vec::<QuadTreeIterMutEntry<T>>::new(),
//     };
//     unsafe {
//       let ptr = &mut *node as *mut QuadTree<T>;
//       iter.push_node(&mut *ptr as &'a mut QuadTree<T>);
//       iter.push_node_back(&mut *ptr as &'a mut QuadTree<T>);
//     }
//     iter
//   }
//
//   fn push_node(&mut self, node: &'a mut QuadTree<T>) {
//     let child_index = match node.children {
//       Some(_) => {0},
//       _ => {0},
//     };
//     // (*self.visit_push)(&mut node);
//     self.visiting.push(QuadTreeIterMutEntry::<T> {
//       node: node,
//       child_index: child_index,
//     });
//     self.step_down();
//   }
//
//   fn push_node_back(&mut self, node: &'a mut QuadTree<T>) {
//     // println!("push_node_back");
//     let child_index = match node.children {
//       Some(_) => {0},
//       _ => {0},
//     };
//     // (*self.visit_push_back)(&mut node);
//     self.visiting_back.push(QuadTreeIterMutEntry::<T> {
//       node: node,
//       child_index: child_index,
//     });
//   }
//
//   fn step_down(&mut self) {
//     let maybe_node = match self.visiting.last_mut() {
//       Some(ref mut entry) => {
//         if entry.child_index < 4 {
//           match entry.node.children {
//             Some(ref mut children) => {
//               unsafe {
//                 let ptr = &mut children[entry.child_index] as *mut QuadTree<T>;
//                 let result = Some(&mut *ptr as &'a mut QuadTree<T>);
//                 entry.child_index += 1;
//                 result
//               }
//             },
//             _ => {
//               // entry.child_index = 0;
//               None
//             },
//           }
//         }
//         else {
//           None
//         }
//       },
//       _ => {None},
//     };
//     match maybe_node {
//       Some(node) => {
//         self.push_node(node);
//       },
//       _ => {},
//     }
//   }
//
//   fn step_back_down(&mut self) {
//     // println!("step_back_down");
//     let maybe_node = match self.visiting_back.last_mut() {
//       Some(ref mut entry) => {
//         // println!("child_index {}", entry.child_index);
//         if entry.child_index < 4 {
//           // println!("children: {}", entry.node.children.is_some());
//           match entry.node.children {
//             Some(ref mut children) => {
//               unsafe {
//                 let ptr = &mut children[3 - entry.child_index] as *mut QuadTree<T>;
//                 let result = Some(&mut *ptr as &'a mut QuadTree<T>);
//                 entry.child_index += 1;
//                 result
//               }
//             },
//             _ => {
//               // entry.child_index = 4;
//               None
//             },
//           }
//         }
//         else {
//           None
//         }
//       },
//       _ => {None},
//     };
//     match maybe_node {
//       Some(node) => {
//         self.push_node_back(node);
//       },
//       _ => {
//         self.step_back_up();
//       },
//     }
//   }
//
//   fn step_up(&mut self) {
//     // (*self.visit_pop)();
//     self.visiting.pop();
//     if self.visiting.len() > 0 {
//       self.step_down();
//     }
//   }
//
//   fn step_back_up(&mut self) {
//     // println!("step_back_up");
//     loop {
//       // (*self.visit_pop_back)();
//       self.visiting_back.pop();
//       match self.visiting_back.last() {
//         Some(ref entry) => {
//           if entry.child_index < 4 {
//             break;
//           }
//         },
//         _ => {break;},
//       }
//     }
//     if self.visiting_back.len() > 0 {
//       self.step_back_down();
//     }
//   }
//
//   fn check_end(&mut self) {
//     // println!("check_end");
//     let is_end = match self.visiting.last() {
//       Some(ref front_entry) => {
//         match self.visiting_back.last() {
//           Some(ref back_entry) => {
//             let front_ptr = &*front_entry.node as *const QuadTree<T>;
//             let back_ptr = &*back_entry.node as *const QuadTree<T>;
//             // println!("end? {} {} {} {}", front_ptr == back_ptr, back_entry.node.children.is_some(), front_entry.child_index, back_entry.child_index);
//             front_ptr == back_ptr &&
//               (
//                 front_entry.node.children.is_some() &&
//                 front_entry.child_index == 4 - back_entry.child_index ||
//                 front_entry.node.children.is_none()
//               )
//           },
//           _ => {false},
//         }
//       },
//       _ => {false},
//     };
//     if is_end {
//       while self.visiting.len() > 0 {
//         self.visiting.pop();
//       }
//       while self.visiting_back.len() > 0 {
//         self.visiting_back.pop();
//       }
//     }
//   }
// }
//
// impl<'a, T> Iterator for QuadTreeRefIterMut<'a, T> {
//   type Item = &'a mut QuadTree<T>;
//
//   fn next(&mut self) -> Option<Self::Item> {
//     if !self.first {
//       self.check_end();
//       self.step_up();
//     }
//     else {
//       self.first = false;
//     }
//     let result : Option<&'a mut QuadTree<T>> = match self.visiting.last_mut() {
//       Some(ref mut entry) => {
//         unsafe {
//           let ptr = entry.node as *mut QuadTree<T>;
//           Some(&mut *ptr as &'a mut QuadTree<T>)
//         }
//       },
//       _ => {None},
//     };
//     result
//   }
// }
//
// impl<'a, T> DoubleEndedIterator for QuadTreeRefIterMut<'a, T> {
//   fn next_back(&mut self) -> Option<Self::Item> {
//     if !self.first_back {
//       self.check_end();
//       self.step_back_down();
//     }
//     else {
//       self.first_back = false;
//     }
//     let result = match self.visiting_back.last_mut() {
//       Some(ref mut entry) => {
//         unsafe {
//           let ptr = entry.node as *mut QuadTree<T>;
//           Some(&mut *ptr as &'a mut QuadTree<T>)
//         }
//       },
//       _ => {None},
//     };
//     result
//   }
// }

impl<'a, T> QuadTreeIterMutFilter<'a, T> {
  fn new(node: &'a mut QuadTree<T>, filter: &'a Fn(&QuadTree<T>) -> bool) -> QuadTreeIterMutFilter<'a, T> {
    let mut iter = QuadTreeIterMutFilter::<T> {
      // visit_push: &push_noop,
      // visit_pop: &pop_noop,
      // visit_push_back: &push_noop,
      // visit_pop_back: &pop_noop,
      first: true,
      first_back: true,
      visiting: Vec::<QuadTreeIterMutEntry<T>>::new(),
      visiting_back: Vec::<QuadTreeIterMutEntry<T>>::new(),
      filter_tree: filter,
    };
    unsafe {
      let ptr = &mut *node as *mut QuadTree<T>;
      iter.push_node(&mut *ptr as &'a mut QuadTree<T>);
      iter.push_node_back(&mut *ptr as &'a mut QuadTree<T>);
    }
    iter
  }

  fn push_node(&mut self, node: &'a mut QuadTree<T>) {
    let child_index = match node.children {
      Some(_) => {0},
      _ => {0},
    };
    // (*self.visit_push)(&mut node);
    let filtered = (self.filter_tree)(&*node);
    if filtered {
      self.visiting.push(QuadTreeIterMutEntry::<T> {
        node: node,
        child_index: child_index,
      });
      self.step_down();
    }
    else if self.visiting.len() > 0 {
      self.check_end();
      self.step_down();
    }
  }

  fn push_node_back(&mut self, node: &'a mut QuadTree<T>) {
    // println!("push_node_back");
    let child_index = match node.children {
      Some(_) => {0},
      _ => {0},
    };
    // (*self.visit_push_back)(&mut node);
    let filtered = (self.filter_tree)(&*node);
    if filtered {
      self.visiting_back.push(QuadTreeIterMutEntry::<T> {
        node: node,
        child_index: child_index,
      });
    }
    else if self.visiting_back.len() > 0 {
      self.check_end();
      self.step_back_down();
    }
  }

  fn step_down(&mut self) {
    let maybe_node = match self.visiting.last_mut() {
      Some(ref mut entry) => {
        if entry.child_index < 4 {
          match entry.node.children {
            Some(ref mut children) => {
              unsafe {
                let ptr = &mut children[entry.child_index] as *mut QuadTree<T>;
                let result = Some(&mut *ptr as &'a mut QuadTree<T>);
                entry.child_index += 1;
                result
              }
            },
            _ => {
              // entry.child_index = 0;
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
    match maybe_node {
      Some(node) => {
        self.push_node(node);
      },
      _ => {},
    }
  }

  fn step_back_down(&mut self) {
    // println!("step_back_down");
    let maybe_node = match self.visiting_back.last_mut() {
      Some(ref mut entry) => {
        // println!("child_index {}", entry.child_index);
        if entry.child_index < 4 {
          // println!("children: {}", entry.node.children.is_some());
          match entry.node.children {
            Some(ref mut children) => {
              unsafe {
                let ptr = &mut children[3 - entry.child_index] as *mut QuadTree<T>;
                let result = Some(&mut *ptr as &'a mut QuadTree<T>);
                entry.child_index += 1;
                result
              }
            },
            _ => {
              // entry.child_index = 4;
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
    match maybe_node {
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
    self.visiting.pop();
    if self.visiting.len() > 0 {
      self.step_down();
    }
  }

  fn step_back_up(&mut self) {
    // println!("step_back_up");
    loop {
      // (*self.visit_pop_back)();
      self.visiting_back.pop();
      match self.visiting_back.last() {
        Some(ref entry) => {
          if entry.child_index < 4 {
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
    // println!("check_end");
    let is_end = match self.visiting.last() {
      Some(ref front_entry) => {
        match self.visiting_back.last() {
          Some(ref back_entry) => {
            let front_ptr = &*front_entry.node as *const QuadTree<T>;
            let back_ptr = &*back_entry.node as *const QuadTree<T>;
            // println!("end? {} {} {} {}", front_ptr == back_ptr, back_entry.node.children.is_some(), front_entry.child_index, back_entry.child_index);
            front_ptr == back_ptr &&
              (
                front_entry.node.children.is_some() &&
                front_entry.child_index == 4 - back_entry.child_index ||
                front_entry.node.children.is_none()
              )
          },
          _ => {false},
        }
      },
      _ => {false},
    };
    if is_end {
      while self.visiting.len() > 0 {
        self.visiting.pop();
      }
      while self.visiting_back.len() > 0 {
        self.visiting_back.pop();
      }
    }
  }
}

impl<'a, T> Iterator for QuadTreeIterMutFilter<'a, T> {
  type Item = &'a mut QuadTree<T>;

  fn next(&mut self) -> Option<Self::Item> {
    if !self.first {
      self.check_end();
      self.step_up();
    }
    else {
      self.first = false;
    }
    let result : Option<&'a mut QuadTree<T>> = match self.visiting.last_mut() {
      Some(ref mut entry) => {
        unsafe {
          let ptr = entry.node as *mut QuadTree<T>; 
          Some(&mut *ptr as &'a mut QuadTree<T>)
        }
      },
      _ => {None},
    };
    result
  }
}

impl<'a, T> DoubleEndedIterator for QuadTreeIterMutFilter<'a, T> {
  fn next_back(&mut self) -> Option<Self::Item> {
    if !self.first_back {
      self.check_end();
      self.step_back_down();
    }
    else {
      self.first_back = false;
    }
    let result = match self.visiting_back.last_mut() {
      Some(ref mut entry) => {
        unsafe {
          let ptr = entry.node as *mut QuadTree<T>; 
          Some(&mut *ptr as &'a mut QuadTree<T>)
        }
      },
      _ => {None},
    };
    result
  }
}

#[test]
fn test_deeper_quad_tree_iter_filter_mut() {
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
      QuadTree::<i32> {
        value: 3,
        children: Some(Box::new([
          QuadTree::<i32>::new(9),
          QuadTree::<i32>::new(10),
          QuadTree::<i32>::new(11),
          QuadTree::<i32>::new(12),
        ])),
      },
      QuadTree::<i32>::new(4),
    ])),
  };
  for node in tree.iter_filter_mut(&|node| node.value % 2 == 0) {
    node.value += 1;
  }
  {
    let mut iter = tree.iter();
    assert_eq!(iter.next().unwrap().value, 1);
    assert_eq!(iter.next().unwrap().value, 5);
    assert_eq!(iter.next().unwrap().value, 7);
    assert_eq!(iter.next().unwrap().value, 7);
    assert_eq!(iter.next().unwrap().value, 9);
    assert_eq!(iter.next().unwrap().value, 3);
    assert_eq!(iter.next().unwrap().value, 9);
    assert_eq!(iter.next().unwrap().value, 10);
    assert_eq!(iter.next().unwrap().value, 11);
    assert_eq!(iter.next().unwrap().value, 12);
    assert_eq!(iter.next().unwrap().value, 3);
    assert_eq!(iter.next().unwrap().value, 5);
    assert_eq!(iter.next().unwrap().value, 1);
    assert_eq!(iter.next().is_none(), true);
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

  pub fn last_mut(&mut self) -> Option<&mut T> {
    if let Some(item) = self.list.back_mut() {
      Some(unsafe { &mut **item })
    }
    else {
      None
    }
  }

  pub fn iter(&self) -> iter::Map<linked_list::Iter<&'a mut T>, fn(& &'a mut T) -> &'a T> {
    fn deref<'a, T>(t: &&'a mut T) -> &'a T {
      unsafe {& *(& **t as *const T) as &'a T}
    }
    self.list.iter().map(deref)
  }

  pub fn iter_mut(&mut self) -> iter::Map<linked_list::IterMut<&'a mut T>, fn(&mut &'a mut T) -> &'a mut T> {
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
    let child_index = match node.children {
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
      child_index: child_index,
    });
    self.step_down();
  }

  fn push_node_back(&mut self, node: &'a mut QuadTree<T>) {
    let child_index = match node.children {
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
      child_index: child_index,
    });
  }

  fn step_down(&mut self) {
    let maybe_node = match self.visiting.back_mut() {
      Some(ref mut entry) => {
        if entry.child_index < 4 {
          match entry.node.children {
            Some(ref mut children) => {
              unsafe {
                let ptr = &mut children[entry.child_index] as *mut QuadTree<T>;
                let result = Some(&mut *ptr as &'a mut QuadTree<T>);
                entry.child_index += 1;
                result
              }
            },
            _ => {
              entry.child_index = 4;
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
    match maybe_node {
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
    let maybe_node = match self.visiting_back.back_mut() {
      Some(ref mut entry) => {
        if entry.child_index < 4 {
          match entry.node.children {
            Some(ref mut children) => {
              unsafe {
                let ptr = &mut children[3 - entry.child_index] as *mut QuadTree<T>;
                let result = Some(&mut *ptr as &'a mut QuadTree<T>);
                entry.child_index += 1;
                result
              }
            },
            _ => {
              entry.child_index = 4;
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
    match maybe_node {
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
          if entry.child_index < 4 {
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
            let front_ptr = &*front_entry.node as *const QuadTree<T>;
            let back_ptr = &*back_entry.node as *const QuadTree<T>;
            front_ptr == back_ptr
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
fn test_quad_tree_iter_stack() {
  let mut tree = QuadTree::<i32> {
    value: 0,
    children: Some(Box::new([
      QuadTree::<i32>::new(1),
      QuadTree::<i32>::new(2),
      QuadTree::<i32>::new(3),
      QuadTree::<i32>::new(4),
    ])),
  };
  {
    let mut iter_stack = tree.iter_stack_mut();
    {
      let mut iter = iter_stack.next().unwrap().iter();
      assert_eq!(*(iter.next().unwrap()), 0);
      assert_eq!(*(iter.next().unwrap()), 1);
      assert_eq!(iter.next().is_none(), true);
    }
    {
      let mut iter = iter_stack.next().unwrap().iter();
      assert_eq!(*iter.next().unwrap(), 0);
      assert_eq!(*iter.next().unwrap(), 2);
      assert_eq!(iter.next().is_none(), true);
    }
    {
      let mut iter = iter_stack.next().unwrap().iter();
      assert_eq!(*iter.next().unwrap(), 0);
      assert_eq!(*iter.next().unwrap(), 3);
      assert_eq!(iter.next().is_none(), true);
    }
    {
      let mut iter = iter_stack.next().unwrap().iter();
      assert_eq!(*iter.next().unwrap(), 0);
      assert_eq!(*iter.next().unwrap(), 4);
      assert_eq!(iter.next().is_none(), true);
    }
    {
      let mut iter = iter_stack.next().unwrap().iter();
      assert_eq!(*iter.next().unwrap(), 0);
      assert_eq!(iter.next().is_none(), true);
    }
    assert_eq!(iter_stack.next().is_none(), true);
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

impl TreeSplitable for i32 {
  fn tree_split(&mut self, children: &mut [&mut i32]) {
    *children[0] = *self / 4 + *self % 4;
    *children[1] = *self / 4;
    *children[2] = *self / 4;
    *children[3] = *self / 4;
    *self = 0;
  }
}

impl TreeJoinable for i32 {
  fn tree_join(&mut self, children: &[&i32]) {
    *self = *children[0] + *children[1] + *children[2] + *children[3];
  }
}

impl TreeSplitableWith for i32 {
  fn tree_split_with(&mut self, children: &mut [&mut i32], _: &mut Any) {
    *children[0] = *self / 4 + *self % 4;
    *children[1] = *self / 4;
    *children[2] = *self / 4;
    *children[3] = *self / 4;
    *self = 0;
  }
}

impl TreeJoinableWith for i32 {
  fn tree_join_with(&mut self, children: &[&i32], _: &mut Any) {
    *self = *children[0] + *children[1] + *children[2] + *children[3];
  }
}

#[test]
fn test_quad_tree_split() {
  let mut tree = QuadTree::<i32>::new(8);
  {
    let mut iter = tree.iter_mut();
    iter.next().unwrap().split();
  }
  // tree.split();
  let mut iter = tree.iter();
  assert_eq!(iter.next().unwrap().value, 2);
  assert_eq!(iter.next().unwrap().value, 2);
  assert_eq!(iter.next().unwrap().value, 2);
  assert_eq!(iter.next().unwrap().value, 2);
  assert_eq!(iter.next().unwrap().value, 0);
  assert_eq!(iter.next().is_none(), true);
}

#[test]
fn test_quad_tree_split_in_iter() {
  let mut tree = QuadTree::<i32>::new(8);
  {
    let mut iter = tree.iter_mut().rev();
    iter.next().unwrap().split();
    assert_eq!(iter.next().unwrap().value, 2);
    assert_eq!(iter.next().unwrap().value, 2);
    assert_eq!(iter.next().unwrap().value, 2);
    assert_eq!(iter.next().unwrap().value, 2);
  }
  // tree.split();
  let mut iter = tree.iter();
  assert_eq!(iter.next().unwrap().value, 2);
  assert_eq!(iter.next().unwrap().value, 2);
  assert_eq!(iter.next().unwrap().value, 2);
  assert_eq!(iter.next().unwrap().value, 2);
  assert_eq!(iter.next().unwrap().value, 0);
  assert_eq!(iter.next().is_none(), true);
}

#[test]
fn test_quad_tree_join() {
  let mut tree = QuadTree::<i32> {
    value: 0,
    children: Some(Box::new([
      QuadTree::<i32>::new(1),
      QuadTree::<i32>::new(2),
      QuadTree::<i32>::new(3),
      QuadTree::<i32>::new(4),
    ])),
  };
  tree.join();
  let mut iter = tree.iter();
  assert_eq!(iter.next().unwrap().value, 10);
  assert_eq!(iter.next().is_none(), true);
}

#[test]
fn test_quad_tree_split_with() {
  let mut tree = QuadTree::<i32>::new(8);
  {
    let mut iter = tree.iter_mut();
    iter.next().unwrap().split_with(&mut 0 as &mut i32);
  }
  // tree.split();
  let mut iter = tree.iter();
  assert_eq!(iter.next().unwrap().value, 2);
  assert_eq!(iter.next().unwrap().value, 2);
  assert_eq!(iter.next().unwrap().value, 2);
  assert_eq!(iter.next().unwrap().value, 2);
  assert_eq!(iter.next().unwrap().value, 0);
  assert_eq!(iter.next().is_none(), true);
}

#[test]
fn test_quad_tree_join_with() {
  let mut tree = QuadTree::<i32> {
    value: 0,
    children: Some(Box::new([
      QuadTree::<i32>::new(1),
      QuadTree::<i32>::new(2),
      QuadTree::<i32>::new(3),
      QuadTree::<i32>::new(4),
    ])),
  };
  tree.join_with(&mut 0 as &mut i32);
  let mut iter = tree.iter();
  assert_eq!(iter.next().unwrap().value, 10);
  assert_eq!(iter.next().is_none(), true);
}
