use std::future::Future;
use std::pin::Pin;

use serde::{Deserialize, Serialize};

use crate::azure::pim::PimClient;
use crate::cmd::pim::Pim;

pub async fn fetch_aad_role_info(pim_client: &PimClient) -> Vec<AadRoleInfo> {
    let groups = pim_client.fetch_group_pim().await;
    if groups.is_empty() {
        return vec![];
    }
    pim_client.get_aad_roles().await
        .into_iter()
        .map(|aad_role| AadRoleInfo {
            role_assignment_id: aad_role.id,
            tenant_id: aad_role.resourceId,
            role_definition_id: aad_role.roleDefinitionId,
            role_definition_name: aad_role.roleDefinition.displayName,
            role_name: aad_role.roleDefinition.resource.displayName,
            role_description: None,
        })
        .collect()
}

#[derive(Serialize, Deserialize)]
pub struct AadRoleInfo {
    pub role_assignment_id: String,
    pub tenant_id: String,
    pub role_definition_id: String,
    pub role_definition_name: String,
    pub role_name: String,
    pub role_description: Option<String>,
}

pub struct AadRolePim<'a> {
    pim_client: &'a PimClient,
    aad_role_info: AadRoleInfo,
}

impl<'a> AadRolePim<'a> {
    pub fn new(pim_client: &'a PimClient, aad_role_info: AadRoleInfo) -> AadRolePim<'a> {
        AadRolePim {
            pim_client,
            aad_role_info,
        }
    }
}

impl<'a> Pim<'a> for AadRolePim<'a> {
    fn group_by(&self) -> String {
        self.aad_role_info.role_description.clone().unwrap_or(self.aad_role_info.role_name.to_owned())
    }

    fn resource_id(&self) -> String {
        format!("{} ({})", self.aad_role_info.role_name, self.aad_role_info.role_definition_name)
    }

    fn ensure_token(&self) -> Pin<Box<dyn Future<Output=()> + 'a>> {
        Box::pin(self.pim_client.ensure_token())
    }

    fn activate(&self, reason: String, duration: String) -> Pin<Box<dyn Future<Output=()> + 'a>> {
        let tenant_id = self.aad_role_info.tenant_id.to_owned();
        let role_assignment_id = self.aad_role_info.role_assignment_id.to_owned();
        let role_definition_id = self.aad_role_info.role_definition_id.to_owned();
        Box::pin(async {
            println!(
                "{}",
                self.pim_client.request_aad_role_access(
                    reason,
                    duration,
                    tenant_id,
                    role_assignment_id,
                    role_definition_id,
                ).await
            );
        })
    }
}
