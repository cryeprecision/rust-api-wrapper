#![allow(dead_code)]

use crate::model::{request, QueryResult};

use futures::stream::{Stream, StreamExt};
use pin_project::pin_project;
use reqwest::Client;
use std::pin::Pin;
use std::task::{Context, Poll};

#[pin_project]
pub struct QueryStream<'a, St>
where
    St: Stream<Item = QueryResult<'a>>,
{
    #[pin]
    pub stream: St,
}

impl<'a, St> Stream for QueryStream<'a, St>
where
    St: Stream<Item = QueryResult<'a>>,
{
    type Item = QueryResult<'a>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().stream.poll_next(cx)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.size_hint()
    }
}

/// Run each query that the iterator `queries` yields.
///
/// # Concurrency
///
/// Up to 10 queries are run concurrently.
pub fn run_queries<'a>(
    client: &'a Client,
    queries: impl Iterator<Item = &'a str>,
) -> QueryStream<'a, impl Stream<Item = QueryResult<'a>>> {
    QueryStream {
        stream: futures::stream::iter(queries)
            .map(|query| request(client, query))
            .buffered(10),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() {
        let queries = &["laptop", "food", "hd", "perfume"];
        let client = Client::new();

        {
            run_queries(&client, queries.iter().map(|&q| q))
                .for_each(|result| async move {
                    // here, `result` is correctly annotated as type `QueryResult`
                    println!("{:?}", result);
                })
                .await;
        }

        println!("");
        println!("--------------------------------");
        println!("");

        {
            let mut futures = run_queries(&client, queries.iter().map(|&q| q));
            while let Some(result) = futures.next().await {
                // here, `result` is correctly annotated as type `QueryResult`
                println!("{:?}", result);
            }
        }
    }
}
