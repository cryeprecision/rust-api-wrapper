#![allow(dead_code)]

use crate::model::{request, QueryResult};

use futures::stream::{Stream, StreamExt};
use reqwest::Client;

/// Run each query that the iterator `queries` yields.
///
/// # Concurrency
///
/// Up to 10 queries are run concurrently.
pub fn run_queries<'a>(
    client: &'a Client,
    queries: impl Iterator<Item = &'a str>,
) -> impl Stream<Item = QueryResult<'a>> {
    futures::stream::iter(queries)
        .map(|query| request(client, query))
        .buffered(10)
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
                // here, `result` is annotated as type `{unknown}`

                // although, rust-analyzer doesn't know the type, the compiler can figure it out
                // let _: () = result; // expected `()`, found struct `QueryResult`

                println!("{:?}", result);
            }
        }
    }
}
