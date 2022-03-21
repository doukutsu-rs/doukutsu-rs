use std::cell::Ref;

pub trait RSFuture {
    type Output;

    fn poll(&self) -> Option<Ref<Self::Output>>;

    fn is_done(&self) -> bool {
        self.poll().is_some()
    }
}
