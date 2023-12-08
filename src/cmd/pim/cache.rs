use std::fs::File;
use std::future::Future;
use std::path::PathBuf;
use home::home_dir;
use serde::Serialize;
use serde::de::DeserializeOwned;
use crate::cmd::pim::aad_roles::AadRoleInfo;
use crate::cmd::pim::group::GroupInfo;
use crate::cmd::pim::role::RoleInfo;

pub struct Cache {
    pub refresh: bool,
}

impl Cache {
    fn get_cache_dir() -> PathBuf {
        home_dir().unwrap().join(".config").join("pimple").join("cache")
    }

    fn get_cache_file(cache_type: &str) -> PathBuf {
        Cache::get_cache_dir().join(cache_type)
    }

    fn get_cache<T: DeserializeOwned>(cache_type: &str) -> Option<T> {
        let cache_file = Cache::get_cache_file(cache_type);
        return if cache_file.exists() {
            serde_json::from_reader(File::open(cache_file).unwrap()).unwrap()
        } else {
            None
        };
    }

    fn save_cache<T: Serialize>(cache_type: &str, data: &T) {
        let cache_dir = Cache::get_cache_dir();
        if !cache_dir.exists() {
            std::fs::create_dir_all(cache_dir).unwrap();
        }
        let cache_file = File::options()
            .write(true)
            .truncate(true)
            .create(true)
            .open(Cache::get_cache_file(cache_type))
            .unwrap();
        serde_json::to_writer(cache_file, data).unwrap()
    }

    async fn fetch_cache<Fut, O: DeserializeOwned + Serialize>(
        &self,
        default: impl FnOnce() -> Fut,
        path: &str,
    ) -> O where Fut: Future<Output=O> {
        if !self.refresh {
            if let Some(cache) = Cache::get_cache(path) {
                return cache;
            }
        }
        let data = default().await;
        Cache::save_cache(path, &data);
        data
    }

    pub async fn fetch_group_cache<Fut>(
        &self,
        default: impl FnOnce() -> Fut,
    ) -> Vec<GroupInfo> where Fut: Future<Output=Vec<GroupInfo>> {
        self.fetch_cache(default, "groups.json").await
    }

    pub async fn fetch_role_info_cache<Fut>(
        &self,
        default: impl FnOnce() -> Fut,
    ) -> Vec<RoleInfo> where Fut: Future<Output=Vec<RoleInfo>> {
        self.fetch_cache(default, "roles.json").await
    }

    pub async fn fetch_aad_role_cache<Fut>(
        &self,
        default: impl FnOnce() -> Fut,
    ) -> Vec<AadRoleInfo> where Fut: Future<Output=Vec<AadRoleInfo>> {
        self.fetch_cache(default, "aad_roles.json").await
    }
}
