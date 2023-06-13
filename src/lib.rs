use std::{env, future::Future};

use dotenv::dotenv;
use flowsnet_platform_sdk::logger;
use github_flows::{listen_to_event, EventPayload, GithubLogin};
use slack_flows::send_message_to_channel;

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn run() {
    dotenv().ok();
    logger::init();

    let owner = env::var("owner").unwrap_or("jetjinser".to_string());
    let repo = env::var("repo").unwrap_or("fot".to_string());

    let team = env::var("team").unwrap_or("ham-5b68442".to_string());
    let channel = env::var("channel").unwrap_or("general".to_string());

    let login = env::var("login")
        .map(GithubLogin::Provided)
        .unwrap_or(GithubLogin::Default);

    let events = vec!["discussion"];

    listen_to_event(&login, &owner, &repo, events, |payload| {
        handler(payload, |msg| send_message_to_channel(&team, &channel, msg))
    })
    .await;
}

async fn handler<F, Fut>(payload: EventPayload, send: F)
where
    F: FnOnce(String) -> Fut,
    Fut: Future<Output = ()>,
{
    match payload {
        EventPayload::UnknownEvent(ep) => {
            let action = &ep["action"];

            if let Some(act) = action.as_str() {
                if act != "created" {
                    log::info!("action `{}` != `created`", act);
                    return;
                }
            }

            let discussion = &ep["discussion"];
            match discussion.as_object() {
                Some(dis) => {
                    // title: string, required
                    let title = dis.get("title").and_then(|value| value.as_str()).unwrap();
                    // body: string | nulg, required
                    let body = dis
                        .get("body")
                        .and_then(|value| value.as_str())
                        .unwrap_or("<empty body>");
                    // title: string, required
                    let html_url = dis
                        .get("html_url")
                        .and_then(|value| value.as_str())
                        .unwrap();

                    // title: string, required
                    let login = dis
                        .get("user")
                        .and_then(|value| value.as_object())
                        .and_then(|obj| obj.get("login"))
                        .and_then(|value| value.as_str())
                        .unwrap();

                    let taken_body: String = body.chars().take(200).collect();
                    let end = if body.len() > 200 { "..." } else { "" };
                    let msg = format!(
                        "New Discussion Created by _{login}_:\n*{title}*\n{taken_body}{end}\n\nopen: {html_url}",
                    );

                    send(msg).await;
                }
                None => {
                    log::info!("not discussion payload")
                }
            }
        }
        _ => {
            log::info!("uncovered payload")
        }
    }
}
