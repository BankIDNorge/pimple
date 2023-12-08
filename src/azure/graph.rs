use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::azure::LazyToken;

const MS_GRAPH_URL: &'static str = "https://graph.microsoft.com";

pub struct GraphClient {
    client: Client,
    token: LazyToken,
}

impl GraphClient {
    pub fn new() -> Self {
        GraphClient {
            client: Client::new(),
            token: LazyToken::new(MS_GRAPH_URL),
        }
    }

    pub async fn get_groups(&self, filter: &str) -> Vec<GraphGroup> {
        self.client.get(format!("{}/v1.0/groups?{}", MS_GRAPH_URL, filter))
            .header("Authorization", format!("Bearer {}", self.token.token().await))
            .send()
            .await
            .unwrap()
            .json::<GraphGroups>()
            .await
            .unwrap()
            .value
    }
}

#[derive(Serialize, Deserialize)]
pub struct GraphGroup {
    pub id: String,
    pub description: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: String,
}

#[derive(Serialize, Deserialize)]
struct GraphGroups {
    pub value: Vec<GraphGroup>,
}
