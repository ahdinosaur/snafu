#![allow(missing_docs)] // FIXME

use std::{
    future::Future,
    pin::Pin,
    task::{Context as TaskContext, Poll},
};
use IntoError;

pub trait FutureExt<T, E>
where
    Self: Future<Output = Result<T, E>>,
    Self: Sized,
{
    fn context<C>(self, context: C) -> Context<Self, C>
    where
        C: IntoError<SourceError = E>;
}

impl<Fut, T, E> FutureExt<T, E> for Fut
where
    Self: Future<Output = Result<T, E>>,
{
    fn context<C>(self, context: C) -> Context<Self, C>
    where
        C: IntoError<SourceError = E>,
    {
        Context {
            inner: self,
            context: Some(context),
        }
    }
}

#[derive(Debug)]
#[must_use = "futures do nothing unless polled"]
pub struct Context<Fut, C> {
    inner: Fut,
    context: Option<C>,
}

impl<Fut, C, E, T> Future for Context<Fut, C>
where
    Fut: Future<Output = Result<T, E>>,
    C: IntoError<SourceError = E>,
{
    type Output = Result<T, C::Error>;

    fn poll(mut self: Pin<&mut Self>, ctx: &mut TaskContext) -> Poll<Self::Output> {
        // FIXME doc safety
        unsafe {
            let inner = self.as_mut().map_unchecked_mut(|slf| &mut slf.inner);
            let inner_res = inner.poll(ctx);

            inner_res.map_err(|error| {
                self.get_unchecked_mut()
                    .context
                    .take()
                    .expect("Cannot poll Context after it resolves")
                    .into_error(error)
            })
        }
    }
}
