use std::fs::File;
use std::path::PathBuf;

use clap::Args;
use serde_json::Map;

use crate::kubernetes::get_default_kubeconfig;

#[derive(Args)]
pub struct RefreshAksArgs {
    #[clap(long, env, default_value = get_default_kubeconfig().into_os_string())]
    kubeconfig: PathBuf,
}

pub fn refresh(args: &RefreshAksArgs) {
    let kubelogin_cache_folder = args.kubeconfig.parent().unwrap()
        .join("cache")
        .join("kubelogin");

    kubelogin_cache_folder.read_dir()
        .unwrap()
        .map(|value| value.unwrap())
        .filter(|name| {
            name.file_type().unwrap().is_file() && name.file_name().to_str().unwrap().ends_with(".json")
        })
        .for_each(|file| {
            let mut json: Map<String, serde_json::Value> = {
                serde_json::from_reader(File::open(file.path()).unwrap()).unwrap()
            };
            json.remove("expires_on");
            serde_json::to_writer(File::options().write(true).append(false).open(file.path()).unwrap(), &json).unwrap();
        })
}
