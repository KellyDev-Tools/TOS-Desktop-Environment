//! Container Security Implementation
//!
//! Security policies, seccomp profiles, and capability management
//! for containerized TOS components.

use super::{ContainerConfig, ContainerResult, ContainerError};
use serde::{Deserialize, Serialize};

/// Security policy for containers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    /// Policy name
    pub name: String,
    /// Policy version
    pub version: String,
    /// Seccomp profile
    pub seccomp: SeccompProfile,
    /// AppArmor profile
    pub apparmor: Option<String>,
    /// SELinux options
    pub selinux: Vec<String>,
    /// Capabilities
    pub capabilities: CapabilityPolicy,
    /// No new privileges
    pub no_new_privileges: bool,
    /// Read-only root filesystem
    pub read_only_rootfs: bool,
    /// User namespace remapping
    pub userns_remap: bool,
    /// Drop all capabilities by default
    pub drop_all_caps: bool,
    /// Allowed syscalls (if seccomp is custom)
    pub allowed_syscalls: Vec<String>,
    /// Denied syscalls
    pub denied_syscalls: Vec<String>,
    /// Security options
    pub security_opts: Vec<String>,
    /// Graphical isolation (X11/Wayland)
    pub display_isolation: DisplayIsolation,
    /// Audio isolation (PulseAudio/PipeWire)
    pub audio_isolation: AudioIsolation,
    /// Network isolation level
    pub network_isolation: NetworkIsolation,
}

/// Graphical isolation levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisplayIsolation {
    /// No display access
    None,
    /// Shared display with host (insecure)
    Shared,
    /// Virtualized / Proxied display (via Wayland-Proxy or separate X11)
    Virtualized,
}

/// Audio isolation levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioIsolation {
    /// No audio access
    None,
    /// Shared audio with host (passthrough)
    Shared,
    /// Filtered / Proxied audio
    Filtered,
}

/// Network isolation levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkIsolation {
    /// No network access
    None,
    /// Shared with host
    Host,
    /// Private bridge network (default)
    Bridge,
    /// Internal only (no outbound internet)
    Internal,
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            name: "tos-default".to_string(),
            version: "1.0.0".to_string(),
            seccomp: SeccompProfile::Default,
            apparmor: Some("docker-default".to_string()),
            selinux: Vec::new(),
            capabilities: CapabilityPolicy::default(),
            no_new_privileges: true,
            read_only_rootfs: true,
            userns_remap: false,
            drop_all_caps: true,
            allowed_syscalls: Vec::new(),
            denied_syscalls: Vec::new(),
            security_opts: vec!["no-new-privileges:true".to_string()],
            display_isolation: DisplayIsolation::None,
            audio_isolation: AudioIsolation::None,
            network_isolation: NetworkIsolation::Bridge,
        }
    }
}

impl SecurityPolicy {
    /// Create a minimal security policy (less restrictive)
    pub fn minimal() -> Self {
        Self {
            name: "tos-minimal".to_string(),
            version: "1.0.0".to_string(),
            seccomp: SeccompProfile::Unconfined,
            apparmor: None,
            selinux: Vec::new(),
            capabilities: CapabilityPolicy::all(),
            no_new_privileges: false,
            read_only_rootfs: false,
            userns_remap: false,
            drop_all_caps: false,
            allowed_syscalls: Vec::new(),
            denied_syscalls: Vec::new(),
            security_opts: Vec::new(),
            display_isolation: DisplayIsolation::Shared,
            audio_isolation: AudioIsolation::Shared,
            network_isolation: NetworkIsolation::Host,
        }
    }
    
    /// Create a restricted security policy (most secure)
    pub fn restricted() -> Self {
        Self {
            name: "tos-restricted".to_string(),
            version: "1.0.0".to_string(),
            seccomp: SeccompProfile::Restricted,
            apparmor: Some("tos-restricted".to_string()),
            selinux: vec!["type:tos_t".to_string(), "level:s0:c1,c2".to_string()],
            capabilities: CapabilityPolicy::minimal(),
            no_new_privileges: true,
            read_only_rootfs: true,
            userns_remap: true,
            drop_all_caps: true,
            allowed_syscalls: Vec::new(),
            denied_syscalls: Vec::new(),
            security_opts: vec![
                "no-new-privileges:true".to_string(),
                "seccomp:restricted".to_string(),
            ],
            display_isolation: DisplayIsolation::Virtualized,
            audio_isolation: AudioIsolation::Filtered,
            network_isolation: NetworkIsolation::Internal,
        }
    }
    
    /// Validate a container config against this policy
    pub fn validate(&self, config: &ContainerConfig) -> ContainerResult<()> {
        // Check no_new_privileges
        if self.no_new_privileges && !config.security_options.no_new_privileges {
            return Err(ContainerError::Security(
                "Policy requires no-new-privileges".to_string()
            ));
        }
        
        // Check read-only rootfs
        if self.read_only_rootfs && !config.read_only {
            return Err(ContainerError::Security(
                "Policy requires read-only root filesystem".to_string()
            ));
        }
        
        // Check privileged mode
        if config.privileged {
            return Err(ContainerError::Security(
                "Privileged containers are not allowed".to_string()
            ));
        }
        
        // Check capabilities
        if self.drop_all_caps {
            // Verify all capabilities are dropped
            if !config.cap_drop.contains(&"ALL".to_string()) {
                return Err(ContainerError::Security(
                    "Policy requires dropping all capabilities".to_string()
                ));
            }
        }
        
        // Check allowed capabilities
        for cap in &config.cap_add {
            if !self.capabilities.is_allowed(cap) {
                return Err(ContainerError::Security(
                    format!("Capability {} is not allowed by policy", cap)
                ));
            }
        }
        
        Ok(())
    }
    
    /// Apply policy to container config
    pub fn apply(&self, config: &mut ContainerConfig) {
        // Apply security options
        config.security_options.no_new_privileges = self.no_new_privileges;
        config.read_only = self.read_only_rootfs;
        
        // Apply seccomp
        if let SeccompProfile::Custom(ref path) = self.seccomp {
            config.security_options.seccomp_profile = Some(path.clone());
        }
        
        // Apply AppArmor
        config.security_options.apparmor_profile = self.apparmor.clone();
        
        // Apply SELinux
        config.security_options.selinux_options = self.selinux.clone();
        
        // Apply capabilities
        if self.drop_all_caps {
            config.cap_drop = vec!["ALL".to_string()];
            config.cap_add = self.capabilities.allowed.clone();
        }
        
        // Apply security opts
        config.security_options.security_opts = self.security_opts.clone();
    }
}

/// Seccomp profile types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SeccompProfile {
    /// Default Docker seccomp profile
    Default,
    /// Unconfined (no seccomp)
    Unconfined,
    /// Restricted profile (minimal syscalls)
    Restricted,
    /// Custom profile from file path
    Custom(String),
}

impl SeccompProfile {
    /// Get profile name
    pub fn name(&self) -> String {
        match self {
            Self::Default => "default".to_string(),
            Self::Unconfined => "unconfined".to_string(),
            Self::Restricted => "restricted".to_string(),
            Self::Custom(path) => format!("custom:{}", path),
        }
    }
}

/// Capability policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityPolicy {
    /// Allowed capabilities
    pub allowed: Vec<String>,
    /// Required capabilities (cannot be dropped)
    pub required: Vec<String>,
    /// Denied capabilities (cannot be added)
    pub denied: Vec<String>,
}

impl Default for CapabilityPolicy {
    fn default() -> Self {
        Self::standard()
    }
}

impl CapabilityPolicy {
    /// Standard capability set for TOS containers
    pub fn standard() -> Self {
        Self {
            allowed: vec![
                "CHOWN".to_string(),
                "DAC_OVERRIDE".to_string(),
                "FSETID".to_string(),
                "FOWNER".to_string(),
                "MKNOD".to_string(),
                "NET_RAW".to_string(),
                "SETGID".to_string(),
                "SETUID".to_string(),
                "SETFCAP".to_string(),
                "SETPCAP".to_string(),
                "NET_BIND_SERVICE".to_string(),
                "SYS_CHROOT".to_string(),
                "KILL".to_string(),
                "AUDIT_WRITE".to_string(),
            ],
            required: Vec::new(),
            denied: vec![
                "SYS_ADMIN".to_string(),
                "SYS_PTRACE".to_string(),
                "SYS_MODULE".to_string(),
                "SYS_RAWIO".to_string(),
                "SYS_BOOT".to_string(),
                "MAC_ADMIN".to_string(),
                "MAC_OVERRIDE".to_string(),
                "NET_ADMIN".to_string(),
            ],
        }
    }
    
    /// Minimal capability set (most restrictive)
    pub fn minimal() -> Self {
        Self {
            allowed: vec![
                "CHOWN".to_string(),
                "SETGID".to_string(),
                "SETUID".to_string(),
                "KILL".to_string(),
            ],
            required: Vec::new(),
            denied: vec![
                "SYS_ADMIN".to_string(),
                "SYS_PTRACE".to_string(),
                "SYS_MODULE".to_string(),
                "SYS_RAWIO".to_string(),
                "SYS_BOOT".to_string(),
                "MAC_ADMIN".to_string(),
                "MAC_OVERRIDE".to_string(),
                "NET_ADMIN".to_string(),
                "NET_RAW".to_string(),
                "DAC_OVERRIDE".to_string(),
                "FOWNER".to_string(),
                "MKNOD".to_string(),
                "SETFCAP".to_string(),
                "SETPCAP".to_string(),
            ],
        }
    }
    
    /// All capabilities allowed (least restrictive)
    pub fn all() -> Self {
        Self {
            allowed: all_capabilities(),
            required: Vec::new(),
            denied: Vec::new(),
        }
    }
    
    /// Check if a capability is allowed
    pub fn is_allowed(&self, cap: &str) -> bool {
        let cap_upper = cap.to_uppercase();
        self.allowed.contains(&cap_upper) && !self.denied.contains(&cap_upper)
    }
    
    /// Check if a capability is required
    pub fn is_required(&self, cap: &str) -> bool {
        self.required.contains(&cap.to_uppercase())
    }
    
    /// Check if a capability is denied
    pub fn is_denied(&self, cap: &str) -> bool {
        self.denied.contains(&cap.to_uppercase())
    }
}

/// Linux capability
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Capability {
    Chown,           // 0
    DacOverride,     // 1
    DacReadSearch,   // 2
    Fowner,          // 3
    Fsetid,          // 4
    Kill,            // 5
    Setgid,          // 6
    Setuid,          // 7
    Setpcap,         // 8
    LinuxImmutable,  // 9
    NetBindService,  // 10
    NetBroadcast,    // 11
    NetAdmin,        // 12
    NetRaw,          // 13
    IpcLock,         // 14
    IpcOwner,        // 15
    SysModule,       // 16
    SysRawio,        // 17
    SysChroot,       // 18
    SysPtrace,       // 19
    SysPacct,        // 20
    SysAdmin,        // 21
    SysBoot,         // 22
    SysNice,         // 23
    SysResource,     // 24
    SysTime,         // 25
    SysTtyConfig,    // 26
    Mknod,           // 27
    Lease,           // 28
    AuditWrite,      // 29
    AuditControl,    // 30
    Setfcap,         // 31
    MacOverride,     // 32
    MacAdmin,        // 33
    Syslog,          // 34
    WakeAlarm,       // 35
    BlockSuspend,    // 36
    AuditRead,       // 37
    Perfmon,         // 38
    Bpf,             // 39
    CheckpointRestore, // 40
}

impl Capability {
    /// Get capability name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Chown => "CHOWN",
            Self::DacOverride => "DAC_OVERRIDE",
            Self::DacReadSearch => "DAC_READ_SEARCH",
            Self::Fowner => "FOWNER",
            Self::Fsetid => "FSETID",
            Self::Kill => "KILL",
            Self::Setgid => "SETGID",
            Self::Setuid => "SETUID",
            Self::Setpcap => "SETPCAP",
            Self::LinuxImmutable => "LINUX_IMMUTABLE",
            Self::NetBindService => "NET_BIND_SERVICE",
            Self::NetBroadcast => "NET_BROADCAST",
            Self::NetAdmin => "NET_ADMIN",
            Self::NetRaw => "NET_RAW",
            Self::IpcLock => "IPC_LOCK",
            Self::IpcOwner => "IPC_OWNER",
            Self::SysModule => "SYS_MODULE",
            Self::SysRawio => "SYS_RAWIO",
            Self::SysChroot => "SYS_CHROOT",
            Self::SysPtrace => "SYS_PTRACE",
            Self::SysPacct => "SYS_PACCT",
            Self::SysAdmin => "SYS_ADMIN",
            Self::SysBoot => "SYS_BOOT",
            Self::SysNice => "SYS_NICE",
            Self::SysResource => "SYS_RESOURCE",
            Self::SysTime => "SYS_TIME",
            Self::SysTtyConfig => "SYS_TTY_CONFIG",
            Self::Mknod => "MKNOD",
            Self::Lease => "LEASE",
            Self::AuditWrite => "AUDIT_WRITE",
            Self::AuditControl => "AUDIT_CONTROL",
            Self::Setfcap => "SETFCAP",
            Self::MacOverride => "MAC_OVERRIDE",
            Self::MacAdmin => "MAC_ADMIN",
            Self::Syslog => "SYSLOG",
            Self::WakeAlarm => "WAKE_ALARM",
            Self::BlockSuspend => "BLOCK_SUSPEND",
            Self::AuditRead => "AUDIT_READ",
            Self::Perfmon => "PERFMON",
            Self::Bpf => "BPF",
            Self::CheckpointRestore => "CHECKPOINT_RESTORE",
        }
    }
    
    /// Get capability from name
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_uppercase().as_str() {
            "CHOWN" => Some(Self::Chown),
            "DAC_OVERRIDE" => Some(Self::DacOverride),
            "DAC_READ_SEARCH" => Some(Self::DacReadSearch),
            "FOWNER" => Some(Self::Fowner),
            "FSETID" => Some(Self::Fsetid),
            "KILL" => Some(Self::Kill),
            "SETGID" => Some(Self::Setgid),
            "SETUID" => Some(Self::Setuid),
            "SETPCAP" => Some(Self::Setpcap),
            "LINUX_IMMUTABLE" => Some(Self::LinuxImmutable),
            "NET_BIND_SERVICE" => Some(Self::NetBindService),
            "NET_BROADCAST" => Some(Self::NetBroadcast),
            "NET_ADMIN" => Some(Self::NetAdmin),
            "NET_RAW" => Some(Self::NetRaw),
            "IPC_LOCK" => Some(Self::IpcLock),
            "IPC_OWNER" => Some(Self::IpcOwner),
            "SYS_MODULE" => Some(Self::SysModule),
            "SYS_RAWIO" => Some(Self::SysRawio),
            "SYS_CHROOT" => Some(Self::SysChroot),
            "SYS_PTRACE" => Some(Self::SysPtrace),
            "SYS_PACCT" => Some(Self::SysPacct),
            "SYS_ADMIN" => Some(Self::SysAdmin),
            "SYS_BOOT" => Some(Self::SysBoot),
            "SYS_NICE" => Some(Self::SysNice),
            "SYS_RESOURCE" => Some(Self::SysResource),
            "SYS_TIME" => Some(Self::SysTime),
            "SYS_TTY_CONFIG" => Some(Self::SysTtyConfig),
            "MKNOD" => Some(Self::Mknod),
            "LEASE" => Some(Self::Lease),
            "AUDIT_WRITE" => Some(Self::AuditWrite),
            "AUDIT_CONTROL" => Some(Self::AuditControl),
            "SETFCAP" => Some(Self::Setfcap),
            "MAC_OVERRIDE" => Some(Self::MacOverride),
            "MAC_ADMIN" => Some(Self::MacAdmin),
            "SYSLOG" => Some(Self::Syslog),
            "WAKE_ALARM" => Some(Self::WakeAlarm),
            "BLOCK_SUSPEND" => Some(Self::BlockSuspend),
            "AUDIT_READ" => Some(Self::AuditRead),
            "PERFMON" => Some(Self::Perfmon),
            "BPF" => Some(Self::Bpf),
            "CHECKPOINT_RESTORE" => Some(Self::CheckpointRestore),
            _ => None,
        }
    }
}

/// Get all capabilities as strings
fn all_capabilities() -> Vec<String> {
    use Capability::*;
    vec![
        Chown, DacOverride, DacReadSearch, Fowner, Fsetid, Kill,
        Setgid, Setuid, Setpcap, LinuxImmutable, NetBindService,
        NetBroadcast, NetAdmin, NetRaw, IpcLock, IpcOwner,
        SysModule, SysRawio, SysChroot, SysPtrace, SysPacct,
        SysAdmin, SysBoot, SysNice, SysResource, SysTime,
        SysTtyConfig, Mknod, Lease, AuditWrite, AuditControl,
        Setfcap, MacOverride, MacAdmin, Syslog, WakeAlarm,
        BlockSuspend, AuditRead, Perfmon, Bpf, CheckpointRestore,
    ].iter().map(|c| c.name().to_string()).collect()
}

/// Seccomp profile generator
pub struct SeccompProfileGenerator;

impl SeccompProfileGenerator {
    /// Generate a default seccomp profile
    pub fn default_profile() -> SeccompProfileData {
        SeccompProfileData {
            default_action: SeccompAction::Errno,
            syscalls: vec![
                // Allowed syscalls for TOS containers
                SyscallRule {
                    names: vec![
                        "accept".to_string(), "accept4".to_string(),
                        "access".to_string(), "alarm".to_string(),
                        "bind".to_string(), "brk".to_string(),
                        "capget".to_string(), "capset".to_string(),
                        "chdir".to_string(), "chmod".to_string(),
                        "chown".to_string(), "chown32".to_string(),
                        "clock_getres".to_string(), "clock_gettime".to_string(),
                        "clock_nanosleep".to_string(), "clone".to_string(),
                        "clone3".to_string(), "close".to_string(),
                        "close_range".to_string(), "connect".to_string(),
                        "copy_file_range".to_string(), "creat".to_string(),
                        "dup".to_string(), "dup2".to_string(), "dup3".to_string(),
                        "epoll_create".to_string(), "epoll_create1".to_string(),
                        "epoll_ctl".to_string(), "epoll_ctl_old".to_string(),
                        "epoll_pwait".to_string(), "epoll_pwait2".to_string(),
                        "epoll_wait".to_string(), "epoll_wait_old".to_string(),
                        "eventfd".to_string(), "eventfd2".to_string(),
                        "execve".to_string(), "execveat".to_string(),
                        "exit".to_string(), "exit_group".to_string(),
                        "faccessat".to_string(), "faccessat2".to_string(),
                        "fadvise64".to_string(), "fadvise64_64".to_string(),
                        "fallocate".to_string(), "fanotify_mark".to_string(),
                        "fchdir".to_string(), "fchmod".to_string(),
                        "fchmodat".to_string(), "fchown".to_string(),
                        "fchown32".to_string(), "fchownat".to_string(),
                        "fcntl".to_string(), "fcntl64".to_string(),
                        "fdatasync".to_string(), "fgetxattr".to_string(),
                        "flistxattr".to_string(), "flock".to_string(),
                        "fork".to_string(), "fremovexattr".to_string(),
                        "fsetxattr".to_string(), "fstat".to_string(),
                        "fstat64".to_string(), "fstatat64".to_string(),
                        "fstatfs".to_string(), "fstatfs64".to_string(),
                        "fsync".to_string(), "ftruncate".to_string(),
                        "ftruncate64".to_string(), "futex".to_string(),
                        "futex_time64".to_string(), "getcpu".to_string(),
                        "getcwd".to_string(), "getdents".to_string(),
                        "getdents64".to_string(), "getegid".to_string(),
                        "getegid32".to_string(), "geteuid".to_string(),
                        "geteuid32".to_string(), "getgid".to_string(),
                        "getgid32".to_string(), "getgroups".to_string(),
                        "getgroups32".to_string(), "getitimer".to_string(),
                        "getpeername".to_string(), "getpgid".to_string(),
                        "getpgrp".to_string(), "getpid".to_string(),
                        "getppid".to_string(), "getpriority".to_string(),
                        "getrandom".to_string(), "getresgid".to_string(),
                        "getresgid32".to_string(), "getresuid".to_string(),
                        "getresuid32".to_string(), "getrlimit".to_string(),
                        "get_robust_list".to_string(), "getrusage".to_string(),
                        "getsid".to_string(), "getsockname".to_string(),
                        "getsockopt".to_string(), "get_thread_area".to_string(),
                        "gettid".to_string(), "gettimeofday".to_string(),
                        "getuid".to_string(), "getuid32".to_string(),
                        "getxattr".to_string(), "inotify_add_watch".to_string(),
                        "inotify_init".to_string(), "inotify_init1".to_string(),
                        "inotify_rm_watch".to_string(), "io_cancel".to_string(),
                        "ioctl".to_string(), "io_destroy".to_string(),
                        "io_getevents".to_string(), "io_pgetevents".to_string(),
                        "io_pgetevents_time64".to_string(), "ioprio_get".to_string(),
                        "ioprio_set".to_string(), "io_setup".to_string(),
                        "io_submit".to_string(), "io_uring_enter".to_string(),
                        "io_uring_register".to_string(), "io_uring_setup".to_string(),
                        "kill".to_string(), "lchown".to_string(),
                        "lchown32".to_string(), "lgetxattr".to_string(),
                        "link".to_string(), "linkat".to_string(),
                        "listen".to_string(), "listxattr".to_string(),
                        "llistxattr".to_string(), "lremovexattr".to_string(),
                        "lseek".to_string(), "lsetxattr".to_string(),
                        "lstat".to_string(), "lstat64".to_string(),
                        "madvise".to_string(), "membarrier".to_string(),
                        "memfd_create".to_string(), "mincore".to_string(),
                        "mkdir".to_string(), "mkdirat".to_string(),
                        "mknod".to_string(), "mknodat".to_string(),
                        "mlock".to_string(), "mlock2".to_string(),
                        "mlockall".to_string(), "mmap".to_string(),
                        "mmap2".to_string(), "mprotect".to_string(),
                        "mremap".to_string(), "msgctl".to_string(),
                        "msgget".to_string(), "msgrcv".to_string(),
                        "msgsnd".to_string(), "msync".to_string(),
                        "munlock".to_string(), "munlockall".to_string(),
                        "munmap".to_string(), "nanosleep".to_string(),
                        "newfstatat".to_string(), "open".to_string(),
                        "openat".to_string(), "openat2".to_string(),
                        "pause".to_string(), "pidfd_open".to_string(),
                        "pidfd_send_signal".to_string(), "pipe".to_string(),
                        "pipe2".to_string(), "poll".to_string(),
                        "ppoll".to_string(), "ppoll_time64".to_string(),
                        "prctl".to_string(), "pread64".to_string(),
                        "preadv".to_string(), "preadv2".to_string(),
                        "prlimit64".to_string(), "pselect6".to_string(),
                        "pselect6_time64".to_string(), "pwrite64".to_string(),
                        "pwritev".to_string(), "pwritev2".to_string(),
                        "read".to_string(), "readahead".to_string(),
                        "readdir".to_string(), "readlink".to_string(),
                        "readlinkat".to_string(), "readv".to_string(),
                        "recv".to_string(), "recvfrom".to_string(),
                        "recvmmsg".to_string(), "recvmmsg_time64".to_string(),
                        "recvmsg".to_string(), "remap_file_pages".to_string(),
                        "removexattr".to_string(), "rename".to_string(),
                        "renameat".to_string(), "renameat2".to_string(),
                        "restart_syscall".to_string(), "rmdir".to_string(),
                        "rseq".to_string(), "rt_sigaction".to_string(),
                        "rt_sigpending".to_string(), "rt_sigprocmask".to_string(),
                        "rt_sigqueueinfo".to_string(), "rt_sigreturn".to_string(),
                        "rt_sigsuspend".to_string(), "rt_sigtimedwait".to_string(),
                        "rt_sigtimedwait_time64".to_string(), "rt_tgsigqueueinfo".to_string(),
                        "sched_getaffinity".to_string(), "sched_getattr".to_string(),
                        "sched_getparam".to_string(), "sched_get_priority_max".to_string(),
                        "sched_get_priority_min".to_string(), "sched_getscheduler".to_string(),
                        "sched_rr_get_interval".to_string(), "sched_rr_get_interval_time64".to_string(),
                        "sched_setaffinity".to_string(), "sched_setattr".to_string(),
                        "sched_setparam".to_string(), "sched_setscheduler".to_string(),
                        "sched_yield".to_string(), "seccomp".to_string(),
                        "select".to_string(), "semctl".to_string(),
                        "semget".to_string(), "semop".to_string(),
                        "semtimedop".to_string(), "semtimedop_time64".to_string(),
                        "send".to_string(), "sendfile".to_string(),
                        "sendfile64".to_string(), "sendmmsg".to_string(),
                        "sendmsg".to_string(), "sendto".to_string(),
                        "setfsgid".to_string(), "setfsgid32".to_string(),
                        "setfsuid".to_string(), "setfsuid32".to_string(),
                        "setgid".to_string(), "setgid32".to_string(),
                        "setgroups".to_string(), "setgroups32".to_string(),
                        "setitimer".to_string(), "setpgid".to_string(),
                        "setpriority".to_string(), "setregid".to_string(),
                        "setregid32".to_string(), "setresgid".to_string(),
                        "setresgid32".to_string(), "setresuid".to_string(),
                        "setresuid32".to_string(), "setreuid".to_string(),
                        "setreuid32".to_string(), "setrlimit".to_string(),
                        "set_robust_list".to_string(), "setsid".to_string(),
                        "setsockopt".to_string(), "set_thread_area".to_string(),
                        "set_tid_address".to_string(), "setuid".to_string(),
                        "setuid32".to_string(), "setxattr".to_string(),
                        "shmat".to_string(), "shmctl".to_string(),
                        "shmdt".to_string(), "shmget".to_string(),
                        "shutdown".to_string(), "sigaltstack".to_string(),
                        "signalfd".to_string(), "signalfd4".to_string(),
                        "sigpending".to_string(), "sigprocmask".to_string(),
                        "sigreturn".to_string(), "socket".to_string(),
                        "socketcall".to_string(), "socketpair".to_string(),
                        "splice".to_string(), "stat".to_string(),
                        "stat64".to_string(), "statfs".to_string(),
                        "statfs64".to_string(), "statx".to_string(),
                        "symlink".to_string(), "symlinkat".to_string(),
                        "sync".to_string(), "sync_file_range".to_string(),
                        "syncfs".to_string(), "sysinfo".to_string(),
                        "tee".to_string(), "tgkill".to_string(),
                        "time".to_string(), "timer_create".to_string(),
                        "timer_delete".to_string(), "timer_getoverrun".to_string(),
                        "timer_gettime".to_string(), "timer_gettime64".to_string(),
                        "timer_settime".to_string(), "timer_settime64".to_string(),
                        "timerfd_create".to_string(), "timerfd_gettime".to_string(),
                        "timerfd_gettime64".to_string(), "timerfd_settime".to_string(),
                        "timerfd_settime64".to_string(), "times".to_string(),
                        "tkill".to_string(), "truncate".to_string(),
                        "truncate64".to_string(), "ugetrlimit".to_string(),
                        "umask".to_string(), "uname".to_string(),
                        "unlink".to_string(), "unlinkat".to_string(),
                        "utime".to_string(), "utimensat".to_string(),
                        "utimensat_time64".to_string(), "utimes".to_string(),
                        "vfork".to_string(), "wait4".to_string(),
                        "waitid".to_string(), "waitpid".to_string(),
                        "write".to_string(), "writev".to_string(),
                    ],
                    action: SeccompAction::Allow,
                },
            ],
        }
    }
    
    /// Generate a restricted profile
    pub fn restricted_profile() -> SeccompProfileData {
        let mut profile = Self::default_profile();
        // Remove potentially dangerous syscalls
        profile.syscalls.retain(|rule| {
            !rule.names.iter().any(|name| {
                matches!(name.as_str(), 
                    "clone" | "clone3" | "unshare" | "setns" |
                    "pivot_root" | "chroot" | "mount" | "umount" | "umount2"
                )
            })
        });
        profile
    }
}

/// Seccomp profile data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeccompProfileData {
    pub default_action: SeccompAction,
    pub syscalls: Vec<SyscallRule>,
}

/// Seccomp action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeccompAction {
    Allow,
    Errno,
    Kill,
    Trap,
    Trace,
    Log,
}

/// Syscall rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyscallRule {
    pub names: Vec<String>,
    pub action: SeccompAction,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_security_policy_default() {
        let policy = SecurityPolicy::default();
        assert!(policy.no_new_privileges);
        assert!(policy.read_only_rootfs);
        assert!(policy.drop_all_caps);
    }
    
    #[test]
    fn test_security_policy_minimal() {
        let policy = SecurityPolicy::minimal();
        assert!(!policy.no_new_privileges);
        assert!(!policy.read_only_rootfs);
        assert!(!policy.drop_all_caps);
    }
    
    #[test]
    fn test_security_policy_restricted() {
        let policy = SecurityPolicy::restricted();
        assert!(policy.no_new_privileges);
        assert!(policy.read_only_rootfs);
        assert!(policy.userns_remap);
    }
    
    #[test]
    fn test_capability_policy() {
        let policy = CapabilityPolicy::standard();
        assert!(policy.is_allowed("CHOWN"));
        assert!(policy.is_allowed("chown")); // case insensitive
        assert!(!policy.is_allowed("SYS_ADMIN"));
        assert!(policy.is_denied("SYS_ADMIN"));
    }
    
    #[test]
    fn test_capability_from_name() {
        assert_eq!(Capability::from_name("CHOWN"), Some(Capability::Chown));
        assert_eq!(Capability::from_name("SYS_ADMIN"), Some(Capability::SysAdmin));
        assert_eq!(Capability::from_name("INVALID"), None);
    }
    
    #[test]
    fn test_seccomp_profile() {
        let profile = SeccompProfile::Default;
        assert_eq!(profile.name(), "default");
        
        let profile = SeccompProfile::Custom("/path/to/profile.json".to_string());
        assert!(profile.name().contains("custom"));
    }
    
    #[test]
    fn test_seccomp_profile_generator() {
        let default = SeccompProfileGenerator::default_profile();
        assert!(matches!(default.default_action, SeccompAction::Errno));
        assert!(!default.syscalls.is_empty());
        
        let restricted = SeccompProfileGenerator::restricted_profile();
        assert!(restricted.syscalls.len() < default.syscalls.len());
    }
}
