#![allow(dead_code)]

use serde::Deserialize;
use reqwest::Client;

/// The JSON structure for a single product returned from the API.
///
/// # Example
///
/// `{ "id": 11, "title": "perfume Oil", "category": "fragrances", ... }`
#[derive(Deserialize)]
pub struct Product {
    pub id: usize,
    pub title: String,
    pub category: String,
}

/// The JSON structure returned from the API.
///
/// # Example
///
/// `{ "products": [...], "total": 5, "skip": 0, "limit": 5 }`
#[derive(Debug, Deserialize)]
pub struct Response {
    pub total: usize,
    pub skip: usize,
    pub limit: usize,
    pub products: Vec<Product>,
}

/// A query-string and the corresponding result returned from the API.
pub struct QueryResult<'a> {
    pub response: reqwest::Result<Response>,
    pub query: &'a str,
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
pub async fn request<'a>(client: &'a Client, query: &'a str) -> QueryResult<'a> {
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
