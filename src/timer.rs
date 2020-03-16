
use futures_01::future::Future as Future01;
use futures_01::stream::Stream as Stream01;
use futures_01::Poll as Poll01;
use futures_util::compat::Compat;
use futures_util::stream::Stream;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio_02::time::Delay as Delay2;
use tokio_02::time::Instant;
use tokio_02::time::Interval as Interval2;

struct PhantomError<S, E> {
    phantom: PhantomData<fn() -> E>,
    inner: S,
}

impl<S, E> PhantomError<S, E> {
    fn get_mut(&mut self) -> &mut S {
        &mut self.inner
    }
}

impl<S, E> futures_util::stream::Stream for PhantomError<S, E>
where
    S: Stream + Unpin,
{
    type Item = Result<S::Item, E>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> std::task::Poll<Option<Self::Item>> {
        let inner = Pin::into_inner(self);
        let stream = Pin::new(&mut inner.inner);
        match stream.poll_next(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Some(val)) => Poll::Ready(Some(Ok(val))),
            Poll::Ready(None) => Poll::Ready(None),
        }
    }
}

impl<F, E> std::future::Future for PhantomError<F, E>
where
    F: std::future::Future + Unpin,
{
    type Output = Result<F::Output, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> std::task::Poll<Self::Output> {
        let inner = Pin::into_inner(self);
        let stream = Pin::new(&mut inner.inner);
        match stream.poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(val) => Poll::Ready(Ok(val)),
        }
    }
}

pub struct Interval(Compat<PhantomError<Interval2, tokio_timer_02::Error>>);

impl Interval {
    pub fn new_interval(duration: std::time::Duration) -> Self {
        Interval(Compat::new(PhantomError {
            inner: tokio_02::time::interval(duration),
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

pub struct Delay {
    inner: Compat<PhantomError<Delay2, tokio_timer_02::Error>>,
}

impl Delay {
    pub fn new(instant: Instant) -> Delay {
        Delay {
            inner: Compat::new(PhantomError {
                inner: tokio_02::time::delay_until(instant),
                phantom: PhantomData,
            }),
        }
    }

    pub fn reset(&mut self, deadline: Instant) {
        self.inner.get_mut().get_mut().reset(deadline)
    }
}

impl Future01 for Delay {
    type Item = ();
    type Error = tokio_timer_02::Error;

    fn poll(&mut self) -> Poll01<Self::Item, Self::Error> {
        self.inner.poll()
    }
}
