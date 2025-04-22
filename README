# Item-Stream

Allows for producing asyncronous values from the outputs of streams. Creates an abstraction over the Stream + Futures(Un)Ordered fields of a struct.

```
pub struct ItemStream<STREAM, STEP, FUTS> {
    stream: STREAM,
    step_fn: STEP,
    futures_handler: FUTS,
    stream_dead: bool,
}

impl<STREAM, STEP, FUTS, O, F> Stream for ItemStream<STREAM, STEP, FUTS>
where
    STREAM: Stream + Unpin,
    STEP: FnMut(STREAM::Item) -> F + Unpin,
    FUTS: FuturesArray<F> + Unpin,
    F: Future<Output = O>,
{
    type Item = O;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        if !this.stream_dead {
            while let Poll::Ready(stream_out_opt) = this.stream.poll_next_unpin(cx) {
                if let Some(stream_out) = stream_out_opt {
                    this.futures_handler.push_fut((this.step_fn)(stream_out));

                    if let Poll::Ready(Some(fut_out)) = this.futures_handler.poll_next_unpin(cx) {
                        return Poll::Ready(Some(fut_out));
                    }
                } else {
                    this.stream_dead = true;
                    break;
                }
            }
        } else if this.futures_handler.is_empty() {
            return Poll::Ready(None);
        }

        if let Poll::Ready(Some(fut_out)) = this.futures_handler.poll_next_unpin(cx) {
            return Poll::Ready(Some(fut_out));
        }

        Poll::Pending
    }
}

```