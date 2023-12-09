use axum::{
    response::Html,
    extract::Path,
    routing::get,
    Router,
};
use std::env;

async fn load_html(snippet_name: &str) -> String {
    let path = format!("website/{}", snippet_name);
    let html = tokio::fs::read_to_string(path).await.unwrap();
    format!("{}", html)
}

async fn load_json(name: &str) -> serde_json::Value {
    let path = format!("data/{}.json", name);
    let json_str = tokio::fs::read_to_string(path).await.unwrap();
    let json_data: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    return json_data.get(name).unwrap().to_owned();
}

async fn homepage_handler() -> Html<String> {
    let layout = load_html("layout.html").await;
    let homepage_snippet = load_html("snippets/homepage.html").await;
    let html = layout.replace("{{body}}", &homepage_snippet);
    Html(html)
}

async fn category_handler(Path(category_id): Path<String>) -> Html<String> {
    let category_id = category_id.parse::<i64>().unwrap();
    let categories = load_json("categories").await;
    let forums = load_json("forums").await;
    let posts = load_json("posts").await;

    let section_tree_html = forums
        .as_array().unwrap().iter()
        .filter(|c| c["category_id"] == category_id)
        .map(|section| {
            let article_item_html = String::from(r#"<li><a href="/hc/kb/article/{{article_id}}/" title="{{article_title}}">{{article_title}}</a></li>"#);
            let article_list_html = posts
                .as_array().unwrap().iter()
                .filter(|p| p["forum_id"] == section["id"])
                .map(|p| {
                    let article_item_html = article_item_html
                        .replace("{{article_id}}", &p["id"].to_string())
                        .replace("{{article_title}}", &p["title"].as_str().unwrap());
                    article_item_html
                }).collect::<Vec<String>>().join("");
            let section_item_html = String::from(r#"<div class="section section-{{section_id}}"><h3><a href="/hc/kb/section/{{section_id}}/">{{section_title}}</a></h3><ul class="article-list">{{article_list}}</ul></div>"#);
            let section_item_html = section_item_html
                .replace("{{section_id}}", &section["id"].to_string())
                .replace("{{section_title}}", &section["title"].as_str().unwrap())
                .replace("{{article_list}}", &article_list_html);
            section_item_html
        }).collect::<Vec<String>>().join("");

    let category = categories
        .as_array().unwrap().iter()
        .find(|c| c["id"] == category_id).unwrap();
    let category_html = load_html("snippets/category.html").await;
    let category_html = category_html
        .replace("{{category_name}}", &category["title"].as_str().unwrap())
        .replace("{{section_tree}}", &section_tree_html);

    let layout_html = load_html("layout.html").await;
    let layout = layout_html.replace("{{body}}", &category_html);
    Html(layout)
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async { "Hello, 嘿店!" }))
        .route("/hc", get(homepage_handler))
        .route("/hc/kb/category/:id", get(category_handler))
        .route("/hc/kb/category/:id/", get(category_handler))
        ;

    let port = env::var("PORT").unwrap_or_else(|_| String::from("3000"));
    let host = env::var("HOST").unwrap_or_else(|_| String::from("0.0.0.0"));
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port)).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
