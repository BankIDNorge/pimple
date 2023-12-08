use std::future::{Future, join};
use std::io::Write;
use std::pin::Pin;

use clap::Args;
use futures::future::join_all;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, BufReader, Lines};

use crate::azure::{graph, management, pim};
use crate::cmd::pim::aad_roles::AadRolePim;
use crate::cmd::pim::cache::Cache;
use crate::cmd::pim::group::GroupPim;
use crate::cmd::pim::role::RolePim;

mod aad_roles;
mod cache;
mod group;
mod role;

#[derive(Args)]
pub struct PimArgs {
    #[arg(short, long, help = "Fetch updated cache roles from Azure")]
    refresh: bool,
}

pub async fn pim(args: &PimArgs) {
    let cache = Cache {
        refresh: args.refresh
    };

    let pim_client = pim::PimClient::new();
    let graph_client = graph::GraphClient::new();
    let management_client = management::ManagementClient::new();

    let (group_pim_info, role_pim_info, aad_pim_info) = join!(
        cache.fetch_group_cache(async || { group::fetch_group_info(&pim_client, &graph_client).await }),
        cache.fetch_role_info_cache(async || { role::fetch_role_info(&management_client).await }),
        cache.fetch_aad_role_cache(async || { aad_roles::fetch_aad_role_info(&pim_client).await })
    ).await;

    let group_pim = group_pim_info
        .into_iter()
        .map(|info| GroupPim::new(&pim_client, info))
        .collect::<Vec<GroupPim>>();

    let role_pim = role_pim_info
        .into_iter()
        .map(|info| RolePim::new(&management_client, info))
        .collect::<Vec<RolePim>>();

    let aad_pim = aad_pim_info
        .into_iter()
        .map(|info| AadRolePim::new(&pim_client, info))
        .collect::<Vec<AadRolePim>>();

    let grouped_group_pim = group(&group_pim);
    let grouped_role_pim = group(&role_pim);
    let grouped_aad_pim = group(&aad_pim);

    let mut pim_indexed: Vec<&dyn Pim> = Vec::with_capacity(group_pim.len() + role_pim.len() + aad_pim.len());

    print_and_add(&grouped_group_pim, pim_indexed.as_mut());
    print_and_add(&grouped_role_pim, pim_indexed.as_mut());
    print_and_add(&grouped_aad_pim, pim_indexed.as_mut());

    let reader = BufReader::new(tokio::io::stdin());
    let mut lines = reader.lines();
    let roles_string = prompt(&mut lines, "Please select role(s) separated by ',': ").await;
    let roles = roles_string.split(",")
        .map(|role| role.parse::<usize>().expect("Role ID(s) must be numeric"));

    let (duration, reason) = {
        let handles = (&roles).clone().into_iter()
            .map(|v| pim_indexed[v - 1].ensure_token());

        let (prompts, _) = join!(reason_and_duration(&mut lines), join_all(handles)).await;
        prompts
    };

    join_all(roles.map(|role| {
        pim_indexed[role - 1].activate(reason.clone(), duration.clone())
    })).await;
}

async fn reason_and_duration<T>(lines: &mut Lines<T>) -> (String, String) where T: AsyncBufRead + Unpin {
    (
        prompt(lines, "Select a duration(ISO8601, example `PT30M` or `PT1H`): ").await,
        prompt(lines, "Reason: ").await
    )
}

fn group<'a, T>(pims: &'a Vec<T>) -> Vec<&[T]> where T: Pim<'a> {
    pims.as_slice()
        .group_by(|a, b| a.group_by() == b.group_by())
        .collect()
}

async fn prompt<T>(lines: &mut Lines<T>, prompt: &'static str) -> String where T: AsyncBufRead + Unpin {
    print!("{}", prompt);
    std::io::stdout().flush().unwrap();
    lines.next_line().await.unwrap().unwrap().trim().to_owned()
}

fn print_and_add<'a, T>(grouped: &Vec<&'a [T]>, pim_indexed: &mut Vec<&'a dyn Pim<'a>>) where T: Pim<'a> {
    grouped.into_iter().for_each(|values| {
        println!("{}", values.first().unwrap().group_by());
        values.into_iter().for_each(|group| {
            pim_indexed.push(group);
            println!("{}.\t{}", pim_indexed.len(), group.resource_id());
        })
    });
}

trait Pim<'a> {
    fn group_by(&self) -> String;
    fn resource_id(&self) -> String;
    fn ensure_token(&self) -> Pin<Box<dyn Future<Output=()> + 'a>>;
    fn activate(&self, reason: String, duration: String) -> Pin<Box<dyn Future<Output = ()> + 'a>>;
}
