use axum::{
    response::Html,
    routing::get,
    Router,
};
use std::env;

async fn load_html_snippet(snippet_name: &str) -> String {
    let path = format!("website/{}", snippet_name);
    let html = tokio::fs::read_to_string(path).await.unwrap();
    format!("{}", html)
}

async fn homepage_handler() -> Html<String> {
    let layout = load_html_snippet("layout.html").await;
    let homepage_snippet = load_html_snippet("snippets/homepage.html").await;
    // replace the {{content}} placeholder with the homepage snippet
    let html = layout.replace("{{ body }}", &homepage_snippet);
    Html(html)
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async { "Hello, 嘿店!" }))
        .route("/hc", get(homepage_handler));

    let port = env::var("PORT").unwrap_or_else(|_| String::from("3000"));
    let host = env::var("HOST").unwrap_or_else(|_| String::from("0.0.0.0"));
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port)).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
