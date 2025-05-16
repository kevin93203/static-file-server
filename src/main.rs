use axum::{response::{Html, IntoResponse}, extract::Path, routing::get, Router};
use tokio::net::TcpListener;
use std::path::{Path as FsPath};
use std::fs;
use axum::body::Body;
use axum::http::{Response, StatusCode};

async fn static_or_dir(path: Option<Path<String>>) -> impl IntoResponse {
    let path = match path {
        Some(Path(p)) => p,
        None => "".to_string(),
    };
    let fs_path = FsPath::new(".").join(&path);
    if fs_path.is_dir() {
        let entries = match fs::read_dir(&fs_path) {
            Ok(e) => e,
            Err(_) => return Html("<h1>無法讀取目錄</h1>").into_response(),
        };
        let mut html = String::from("<html><head><meta charset='utf-8'><title>目錄列表</title></head><body>");
        html.push_str(&format!("<h2>目錄：/{}</h2><ul>", path));
        if path != "" {
            let parent = if let Some(idx) = path.rfind('/') {
                &path[..idx]
            } else {
                ""
            };
            html.push_str(&format!("<li><a href='/{}'>.. (上層目錄)</a></li>", parent));
        }
        for entry in entries {
            if let Ok(entry) = entry {
                let file_name = entry.file_name().to_string_lossy().to_string();
                let file_type = entry.file_type().ok();
                let href = if path.is_empty() {
                    file_name.clone()
                } else {
                    format!("{}/{}", path, file_name)
                };
                if let Some(ft) = file_type {
                    if ft.is_dir() {
                        html.push_str(&format!("<li><a href='/{}'>{}/</a></li>", href, file_name));
                    } else {
                        html.push_str(&format!("<li><a href='/{}'>{}</a></li>", href, file_name));
                    }
                }
            }
        }
        html.push_str("</ul></body></html>");
        Html(html).into_response()
    } else if fs_path.is_file() {
        match fs::read(&fs_path) {
            Ok(content) => {
                let mime = mime_guess::from_path(&fs_path).first_or_octet_stream();
                Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", mime.as_ref())
                    .body(Body::from(content))
                    .unwrap()
            },
            Err(_) => StatusCode::NOT_FOUND.into_response(),
        }
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/*path", get(static_or_dir))
        .route("/", get(static_or_dir));

    let addr = "127.0.0.1:3000";
    println!("Server running at http://{}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
