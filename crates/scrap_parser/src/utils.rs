use std::ops::{Deref, DerefMut};

use thin_vec::ThinVec;

#[derive(Debug, Clone)]
/// A local vector type that uses a thin vector for storage.
pub struct LocalVec<T>(ThinVec<T>);

impl<T> Default for LocalVec<T> {
    fn default() -> Self {
        LocalVec(ThinVec::new())
    }
}

impl<T> LocalVec<T> {
    pub fn new() -> Self {
        LocalVec(ThinVec::new())
    }
}

impl<T> Deref for LocalVec<T> {
    type Target = ThinVec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for LocalVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> chumsky::container::Container<T> for LocalVec<T> {
    fn with_capacity(n: usize) -> Self {
        LocalVec(ThinVec::with_capacity(n))
    }

    fn push(&mut self, item: T) {
        self.0.push(item);
    }
}
