use anyhow::anyhow;
use tiny_http::{Response, StatusCode};

use crate::notifications::WeakNotifyHandle;

pub fn run_http(port: u16, notifier: WeakNotifyHandle) {
    let Ok(srv) = tiny_http::Server::http(("localhost", port))
        .inspect_err(|e| tracing::warn!(%e, "creating server"))
    else {
        return;
    };
    std::thread::spawn(move || {
        for req in srv.incoming_requests() {
            let op = req.url().trim_start_matches('/');
            let res = match op {
                "warn-rain" => notifier.set_enabled(true),
                "unwarn-rain" => notifier.set_enabled(false),
                _ => Err(anyhow!("Unknown path")),
            };
            let res = match res {
                Ok(_) => req.respond(Response::empty(StatusCode(200))),
                Err(e) => req.respond(
                    Response::from_string(e.to_string()).with_status_code(StatusCode(400)),
                ),
            };
            if let Err(e) = res {
                tracing::warn!(%e, "responding to request");
            }
        }
    });
}
