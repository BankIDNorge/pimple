use std::future::Future;
use std::pin::Pin;
use crate::azure::{graph, pim};
use crate::cmd::pim::Pim;
use serde::{Serialize, Deserialize};

pub async fn fetch_group_info(pim_client: &pim::PimClient, graph_client: &graph::GraphClient) -> Vec<GroupInfo> {
    let groups = pim_client.fetch_group_pim().await;
    if groups.is_empty() {
        return vec![];
    }

    let group_query = create_group_query(&groups);
    let extra_group_info = graph_client.get_groups(&group_query).await;
    return groups.into_iter()
        .map(|group| {
            GroupInfo {
                role_assignment_id: group.id,
                group_description: (&extra_group_info).into_iter()
                    .find(|g| g.id == group.resourceId)
                    .map(|g| g.description.clone().unwrap_or_else(|| g.display_name.clone())),
                group_object_id: group.resourceId,
                role_definition_id: group.roleDefinitionId,
                role_definition_name: group.roleDefinition.displayName,
                group_name: group.roleDefinition.resource.displayName,
            }
        })
        .collect();
}

fn create_group_query(groups: &Vec<pim::AssignableGroup>) -> String {
    let string_size = groups.len() * (36 + 11); // length of UUID + ", "
    let group_ids = (&groups).into_iter()
        .fold(String::with_capacity(string_size), |mut result, v| {
            result.push_str("or id eq '");
            result.push_str(&v.resourceId);
            result.push_str("'");
            result
        });
    let group_query = format!("$filter={}", &group_ids[3..]);
    group_query
}

#[derive(Serialize, Deserialize)]
pub struct GroupInfo {
    pub role_assignment_id: String,
    pub group_object_id: String,
    pub role_definition_id: String,
    pub role_definition_name: String,
    pub group_name: String,
    pub group_description: Option<String>,
}

pub struct GroupPim<'a> {
    pim_client: &'a pim::PimClient,
    group_info: GroupInfo,
}

impl<'a> GroupPim<'a> {
    pub fn new(pim_client: &'a pim::PimClient, group_info: GroupInfo) -> GroupPim<'a> {
        GroupPim {
            pim_client,
            group_info,
        }
    }
}

impl<'a> Pim<'a> for GroupPim<'a> {
    fn group_by(&self) -> String {
        self.group_info.group_description.clone().unwrap_or(self.group_info.group_name.to_owned())
    }

    fn resource_id(&self) -> String {
        format!("{} ({})", self.group_info.group_name, self.group_info.role_definition_name)
    }

    fn activate(&self, reason: String, duration: String) -> Pin<Box<dyn Future<Output = ()> + 'a>> {

        let group_id = self.group_info.group_object_id.to_owned();
        let role_assignment_id = self.group_info.role_assignment_id.to_owned();
        let role_definition_id = self.group_info.role_definition_id.to_owned();
        Box::pin(async {
            println!(
                "{}",
                self.pim_client.request_group_access(
                    reason,
                    duration,
                    group_id,
                    role_assignment_id,
                    role_definition_id,
                ).await
            )
        })
    }


    fn ensure_token(&self) -> Pin<Box<dyn Future<Output = ()> + 'a>> {
        Box::pin(self.pim_client.ensure_token())
    }
}
