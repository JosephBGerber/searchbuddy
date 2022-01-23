#![feature(try_blocks)]

use axum::extract::Query;
use axum::routing::get;
use axum::{Json, Router};
use log::error;
use serde::Deserialize;
use shared::{get_channel_id, Chatroom};
use std::collections::HashMap;
use std::error::Error;
use tokio::runtime::Runtime;
use url::Url;

#[derive(Deserialize)]
struct ChatroomQuery {
    search: String,
}

struct Location {
    url: Url,
    terms: Vec<String>,
}

async fn get_chatrooms(Query(query): Query<ChatroomQuery>) -> Json<Vec<Chatroom>> {
    let terms: Vec<&str> = query.search.split(" ").collect();

    // TODO - This is a dumb implementation for the MVP.
    //  In the future, there will be a service that chatrooms will register themselves with.
    let urls = vec![Url::parse("http://0.0.0.0:3000").unwrap()];

    let instances = locate_instances(&urls, &terms);

    let client = reqwest::Client::new();

    let mut chatrooms = Vec::new();

    for Location { url, terms } in instances {
        let response: Result<HashMap<String, (u32, u32)>, Box<dyn Error>> = try {
            client
                .post(url.join("chatrooms")?)
                .json(&terms)
                .send()
                .await?
                .json::<HashMap<String, (u32, u32)>>()
                .await?
        };

        match response {
            Ok(response) => {
                for (term, (chatroom_id, count)) in response {
                    let mut url = url.join("ws").unwrap();
                    url.set_scheme("ws").unwrap();

                    chatrooms.push(Chatroom {
                        term,
                        online: true,
                        chatroom_id,
                        num_users: count,
                        url: url.join("ws").unwrap().to_string(),
                    });
                }
            }
            Err(error) => {
                error!("Error occurred while querying chatroom instance {}", error);
            }
        }
    }

    Json(chatrooms)
}

fn locate_instances(urls: &Vec<Url>, terms: &[&str]) -> Vec<Location> {
    let mut instances = Vec::new();

    for url in urls {
        instances.push(Location {
            url: url.clone(),
            terms: Vec::new(),
        });
    }

    for term in terms {
        let hash = get_channel_id(*term);
        let index = hash as usize % urls.len();
        instances[index].terms.push(term.to_string());
    }

    instances
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let runtime = Runtime::new()?;

    let app = Router::new().route("/chatrooms", get(get_chatrooms));

    runtime.block_on(async {
        axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
            .serve(app.into_make_service())
            .await
            .unwrap();
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::locate_instances;
    use url::Url;

    #[test]
    fn locate_instances_returns_instances() {
        let urls = vec![
            Url::parse("http://instance1.example.com").unwrap(),
            Url::parse("http://instance2.example.com").unwrap(),
        ];
        let terms = ["a", "b", "c", "d", "e", "f", "g", "h"];
        let instances = locate_instances(&urls, &terms);
        assert_eq!(instances.len(), 2);
        assert_eq!(instances[0].url, urls[0]);
        assert_eq!(instances[0].terms.len(), 5);
        assert_eq!(instances[1].url, urls[1]);
        assert_eq!(instances[1].terms.len(), 3);
    }
}
