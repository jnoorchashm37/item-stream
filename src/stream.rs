use futures::{
    StreamExt,
    stream::{FuturesOrdered, FuturesUnordered},
};
use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::Stream;

use crate::FuturesArray;

pub struct ItemStream<STREAM, STEP, FUTS, ARGS> {
    stream: STREAM,
    step_fn: STEP,
    futures_handler: FUTS,
    args: ARGS,
}

impl<STREAM, STEP, FUTS, O, F, ARGS> ItemStream<STREAM, STEP, FUTS, ARGS>
where
    STREAM: Stream,
    STEP: FnMut(STREAM::Item, ARGS) -> F,
    FUTS: FuturesArray<F>,
    F: Future<Output = O>,
{
    pub fn new(stream: STREAM, step_fn: STEP, futures_handler: FUTS, args: ARGS) -> Self {
        Self {
            stream,
            step_fn,
            futures_handler,
            args,
        }
    }

    pub fn add_to_stream(&mut self, item: STREAM::Item, args: ARGS) {
        self.futures_handler.push_fut((self.step_fn)(item, args));
    }
}

impl<STREAM, STEP, O, F, ARGS> ItemStream<STREAM, STEP, FuturesOrdered<F>, ARGS>
where
    STREAM: Stream,
    STEP: FnMut(STREAM::Item, ARGS) -> F,
    F: Future<Output = O>,
{
    pub fn new_ordered(stream: STREAM, step_fn: STEP, args: ARGS) -> Self {
        Self {
            stream,
            step_fn,
            futures_handler: FuturesOrdered::new(),
            args,
        }
    }
}

impl<STREAM, STEP, O, F, ARGS> ItemStream<STREAM, STEP, FuturesUnordered<F>, ARGS>
where
    STREAM: Stream,
    STEP: FnMut(STREAM::Item, ARGS) -> F,
    F: Future<Output = O>,
{
    pub fn new_unordered(stream: STREAM, step_fn: STEP, args: ARGS) -> Self {
        Self {
            stream,
            step_fn,
            futures_handler: FuturesUnordered::new(),
            args,
        }
    }
}

impl<STREAM, STEP, FUTS, O, F, ARGS> Stream for ItemStream<STREAM, STEP, FUTS, ARGS>
where
    STREAM: Stream + Unpin,
    STEP: FnMut(STREAM::Item, ARGS) -> F + Unpin,
    FUTS: FuturesArray<F> + Unpin,
    F: Future<Output = O>,
    ARGS: Clone + Unpin,
{
    type Item = O;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        while let Poll::Ready(stream_out_opt) = this.stream.poll_next_unpin(cx) {
            if let Some(stream_out) = stream_out_opt {
                this.futures_handler
                    .push_fut((this.step_fn)(stream_out, this.args.clone()));

                if let Poll::Ready(Some(fut_out)) = this.futures_handler.poll_next_unpin(cx) {
                    return Poll::Ready(Some(fut_out));
                }
            } else {
                if this.futures_handler.is_empty() {
                    return Poll::Ready(None);
                }
                break;
            }
        }

        if let Poll::Ready(Some(fut_out)) = this.futures_handler.poll_next_unpin(cx) {
            return Poll::Ready(Some(fut_out));
        }

        Poll::Pending
    }
}
