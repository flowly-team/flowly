use std::marker::PhantomData;

use futures::Stream;

use crate::{Context, Service};

// const SPAWN_EACH_MAX_TASKS: usize = 256;

// struct SpawnEachTask<I: Send, S: Service<I>> {
//     tx: flowly_spsc::Sender<I>,
//     _handle: tokio::task::JoinHandle<()>,
//     m: PhantomData<S>,
// }

// impl<I, S> SpawnEachTask<I, S>
// where
//     S::Out: Send + 'static,
//     I: Send + 'static,
//     S: Service<I> + Send + 'static,
// {
//     fn new(mut s: S, out_tx: mpsc::Sender<S::Out>, cx: Context, input: I) -> Self {
//         let (mut tx, mut rx) = flowly_spsc::channel(1);
//         tx.try_send(input).unwrap();

//         Self {
//             tx,
//             _handle: tokio::spawn(async move {
//                 println!("  recv");
//                 'recv: while let Some(item) = rx.recv().await {
//                     println!("  recv got");
//                     let mut s = pin!(s.handle(item, &cx));

//                     while let Some(x) = s.next().await {
//                         println!("  send");
//                         if let Err(..) = out_tx.send(x).await {
//                             log::error!("cannot send the message. channel closed!");
//                             break 'recv;
//                         }
//                     }
//                 }
//             }),
//             m: PhantomData,
//         }
//     }
// }

pub struct SpawnEach<I: Send + 'static, S: Service<I>> {
    service: S,
    // sender: mpsc::Sender<S::Out>,
    // receiver: Mutex<mpsc::Receiver<S::Out>>,
    // tasks: Vec<SpawnEachTask<I, S>>,
    _m: PhantomData<I>,
}

impl<I, S> SpawnEach<I, S>
where
    I: Send,
    S: Service<I> + Send,
    S::Out: Send,
{
    pub(crate) fn new(service: S) -> Self {
        // let (sender, rx) = mpsc::channel(1);

        Self {
            service,
            _m: PhantomData,
            // sender,
            // receiver: Mutex::new(rx),
            // tasks: Vec::new(),
        }
    }

    //     #[inline]
    //     fn drain_rx(&mut self) -> impl Stream<Item = S::Out> + Send {
    //         futures::stream::poll_fn(move |cx| {
    //             if let Some(mut lock) = self.receiver.try_lock().ok() {
    //                 Poll::Ready(match lock.poll_recv(cx) {
    //                     Poll::Ready(Some(val)) => Some(val),
    //                     _ => None,
    //                 })
    //             } else {
    //                 Poll::Ready(None)
    //             }
    //         })
    //     }
}

impl<I, S> Service<I> for SpawnEach<I, S>
where
    I: Send,
    S: Service<I> + Clone + Send + 'static,
    S::Out: Send,
{
    type Out = S::Out;

    fn handle(&mut self, input: I, cx: &Context) -> impl Stream<Item = Self::Out> + Send {
        self.service.handle(input, cx)
        // for task in &mut self.tasks {
        //     if let Err(err) = task.tx.try_send(input) {
        //         input = err.val;
        //     } else {
        //         return self.drain_rx().right_stream();
        //     }
        // }

        // if self.tasks.len() < SPAWN_EACH_MAX_TASKS {
        //     self.tasks.push(SpawnEachTask::new(
        //         self.service.clone(),
        //         self.sender.clone(),
        //         cx.clone(),
        //         input,
        //     ));

        //     self.drain_rx().right_stream()
        // } else {
        //     // random task index
        //     let index = (std::time::SystemTime::now()
        //         .duration_since(UNIX_EPOCH)
        //         .unwrap_or_default()
        //         .as_nanos()
        //         % self.tasks.len() as u128) as usize;

        //     async move {
        //         if let Err(..) = self.tasks[index].tx.send(input).await {
        //             log::error!("cannot send the message. channel closed!");
        //         }

        //         self.drain_rx()
        //     }
        //     .into_stream()
        //     .flatten()
        //     .left_stream()
        // }
    }
}

// pub struct Spawn<S> {
//     pub(crate) service: S,
// }

// pub struct SpawnLocal<S> {
//     pub(crate) service: S,
// }

// impl<I: Send, S: Service<I> + Send + 'static> Service<I> for Spawn<S>
// where
//     S::Out: Send,
// {
//     type Out = S::Out;

//     #[inline]
//     fn handle(&mut self, input: I, cx: &Context) -> impl Stream<Item = S::Out> {
//         let _ = cx;
//         let _ = input;

//         unimplemented!();

//         futures::stream::empty()
//         // let (tx, mut rx) = mpsc::channel(self.buffer);

//         // let fut = Box::pin(async move {
//         //     let mut stream = pin!(self.service.handle(input));

//         //     while let Some(item) = stream.next().await {
//         //         if (tx.send(item).await).is_err() {
//         //             break;
//         //         }
//         //     }
//         // }) as Pin<Box<dyn Future<Output = ()> + Send>>;

//         // // SAFTY:
//         // // This is safe because:
//         // //  - input stream will be dropped as soon as it drained.
//         // //  - it is garateed that input lives as long as it contains items
//         // tokio::spawn(unsafe {
//         //     std::mem::transmute::<
//         //         Pin<Box<dyn Future<Output = ()> + Send>>,
//         //         Pin<Box<dyn Future<Output = ()> + Send + 'static>>,
//         //     >(fut)
//         // });

//         // poll_fn(move |cx| rx.poll_recv(cx))
//     }
// }
