use crate::{Invoke, Lifetime, MacroInvoke, Push, TypeParam, TypedIndex, ValueNode};
use std::cell::{Cell, RefCell};
use std::thread::LocalKey;

thread_local! {
    pub(crate) static VALUES: RefCell<Vec<ValueNode>> = RefCell::new(Vec::new());
    pub(crate) static INVOKES: RefCell<Vec<Invoke>> = RefCell::new(Vec::new());
    pub(crate) static MACROS: RefCell<Vec<MacroInvoke>> = RefCell::new(Vec::new());
    pub(crate) static TYPE_PARAMS: Cell<usize> = Cell::new(0);
    pub(crate) static LIFETIMES: Cell<usize> = Cell::new(1);
}

pub(crate) const STATIC_LIFETIME: Lifetime = Lifetime(0);

pub(crate) trait GlobalBorrow<T> {
    fn with_borrow<R, F>(&'static self, f: F) -> R
    where
        F: FnOnce(&Vec<T>) -> R;

    fn with_borrow_mut<R, F>(&'static self, f: F) -> R
    where
        F: FnOnce(&mut Vec<T>) -> R;
}

impl<T> GlobalBorrow<T> for LocalKey<RefCell<Vec<T>>> {
    fn with_borrow<R, F>(&'static self, f: F) -> R
    where
        F: FnOnce(&Vec<T>) -> R,
    {
        self.with(|data| f(&*data.borrow()))
    }

    fn with_borrow_mut<R, F>(&'static self, f: F) -> R
    where
        F: FnOnce(&mut Vec<T>) -> R,
    {
        self.with(|data| f(&mut *data.borrow_mut()))
    }
}

pub(crate) trait GlobalPush {
    type Element: TypedIndex;

    fn index_push(&'static self, element: Self::Element) -> <Self::Element as TypedIndex>::Index;
}

impl<T> GlobalPush for LocalKey<RefCell<Vec<T>>>
where
    T: TypedIndex,
{
    type Element = T;

    fn index_push(&'static self, element: T) -> T::Index {
        self.with(|data| data.borrow_mut().index_push(element))
    }
}

pub(crate) trait GlobalCounter<T> {
    fn count(&'static self) -> T;
}

impl GlobalCounter<Lifetime> for LocalKey<Cell<usize>> {
    fn count(&'static self) -> Lifetime {
        self.with(|counter| {
            let count = counter.get();
            counter.set(count + 1);
            Lifetime(count)
        })
    }
}

impl GlobalCounter<TypeParam> for LocalKey<Cell<usize>> {
    fn count(&'static self) -> TypeParam {
        self.with(|counter| {
            let count = counter.get();
            counter.set(count + 1);
            TypeParam(count)
        })
    }
}

pub(crate) fn clear() {
    // It's not safe to reset TYPE_PARAMS and LIFETIMES as this might
    // interfere with cached values in generic parameters in functions in the
    // reflect! macro
    VALUES.with(|data| data.borrow_mut().clear());
    INVOKES.with(|data| data.borrow_mut().clear());
    MACROS.with(|data| data.borrow_mut().clear());
}
