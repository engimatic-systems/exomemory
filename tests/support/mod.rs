#![allow(dead_code)]
use exomemory::{policy::Policy, types::*};
use std::{fs, path::PathBuf};
pub struct Run {
    pub root: PathBuf,
    pub scope: PathBuf,
    pub policy: Policy,
    pub context: Context,
}
impl Run {
    pub fn new() -> Self {
        let parent = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/.runtime");
        fs::create_dir_all(&parent).unwrap();
        let root = parent.join(format!(
            "r{}",
            &uuid::Uuid::new_v4().simple().to_string()[..12]
        ));
        fs::create_dir(&root).unwrap();
        let scope = root.join("scope");
        fs::create_dir(&scope).unwrap();
        for dir in ["logs", "harness"] {
            fs::create_dir(root.join(dir)).unwrap();
        }
        let policy = Policy::test_profile(&root, &scope, "fixture-tenant".into()).unwrap();
        let context = Context {
            tenant: policy.tenant.clone(),
            session: "s1".into(),
            cwd: scope.clone(),
        };
        Self {
            root,
            scope,
            policy,
            context,
        }
    }
}
pub fn draft(run: &Run) -> Draft {
    Draft {
        body: "The test suite took 12 seconds.".into(),
        blurb: "Test suite: 12 seconds".into(),
        uses: vec!["plan tests".into()],
        topics: vec!["testing".into()],
        triggers: vec![Trigger::Regex {
            id: "test-duration".into(),
            pattern: "(?i)test".into(),
        }],
        backend: Backend {
            kind: "sqlite".into(),
            instance: "local".into(),
        },
        scope: Scope::Dir {
            root: run.scope.clone(),
        },
        provenance: Provenance {
            origin: "synthetic fixture".into(),
            actor: "controlled harness".into(),
        },
        trust: Trust::Untrusted,
        overwritable: true,
    }
}
pub struct Daemon {
    child: std::process::Child,
    pub socket: PathBuf,
}
impl Daemon {
    pub fn start(run: &Run) -> Self {
        use std::{
            os::unix::net::UnixStream,
            process::{Command, Stdio},
            time::{Duration, Instant},
        };
        let socket = run.root.join("exo.sock");
        let mut child = Command::new(env!("CARGO_BIN_EXE_exod"))
            .args([
                "--runtime",
                run.root.to_str().unwrap(),
                "--scope-root",
                run.scope.to_str().unwrap(),
                "--tenant",
                &run.policy.tenant,
                "--database",
                run.root.join("memory.sqlite").to_str().unwrap(),
                "--socket",
                socket.to_str().unwrap(),
            ])
            .stdout(Stdio::from(
                fs::File::create(run.root.join("logs/daemon.stdout")).unwrap(),
            ))
            .stderr(Stdio::from(
                fs::File::create(run.root.join("logs/daemon.stderr")).unwrap(),
            ))
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if UnixStream::connect(&socket).is_ok() {
                break;
            }
            assert!(child.try_wait().unwrap().is_none(), "daemon exited");
            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                panic!("daemon startup timeout");
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        Self { child, socket }
    }
}
impl Drop for Daemon {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
pub fn ctl(
    run: &Run,
    socket: &std::path::Path,
    cwd: &std::path::Path,
    args: &[&str],
    input: Option<&Draft>,
) -> (i32, serde_json::Value) {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    let mut child = Command::new(env!("CARGO_BIN_EXE_exoctl"))
        .args([
            "--runtime",
            run.root.to_str().unwrap(),
            "--scope-root",
            run.scope.to_str().unwrap(),
            "--tenant",
            &run.policy.tenant,
            "--session",
            &run.context.session,
            "--cwd",
            cwd.to_str().unwrap(),
            "--socket",
            socket.to_str().unwrap(),
        ])
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    if let Some(draft) = input {
        child
            .stdin
            .take()
            .unwrap()
            .write_all(&serde_json::to_vec(draft).unwrap())
            .unwrap();
    }
    let output = child.wait_with_output().unwrap();
    (
        output.status.code().unwrap(),
        serde_json::from_slice(&output.stdout).unwrap(),
    )
}
pub fn event(run: &Run, number: u64, presence_version: u64) -> Event {
    Event {
        context: run.context.clone(),
        live_context: "main".into(),
        number,
        presence_version,
        kind: EventKind::Prompt,
        text: "Run the test suite".into(),
    }
}
