use crate::prelude::Global;

pub trait Accessor<'scope> {

}

impl<'scope> Accessor<'scope> for Global<'scope> {
    
}

pub(crate) mod private {
    use std::cell::RefCell;

    use crate::{private::Private, memory::ledger::Ledger};

    pub trait AccessorPriv<'scope> {
        fn ledger(&self, _: Private) -> &'scope RefCell<Ledger>;
    }

    
}