use std::path::PathBuf;
use home::home_dir;

pub fn get_default_kubeconfig() -> PathBuf {
    home_dir().unwrap().join(".kube").join("config")
}
