use std::fmt::{Display, Formatter};
use base64::Engine;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use futures::lock::Mutex;

use serde_json::Value;

pub mod graph;
pub mod management;
pub mod pim;

pub struct LazyToken {
    resource_uri: String,
    token: Mutex<Option<Token>>
}

impl LazyToken {
    fn new<S: Into<String>>(resource_uri: S) -> LazyToken {
        LazyToken {
            resource_uri: resource_uri.into(),
            token: Mutex::new(None)
        }
    }

    async fn fetch_token(resource_uri: &str) -> Token {
        #[cfg(target_family = "unix")]
        const EXECUTABLE: &'static str = "az";
        #[cfg(target_family = "windows")]
        const EXECUTABLE: &'static str = "az.cmd";

        let output = tokio::process::Command::new(EXECUTABLE)
            .arg("account")
            .arg("get-access-token")
            .arg("--resource")
            .arg(resource_uri)
            .output()
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&output.stdout).unwrap();
        let access_token = json.get("accessToken").unwrap().as_str().unwrap().to_string();
        let claims_base64 = access_token.split(".").skip(1).next().unwrap();
        let claims = serde_json::from_slice::<Value>(&BASE64_URL_SAFE_NO_PAD.decode(claims_base64).unwrap()).unwrap();
        Token {
            subject_id: claims.get("oid").unwrap().as_str().unwrap().to_owned(),
            access_token,
            tenant_id: claims.get("tid").unwrap().as_str().unwrap().to_owned()
        }
    }

    pub async fn token(&self) -> Token {
        let mut handle = self.token.lock().await;
        if let Some(token) = handle.as_ref() {
            token.clone()
        } else {
            *handle = Some(Self::fetch_token(&self.resource_uri).await);
            handle.as_ref().unwrap().clone()
        }
    }
}

#[derive(Clone)]
pub struct Token {
    pub subject_id: String,
    access_token: String,
    pub tenant_id: String,
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.access_token)
    }
}
