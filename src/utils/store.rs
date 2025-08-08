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
    data: Rc<RefCell<HashMap<T::Id, Rc<T>>>>,
}

impl<T: Storable> Store<T> {
    pub fn new() -> Self {
        Store {
            data: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn get_list(&self) -> Vec<Rc<T>> {
        self.data.borrow().values().cloned().collect()
    }

    pub fn get(&self, id: &T::Id) -> Option<Rc<T>> {
        self.data.borrow().get(id).cloned()
    }

    pub fn get_bulk<I>(&self, ids: I) -> Vec<Rc<T>>
    where
        I: IntoIterator,
        I::Item: std::borrow::Borrow<T::Id>,
    {
        let data = self.data.borrow();
        ids.into_iter()
            .filter_map(|id| data.get(std::borrow::Borrow::borrow(&id)).cloned())
            .collect()
    }

    pub fn add(&mut self, data: T) {
        self.data.borrow_mut().entry(data.get_id()).or_insert_with(|| Rc::new(data));
    }

    pub fn add_bulk(&mut self, data: Vec<T>) {
        let mut store = self.data.borrow_mut();
        for item in data {
            store.entry(item.get_id()).or_insert_with(|| Rc::new(item));
        }
    }

    pub fn update(&mut self, id: T::Id, f: impl FnOnce(&mut T))
    where
        T: Clone,
    {
        if let Some(item) = self.data.borrow_mut().get_mut(&id) {
            f(Rc::make_mut(item));
        }
    }


    // Return Option<Rc<T>>
    pub fn remove(&mut self, item: &T) -> Option<Rc<T>> {
        self.data.borrow_mut().remove(&item.get_id())
    }
}
