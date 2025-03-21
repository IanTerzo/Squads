use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use gtk::traits::{BoxExt, WidgetExt};
use ipc_channel::ipc::IpcSender;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::env;
use tao::{
    dpi::{LogicalPosition, LogicalSize},
    event_loop::EventLoop,
    platform::unix::WindowExtUnix,
    window::WindowBuilder,
};
use url::form_urlencoded;
use wry::{WebViewBuilder, WebViewBuilderExtUnix};

// Structs must be the exact same as in auth.rs

#[derive(Serialize, Deserialize, Debug)]
struct AuthorizationCode {
    pub code: String,
    pub code_verifier: String,
}
#[derive(Serialize, Deserialize, Debug)]
struct Cookie {
    name: String,
    value: String,
    domain: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct AuthorizationInfo {
    authorization_codes: Option<AuthorizationCode>,
    cookies: Option<Vec<Cookie>>,
    success: bool,
}

fn gen_code_challenge() -> String {
    let code_verifier: String = (0..64)
        .map(|_| rand::rng().random_range(33..=126) as u8 as char)
        .collect();

    let hash = Sha256::digest(code_verifier.as_bytes());

    let code_challenge = URL_SAFE_NO_PAD.encode(hash);

    code_challenge
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let server_name = args[1].clone();

    let base_url = "https://login.microsoftonline.com/common/oauth2/v2.0/authorize";

    let challenge = gen_code_challenge();

    let params = vec![
        ("client_id", "5e3ce6c0-2b1f-4285-8d4b-75ee78787346"),
        ("scope", "openId profile openid offline_access"),
        ("redirect_uri", "https://teams.microsoft.com/v2"),
        ("response_mode", "fragment"),
        ("response_type", "code"),
        ("x-client-SKU", "msal.js.browser"),
        ("x-client-VER", "3.18.0"),
        ("client_info", "1"),
        ("code_challenge", challenge.as_str()),
        ("code_challenge_method", "plain"),
    ];

    let encoded_params = form_urlencoded::Serializer::new(String::new())
        .extend_pairs(&params)
        .finish();
    let auth_url = format!("{}?{}", base_url, encoded_params);

    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("Squads - Login")
        .build(&event_loop)
        .unwrap();

    let builder = WebViewBuilder::new()
        .with_url(auth_url)
        .with_bounds(wry::Rect {
            position: LogicalPosition::new(0, 0).into(),
            size: LogicalSize::new(1000, 700).into(),
        });

    #[cfg(not(target_os = "linux"))]
    let webview = builder.build(&window).unwrap();
    #[cfg(target_os = "linux")]
    let webview = {
        let vbox = window.default_vbox().unwrap();
        let overlay = gtk::Overlay::new();
        overlay.show_all();
        vbox.pack_start(&overlay, true, true, 0);
        builder.build_gtk(&overlay).unwrap()
    };

    event_loop.run(move |event, _, control_flow| {
        *control_flow = tao::event_loop::ControlFlow::Wait;

        match event {
            tao::event::Event::WindowEvent {
                event: tao::event::WindowEvent::CloseRequested,
                ..
            } => {
                let tx = IpcSender::connect(server_name.clone()).unwrap();
                tx.send(AuthorizationInfo {
                    authorization_codes: None,
                    cookies: None,
                    success: false,
                })
                .unwrap();
                *control_flow = tao::event_loop::ControlFlow::Exit
            }

            tao::event::Event::NewEvents(_) => {
                // Poll the webview to check the current URL.
                let current_url = webview.url().unwrap();

                if current_url.contains("https://teams.microsoft.com/v2/#code=") {
                    let code = current_url
                        .split("code=")
                        .nth(1)
                        .and_then(|s| s.split('&').next())
                        .unwrap()
                        .to_string();

                    let cookies = webview.cookies().unwrap();

                    let mut cookies_parsed = vec![];
                    for cookie in cookies {
                        cookies_parsed.push(Cookie {
                            name: cookie.name().to_string(),
                            value: cookie.value().to_string(),
                            domain: cookie.domain().map(|s| s.to_string()),
                        });
                    }

                    let authorization_codes = AuthorizationCode {
                        code,
                        code_verifier: challenge.clone(),
                    };

                    let tx = IpcSender::connect(server_name.clone()).unwrap();
                    tx.send(AuthorizationInfo {
                        authorization_codes: Some(authorization_codes),
                        cookies: Some(cookies_parsed),
                        success: true,
                    })
                    .unwrap();

                    *control_flow = tao::event_loop::ControlFlow::Exit;
                }
            }
            _ => (),
        }
    });
}
