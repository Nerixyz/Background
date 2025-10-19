pub trait OptionExt<T> {
    fn or_assign(&mut self, other: Option<T>);
}

impl<T> OptionExt<T> for Option<T> {
    fn or_assign(&mut self, other: Option<T>) {
        if self.is_none() {
            *self = other;
        }
    }
}
