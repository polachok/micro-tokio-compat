mod compat;
mod idle;
pub mod runtime;
mod task_executor;

pub use runtime::Runtime;
pub use task_executor::TaskExecutor;

pub mod timer {
    use futures_01::stream::Stream as Stream01;
    use futures_01::Poll as Poll01;
    use futures_util::compat::Compat;
    use futures_util::stream::Stream;
    use std::marker::PhantomData;
    use std::pin::Pin;
    use std::task::{Context, Poll};
    use tokio_02::time::Interval as Interval2;

    struct PhantomError<S, E> {
        phantom: PhantomData<fn() -> E>,
        stream: S,
    }

    impl<S, E> futures_util::stream::Stream for PhantomError<S, E>
    where
        S: Stream + Unpin,
    {
        type Item = Result<S::Item, E>;

        fn poll_next(
            self: Pin<&mut Self>,
            cx: &mut Context,
        ) -> std::task::Poll<Option<Self::Item>> {
            let inner = Pin::into_inner(self);
            let stream = Pin::new(&mut inner.stream);
            match stream.poll_next(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Some(val)) => Poll::Ready(Some(Ok(val))),
                Poll::Ready(None) => Poll::Ready(None),
            }
        }
    }

    pub struct Interval(Compat<PhantomError<Interval2, tokio_timer_02::Error>>);

    impl Interval {
        pub fn new_interval(duration: std::time::Duration) -> Self {
            Interval(Compat::new(PhantomError {
                stream: tokio_02::time::interval(duration),
                phantom: PhantomData,
            }))
        }
    }

    impl Stream01 for Interval {
        type Item = tokio_02::time::Instant;
        type Error = tokio_timer_02::Error;

        fn poll(&mut self) -> Poll01<Option<Self::Item>, Self::Error> {
            self.0.poll()
        }
    }
}
