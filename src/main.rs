use axum::{response::{Html, IntoResponse}, extract::Path, routing::get, Router};
use tokio::net::TcpListener;
use std::path::{Path as FsPath};
use std::fs;
use axum::body::Body;
use axum::http::{Response, StatusCode};
use chrono::prelude::*;

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
        let mut dir_entries = vec![];
        for entry in entries {
            if let Ok(entry) = entry {
                let file_name = entry.file_name().to_string_lossy().to_string();
                let file_type = entry.file_type().ok();
                let metadata = entry.metadata().ok();
                dir_entries.push((file_name, file_type, metadata));
            }
        }
        dir_entries.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
        let mut html = String::new();
        html.push_str(&format!("<html>\n<head><title>Index of /{}</title>\n</head>\n", path));
        html.push_str(&format!("<body>\n<h1>Index of /{}</h1>\n<hr><pre><a href=\"../\">../</a>\n", path));
        for (file_name, file_type, metadata) in dir_entries {
            let is_dir = file_type.as_ref().map(|ft| ft.is_dir()).unwrap_or(false);
            let href = if path.is_empty() {
                file_name.clone()
            } else {
                format!("{}/{}", path, file_name)
            };
            let display_name = if is_dir {
                format!("{}/", file_name)
            } else {
                file_name.clone()
            };
            let (date_str, size_str) = if let Some(meta) = metadata {
                let modified_time = meta.modified().ok().map(|t| {
                    let datetime: DateTime<Local> = t.into();
                    datetime.format("%d-%b-%Y %H:%M").to_string()
                }).unwrap_or_else(|| "-".to_string());
                let size = if is_dir {
                    "-".to_string()
                } else {
                    let sz = meta.len();
                    if sz > 1024*1024*1024-1 {
                        format!("{:>6}G", (sz + 1024*1024*1024/2)/(1024*1024*1024))
                    } else if sz > 1024*1024-1 {
                        format!("{:>6}M", (sz + 1024*1024/2)/(1024*1024))
                    } else if sz > 9999 {
                        format!("{:>6}K", (sz + 1024/2)/1024)
                    } else {
                        format!("{:>6}", sz)
                    }
                };
                (modified_time, size)
            } else {
                ("-".to_string(), "-".to_string())
            };
            let padding = " ".repeat(50_usize.saturating_sub(display_name.len())); // 指定數字類型為 i32
            html.push_str(&format!("<a href=\"/{}\">{}</a>{}{} {:>10}\n", href, display_name, padding, date_str, size_str)); // 調整寬度以確保對齊
        }
        html.push_str("</pre><hr>\n</body>\n</html>");
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
