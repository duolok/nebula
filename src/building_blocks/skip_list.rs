use rand::Rng;
use std::rc::Rc;
use std::cell::RefCell;
use std::default::Default;

pub struct SkipListNode<T> {
    value: T,
    next_nodes: Vec<Option<Rc<RefCell<SkipListNode<T>>>>>,
}

pub struct SkipList<T> {
    head: Rc<RefCell<SkipListNode<T>>>,
    max_level: usize,
}

impl<T: Ord + Default> SkipList<T> {
    pub fn new() -> Self {
        let max_level = 10; 
        let head = Rc::new(RefCell::new(SkipListNode {
            value: Default::default(),
            next_nodes: vec![None; max_level],
        }));

        SkipList { head, max_level }
    }

    pub fn insert(&mut self, value: T) {
        let level = self.roll();

        let new_node = Rc::new(RefCell::new(SkipListNode {
            value,
            next_nodes: vec![None; level],
        }));

        let mut current = Rc::clone(&self.head);
        let mut update = vec![Rc::clone(&self.head); level];

        for i in (0..level).rev() {
            let mut next_reference = current.borrow().next_nodes[i].as_ref().map(Rc::clone);
            while let Some(next) = next_reference {
                let next_value = &next.borrow().value;
                if next_value < &new_node.borrow().value {
                    current = Rc::clone(&next);
                    next_reference = current.borrow().next_nodes[i].as_ref().map(Rc::clone);
                } else {
                    break;
                }
            }
            update[i] = Rc::clone(&current);
        }

        for i in 0..level {
            let mut new_node_borrow_mut = new_node.borrow_mut();
            new_node_borrow_mut.next_nodes[i] = update[i].borrow_mut().next_nodes[i].take();
            update[i].borrow_mut().next_nodes[i] = Some(Rc::clone(&new_node));
        }
    }

    pub fn contains(&self, value: T) -> bool {
        self.search(value).is_some()
    }

    pub fn search(&self, value: T)  -> Option<Rc<RefCell<SkipListNode<T>>>> {
        let mut current = Rc::clone(&self.head);

        for i in (0..self.max_level).rev() {
                let mut next_reference = current.borrow().next_nodes[i].as_ref().map(Rc::clone);
                while let Some(next) = next_reference {
                    let next_value = &next.borrow().value;
                    if next_value < &value {
                        current = Rc::clone(&next);
                        next_reference = current.borrow().next_nodes[i].as_ref().map(Rc::clone);
                    } else if next_value == &value {
                        return Some(Rc::clone(&next));
                    } else {
                        break;
                    }
                }
            }
        None
    }


    pub fn delete(&mut self, value: T) {
        let mut current = Rc::clone(&self.head);
        let mut update = vec![Rc::clone(&self.head); self.max_level];

        for i in (0..self.max_level).rev() {
            let mut next_reference = current.borrow().next_nodes[i].as_ref().map(Rc::clone);
            while let Some(next) = next_reference {
                let next_value = &next.borrow().value;
                if next_value < &value {
                    current = Rc::clone(&next);
                    next_reference = current.borrow().next_nodes[i].as_ref().map(Rc::clone);
                } else {
                    break;
                }
            }
            update[i] = Rc::clone(&current);
        }

        let target = current.borrow().next_nodes[0].as_ref().map(Rc::clone);
        if let Some(node_to_delete) = target {
            if node_to_delete.borrow().value == value {
                for i in 0..self.max_level {
                    let previous = &update[i];
                    let mut previous_next = previous.borrow_mut().next_nodes[i].take();

                    if let Some(next) = &previous_next {
                        if Rc::ptr_eq(&node_to_delete, next) {
                            previous_next = node_to_delete.borrow_mut().next_nodes[i].take();
                        }
                    }
                    previous.borrow_mut().next_nodes[i] = previous_next;
                }
            }
        }
    }

    fn roll(&mut self) -> usize {
        let mut level = 1;
        let probability = 0.5; 
        let mut rng = rand::thread_rng();
        
        while rng.gen::<f64>() < probability && level < self.max_level {
            level += 1;
        }

        level
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_correct_initial_state_test() {
        let skip_list: SkipList<i32> = SkipList::new();

        let head_node = skip_list.head.borrow();
        assert_eq!(head_node.value, Default::default());
        assert_eq!(head_node.next_nodes.len(), 10); 

        for next_node in &head_node.next_nodes {
            assert!(next_node.is_none());
        }

        assert_eq!(skip_list.max_level, 10);
    }

    #[test]
    fn insert_test() {
        let mut skip_list = SkipList::new();

        skip_list.insert(3);
        skip_list.insert(7);
        skip_list.insert(1);
        skip_list.insert(9);

        assert_eq!(skip_list.contains(5), false);
        assert_eq!(skip_list.contains(7),true);
        assert_eq!(skip_list.contains(11), false);
        assert_eq!(skip_list.contains(1), true);
    }

    #[test]
    fn delete_test() {
        let mut skip_list = SkipList::new();

        skip_list.insert(3);
        skip_list.insert(7);
        skip_list.insert(1);
        skip_list.insert(9);

        skip_list.delete(7);
        skip_list.delete(1);

        assert_eq!(skip_list.contains(7), false);
        assert_eq!(skip_list.contains(3), true);
        assert_eq!(skip_list.contains(1), false);
        assert_eq!(skip_list.contains(9), true);
    }

    #[test]
    fn search_existing_value_test() {
        let mut skip_list = SkipList::new();
        skip_list.insert(10);
        skip_list.insert(15);
        skip_list.insert(20);

        assert_eq!(skip_list.search(10).map(|node| node.borrow().value), Some(10));
        assert_eq!(skip_list.search(15).map(|node| node.borrow().value), Some(15));
        assert_eq!(skip_list.search(20).map(|node| node.borrow().value), Some(20));
    }

    #[test]
    fn search_non_existing_value_test() {
        let mut skip_list = SkipList::new();
        skip_list.insert(10);
        skip_list.insert(15);
        skip_list.insert(20);

        assert_eq!(skip_list.search(5).map(|node| node.borrow().value), None);
        assert_eq!(skip_list.search(25).map(|node| node.borrow().value), None);
        assert_eq!(skip_list.search(100).map(|node| node.borrow().value), None);
    }
}
