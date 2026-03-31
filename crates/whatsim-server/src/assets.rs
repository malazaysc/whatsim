use axum::http::{header, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use rust_embed::{Embed, EmbeddedFile};

/// Embeds the built frontend assets from `web/dist`.
///
/// When the directory exists but is empty (e.g. during development before
/// running `npm run build`), the embed will contain no usable files and
/// requests will fall through to the placeholder handler.
#[derive(Embed)]
#[folder = "../../web/dist/"]
pub struct FrontendAssets;

fn get_embedded(path: &str) -> Option<EmbeddedFile> {
    <FrontendAssets as Embed>::get(path)
}

/// Serve a static file from the embedded frontend assets.
pub async fn static_handler(uri: axum::http::Uri) -> Response {
    let path = uri.path().trim_start_matches('/');

    // Try the exact path first.
    if let Some(content) = get_embedded(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        return (
            StatusCode::OK,
            [(header::CONTENT_TYPE, mime.as_ref())],
            content.data,
        )
            .into_response();
    }

    // SPA fallback: serve index.html for any path that doesn't match a static
    // file, so that client-side routing works.
    if let Some(index) = get_embedded("index.html") {
        return (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/html")],
            index.data,
        )
            .into_response();
    }

    // Development placeholder when web/dist has not been built yet.
    Html(
        r#"<!DOCTYPE html>
<html>
<head><title>Whatsim</title></head>
<body style="font-family: system-ui, sans-serif; max-width: 600px; margin: 80px auto; text-align: center;">
  <h1>Whatsim</h1>
  <p>The frontend has not been built yet.</p>
  <p>Run <code>cd web && npm install && npm run build</code> to build it,
     or use the API directly at <code>/api/*</code>.</p>
</body>
</html>"#
            .to_string(),
    )
    .into_response()
}
