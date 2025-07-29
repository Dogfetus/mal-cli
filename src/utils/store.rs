// for single sorce of truth
#![allow(dead_code)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub trait Storable {
    type Id: Eq + std::hash::Hash + Copy;

    fn get_id(&self) -> Self::Id;
}

#[derive(Debug, Clone)]
pub struct Store<T: Storable> {
    data: Rc<RefCell<HashMap<T::Id, T>>>,
}

impl<T: Storable> Store<T> {
    pub fn new() -> Self {
        Store {
            data: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn get_list(&self) -> Vec<T>
    where
        T: Clone,
    {
        self.data.borrow().values().cloned().collect()
    }

    pub fn get(&self, id: &T::Id) -> Option<T>
    where
        T: Clone,
    {
        self.data.borrow().get(id).cloned()
    }

    pub fn get_bulk(&self, ids: Vec<T::Id>) -> Vec<T>
    where
        T: Clone,
    {
        let data = self.data.borrow();
        ids.iter()
            .filter_map(|id| data.get(id).cloned())
            .collect()
    }

    pub fn add(&mut self, data: T) {
        self.data.borrow_mut().insert(data.get_id(), data);
    }

    pub fn add_bulk(&mut self, data: Vec<T>) {
        for item in data {
            self.data.borrow_mut().insert(item.get_id(), item);
        }
    }

    pub fn update(&mut self, id: T::Id, f: impl FnOnce(&mut T))
    where
        T: Clone,
    {
        if let Some(item) = self.data.borrow_mut().get_mut(&id) {
            f(item);
        }
    }

    pub fn remove(&mut self, id: T) -> Option<T> {
        self.data.borrow_mut().remove(&id.get_id())
    }

    // pub fn update<F>(&mut self, updater: F)
    // where
    //     F: FnOnce(&mut Vec<T>),
    // {
    // }
}
