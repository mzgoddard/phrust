use std::any::Any;

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
  pub parent: Option<*mut QuadTree<T>>,
  pub children: Option<Box<[QuadTree<T>; 4]>>,
}

struct PointerIterEntry<T> {
  node: *const QuadTree<T>,
  child: *const QuadTree<T>,
  end: *const QuadTree<T>,
}

#[allow(dead_code)]
pub struct Iter<'a, T: 'a> {
  start: Option<&'a QuadTree<T>>,
  visiting: Vec<PointerIterEntry<T>>,
  visiting_back: Vec<PointerIterEntry<T>>,
}

impl<T> QuadTree<T> {
  pub fn new(value: T) -> QuadTree<T> {
    QuadTree::<T> {
      value: value,
      parent: None,
      children: None,
    }
  }

  pub fn is_leaf(&self) -> bool {
    self.children.is_none()
  }

  pub fn iter(&self) -> Iter<T> {
    Iter::<T>::new(self)
  }
}

impl<T> QuadTree<T> where T: TreeSplitable + TreeJoinable + Default {
  pub fn split(&mut self) {
    let parent_ptr = Some(&mut *self as *mut QuadTree<T>);
    self.children = Some(Box::new([
      QuadTree::<T> {
        value: T::default(),
        parent: parent_ptr,
        children: None,
      },
      QuadTree::<T> {
        value: T::default(),
        parent: parent_ptr,
        children: None,
      },
      QuadTree::<T> {
        value: T::default(),
        parent: parent_ptr,
        children: None,
      },
      QuadTree::<T> {
        value: T::default(),
        parent: parent_ptr,
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
    let parent_ptr = Some(&mut *self as *mut QuadTree<T>);
    self.children = Some(Box::new([
      QuadTree::<T> {
        value: T::default(),
        parent: parent_ptr,
        children: None,
      },
      QuadTree::<T> {
        value: T::default(),
        parent: parent_ptr,
        children: None,
      },
      QuadTree::<T> {
        value: T::default(),
        parent: parent_ptr,
        children: None,
      },
      QuadTree::<T> {
        value: T::default(),
        parent: parent_ptr,
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
          if let Some(_) = unsafe { &*child }.children {
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
    parent: None,
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
    parent: None,
    children: Some(Box::new([
      QuadTree::<i32>::new(1),
      QuadTree::<i32> {
        value: 2,
        parent: None,
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
    parent: None,
    children: Some(Box::new([
      QuadTree::<i32>::new(1),
      QuadTree::<i32> {
        value: 2,
        parent: None,
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
    parent: None,
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
    parent: None,
    children: Some(Box::new([
      QuadTree::<i32>::new(1),
      QuadTree::<i32> {
        value: 2,
        parent: None,
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
    parent: None,
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
    parent: None,
    children: Some(Box::new([
      QuadTree::<i32>::new(1),
      QuadTree::<i32> {
        value: 2,
        parent: None,
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
  tree.split();
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
  let tree_p = &mut tree as *mut QuadTree<i32>;
  {
    let mut iter = tree.iter().rev();
    iter.next();
    unsafe { (*tree_p).split(); }
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
    parent: None,
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
  tree.split_with(&mut 0 as &mut i32);
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
    parent: None,
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
