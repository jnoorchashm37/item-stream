use std::task::Poll;

use futures::StreamExt;

use crate::ItemStream;

#[tokio::test(flavor = "multi_thread")]
async fn test_ordered() {
    let values = [0, 1, 2, 3, 4];
    let stream = futures::stream::iter(values);
    let step_fn = async |i| (i, i * 2);

    let mut item_stream = ItemStream::new_ordered(stream, step_fn);

    std::future::poll_fn(|cx| {
        loop {
            if let Poll::Ready(val) = item_stream.poll_next_unpin(cx) {
                if let Some((idx, stream_out)) = val {
                    assert_eq!(values[idx], stream_out / 2);
                } else {
                    break Poll::Ready(());
                }
            }
        }
    })
    .await;
}

#[tokio::test]
async fn test_unordered() {
    let values = [0, 1, 2, 3, 4];
    let stream = futures::stream::iter(values);
    let step_fn = async |i| (i, i * 2);

    let mut item_stream = ItemStream::new_unordered(stream, step_fn);

    std::future::poll_fn(|cx| {
        loop {
            if let Poll::Ready(val) = item_stream.poll_next_unpin(cx) {
                if let Some((idx, stream_out)) = val {
                    assert_eq!(values[idx], stream_out / 2);
                } else {
                    break Poll::Ready(());
                }
            }
        }
    })
    .await;
}

#[tokio::test]
async fn test_pushback() {
    let values = [0, 1, 2, 3, 4];
    let stream = futures::stream::iter(values);
    let step_fn = async |i| {
        if i % 2 == 0 {
            let out: Result<(i32, i32), i32> = Ok((i, i * 2));
            out
        } else {
            let out: Result<(i32, i32), i32> = Err(i);
            out
        }
    };

    let mut item_stream = ItemStream::new_unordered(stream, step_fn);

    std::future::poll_fn(|cx| {
        loop {
            if let Poll::Ready(val) = item_stream.poll_next_unpin(cx) {
                if let Some(Ok((idx, stream_out))) = val {
                    if idx >= 10 {
                        assert_eq!(values[(idx / 10) as usize], stream_out / 10 / 2);
                    } else {
                        assert_eq!(values[idx as usize], stream_out / 2);
                    }
                } else if let Some(Err(i)) = val {
                    item_stream.add_to_stream(i * 10);
                } else {
                    break Poll::Ready(());
                }
            }
        }
    })
    .await;
}
