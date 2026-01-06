use helix_daemon::{Client, Command, Request, ResponseResult, Server};
use std::time::Duration;
use tempfile::tempdir;

#[tokio::test]
async fn test_ping_command() {
    let dir = tempdir().unwrap();
    let socket_path = dir.path().join("test.sock").to_string_lossy().to_string();

    let server = Server::new(&socket_path);
    let server_handle = tokio::spawn({
        let server = Server::new(&socket_path);
        async move { server.run().await }
    });

    tokio::time::sleep(Duration::from_millis(50)).await;

    let client = Client::with_socket_path(&socket_path);
    let version = client.ping().await.unwrap();

    assert_eq!(version, env!("CARGO_PKG_VERSION"));

    client.shutdown("test complete").await.unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;

    drop(server);
    let _ = server_handle.await;
}

#[tokio::test]
async fn test_status_command() {
    let dir = tempdir().unwrap();
    let socket_path = dir.path().join("test.sock").to_string_lossy().to_string();

    tokio::spawn({
        let socket_path = socket_path.clone();
        async move {
            let server = Server::new(&socket_path);
            server.run().await
        }
    });

    tokio::time::sleep(Duration::from_millis(50)).await;

    let client = Client::with_socket_path(&socket_path);
    let request = Request::new(
        "/test/repo",
        "decisions",
        Command::Status(helix_daemon::StatusPayload::default()),
    );
    let response = client.send(request).await.unwrap();

    match response.result {
        ResponseResult::Ok { payload } => {
            if let helix_daemon::ResponsePayload::Status(status) = payload {
                assert!(status.uptime_ms > 0);
                assert!(status.queues.is_empty());
            } else {
                panic!("Unexpected payload type");
            }
        }
        ResponseResult::Error { error } => panic!("Unexpected error: {error:?}"),
    }

    client.shutdown("test complete").await.unwrap();
}

#[tokio::test]
async fn test_protocol_version_mismatch() {
    let dir = tempdir().unwrap();
    let socket_path = dir.path().join("test.sock").to_string_lossy().to_string();

    tokio::spawn({
        let socket_path = socket_path.clone();
        async move {
            let server = Server::new(&socket_path);
            server.run().await
        }
    });

    tokio::time::sleep(Duration::from_millis(50)).await;

    let client = Client::with_socket_path(&socket_path);
    let mut request = Request::new("/test/repo", "test", Command::Ping);
    request.version = 999;

    let response = client.send(request).await.unwrap();

    match response.result {
        ResponseResult::Error { error } => {
            assert_eq!(error.code, helix_daemon::ErrorCode::IncompatibleVersion);
        }
        ResponseResult::Ok { .. } => panic!("Expected error for version mismatch"),
    }

    client.shutdown("test complete").await.unwrap();
}

#[tokio::test]
async fn test_enqueue_sync_coalescing() {
    let dir = tempdir().unwrap();
    let socket_path = dir.path().join("test.sock").to_string_lossy().to_string();

    tokio::spawn({
        let socket_path = socket_path.clone();
        async move {
            let server = Server::new(&socket_path);
            server.run().await
        }
    });

    tokio::time::sleep(Duration::from_millis(50)).await;

    let client = Client::with_socket_path(&socket_path);

    let sync_id1 = client
        .enqueue_sync("/test/repo", "decisions", ".decisions", false)
        .await
        .unwrap();

    let sync_id2 = client
        .enqueue_sync("/test/repo", "decisions", ".decisions", false)
        .await
        .unwrap();

    assert_eq!(sync_id1, sync_id2);

    let sync_id3 = client
        .enqueue_sync("/test/repo", "decisions", ".decisions", true)
        .await
        .unwrap();

    assert_ne!(sync_id1, sync_id3);

    client.shutdown("test complete").await.unwrap();
}

#[tokio::test]
async fn test_status_shows_queued_jobs() {
    let dir = tempdir().unwrap();
    let socket_path = dir.path().join("test.sock").to_string_lossy().to_string();

    tokio::spawn({
        let socket_path = socket_path.clone();
        async move {
            let server = Server::new(&socket_path);
            server.run().await
        }
    });

    tokio::time::sleep(Duration::from_millis(50)).await;

    let client = Client::with_socket_path(&socket_path);

    client
        .enqueue_sync("/repo1", "decisions", ".decisions", false)
        .await
        .unwrap();

    client
        .enqueue_sync("/repo2", "hbd", ".tickets", false)
        .await
        .unwrap();

    let request = Request::new(
        "",
        "",
        Command::Status(helix_daemon::StatusPayload::default()),
    );
    let response = client.send(request).await.unwrap();

    match response.result {
        ResponseResult::Ok { payload } => {
            if let helix_daemon::ResponsePayload::Status(status) = payload {
                assert_eq!(status.queues.len(), 2);
            } else {
                panic!("Unexpected payload type");
            }
        }
        ResponseResult::Error { error } => panic!("Unexpected error: {error:?}"),
    }

    client.shutdown("test complete").await.unwrap();
}
