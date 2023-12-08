use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::azure::LazyToken;

const MANAGEMENT_URL: &'static str = "https://management.azure.com";

pub struct ManagementClient {
    client: Client,
    token: LazyToken,
}

impl ManagementClient {
    pub fn new() -> Self {
        ManagementClient {
            token: LazyToken::new(MANAGEMENT_URL),
            client: Client::new(),
        }
    }

    pub async fn ensure_token(&self) {
        self.token.token().await;
    }

    pub async fn get_available_roles(&self) -> Vec<Role> {
        self.client.get(format!("{}/providers/Microsoft.Authorization/roleEligibilityScheduleInstances?api-version=2020-10-01&$filter=asTarget()", MANAGEMENT_URL))
            .header("Authorization", format!("Bearer {}", self.token.token().await))
            .send()
            .await
            .unwrap()
            .json::<RoleResponse>()
            .await
            .unwrap()
            .value
    }

    pub async fn activate_role(&self, reason: String, duration: String, scope: String, role_definition_id: String, role_assignment_id: String) -> String {
        let token = self.token.token().await;
        self.client.put(format!("{}{}/providers/Microsoft.Authorization/roleAssignmentScheduleRequests/{}?api-version=2020-10-01", MANAGEMENT_URL, scope, role_assignment_id))
            .header("Authorization", format!("Bearer {}", &token.access_token))
            .json(&RoleAssignmentRequest {
                properties: RoleAssignmentProperties {
                    role_definition_id,
                    principal_id: token.subject_id.to_owned(),
                    request_type: "SelfActivate".to_string(),
                    justification: reason,
                    schedule_info: RoleAssignmentScheduleInfo {
                        expiration: RoleAssignmentExpiration {
                            expiry_type: "AfterDuration".to_string(),
                            duration,
                        },
                    },
                },
            })
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap()
    }
}

#[derive(Serialize, Deserialize)]
pub struct RoleNamedResource {
    pub id: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
}

#[derive(Serialize, Deserialize)]
pub struct ExpandedProperties {
    #[serde(rename = "roleDefinition")]
    pub role_definition: RoleNamedResource,
    pub scope: RoleNamedResource,
}

#[derive(Serialize, Deserialize)]
pub struct RoleProperties {
    #[serde(rename = "roleDefinitionId")]
    pub role_definition_id: String,
    #[serde(rename = "expandedProperties")]
    pub expanded_properties: ExpandedProperties,
}

#[derive(Serialize, Deserialize)]
pub struct Role {
    pub properties: RoleProperties,
    pub name: String,
    pub id: String,
}

#[derive(Serialize, Deserialize)]
struct RoleResponse {
    pub value: Vec<Role>,
}

#[derive(Serialize)]
struct RoleAssignmentRequest {
    #[serde(rename = "Properties")]
    properties: RoleAssignmentProperties,
}

#[derive(Serialize)]
struct RoleAssignmentProperties {
    #[serde(rename = "RoleDefinitionId")]
    role_definition_id: String,
    #[serde(rename = "PrincipalId")]
    principal_id: String,
    #[serde(rename = "RequestType")]
    request_type: String,
    #[serde(rename = "Justification")]
    justification: String,
    #[serde(rename = "ScheduleInfo")]
    schedule_info: RoleAssignmentScheduleInfo,
}

#[derive(Serialize)]
struct RoleAssignmentScheduleInfo {
    #[serde(rename = "Expiration")]
    expiration: RoleAssignmentExpiration,
}

#[derive(Serialize)]
struct RoleAssignmentExpiration {
    #[serde(rename = "Type")]
    expiry_type: String,
    //"AfterDuration",
    #[serde(rename = "Duration")]
    duration: String,
}
