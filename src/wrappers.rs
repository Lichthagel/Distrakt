use std::ops::Deref;

pub struct Wrapper<T>(T);

impl<T> Wrapper<T> {
    pub fn new(content: T) -> Self {
        Self {
            0: content
        }
    }
}

impl<T: 'static> typemap::Key for Wrapper<T> {
    type Value = Wrapper<T>;
}

impl<T> Deref for Wrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}