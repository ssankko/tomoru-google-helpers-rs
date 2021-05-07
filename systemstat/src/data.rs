//! This module provides the data structures that represent system information.
//!
//! They're always the same across all platforms.

pub use bytesize::ByteSize;
pub use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
pub use std::collections::BTreeMap;
use std::io;
pub use std::net::{Ipv4Addr, Ipv6Addr};
use std::ops::Sub;
pub use std::time::Duration;

#[inline(always)]
pub fn saturating_sub_bytes(l: ByteSize, r: ByteSize) -> ByteSize {
    ByteSize::b(l.as_u64().saturating_sub(r.as_u64()))
}

/// A wrapper for a measurement that takes time.
///
/// Time should pass between getting the object and calling .done() on it.
pub struct DelayedMeasurement<T> {
    res: Box<dyn Fn() -> io::Result<T> + Send + Sync>,
}

impl<T> DelayedMeasurement<T> {
    #[inline(always)]
    pub fn new(f: Box<dyn Fn() -> io::Result<T> + Send + Sync>) -> DelayedMeasurement<T> {
        DelayedMeasurement { res: f }
    }

    #[inline(always)]
    pub fn done(&self) -> io::Result<T> {
        (self.res)()
    }
}

#[cfg(not(target_os = "linux"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformCpuLoad {}

#[cfg(target_os = "linux")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformCpuLoad {
    pub iowait: f32,
}

impl PlatformCpuLoad {
    #[cfg(target_os = "linux")]
    #[inline(always)]
    pub fn avg_add(self, rhs: &Self) -> Self {
        PlatformCpuLoad {
            iowait: (self.iowait + rhs.iowait) / 2.0,
        }
    }

    #[cfg(not(target_os = "linux"))]
    #[inline(always)]
    pub fn avg_add(self, _rhs: &Self) -> Self {
        PlatformCpuLoad {}
    }

    #[cfg(target_os = "linux")]
    #[inline(always)]
    pub fn zero() -> Self {
        PlatformCpuLoad { iowait: 0.0 }
    }

    #[cfg(not(target_os = "linux"))]
    #[inline(always)]
    pub fn zero() -> Self {
        PlatformCpuLoad {}
    }

    #[cfg(target_os = "linux")]
    #[inline(always)]
    pub fn from(input: f32) -> Self {
        PlatformCpuLoad { iowait: input }
    }

    #[cfg(not(target_os = "linux"))]
    #[inline(always)]
    pub fn from(_input: f32) -> Self {
        PlatformCpuLoad {}
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuLoad {
    pub user: f32,
    pub nice: f32,
    pub system: f32,
    pub interrupt: f32,
    pub idle: f32,
    pub platform: PlatformCpuLoad,
}

impl CpuLoad {
    #[inline(always)]
    pub fn avg_add(self, rhs: &Self) -> Self {
        CpuLoad {
            user: (self.user + rhs.user) / 2.0,
            nice: (self.nice + rhs.nice) / 2.0,
            system: (self.system + rhs.system) / 2.0,
            interrupt: (self.interrupt + rhs.interrupt) / 2.0,
            idle: (self.idle + rhs.idle) / 2.0,
            platform: self.platform.avg_add(&rhs.platform),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CpuTime {
    pub user: usize,
    pub nice: usize,
    pub system: usize,
    pub interrupt: usize,
    pub idle: usize,
    pub other: usize,
}

impl<'a> Sub<&'a CpuTime> for CpuTime {
    type Output = CpuTime;

    #[inline(always)]
    fn sub(self, rhs: &CpuTime) -> CpuTime {
        CpuTime {
            user: self.user.saturating_sub(rhs.user),
            nice: self.nice.saturating_sub(rhs.nice),
            system: self.system.saturating_sub(rhs.system),
            interrupt: self.interrupt.saturating_sub(rhs.interrupt),
            idle: self.idle.saturating_sub(rhs.idle),
            other: self.other.saturating_sub(rhs.other),
        }
    }
}

impl CpuTime {
    pub fn to_cpuload(&self) -> CpuLoad {
        let total = self.user + self.nice + self.system + self.interrupt + self.idle + self.other;
        if total == 0 {
            CpuLoad {
                user: 0.0,
                nice: 0.0,
                system: 0.0,
                interrupt: 0.0,
                idle: 0.0,
                platform: PlatformCpuLoad::zero(),
            }
        } else {
            CpuLoad {
                user: self.user as f32 / total as f32,
                nice: self.nice as f32 / total as f32,
                system: self.system as f32 / total as f32,
                interrupt: self.interrupt as f32 / total as f32,
                idle: self.idle as f32 / total as f32,
                platform: PlatformCpuLoad::from(self.other as f32 / total as f32),
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadAverage {
    pub one: f32,
    pub five: f32,
    pub fifteen: f32,
}

#[cfg(target_os = "windows")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformMemory {
    pub load: u32,
    pub total_phys: ByteSize,
    pub avail_phys: ByteSize,
    pub total_pagefile: ByteSize,
    pub avail_pagefile: ByteSize,
    pub total_virt: ByteSize,
    pub avail_virt: ByteSize,
    pub avail_ext: ByteSize,
}

#[cfg(target_os = "freebsd")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformMemory {
    pub active: ByteSize,
    pub inactive: ByteSize,
    pub wired: ByteSize,
    pub cache: ByteSize,
    pub zfs_arc: ByteSize,
    pub free: ByteSize,
}

#[cfg(target_os = "openbsd")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformMemory {
    pub active: ByteSize,
    pub inactive: ByteSize,
    pub wired: ByteSize,
    pub cache: ByteSize,
    pub free: ByteSize,
    pub paging: ByteSize,
}

#[cfg(target_os = "macos")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformMemory {
    pub active: ByteSize,
    pub inactive: ByteSize,
    pub wired: ByteSize,
    pub cache: ByteSize,
    pub free: ByteSize,
}

#[cfg(any(target_os = "linux", target_os = "android"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformMemory {
    pub meminfo: BTreeMap<String, ByteSize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub total: ByteSize,
    pub free: ByteSize,
    pub platform_memory: PlatformMemory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryLife {
    pub remaining_capacity: f32,
    pub remaining_time: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filesystem {
    pub files: usize,
    pub files_total: usize,
    pub files_avail: usize,
    pub free: ByteSize,
    pub avail: ByteSize,
    pub total: ByteSize,
    pub name_max: usize,
    pub fs_type: String,
    pub fs_mounted_from: String,
    pub fs_mounted_on: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDeviceStats {
    pub name: String,
    pub read_ios: usize,
    pub read_merges: usize,
    pub read_sectors: usize,
    pub read_ticks: usize,
    pub write_ios: usize,
    pub write_merges: usize,
    pub write_sectors: usize,
    pub write_ticks: usize,
    pub in_flight: usize,
    pub io_ticks: usize,
    pub time_in_queue: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IpAddr {
    Empty,
    Unsupported,
    V4(Ipv4Addr),
    V6(Ipv6Addr),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkAddrs {
    pub addr: IpAddr,
    pub netmask: IpAddr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Network {
    pub name: String,
    pub addrs: Vec<NetworkAddrs>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub rx_bytes: ByteSize,
    pub tx_bytes: ByteSize,
    pub rx_packets: u64,
    pub tx_packets: u64,
    pub rx_errors: u64,
    pub tx_errors: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocketStats {
    pub tcp_sockets_in_use: usize,
    pub tcp_sockets_orphaned: usize,
    pub udp_sockets_in_use: usize,
    pub tcp6_sockets_in_use: usize,
    pub udp6_sockets_in_use: usize,
}
