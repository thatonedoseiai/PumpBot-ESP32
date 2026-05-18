use std::marker::PhantomData;

pub struct OutputCtl<'a> {
    _marker: PhantomData<&'a ()>,
}

impl OutputCtl<'_> {
    pub fn new() -> Self {
        OutputCtl {
            _marker: PhantomData
        }
    }
}
