use serde::{Deserialize, Serialize};

pub use business::*;
pub use sys_info::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Health {
    pub is_busy: bool,
    pub info: SystemInfo,
}

pub async fn health() -> Health {
    Health {
        is_busy: business::is_busy(),
        info: sys_info::sys_info().await,
    }
}

mod business {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    use once_cell::sync::Lazy;

    pub static BUSINESS_COUNTER: Lazy<Arc<AtomicUsize>> =
        Lazy::new(|| Arc::new(AtomicUsize::new(0)));

    pub struct BusinessToken;

    impl BusinessToken {
        #[inline]
        pub fn new() -> BusinessToken {
            BUSINESS_COUNTER.fetch_add(1, Ordering::SeqCst);
            BusinessToken
        }
    }

    impl Default for BusinessToken {
        #[inline]
        fn default() -> Self {
            Self::new()
        }
    }

    impl Drop for BusinessToken {
        #[inline]
        fn drop(&mut self) {
            BUSINESS_COUNTER.fetch_sub(1, Ordering::SeqCst);
        }
    }

    #[inline]
    pub fn is_busy() -> bool {
        BUSINESS_COUNTER.load(Ordering::SeqCst) != 0
    }

    #[inline]
    pub fn busy() -> BusinessToken {
        Default::default()
    }
}

mod sys_info {
    use std::sync::Arc;

    use once_cell::sync::Lazy;
    use serde::{Deserialize, Serialize};
    use systemstat::*;
    use tokio::sync::RwLock;

    type Result<T> = std::result::Result<T, String>;

    static CPU_LOAD: Lazy<Arc<RwLock<Result<Vec<CpuLoad>>>>> = Lazy::new(|| {
        let data: Arc<RwLock<Result<Vec<CpuLoad>>>> = Arc::new(RwLock::new(Err("".to_owned())));
        tokio::spawn({
            let data = data.clone();
            let sys = System::new();
            async move {
                loop {
                    match sys.cpu_load() {
                        Ok(mes) => {
                            tokio::time::sleep(Duration::from_secs(1)).await;
                            *data.write().await = mes.done().map_err(|x| x.to_string());
                        }
                        Err(err) => {
                            *data.write().await = Err(err.to_string());
                            tokio::time::sleep(Duration::from_secs(1)).await;
                        }
                    }
                }
            }
        });
        data
    });

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SystemInfo {
        pub mounts: Result<Vec<Filesystem>>,
        pub block_device_statistics: Result<BTreeMap<String, BlockDeviceStats>>,
        pub networks: Result<BTreeMap<String, Network>>,
        pub memory: Result<Memory>,
        pub load_average: Result<LoadAverage>,
        pub cpu_load: Result<Vec<CpuLoad>>,
        pub uptime: Result<Duration>,
        pub boot_time: Result<DateTime<Utc>>,
        pub socket_stats: Result<SocketStats>,
    }

    pub async fn sys_info() -> SystemInfo {
        let sys = System::new();
        SystemInfo {
            mounts: sys.mounts().map_err(|x| x.to_string()),
            block_device_statistics: sys.block_device_statistics().map_err(|x| x.to_string()),
            networks: sys.networks().map_err(|x| x.to_string()),
            memory: sys.memory().map_err(|x| x.to_string()),
            load_average: sys.load_average().map_err(|x| x.to_string()),
            cpu_load: CPU_LOAD.read().await.clone(),
            uptime: sys.uptime().map_err(|x| x.to_string()),
            boot_time: sys.boot_time().map_err(|x| x.to_string()),
            socket_stats: sys.socket_stats().map_err(|x| x.to_string()),
        }
    }
}
