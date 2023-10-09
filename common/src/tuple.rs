pub trait MappableTuple<T> {
    type Ret<R>;
    fn map<F: Fn(&T) -> R, R>(&self, f: F) -> Self::Ret<R>;
}

impl<T> MappableTuple<T> for (T, T) {
    type Ret<R> = (R, R);

    fn map<F: Fn(&T) -> R, R>(&self, f: F) -> Self::Ret<R> {
        (f(&self.0), f(&self.0))
    }
}
