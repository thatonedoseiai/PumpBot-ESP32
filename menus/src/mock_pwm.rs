use std::marker::PhantomData;

pub struct OutputCtl<'a> {
    _marker: PhantomData<&'a ()>,
}
