use std::sync::Arc;
use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse},
    routing::get,
    Router,
    body::Body,
};
use chrono::prelude::*;
use clap::{Arg, Command};
use std::{
    fs,
    path::{Path as FsPath, PathBuf},
};
use thiserror::Error;
use tokio::net::TcpListener;
use tracing::{error, info};

// 自定義錯誤類型
#[derive(Error, Debug)]
enum ServerError {
    #[error("文件系統錯誤: {0}")]
    Filesystem(#[from] std::io::Error),
    
    #[error("路徑不安全: {0}")]
    UnsafePath(String),
    
    #[error("未找到: {0}")]
    NotFound(String),
    
    #[error("伺服器錯誤: {0}")]
    ServerError(String),
}

// 轉換為 HTTP 響應
impl IntoResponse for ServerError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            ServerError::NotFound(path) => (
                StatusCode::NOT_FOUND,
                format!("找不到路徑: {}", path),
            ),
            ServerError::UnsafePath(path) => (
                StatusCode::FORBIDDEN,
                format!("禁止訪問: {}", path),
            ),
            _ => {
                error!("伺服器錯誤: {:?}", self);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "伺服器內部錯誤".to_string(),
                )
            }
        };
        
        (status, Html(format!("<h1>{}</h1>", message))).into_response()
    }
}

// 服務器配置
#[derive(Clone)]
struct ServerConfig {
    base_path: Arc<String>,
    restricted_files: Vec<String>,
    use_plain_html: bool,
}

// 檢查路徑是否安全
fn is_safe_path(path: &str, config: &ServerConfig) -> Result<PathBuf, ServerError> {
    // 檢查禁止的文件類型
    for restricted in &config.restricted_files {
        if path.contains(restricted) {
            return Err(ServerError::UnsafePath(path.to_string()));
        }
    }
    
    let fs_path = FsPath::new(config.base_path.as_str()).join(path);
    
    // 檢查路徑是否超出基礎目錄範圍
    let canonical_base = fs::canonicalize(config.base_path.as_str())
        .map_err(ServerError::Filesystem)?;
    
    let canonical_path = match fs::canonicalize(&fs_path) {
        Ok(p) => p,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(ServerError::NotFound(path.to_string()));
        }
        Err(e) => return Err(ServerError::Filesystem(e)),
    };
    
    if !canonical_path.starts_with(canonical_base) {
        return Err(ServerError::UnsafePath(path.to_string()));
    }
    
    Ok(fs_path)
}

// 生成目錄索引HTML
fn generate_directory_html(
    path: &str,
    entries: Vec<fs::DirEntry>,
    use_plain_html: bool,
) -> Result<String, ServerError> {
    let mut dir_entries = Vec::new();
    
    for entry in entries {
        let file_name = entry.file_name().to_string_lossy().to_string();
        let file_type = entry.file_type().map_err(ServerError::Filesystem)?;
        let metadata = entry.metadata().map_err(ServerError::Filesystem)?;
        dir_entries.push((file_name, file_type, metadata));
    }
    
    dir_entries.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
    
    if use_plain_html {
        // 原始的HTML格式，類似於原始專案
        let mut html = String::new();
        html.push_str(&format!("<html>\n<head><title>Index of /{}</title>\n</head>\n", path));
        html.push_str(&format!("<body>\n<h1>Index of /{}</h1>\n<hr><pre><a href=\"../\">../</a>\n", path));
        
        for (file_name, file_type, metadata) in dir_entries {
            let is_dir = file_type.is_dir();
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
            
            let (date_str, size_str) = if let Some(meta) = Some(metadata) {
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
            
            let padding = " ".repeat(50_usize.saturating_sub(display_name.len()));
            html.push_str(&format!("<a href=\"/{}\">{}</a>{}{} {:>10}\n", href, display_name, padding, date_str, size_str));
        }
        
        html.push_str("</pre><hr>\n</body>\n</html>");
        return Ok(html);
    } else {
        // 美化版HTML
        let mut html = String::new();
        html.push_str(&format!("<html>\n<head>\n<title>Index of /{}</title>\n", path));
        html.push_str("<style>\n");
        html.push_str("body { font-family: system-ui, -apple-system, sans-serif; padding: 2em; }\n");
        html.push_str("table { border-collapse: collapse; width: 100%; }\n");
        html.push_str("th, td { text-align: left; padding: 8px; }\n");
        html.push_str("tr:nth-child(even) { background-color: #f2f2f2; }\n");
        html.push_str("th { background-color: #4CAF50; color: white; }\n");
        html.push_str("a { text-decoration: none; }\n");
        html.push_str("a:hover { text-decoration: underline; }\n");
        html.push_str("</style>\n</head>\n");
        
        html.push_str(&format!("<body>\n<h1>Index of /{}</h1>\n", path));
        html.push_str("<table>\n<tr><th>Name</th><th>Last Modified</th><th>Size</th></tr>\n");
        
        // 返回上一層目錄的連結
        html.push_str("<tr><td><a href=\"../\">../</a></td><td></td><td></td></tr>\n");
        
        for (file_name, file_type, metadata) in dir_entries {
            let is_dir = file_type.is_dir();
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
            
            let modified_time = metadata.modified().ok().map(|t| {
                let datetime: DateTime<Local> = t.into();
                datetime.format("%d-%b-%Y %H:%M").to_string()
            }).unwrap_or_else(|| "-".to_string());
            
            let size = if is_dir {
                "-"
            } else {
                let sz = metadata.len();
                if sz > 1024*1024*1024 {
                    &format!("{:.1} GB", sz as f64 / (1024.0*1024.0*1024.0))
                } else if sz > 1024*1024 {
                    &format!("{:.1} MB", sz as f64 / (1024.0*1024.0))
                } else if sz > 1024 {
                    &format!("{:.1} KB", sz as f64 / 1024.0)
                } else {
                    &format!("{} B", sz)
                }
            };
            
            html.push_str(&format!(
                "<tr><td><a href=\"/{}\">{}</a></td><td>{}</td><td>{}</td></tr>\n",
                href, display_name, modified_time, size
            ));
        }
        
        html.push_str("</table>\n");
        html.push_str("<hr>\n<p style=\"font-size: 0.8em; color: #666;\">Powered by Rust Static Server</p>\n");
        html.push_str("</body>\n</html>");
        
        Ok(html)
    }
}

// 處理靜態文件或目錄請求
async fn serve_static(
    State(config): State<ServerConfig>,
    path: Option<Path<String>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ServerError> {
    let path_str = match path {
        Some(Path(p)) => p,
        None => "".to_string(),
    };
    
    let fs_path = is_safe_path(&path_str, &config)?;
    
    if fs_path.is_dir() {
        let entries = fs::read_dir(&fs_path).map_err(ServerError::Filesystem)?;
        let html = generate_directory_html(&path_str, entries.collect::<Result<Vec<_>, _>>().map_err(ServerError::Filesystem)?, config.use_plain_html)?;
        return Ok(Html(html).into_response());
    } else if fs_path.is_file() {
        // 檢查If-Modified-Since頭部用於簡單緩存
        if let Some(if_modified_since) = headers.get(header::IF_MODIFIED_SINCE) {
            if let Ok(header_time) = if_modified_since.to_str() {
                if let Ok(metadata) = fs::metadata(&fs_path) {
                    if let Ok(modified) = metadata.modified() {
                        let modified_time: DateTime<Local> = modified.into();
                        let modified_str = modified_time.format("%a, %d %b %Y %H:%M:%S GMT").to_string();
                        
                        if header_time == modified_str {
                            return Ok(StatusCode::NOT_MODIFIED.into_response());
                        }
                    }
                }
            }
        }
        
        let content = fs::read(&fs_path).map_err(ServerError::Filesystem)?;
        let mime = mime_guess::from_path(&fs_path).first_or_octet_stream();
        
        let metadata = fs::metadata(&fs_path).map_err(ServerError::Filesystem)?;
        let modified = metadata.modified().map_err(ServerError::Filesystem)?;
        let modified_time: DateTime<Local> = modified.into();
        let modified_str = modified_time.format("%a, %d %b %Y %H:%M:%S GMT").to_string();
        
        let response = axum::response::Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, mime.as_ref())
            .header(header::LAST_MODIFIED, modified_str)
            .header(header::CACHE_CONTROL, "public, max-age=3600")
            .body(Body::from(content))
            .map_err(|e| ServerError::ServerError(e.to_string()))?;
            
        Ok(response)
    } else {
        Err(ServerError::NotFound(path_str))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日誌
    tracing_subscriber::fmt::init();
    
    let matches = Command::new("Static File Server")
        .version("0.2.0")
        .author("Your Name <youremail@example.com>")
        .about("A simple static file server with security features")
        .arg(
            Arg::new("host")
                .short('H')
                .long("host")
                .value_name("HOST")
                .help("設置伺服器監聽地址")
                .value_parser(clap::value_parser!(String))
                .default_value("127.0.0.1"),
        )
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .value_name("PORT")
                .help("設置伺服器監聽端口")
                .value_parser(clap::value_parser!(u16))
                .default_value("3000"),
        )
        .arg(
            Arg::new("base")
                .short('b')
                .long("base")
                .value_name("BASE")
                .help("設置基礎路徑")
                .value_parser(clap::value_parser!(String))
                .default_value("."),
        )
        .arg(
            Arg::new("plain")
                .short('P')
                .long("plain")
                .help("使用原始HTML樣式，無美化")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("restricted-files")
                .short('r')
                .long("restricted-files")
                .value_name("PATTERNS")
                .help("設置禁止訪問的文件類型，用逗號分隔")
                .value_parser(clap::value_parser!(String))
                .default_value(".env,.git,Cargo.toml,Cargo.lock"),
        )
        .get_matches();

    let host = matches.get_one::<String>("host").unwrap();
    let port = matches.get_one::<u16>("port").unwrap();
    let base_path = Arc::new(matches.get_one::<String>("base").unwrap().clone());

    // 伺服器配置
    let restricted_files = matches
        .get_one::<String>("restricted-files")
        .unwrap()
        .split(',')
        .map(|s| s.trim().to_string())
        .collect::<Vec<String>>();

    let config = ServerConfig {
        base_path,
        restricted_files,
        use_plain_html: matches.get_flag("plain"),
    };

    let addr = format!("{}:{}", host, port);
    info!("伺服器運行在 http://{}", addr);

    // 路由設置
    let app = Router::new()
        .route("/*path", get(serve_static))
        .route("/", get(serve_static))
        .with_state(config);

    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}