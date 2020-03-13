use futures_01::future::Future as Future01;
use futures_util::{compat::Future01CompatExt, future::FutureExt};
use std::cell::RefCell;
use tokio_02::runtime::Handle;
use tokio_executor_01 as executor_01;

use super::idle;

#[derive(Clone, Debug)]
pub(super) struct CompatSpawner<S> {
    pub(super) inner: S,
    pub(super) idle: idle::Idle,
}

struct CompatGuards {
    _executor: executor_01::DefaultGuard,
}

thread_local! {
    static COMPAT_GUARDS: RefCell<Option<CompatGuards>> = RefCell::new(None);
}

pub(super) fn set_guards(executor: impl executor_01::Executor + 'static) {
    let guards = CompatGuards {
        _executor: executor_01::set_default(executor),
    };

    COMPAT_GUARDS.with(move |current| {
        let prev = current.borrow_mut().replace(guards);
        assert!(
            prev.is_none(),
            "default tokio 0.1 runtime set twice; this is a bug"
        );
    });
}

pub(super) fn unset_guards() {
    let _ = COMPAT_GUARDS.try_with(move |current| {
        drop(current.borrow_mut().take());
    });
}

// === impl CompatSpawner ===

impl<T> CompatSpawner<T> {
    pub(super) fn new(inner: T, idle: &idle::Idle) -> Self {
        Self {
            inner,
            idle: idle.clone(),
        }
    }
}

impl<'a> executor_01::Executor for CompatSpawner<&'a Handle> {
    fn spawn(
        &mut self,
        future: Box<dyn futures_01::Future<Item = (), Error = ()> + Send>,
    ) -> Result<(), executor_01::SpawnError> {
        let future = future.compat().map(|_| ());
        let idle = self.idle.reserve();
        self.inner.spawn(idle.with(future));
        Ok(())
    }
}

impl<'a, T> executor_01::TypedExecutor<T> for CompatSpawner<&'a Handle>
where
    T: Future01<Item = (), Error = ()> + Send + 'static,
{
    fn spawn(&mut self, future: T) -> Result<(), executor_01::SpawnError> {
        let idle = self.idle.reserve();
        let future = Box::pin(idle.with(future.compat().map(|_| ())));
        // Use the `tokio` 0.2 `TypedExecutor` impl so we don't have to box the
        // future twice (once to spawn it using `Executor01::spawn` and a second
        // time to pin the compat future).
        self.inner.spawn(future);
        Ok(())
    }
}

impl executor_01::Executor for CompatSpawner<Handle> {
    fn spawn(
        &mut self,
        future: Box<dyn futures_01::Future<Item = (), Error = ()> + Send>,
    ) -> Result<(), executor_01::SpawnError> {
        let future = future.compat().map(|_| ());
        let idle = self.idle.reserve();
        self.inner.spawn(idle.with(future));
        Ok(())
    }
}

impl<T> executor_01::TypedExecutor<T> for CompatSpawner<Handle>
where
    T: Future01<Item = (), Error = ()> + Send + 'static,
{
    fn spawn(&mut self, future: T) -> Result<(), executor_01::SpawnError> {
        let idle = self.idle.reserve();
        let future = Box::pin(idle.with(future.compat().map(|_| ())));
        // Use the `tokio` 0.2 `TypedExecutor` impl so we don't have to box the
        // future twice (once to spawn it using `Executor01::spawn` and a second
        // time to pin the compat future).
        self.inner.spawn(future);
        Ok(())
    }
}
