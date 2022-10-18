use std::future::Future;

use futures_signals::signal::{self, Signal, SignalExt};

// TODO: Doc
pub struct Sig<T>(pub T);

// TODO: Doc
// TODO: Do we need the sized constraints?
// TODO: Do we want a seperate `RefSignalOrValue` and `SignalOrValue`?
pub trait RefSignalOrValue<'a> {
    type Item: 'a;
    type Map<'b, F, R>: RefSignalOrValue<'b, Item = R> + 'b
    where
        'b: 'a,
        F: FnMut(Self::Item) -> R + 'b,
        R: RefSignalOrValue<'b, Item = R> + 'b;

    fn map<'b: 'a, F, R>(self, callback: F) -> Self::Map<'b, F, R>
    where
        R: RefSignalOrValue<'b, Item = R> + 'b,
        F: FnMut(Self::Item) -> R + 'b,
        Self: Sized;

    fn for_each<FVal, FInitSig, FSig, Task, Exec>(
        self,
        fn_val: FVal,
        fn_init_sig: FInitSig,
        executor: &mut Exec,
    ) where
        FVal: FnOnce(&mut Exec, Self::Item),
        FInitSig: FnOnce(&mut Exec) -> FSig,
        FSig: FnMut(Self::Item) -> Task + 'a,
        Task: Future<Output = ()> + 'a,
        Exec: Executor,
        Self: Sized;
}

pub trait SignalOrValue: RefSignalOrValue<'static> {}

impl<T: RefSignalOrValue<'static>> SignalOrValue for T {}

pub trait Executor {
    fn spawn(&mut self, future: impl Future<Output = ()> + 'static);
}

pub trait RefValue<'a> {}

pub trait Value: RefValue<'static> {}

impl<T: Value> RefValue<'static> for T {}

macro_rules! static_values{
    ($($t:ty),*) => {
        $(
            impl Value for $t {}
        )*
    }
}

static_values!(i8, i16, i32, i64);
static_values!(u8, u16, u32, u64);
static_values!(f32, f64);
static_values!(bool, String);

impl<'a> RefValue<'a> for &'a str {}
impl<'a> RefValue<'a> for &'a String {}
impl<'a, T: 'a> RefValue<'a> for Option<T> {}
impl<'a, T: 'a> RefValue<'a> for [T] {}
impl<'a, const COUNT: usize, T: 'a> RefValue<'a> for [T; COUNT] {}

impl<'a, T> RefSignalOrValue<'a> for T
where
    T: RefValue<'a> + 'a,
{
    type Item = Self;
    type Map<'b, F, R> = R
    where
        'b: 'a,
        F: FnMut(Self::Item) -> R + 'b,
        R: RefSignalOrValue<'b, Item = R> + 'b;

    fn map<'b: 'a, F, R>(self, mut callback: F) -> Self::Map<'b, F, R>
    where
        R: RefSignalOrValue<'b, Item = R> + 'b,
        F: FnMut(Self::Item) -> R + 'b,
        Self: Sized,
    {
        callback(self)
    }

    fn for_each<FVal, FInitSig, FSig, Task, Exec>(
        self,
        fn_val: FVal,
        _fn_init_sig: FInitSig,
        executor: &mut Exec,
    ) where
        FVal: FnOnce(&mut Exec, Self::Item),
        FInitSig: FnOnce(&mut Exec) -> FSig,
        FSig: FnMut(Self::Item) -> Task + 'a,
        Task: Future<Output = ()> + 'a,
        Exec: Executor,
        Self: Sized,
    {
        fn_val(executor, self);
    }
}

impl<T, S> RefSignalOrValue<'static> for Sig<S>
where
    T: 'static,
    S: Signal<Item = T> + 'static,
{
    type Item = T;
    type Map<'b, F, R> = Sig<signal::Map<S, F>>
    where
        'b: 'static,
        F: FnMut(Self::Item) -> R + 'b,
        R: RefSignalOrValue<'b, Item = R> + 'b;

    fn map<'b: 'static, F, R>(self, callback: F) -> Self::Map<'b, F, R>
    where
        'b: 'static,
        R: RefSignalOrValue<'b, Item = R> + 'b,
        F: FnMut(Self::Item) -> R + 'b,
        Self: Sized,
    {
        Sig(self.0.map(callback))
    }

    fn for_each<FVal, FInitSig, FSig, Task, Exec>(
        self,
        _fn_val: FVal,
        fn_init_sig: FInitSig,
        executor: &mut Exec,
    ) where
        FVal: FnOnce(&mut Exec, Self::Item),
        FInitSig: FnOnce(&mut Exec) -> FSig,
        FSig: FnMut(Self::Item) -> Task + 'static,
        Task: Future<Output = ()> + 'static,
        Exec: Executor,
        Self: Sized,
    {
        let fn_sig = fn_init_sig(executor);
        executor.spawn(self.0.for_each(fn_sig));
    }
}