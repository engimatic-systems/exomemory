mod support;
use support::*;
#[test]
fn daemon_client_create_read_metadata_restart_and_errors() {
    let run = Run::new();
    let daemon = Daemon::start(&run);
    let (code, created) = ctl(
        &run,
        &daemon.socket,
        &run.scope,
        &["create", "--json", "-"],
        Some(&draft(&run)),
    );
    assert_eq!(code, 0, "{created}");
    let record = &created["result"]["Ok"];
    let key = record["metadata"]["ghost_key"].as_str().unwrap();
    assert_eq!(
        ctl(&run, &daemon.socket, &run.scope, &[key], None).1["result"]["Ok"],
        *record
    );
    assert_eq!(
        ctl(&run, &daemon.socket, &run.scope, &["meta", key], None).1["result"]["Ok"],
        record["metadata"]
    );
    assert_eq!(
        ctl(&run, &daemon.socket, &run.scope, &["missing"], None).0,
        6
    );
    let other = Run::new();
    assert_eq!(ctl(&run, &daemon.socket, &other.scope, &[key], None).0, 4);
    let mut bad = draft(&run);
    bad.backend.kind = "zvec".into();
    assert_eq!(
        ctl(
            &run,
            &daemon.socket,
            &run.scope,
            &["create", "--json", "-"],
            Some(&bad)
        )
        .0,
        5
    );
    bad = draft(&run);
    bad.triggers.clear();
    assert_eq!(
        ctl(
            &run,
            &daemon.socket,
            &run.scope,
            &["create", "--json", "-"],
            Some(&bad)
        )
        .0,
        2
    );
    let socket = daemon.socket.clone();
    drop(daemon);
    assert_eq!(ctl(&run, &socket, &run.scope, &[key], None).0, 3);
    let daemon = Daemon::start(&run);
    assert_eq!(
        ctl(&run, &daemon.socket, &run.scope, &[key], None).1["result"]["Ok"],
        *record
    );
    std::thread::scope(|scope| {
        for _ in 0..8 {
            scope.spawn(|| {
                assert_eq!(ctl(&run, &daemon.socket, &run.scope, &[key], None).0, 0);
            });
        }
    });
}
#[test]
fn direct_socket_requests_cannot_bypass_scope_or_protocol_checks() {
    use exomemory::{transport::frame, types::*};
    use std::{io::Write, os::unix::net::UnixStream};
    let run = Run::new();
    let daemon = Daemon::start(&run);
    let other = Run::new();
    let raw = |value: serde_json::Value| -> Response {
        let mut stream = UnixStream::connect(&daemon.socket).unwrap();
        writeln!(stream, "{value}").unwrap();
        serde_json::from_slice(&frame(stream).unwrap()).unwrap()
    };
    let mut request = serde_json::to_value(Request {
        version: 1,
        context: run.context.clone(),
        operation: Operation::Create { draft: draft(&run) },
    })
    .unwrap();
    request["context"]["cwd"] = serde_json::to_value(&other.scope).unwrap();
    assert_eq!(
        raw(request.clone()).result.unwrap_err().code,
        ErrorCode::Forbidden
    );
    request["context"]["cwd"] = serde_json::to_value(&run.scope).unwrap();
    request["version"] = 2.into();
    assert_eq!(
        raw(request.clone()).result.unwrap_err().code,
        ErrorCode::Unsupported
    );
    request["version"] = 1.into();
    request["operation"]["draft"]["triggers"][0]["kind"] = "vector".into();
    assert_eq!(
        raw(request).result.unwrap_err().code,
        ErrorCode::Unsupported
    );
    assert_eq!(
        raw(serde_json::json!({"unknown":true}))
            .result
            .unwrap_err()
            .code,
        ErrorCode::InvalidInput
    );
    assert_eq!(
        ctl(
            &run,
            &daemon.socket,
            &run.scope,
            &["list-trigger-types"],
            None
        )
        .0,
        0
    );
}
