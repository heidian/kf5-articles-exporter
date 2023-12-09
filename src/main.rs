use axum::{
    response::Html,
    extract::Path,
    routing::get,
    Router,
};
use std::env;
use regex::Regex;


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
        .replace("{{category_name}}", category["title"].as_str().unwrap())
        .replace("{{section_tree}}", &section_tree_html);

    let layout_html = load_html("layout.html").await;
    let html = layout_html.replace("{{body}}", &category_html);
    Html(html)
}

async fn section_handler(Path(section_id): Path<String>) -> Html<String> {
    let section_id = section_id.parse::<i64>().unwrap();
    let forums = load_json("forums").await;
    let posts = load_json("posts").await;

    let article_item_html = String::from(r#"<li><a href="/hc/kb/article/{{article_id}}/" title="{{article_title}}">{{article_title}}</a></li>"#);
    let article_list_html = posts
        .as_array().unwrap().iter()
        .filter(|p| p["forum_id"] == section_id)
        .map(|p| {
            let article_item_html = article_item_html
                .replace("{{article_id}}", &p["id"].to_string())
                .replace("{{article_title}}", &p["title"].as_str().unwrap());
            article_item_html
        }).collect::<Vec<String>>().join("");

    let section = forums
        .as_array().unwrap().iter()
        .find(|c| c["id"] == section_id).unwrap();
    let section_html = load_html("snippets/section.html").await;
    let section_html = section_html
        .replace("{{section_title}}", section["title"].as_str().unwrap())
        .replace("{{article_list}}", &article_list_html)
        ;

    let layout_html = load_html("layout.html").await;
    let html = layout_html.replace("{{body}}", &section_html);
    Html(html)
}

async fn article_handler(Path(article_id): Path<String>) -> Html<String> {
    let article_id = article_id.parse::<i64>().unwrap();
    let posts = load_json("posts").await;

    let article = posts
        .as_array().unwrap().iter()
        .find(|c| c["id"] == article_id).unwrap();

    let article_content = article["content"].as_str().unwrap().to_owned();
    let article_content = article_content.replace("support.hey.shop", "support.heidianer.com");
    let re = Regex::new(r"https:\/\/files.kf5.com\/attachments\/download\/(\w+\/\w+\/\w+)\/").unwrap();
    let article_content = re.replace_all(
        &article_content,
        "https://up.img.heidiancdn.com/kf5/$1"
    );
    let re = Regex::new(r"https:\/\/heidian.kf5.com\/attachments\/download\/(\w+\/\w+)\/").unwrap();
    let article_content = re.replace_all(
        &article_content,
        "https://up.img.heidiancdn.com/kf5/$1"
    );

    let article_html = load_html("snippets/article.html").await;
    let article_html = article_html
        .replace("{{article_title}}", article["title"].as_str().unwrap())
        .replace("{{article_body}}", &article_content);

    let layout_html = load_html("layout.html").await;
    let html = layout_html.replace("{{body}}", &article_html);
    Html(html)
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        // .route("/", get(|| async { "Hello, 嘿店!" }))
        .route("/", get(homepage_handler))
        .route("/hc", get(homepage_handler))
        .route("/hc/", get(homepage_handler))
        .route("/hc/kb/category/:id", get(category_handler))
        .route("/hc/kb/category/:id/", get(category_handler))
        .route("/hc/kb/section/:id", get(section_handler))
        .route("/hc/kb/section/:id/", get(section_handler))
        .route("/hc/kb/article/:id", get(article_handler))
        .route("/hc/kb/article/:id/", get(article_handler))
        ;

    let port = env::var("PORT").unwrap_or_else(|_| String::from("3000"));
    let host = env::var("HOST").unwrap_or_else(|_| String::from("0.0.0.0"));
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port)).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
