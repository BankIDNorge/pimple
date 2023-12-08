use std::future::Future;
use std::pin::Pin;

use serde::{Deserialize, Serialize};

use crate::azure::management;
use crate::cmd::pim::Pim;

pub async fn fetch_role_info(management_client: &management::ManagementClient) -> Vec<RoleInfo> {
    management_client.get_available_roles().await
        .into_iter()
        .map(|role| RoleInfo {
            role_name: role.properties.expanded_properties.role_definition.display_name,
            scope_name: role.properties.expanded_properties.scope.display_name,
            scope: role.properties.expanded_properties.scope.id,
            role_definition_id: role.properties.role_definition_id,
        })
        .collect()
}

#[derive(Serialize, Deserialize)]
pub struct RoleInfo {
    role_name: String,
    scope_name: String,
    scope: String,
    role_definition_id: String,
}

pub struct RolePim<'a> {
    management_client: &'a management::ManagementClient,
    role_info: RoleInfo,
}

impl<'a> RolePim<'a> {
    pub fn new(management_client: &'a management::ManagementClient, role_info: RoleInfo) -> RolePim<'a> {
        RolePim {
            management_client,
            role_info,
        }
    }
}


impl<'a> Pim<'a> for RolePim<'a> {
    fn group_by(&self) -> String {
        self.role_info.role_name.to_owned()
    }

    fn resource_id(&self) -> String {
        self.role_info.scope_name.to_owned()
    }

    fn ensure_token(&self) -> Pin<Box<dyn Future<Output=()> + 'a>> {
        Box::pin(self.management_client.ensure_token())
    }

    fn activate(&self, reason: String, duration: String) -> Pin<Box<dyn Future<Output=()> + 'a>> {
        let scope = self.role_info.scope.clone();
        let role_definition_id = self.role_info.role_definition_id.clone();
        let role_assignment_id = uuid::Uuid::new_v4().to_string();
        Box::pin(async {
            println!(
                "{}",
                self.management_client.activate_role(
                    reason,
                    duration,
                    scope,
                    role_definition_id,
                    role_assignment_id,
                ).await
            );
        })
    }
}
