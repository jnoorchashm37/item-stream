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

pub struct ItemStream<STREAM, STEP, FUTS> {
    stream: STREAM,
    step_fn: STEP,
    futures_handler: FUTS,
}

impl<STREAM, STEP, FUTS, O, F> ItemStream<STREAM, STEP, FUTS>
where
    STREAM: Stream,
    STEP: FnMut(STREAM::Item) -> F,
    FUTS: FuturesArray<F>,
    F: Future<Output = O> + 'static,
{
    pub fn new(stream: STREAM, step_fn: STEP, futures_handler: FUTS) -> Self {
        Self {
            stream,
            step_fn,
            futures_handler,
        }
    }

    pub fn add_to_stream(&mut self, item: STREAM::Item) {
        self.futures_handler.push_fut((self.step_fn)(item));
    }
}

impl<STREAM, STEP, O, F> ItemStream<STREAM, STEP, FuturesOrdered<F>>
where
    STREAM: Stream,
    STEP: FnMut(STREAM::Item) -> F,
    F: Future<Output = O> + 'static,
{
    pub fn new_ordered(stream: STREAM, step_fn: STEP) -> Self {
        Self {
            stream,
            step_fn,
            futures_handler: FuturesOrdered::new(),
        }
    }
}

impl<STREAM, STEP, O, F> ItemStream<STREAM, STEP, FuturesUnordered<F>>
where
    STREAM: Stream,
    STEP: FnMut(STREAM::Item) -> F,
    F: Future<Output = O> + 'static,
{
    pub fn new_unordered(stream: STREAM, step_fn: STEP) -> Self {
        Self {
            stream,
            step_fn,
            futures_handler: FuturesUnordered::new(),
        }
    }
}

impl<STREAM, STEP, FUTS, O, F> Stream for ItemStream<STREAM, STEP, FUTS>
where
    STREAM: Stream + Unpin,
    STEP: FnMut(STREAM::Item) -> F + Unpin,
    FUTS: FuturesArray<F> + Unpin,
    F: Future<Output = O> + 'static,
{
    type Item = O;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        while let Poll::Ready(stream_out_opt) = this.stream.poll_next_unpin(cx) {
            if let Some(stream_out) = stream_out_opt {
                this.futures_handler.push_fut((this.step_fn)(stream_out));

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
