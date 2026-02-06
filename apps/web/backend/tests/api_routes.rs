use std::sync::{Arc, OnceLock};

use aurelia_api::axum_impl::AppState;
use aurelia_api::traits::axum_routes::build_router;
use aurelia_core::lastfm_core::LastFmState;
use aurelia_core::listenbrainz_core::ListenBrainzState;
use axum::body::Body;
use axum::http::Request;
use http_body_util::BodyExt;
use serial_test::serial;
use tempfile::TempDir;
use tower::ServiceExt;

fn setup_state() -> Arc<AppState> {
    static TEST_DIR: OnceLock<TempDir> = OnceLock::new();
    let dir = TEST_DIR.get_or_init(|| TempDir::new().expect("temp dir"));
    let path = dir.path().to_path_buf();

    aurelia_core::db::init(&path).expect("db init");
    let _ = aurelia_core::clear_cache(path.to_string_lossy().to_string());

    Arc::new(AppState {
        app_data_dir: path,
        listenbrainz_state: Arc::new(ListenBrainzState::new()),
        lastfm_state: Arc::new(LastFmState::new()),
    })
}

async fn parse_json(response: axum::response::Response) -> serde_json::Value {
    let collected = response.into_body().collect().await.expect("body");
    serde_json::from_slice(&collected.to_bytes()).expect("json")
}

#[tokio::test]
#[serial]
async fn get_library_returns_empty_payload() {
    let app_state = setup_state();
    let app = build_router().with_state(app_state);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/library")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let json = parse_json(response).await;

    assert_eq!(json["status"], "ok");
    assert_eq!(json["data"]["songs"].as_array().unwrap().len(), 0);
    assert_eq!(json["data"]["artists"].as_array().unwrap().len(), 0);
    assert_eq!(json["data"]["albums"].as_array().unwrap().len(), 0);
}

#[tokio::test]
#[serial]
async fn get_image_builds_url() {
    let app_state = setup_state();
    let app = build_router().with_state(app_state);

    let server_url = "http%3A%2F%2Flocalhost%3A8096";
    let uri = format!(
        "/images/abc?imageType=Primary&serverUrl={server_url}&token=token&width=200&quality=80"
    );

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(uri)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let json = parse_json(response).await;
    let url = json["data"].as_str().unwrap();

    assert!(url.contains("http://localhost:8096/Items/abc/Images/Primary"));
    assert!(url.contains("width=200"));
    assert!(url.contains("quality=80"));
    assert!(url.contains("api_key=token"));
}

#[tokio::test]
#[serial]
async fn get_audio_stream_url_respects_container() {
    let app_state = setup_state();
    let app = build_router().with_state(app_state);

    let server_url = "http%3A%2F%2Flocalhost%3A8096";
    let uri =
        format!("/audio/song123/stream-url?serverUrl={server_url}&token=token&container=flac");

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(uri)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let json = parse_json(response).await;
    let url = json["data"].as_str().unwrap();

    assert!(url.contains("/Audio/song123/stream"));
    assert!(url.contains("static=true"));
}
