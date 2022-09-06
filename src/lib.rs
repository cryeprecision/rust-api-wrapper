#![allow(dead_code)]

use futures::stream::{Stream, StreamExt};
use reqwest::Client;
use serde::Deserialize;

/// The JSON structure for a single product returned from the API.
///
/// # Example
///
/// `{ "id": 11, "title": "perfume Oil", "category": "fragrances", ... }`
#[derive(Deserialize)]
struct Product {
    id: usize,
    title: String,
    category: String,
}

/// The JSON structure returned from the API.
///
/// # Example
///
/// `{ "products": [...], "total": 5, "skip": 0, "limit": 5 }`
#[derive(Debug, Deserialize)]
struct Response {
    total: usize,
    skip: usize,
    limit: usize,
    products: Vec<Product>,
}

/// A query-string and the corresponding result returned from the API.
struct QueryResult<'a> {
    response: reqwest::Result<Response>,
    query: &'a str,
}

impl<'a> From<(reqwest::Result<Response>, &'a str)> for QueryResult<'a> {
    fn from((response, query): (reqwest::Result<Response>, &'a str)) -> Self {
        QueryResult { response, query }
    }
}

impl std::fmt::Debug for Product {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl<'a> std::fmt::Debug for QueryResult<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:^10}] => ", self.query)?;
        match &self.response {
            Ok(val) => write!(f, "{:?}", val.products),
            Err(err) => write!(f, "Error: {:?}", err.status()),
        }
    }
}

/// Run a single query-request to the API.
///
/// # Example
///
/// `https://dummyjson.com/products/search?q=laptop`
async fn request<'a>(client: &'a Client, query: &'a str) -> QueryResult<'a> {
    let response = match client
        .get("https://dummyjson.com/products/search")
        .query(&[("q", query)])
        .send()
        .await
    {
        Ok(response) => response,
        Err(err) => return QueryResult::from((Err(err), query)),
    };

    let json = match response.error_for_status() {
        Err(err) => return QueryResult::from((Err(err), query)),
        Ok(response) => response.json::<Response>().await,
    };

    QueryResult::from((json, query))
}

/// Run each query that the iterator `queries` yields.
///
/// # Concurrency
///
/// Up to 10 queries are run concurrently.
fn run_queries<'a>(
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
