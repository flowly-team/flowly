mod and_then;
mod concurrent_each;
mod except;
mod inspect;
mod map;
mod pass;
mod scope;
mod spawn_each;
mod stub;
mod switch;

pub use and_then::and_then;
pub use concurrent_each::{ConcurrentEach, concurrent_each};
use flowly_core::Either;
pub use map::{filter_map, map, map_if_else, try_filter_map, try_map};
pub use pass::flow;
pub use spawn_each::{SpawnEach, spawn_each};
pub use stub::stub;
pub use switch::switch;
use tokio::sync::watch;

use std::{marker::PhantomData, pin::pin};

use futures::{Stream, StreamExt, future};

pub use crate::except::Except;
pub use crate::scope::{Scope, ScopeEach, scope, scope_each};

#[derive(Clone)]
#[non_exhaustive]
pub struct Context {
    pub abort: watch::Sender<bool>,
    pub abort_recv: watch::Receiver<bool>,
}

impl Context {
    pub async fn fuse_abort<F: Future>(&self, fut: F) -> Option<F::Output> {
        let mut abort_recv = self.abort_recv.clone();
        let fut1 = pin!(abort_recv.changed());
        let fut2 = pin!(fut);

        match futures::future::select(fut1, fut2).await {
            future::Either::Left(..) => None,
            future::Either::Right((val, _)) => Some(val),
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl From<watch::Sender<bool>> for Context {
    fn from(abort: watch::Sender<bool>) -> Self {
        Self {
            abort_recv: abort.subscribe(),
            abort,
        }
    }
}

impl Context {
    pub fn new() -> Self {
        Self::from(watch::Sender::default())
    }
}

pub trait Service<In> {
    type Out;

    fn handle(&mut self, input: In, cx: &Context) -> impl Stream<Item = Self::Out> + Send;
    fn handle_stream(
        &mut self,
        input: impl Stream<Item = In> + Send,
        cx: &Context,
    ) -> impl Stream<Item = Self::Out> + Send
    where
        In: Send,
        Self: Send,
        Self::Out: Send,
    {
        async_stream::stream! {
            let mut input = pin!(input);

            while let Some(item) = input.next().await {
                let mut s = pin!(self.handle(item, cx));

                while let Some(out) = s.next().await {
                    yield out;
                }
            }
        }
    }

    #[inline]
    fn finalize(&mut self, _cx: &Context) -> impl Future<Output = ()>
    where
        Self: Sized,
    {
        async move {}
    }
}

impl<I, O1, E1, O2, E2, S1, S2> Service<I> for (S1, S2)
where
    I: Send,
    O1: Send,
    O2: Send,
    E1: Send,
    E2: Send,
    S1: Service<I, Out = Result<O1, E1>> + Send,
    S2: Service<O1, Out = Result<O2, E2>> + Send,
{
    type Out = Result<O2, Either<E1, E2>>;

    fn handle(&mut self, msg: I, cx: &Context) -> impl Stream<Item = Self::Out> + Send {
        async_stream::stream! {
            let mut s1 = pin!(self.0.handle(msg, cx));

            while let Some(res) = s1.next().await {
                match res {
                    Ok(ok) => {
                        let mut s2 = pin!(self.1.handle(ok, cx));

                        while let Some(i2) = s2.next().await {
                            yield i2.map_err(Either::Right);
                        }
                    },
                    Err(err) => yield Err(Either::Left(err)),
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct Left<S1, S2>(S1, S2);
impl<I, O1, E, O2, S1, S2> Service<I> for Left<S1, S2>
where
    I: Send,
    O1: Send,
    O2: Send,
    E: Send,
    S1: Service<I, Out = Result<O1, E>> + Send,
    S2: Service<O1, Out = O2> + Send,
{
    type Out = Result<O2, E>;

    fn handle(&mut self, msg: I, cx: &Context) -> impl Stream<Item = Self::Out> + Send {
        async_stream::stream! {
            let mut s1 = pin!(self.0.handle(msg, cx));

            while let Some(res) = s1.next().await {
                match res {
                    Ok(ok) => {
                        let mut s2 = pin!(self.1.handle(ok, cx));

                        while let Some(i2) = s2.next().await {
                            yield Ok(i2);
                        }
                    },
                    Err(err) => yield Err(err),
                }
            }
        }
    }
}

pub trait ServiceExt<I: Send>: Service<I> {
    #[inline]
    fn flow<O1, O2, E1, E2, U>(self, service: U) -> (Self, U)
    where
        Self: Sized + Service<I, Out = Result<O1, E1>> + Send,
        U: Send + Service<O1, Out = Result<O2, E2>>,
        O1: Send,
        O2: Send,
        E1: Send,
        E2: Send,
    {
        (self, service)
    }

    #[inline]
    fn except<F>(self, on_err: F) -> Except<Self, F>
    where
        Self: Sized,
    {
        Except {
            service: self,
            on_err,
        }
    }

    /// Adds an inspection step that invokes the supplied callback on every
    /// successful output of the wrapped service.
    ///
    /// This method returns a 2‑tuple consisting of:
    /// * `self` – the original service unchanged.
    /// * an `inspect::Inspect<O, E, F>` instance that intercepts the
    ///   service’s output. For each successful result (`Ok(o)`), the
    ///   closure `f` is called with a reference to `o`. The output is then
    ///   passed through unchanged.
    ///
    /// # Parameters
    ///
    /// * `f` – A callback implementing `Fn(&O)`. The callback receives a
    ///   reference to the successful output value. It can be used for
    ///   logging, metrics, or any side‑effect‑only operation.
    ///
    /// # Return value
    ///
    /// A tuple `(Self, inspect::Inspect<O, E, F>)` that can be used in a
    /// service pipeline (e.g., within the `flow` combinator). The first
    /// element is the original service, and the second element is a service
    /// that performs the inspection.
    ///
    /// # Example
    ///
    /// ```rust
    /// use flowly_service::{Service, flow::Flow, inspect::Inspect};
    ///
    /// let service = MyService::new();
    /// let (orig, inspector) = service.flow_inspect(|value: &Result<i32, _>| {
    ///     println!("Got value: {:?}", value);
    /// });
    /// let flow = Flow::from(orig).and(inspector);
    /// ```
    #[inline]
    fn flow_inspect<O, E, F>(self, f: F) -> Left<Self, inspect::Inspect<O, F>>
    where
        Self: Sized + Service<I, Out = Result<O, E>> + Send,
        F: Fn(&O) + Send,
        O: Send,
    {
        Left(
            self,
            inspect::Inspect::<O, F> {
                cb: f,
                _m: PhantomData,
            },
        )
    }

    /// Creates a concurrent wrapper around the current service that limits the number of
    /// parallel executions.
    ///
    /// This method returns a `ConcurrentEach<I, Self>` instance that delegates work to a pool
    /// of worker tasks. Each worker runs the underlying service independently, allowing
    /// multiple inputs to be processed concurrently. The `limit` argument controls the
    /// maximum number of worker tasks that may run in parallel.
    ///
    /// **Parameters**
    /// - `self`: The service instance to be wrapped. It must implement `Service<I>` and
    ///   satisfy `Send`, `Clone`, and `'static` bounds.
    /// - `limit`: The maximum number of concurrent worker tasks to spawn. If `limit` is
    ///   greater than the current number of tasks, new tasks will be created up to this
    ///   bound.
    ///
    /// **Return value**
    /// A `ConcurrentEach<I, Self>` which itself implements `Service`. When handling an
    /// input, it forwards the input to one of the available workers and returns a stream
    /// of results that can be awaited asynchronously.
    ///
    #[inline]
    fn concurrent_each(self, limit: usize) -> ConcurrentEach<I, Self>
    where
        Self: Sized + Send + Clone + 'static,
        Self::Out: Send,
    {
        ConcurrentEach::new(self, limit)
    }

    /// Creates a new [`SpawnEach`] wrapper around the current service.
    ///
    /// The wrapper spawns a separate task for each input message, forwarding
    /// the results through a bounded `mpsc` channel. This allows the
    /// underlying service to process messages concurrently without
    /// blocking the caller.
    ///
    /// # Parameters
    /// * `self` – The service instance to wrap. The service must implement
    ///   `Service<I>` for some input type `I`.
    ///
    /// # Constraints
    /// * `Self: Sized + Send + Clone + 'static` – The service must be
    ///   clonable and safe to send across threads.
    /// * `Self::Out: Send` – The output type of the service must be
    ///   `Send` because it will be transported across channels.
    ///
    /// # Return value
    /// Returns a [`SpawnEach<I, Self>`] that implements `Service<I>` with
    /// the same input type. The new service can be used just like the
    /// original one, but each invocation of `handle` will spawn a
    /// dedicated task.
    ///
    /// # Example
    /// ```rust
    /// use flowly_service::{Service, spawn_each};
    ///
    /// struct MyService;
    /// impl Service<u32> for MyService {
    ///     type Out = Result<String, std::io::Error>;
    ///     fn handle(&mut self, input: u32, _cx: &crate::Context)
    ///         -> impl futures::Stream<Item = Self::Out> + Send
    ///     {
    ///         // …
    ///     }
    /// }
    ///
    /// let service = MyService;
    /// // Wrap in SpawnEach
    /// let concurrent_service = service.spawn_each();
    /// // Now `concurrent_service` can be used as a Service and will process
    /// // each input concurrently.
    /// ```
    ///
    /// # Note
    /// The default message buffer size is 2.
    #[inline]
    fn spawn_each(self) -> SpawnEach<I, Self>
    where
        Self: Sized + Send + Clone + 'static,
        Self::Out: Send,
    {
        SpawnEach::new(self, 2)
    }

    /// Creates a scoped service wrapper that transforms incoming messages before passing to the wrapped service.
    ///
    /// This method consumes the current service and returns a tuple containing the original service and a new
    /// [`Scope`] service that forwards transformed messages to `s`. The transformation function `f` receives
    /// a reference to the original input `O` and returns either a message `M` for `s` or an error `E1`.\
    ///
    /// # Type Parameters
    /// * `O`: Type of the original input that will be received by the outer service.
    /// * `M`: Type of the message that `s` expects.
    /// * `E1`: Error type returned by the transformation function `f`.
    /// * `S`: The inner service that will handle the transformed messages.
    /// * `F`: Function or closure of type `Fn(&O) -> Result<M, E1>`.
    ///
    /// # Parameters
    /// * `self` – The current service (moved into the returned tuple).  
    /// * `f` – Function that transforms `&O` into `Result<M, E1>`.  
    /// * `s` – The inner service to be invoked after successful transformation.  
    ///
    /// # Returns
    /// A tuple `(Self, Scope<O, M, E1, S, F>)` where:\n
    /// * `Self` is the original service that can continue to be used.\n
    /// * `Scope<O, M, E1, S, F>` is a new service that:\n
    ///   1. Calls `f` with the incoming input.\n
    ///   2. If `f` returns `Ok(m)`, forwards `m` to `s` and collects all emitted outputs into `Vec<O>`.\n
    ///   3. If `f` returns `Err(e)`, immediately returns an error wrapped in `Either::Right(e)` without invoking `s`.\n
    ///
    /// # Example
    /// ```ignore
    /// let (service, scoped) = flow_scope(service, |msg: &Input| {
    ///     if msg.valid { Ok(transformed_msg) } else { Err(TransformError) }
    /// }, inner_service);
    /// ```
    ///
    /// # Constraints
    /// All involved types must be `Send`, and `Self` must implement `Sized + Send`.
    #[inline]
    fn flow_scope<O, M, E1, S, F>(self, f: F, s: S) -> (Self, Scope<O, M, E1, S, F>)
    where
        F: Fn(&O) -> Result<M, E1>,
        Self: Sized + Send,
        O: Send,
        E1: Send,
    {
        (self, scope(f, s))
    }

    #[inline]
    fn flow_scope_each<O, M, E1, S, F>(self, f: F, s: S) -> (Self, ScopeEach<O, M, E1, S, F>)
    where
        F: Fn(&O) -> Result<M, E1>,
        Self: Sized + Send,
        E1: Send,
        O: Send + Clone,
    {
        (self, scope_each(f, s))
    }

    #[inline]
    fn flow_map<O1, O2, E1, F, H>(self, f: F) -> Left<Self, map::Map<O2, F>>
    where
        Self: Sized + Service<I, Out = Result<O1, E1>> + Send,
        F: FnMut(O1) -> H + Send,
        H: Future<Output = O2> + Send,
        O1: Send,
        O2: Send,
        E1: Send,
    {
        Left(self, map::map::<O2, _>(f))
    }

    #[inline]
    fn flow_filter_map<O1, O2, E1, F, H>(self, f: F) -> Left<Self, map::FilterMap<O2, F>>
    where
        Self: Sized + Service<I, Out = Result<O1, E1>> + Send,
        O1: Send,
        O2: Send,
        E1: Send,
        F: FnMut(O1) -> H + Send,
        H: Future<Output = Option<O2>> + Send,
    {
        Left(self, map::filter_map::<O2, _>(f))
    }
}

impl<I: Send, T: Service<I>> ServiceExt<I> for T {}

impl<I: Send, E, S: Service<I, Out = Result<I, E>>> Service<I> for Option<S> {
    type Out = Result<I, E>;

    fn handle(&mut self, input: I, cx: &Context) -> impl Stream<Item = Self::Out> + Send {
        if let Some(srv) = self {
            srv.handle(input, cx).left_stream()
        } else {
            futures::stream::once(async move { Ok(input) }).right_stream()
        }
    }
}
