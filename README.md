# Item-Stream

Allows for producing asyncronous values from the outputs of streams via functions. An abstraction over the Stream + Futures(Un)Ordered fields of a struct.

Similiar to `.then()` on streams values, however the addition to `ARGS` allows the bypassing of `'static` borrows inside the closure via `clone()`

```rust
pub struct ItemStream<STREAM, STEP, FUTS, ARGS> {
    stream: STREAM,
    step_fn: STEP,
    futures_handler: FUTS,
    args: ARGS,
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

```