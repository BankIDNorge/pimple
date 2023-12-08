use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::azure::LazyToken;

const MS_PIM_URL: &'static str = "https://api.azrbac.mspim.azure.com";

pub struct PimClient {
    client: Client,
    token: LazyToken,
}

impl PimClient {
    pub fn new() -> Self {
        PimClient {
            client: Client::new(),
            token: LazyToken::new(MS_PIM_URL),
        }
    }

    pub async fn fetch_group_pim(&self) -> Vec<AssignableGroup> {
        let token = self.token.token().await;
        const EXPAND: &'static str = "$expand=roleDefinition($expand=resource)";
        let filter = format!(
            "$filter=(subject/id eq '{}')+and+(assignmentState eq 'Eligible')",
            token.subject_id
        );
        self.client.get(format!(
            "{}/api/v2/privilegedAccess/aadGroups/roleAssignments?{}&{}",
            MS_PIM_URL,
            EXPAND,
            filter
        ))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .unwrap()
            .json::<GroupPimAssignments>()
            .await
            .unwrap()
            .value
    }

    async fn request_access(
        &self,
        reason: String,
        duration: String,
        resource_id: String,
        role_assignment_id: String,
        role_definition_id: String,
        url: String,
    ) -> String {
        let token = self.token.token().await;
        self.client.post(url)
            .header("Authorization", format!("Bearer {}", &token))
            .json(&RoleAssignment {
                assignment_state: "Active".to_owned(),
                linked_eligible_role_assignment_id: role_assignment_id,
                reason,
                resource_id,
                role_definition_id,
                schedule: RoleAssignmentSchedule {
                    duration,
                    schedule_type: "Once".to_owned(),
                },
                scoped_resource_id: "".to_string(),
                subject_id: token.subject_id.to_owned(),
                ticket_number: "".to_string(),
                ticket_system: "".to_string(),
                assignment_type: "UserAdd".to_string(),
            })
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap()
    }

    pub async fn request_group_access(
        &self,
        reason: String,
        duration: String,
        group_id: String,
        role_assignment_id: String,
        role_definition_id: String,
    ) -> String {
        self.request_access(
            reason,
            duration,
            group_id,
            role_assignment_id,
            role_definition_id,
            format!("{}/api/v2/privilegedAccess/aadGroups/roleAssignmentRequests", MS_PIM_URL)
        ).await
    }

    pub async fn request_aad_role_access(
        &self,
        reason: String,
        duration: String,
        tenant_id: String,
        role_assignment_id: String,
        role_definition_id: String,
    ) -> String {
        self.request_access(
            reason,
            duration,
            tenant_id,
            role_assignment_id,
            role_definition_id,
            format!("{}/api/v2/privilegedAccess/aadroles/roleAssignmentRequests", MS_PIM_URL)
        ).await
    }

    pub async fn get_aad_roles(&self) -> Vec<AssignableGroup> {
        let token = self.token.token().await;
        const EXPAND: &'static str = "$expand=roleDefinition($expand=resource)";
        let filter = format!(
            "$filter=(subject/id eq '{}') and (assignmentState eq 'Eligible')",
            token.subject_id
        );
        let url = format!(
            "{}/api/v2/privilegedAccess/aadroles/roleAssignments?{}&{}",
            MS_PIM_URL,
            EXPAND,
            filter
        );
        self.client.get(url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .unwrap()
            .json::<GroupPimAssignments>()
            .await
            .unwrap()
            .value
    }

    pub async fn ensure_token(&self) {
        self.token.token().await;
    }
}

#[derive(Debug, Eq, PartialEq, Deserialize)]
struct GroupPimAssignments {
    value: Vec<AssignableGroup>,
}

#[allow(non_snake_case)]
#[derive(Debug, Eq, PartialEq, Deserialize)]
pub struct AssignableGroup {
    pub id: String,
    pub resourceId: String,
    // Group id
    pub roleDefinitionId: String,
    // Member/Owner
    pub subjectId: String,
    // Objektid til bruker
    pub roleDefinition: RoleDefinition,
}

#[allow(non_snake_case)]
#[derive(Debug, Eq, PartialEq, Deserialize)]
pub struct RoleDefinition {
    pub displayName: String,
    pub resource: RoleResource,
}

#[allow(non_snake_case)]
#[derive(Debug, Eq, PartialEq, Deserialize)]
pub struct RoleResource {
    pub displayName: String,
}

#[derive(Serialize)]
struct RoleAssignmentSchedule {
    pub duration: String,
    #[serde(rename = "type")]
    pub schedule_type: String,
}

#[derive(Serialize)]
struct RoleAssignment {
    #[serde(rename = "assignmentState")]
    pub assignment_state: String,
    #[serde(rename = "linkedEligibleRoleAssignmentId")]
    pub linked_eligible_role_assignment_id: String,
    pub reason: String,
    #[serde(rename = "resourceId")]
    pub resource_id: String,
    #[serde(rename = "roleDefinitionId")]
    pub role_definition_id: String,
    pub schedule: RoleAssignmentSchedule,
    #[serde(rename = "scopedResourceId")]
    pub scoped_resource_id: String,
    #[serde(rename = "subjectId")]
    pub subject_id: String,
    #[serde(rename = "ticketNumber")]
    pub ticket_number: String,
    #[serde(rename = "ticketSystem")]
    pub ticket_system: String,
    #[serde(rename = "type")]
    pub assignment_type: String,
}

