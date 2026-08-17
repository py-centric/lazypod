use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, UNIX_EPOCH};

// ═══════════════════════════════════════════════════════════════════════════════
// PORT MAPPING: unified struct for Docker (JSON array) and Podman (string) ports
// ═══════════════════════════════════════════════════════════════════════════════

/// A single port mapping from host → container.
/// Both Docker and Podman expose the same four-tuple: `host_ip`, `host_port`, `container_port`, protocol.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PortMapping {
    pub host_ip: String,
    pub host_port: u16,
    pub container_port: u16,
    pub protocol: String, // "tcp" or "udp"
}

impl PortMapping {
    /// Format as "0.0.0.0:8080->80/tcp" (matching `docker ps` display format)
    #[must_use]
    pub fn display(&self) -> String {
        let proto = if self.protocol.starts_with('/') {
            self.protocol.clone()
        } else {
            format!("/{}", self.protocol)
        };
        if self.host_port > 0 {
            format!(
                "{}:{}->{}{}",
                self.host_ip, self.host_port, self.container_port, proto
            )
        } else {
            format!("{}{}", self.container_port, proto)
        }
    }
}

/// Parse Docker-style ports: `[{"PrivatePort":80,"PublicPort":8080,"Type":"tcp","IP":"0.0.0.0"}]`
#[allow(clippy::cast_possible_truncation)]
fn parse_docker_ports(value: &serde_json::Value) -> Vec<PortMapping> {
    let mut ports = Vec::new();
    if let Some(arr) = value.as_array() {
        for entry in arr {
            let container_port = match entry.get("PrivatePort").and_then(serde_json::Value::as_u64)
            {
                Some(p) => p as u16,
                None => continue,
            };
            let host_port = entry
                .get("PublicPort")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0) as u16;
            let protocol = entry
                .get("Type")
                .and_then(|t| t.as_str())
                .unwrap_or("tcp")
                .to_string();
            let host_ip = entry
                .get("IP")
                .and_then(|i| i.as_str())
                .unwrap_or("0.0.0.0")
                .to_string();
            ports.push(PortMapping {
                host_ip,
                host_port,
                container_port,
                protocol,
            });
        }
    }
    ports
}

/// Parse Podman-style ports string: "0.0.0.0:8080->80/tcp, `:::9090->9090/tcp`"
fn parse_podman_ports(s: &str) -> Vec<PortMapping> {
    let mut ports = Vec::new();
    for part in s.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        // Format: "[host_ip:]host_port->container_port/protocol" or "container_port/protocol"
        if let Some(arrow_pos) = part.find("->") {
            let left = &part[..arrow_pos];
            let right = &part[arrow_pos + 2..];

            let (protocol, port_str) = if let Some(slash_pos) = right.find('/') {
                (right[slash_pos + 1..].to_string(), &right[..slash_pos])
            } else {
                ("tcp".to_string(), right)
            };

            let container_port: u16 = port_str.parse().unwrap_or(0);

            // left can be "0.0.0.0:8080" or ":::8080" or just "8080"
            let (host_ip, host_port) = if let Some(colon_pos) = left.rfind(':') {
                let ip = &left[..colon_pos];
                let h_port: u16 = left[colon_pos + 1..].parse().unwrap_or(0);
                let ip = if ip.is_empty() || ip == "::" {
                    "0.0.0.0".to_string()
                } else {
                    ip.to_string()
                };
                (ip, h_port)
            } else {
                ("0.0.0.0".to_string(), left.parse().unwrap_or(0))
            };

            ports.push(PortMapping {
                host_ip,
                host_port,
                container_port,
                protocol,
            });
        } else {
            // No arrow: exposed-only port like "80/tcp"
            let (protocol, port_str) = if let Some(slash_pos) = part.find('/') {
                (part[slash_pos + 1..].to_string(), &part[..slash_pos])
            } else {
                ("tcp".to_string(), part)
            };
            let container_port: u16 = port_str.parse().unwrap_or(0);
            ports.push(PortMapping {
                host_ip: String::new(),
                host_port: 0,
                container_port,
                protocol,
            });
        }
    }
    ports
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Volume {
    #[serde(rename = "name", alias = "Name", default)]
    pub name: String,
    #[serde(rename = "driver", alias = "Driver", default)]
    pub driver: String,
    #[serde(rename = "mountpoint", alias = "Mountpoint", default)]
    pub mountpoint: String,
    #[serde(skip)]
    pub engine: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Network {
    #[serde(rename = "name", alias = "Name", default)]
    pub name: String,
    #[serde(rename = "id", alias = "Id", alias = "ID", default)]
    pub id: String,
    #[serde(rename = "driver", alias = "Driver", default)]
    pub driver: String,
    #[serde(skip)]
    pub engine: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchResult {
    #[serde(alias = "Index", default)]
    pub index: String,
    #[serde(alias = "Name", default)]
    pub name: String,
    #[serde(alias = "Description", default)]
    pub description: String,
    #[serde(alias = "Stars", default)]
    pub stars: u32,
    #[serde(alias = "Official", default)]
    pub official: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Container {
    #[serde(rename = "id", alias = "Id", alias = "ID", default)]
    pub id: String,
    #[serde(rename = "image", alias = "Image", default)]
    pub image: String,
    #[serde(rename = "command", alias = "Command", default)]
    pub command: Option<serde_json::Value>,
    #[serde(rename = "created", alias = "Created", default)]
    pub created: Option<serde_json::Value>,
    #[serde(rename = "state", alias = "State", default)]
    pub state: Option<serde_json::Value>,
    #[serde(rename = "status", alias = "Status", default)]
    pub status: Option<serde_json::Value>,
    #[serde(rename = "names", alias = "Names", default)]
    pub names: Option<serde_json::Value>,
    #[serde(rename = "name", alias = "Name", default)]
    pub name: Option<String>,
    #[serde(rename = "ports", alias = "Ports", default)]
    pub ports: Option<serde_json::Value>,
    /// Podman adds this field when containers are queried with `podman ps --pod`.
    /// Docker containers never have a `pod_id`.
    #[serde(
        rename = "podid",
        alias = "PodID",
        alias = "PodId",
        alias = "pod_id",
        alias = "Pod",
        default
    )]
    pub pod_id: Option<String>,
    #[serde(skip)]
    pub engine: String,
}

impl Container {
    #[must_use]
    pub fn get_names(&self) -> Vec<String> {
        if let Some(n) = &self.name {
            if !n.is_empty() {
                return vec![n.clone()];
            }
        }
        if let Some(v) = &self.names {
            if let Some(arr) = v.as_array() {
                return arr
                    .iter()
                    .filter_map(|x| x.as_str().map(String::from))
                    .collect();
            } else if let Some(s) = v.as_str() {
                return vec![s.to_string()];
            }
        }
        vec![]
    }

    #[must_use]
    pub fn get_command(&self) -> String {
        if let Some(v) = &self.command {
            if let Some(arr) = v.as_array() {
                return arr
                    .iter()
                    .filter_map(|x| x.as_str().map(String::from))
                    .collect::<Vec<_>>()
                    .join(" ");
            } else if let Some(s) = v.as_str() {
                return s.to_string();
            }
        }
        String::new()
    }

    #[must_use]
    pub fn get_state_str(&self) -> String {
        if let Some(s) = &self.state {
            if let Some(st) = s.as_str() {
                return st.to_string();
            }
        }
        "unknown".into()
    }

    #[must_use]
    pub fn get_status_str(&self) -> String {
        if let Some(s) = &self.status {
            if let Some(st) = s.as_str() {
                return st.to_string();
            }
        }
        String::new()
    }

    /// Parse ports into structured `PortMapping` values.
    /// Handles both Docker (JSON array) and Podman (comma-separated string) formats.
    #[must_use]
    pub fn get_ports(&self) -> Vec<PortMapping> {
        if let Some(v) = &self.ports {
            if v.is_array() {
                return parse_docker_ports(v);
            } else if let Some(s) = v.as_str() {
                return parse_podman_ports(s);
            }
        }
        Vec::new()
    }

    /// Convenience: format ports as display strings for UI.
    #[must_use]
    pub fn get_port_strings(&self) -> Vec<String> {
        self.get_ports().iter().map(PortMapping::display).collect()
    }

    #[must_use]
    pub fn is_running(&self) -> bool {
        let state = self.get_state_str().to_lowercase();
        let status = self.get_status_str().to_lowercase();

        if state == "running" || state == "up" || status.starts_with("up") {
            return true;
        }
        if state == "exited" || state == "stopped" || state == "created" || state == "paused" {
            return false;
        }
        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Image {
    #[serde(rename = "id", alias = "Id", alias = "ID", default)]
    pub id: String,
    #[serde(rename = "parentId", alias = "ParentId", alias = "ParentID", default)]
    pub parent_id: Option<String>,
    #[serde(rename = "repoTags", alias = "RepoTags", default)]
    pub repo_tags: Option<serde_json::Value>,
    #[serde(rename = "repository", alias = "Repository", default)]
    pub repository: Option<String>,
    #[serde(rename = "tag", alias = "Tag", default)]
    pub tag: Option<String>,
    #[serde(rename = "names", alias = "Names", default)]
    pub names: Option<serde_json::Value>,
    #[serde(rename = "size", alias = "Size", default)]
    pub size: Option<serde_json::Value>,
    #[serde(rename = "virtualSize", alias = "VirtualSize", default)]
    pub virtual_size: Option<serde_json::Value>,
    #[serde(rename = "created", alias = "Created", default)]
    pub created: Option<serde_json::Value>,
    #[serde(rename = "dangling", alias = "Dangling", default)]
    pub dangling: Option<bool>,
    #[serde(skip)]
    pub engine: String,
}

impl Image {
    #[must_use]
    pub fn is_dangling(&self) -> bool {
        if let Some(true) = self.dangling {
            return true;
        }
        let names = self.get_names();
        if names.is_empty() {
            return true;
        }
        names
            .iter()
            .all(|n| n == "<none>:<none>" || n == "<none>" || n.starts_with("<none>"))
    }
    #[must_use]
    pub fn get_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        if let Some(r) = &self.repository {
            if let Some(t) = &self.tag {
                names.push(format!("{r}:{t}"));
            } else {
                names.push(r.clone());
            }
        }
        if let Some(v) = &self.names {
            if let Some(arr) = v.as_array() {
                names.extend(arr.iter().filter_map(|x| x.as_str().map(String::from)));
            } else if let Some(s) = v.as_str() {
                names.push(s.to_string());
            }
        }
        if let Some(v) = &self.repo_tags {
            if let Some(arr) = v.as_array() {
                names.extend(arr.iter().filter_map(|x| x.as_str().map(String::from)));
            } else if let Some(s) = v.as_str() {
                names.push(s.to_string());
            }
        }
        names
    }

    #[allow(clippy::cast_precision_loss)]
    #[must_use]
    pub fn get_size_str(&self) -> String {
        let val = self.size.as_ref().or(self.virtual_size.as_ref());
        if let Some(v) = val {
            if let Some(s) = v.as_str() {
                if !s.is_empty() {
                    return s.to_string();
                }
            } else if let Some(n) = v.as_i64() {
                if n <= 0 {
                    return "0 B".to_string();
                }
                let units = ["B", "KB", "MB", "GB", "TB"];
                let mut size = n as f64;
                let mut unit_idx = 0;
                while size >= 1024.0 && unit_idx < units.len() - 1 {
                    size /= 1024.0;
                    unit_idx += 1;
                }
                return format!("{size:.2} {}", units[unit_idx]);
            }
        }
        "0 B".to_string()
    }

    #[allow(clippy::cast_sign_loss)]
    #[must_use]
    pub fn get_created_str(&self) -> String {
        if let Some(v) = &self.created {
            if let Some(n) = v.as_i64() {
                // Unix timestamp: can be negative for pre-epoch dates (unlikely but defensive)
                if n >= 0 {
                    let d = UNIX_EPOCH + Duration::from_secs(n as u64);
                    let datetime: DateTime<Utc> = d.into();
                    return datetime.format("%Y-%m-%d %H:%M:%S").to_string();
                }
                // Negative timestamp: fall through to display raw value
                return format!("{n} (raw)");
            } else if let Some(s) = v.as_str() {
                return s.to_string();
            }
        }
        "Unknown".to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Pod {
    #[serde(rename = "id", alias = "Id", alias = "ID", default)]
    pub id: String,
    #[serde(rename = "name", alias = "Name", default)]
    pub name: String,
    #[serde(rename = "status", alias = "Status", default)]
    pub status: String,
    #[serde(rename = "createdat", alias = "CreatedAt", alias = "Created", default)]
    pub created: Option<serde_json::Value>,
    #[serde(rename = "labels", alias = "Labels", default)]
    pub labels: Option<HashMap<String, String>>,
    /// Number of containers in the pod (from `podman pod ps --format json`).
    #[serde(rename = "numberofcontainers", alias = "NumberOfContainers", default)]
    pub num_containers: Option<u32>,
    /// Containers in the pod (from `podman pod ps --format json`).
    /// This is populated directly from the pod ps output, not from a separate query.
    #[serde(rename = "containers", alias = "Containers", default)]
    pub containers: Vec<PodContainer>,
    #[serde(skip)]
    pub engine: String,
}

/// A container within a pod, as reported by `podman pod ps --format json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PodContainer {
    #[serde(rename = "id", alias = "Id", alias = "ID", default)]
    pub id: String,
    #[serde(rename = "names", alias = "Names", alias = "Name", default)]
    pub names: Option<serde_json::Value>,
    #[serde(rename = "status", alias = "Status", default)]
    pub status: Option<String>,
}

impl PodContainer {
    #[must_use]
    pub fn get_name(&self) -> String {
        if let Some(v) = &self.names {
            if let Some(arr) = v.as_array() {
                if let Some(first) = arr.first() {
                    if let Some(s) = first.as_str() {
                        return s.to_string();
                    }
                }
            } else if let Some(s) = v.as_str() {
                return s.to_string();
            }
        }
        self.id[..std::cmp::min(12, self.id.len())].to_string()
    }

    #[must_use]
    pub fn get_status_str(&self) -> String {
        self.status.clone().unwrap_or_else(|| "unknown".to_string())
    }
}

impl Pod {
    #[allow(clippy::cast_sign_loss)]
    #[must_use]
    pub fn get_created_str(&self) -> String {
        if let Some(v) = &self.created {
            if let Some(n) = v.as_i64() {
                if n >= 0 {
                    let d = UNIX_EPOCH + Duration::from_secs(n as u64);
                    let datetime: DateTime<Utc> = d.into();
                    return datetime.format("%Y-%m-%d %H:%M:%S").to_string();
                }
                return format!("{n} (raw)");
            } else if let Some(s) = v.as_str() {
                return s.to_string();
            }
        }
        "Unknown".to_string()
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn get_labels_vec(&self) -> Vec<(String, String)> {
        self.labels
            .as_ref()
            .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default()
    }

    /// True when Podman reports the pod as running.
    #[allow(dead_code)]
    #[must_use]
    pub fn is_running(&self) -> bool {
        let s = self.status.to_lowercase();
        s.contains("running") || s.contains("up")
    }

    /// Get all container names from the pod's Containers array.
    #[allow(dead_code)]
    #[must_use]
    pub fn get_container_names(&self) -> Vec<String> {
        self.containers.iter().map(PodContainer::get_name).collect()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// INSPECT OUTPUT: simplified extraction from the large `docker/podman inspect` JSON blob
// ═══════════════════════════════════════════════════════════════════════════════

/// Simplified view of `docker inspect` / `podman inspect` output.
/// Extracts the most useful fields for display in the TUI details popup.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InspectInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub state: String,
    pub status: String,
    pub created: String,
    pub engine: String,
    pub platform: String,
    pub restart_policy: String,

    // Config
    pub env: Vec<String>,
    pub cmd: Vec<String>,
    pub entrypoint: Vec<String>,
    pub working_dir: String,
    pub user: String,

    // Network
    pub ports: Vec<PortMapping>,
    pub networks: Vec<String>,
    pub ip_address: String,
    pub mac_address: String,
    pub gateway: String,

    // Mounts
    pub mounts: Vec<InspectMount>,

    // Resources
    pub memory_limit: Option<u64>,
    pub cpu_shares: Option<u64>,
    pub pids_limit: Option<i64>,
}

/// A single mount/volume from inspect output.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InspectMount {
    pub source: String,
    pub destination: String,
    pub mode: String,
    pub rw: bool,
    pub mount_type: String, // "bind", "volume", "tmpfs"
}

#[allow(dead_code)]
impl InspectInfo {
    /// Parse the raw inspect JSON (Docker or Podman format) into a simplified `InspectInfo`.
    #[allow(clippy::too_many_lines)]
    #[must_use]
    pub fn from_inspect_json(raw: &serde_json::Value, engine: &str) -> Self {
        let mut info = InspectInfo {
            engine: engine.to_string(),
            ..Default::default()
        };

        // Top-level fields
        info.id = raw
            .get("Id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        info.name = raw
            .get("Name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        // Docker nests name with leading "/"
        if info.name.starts_with('/') {
            info.name = info.name[1..].to_string();
        }

        info.created = raw
            .get("Created")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Image: Docker: Config.Image, Podman: Image or Config.Image
        info.image = raw
            .get("Config")
            .and_then(|c| c.get("Image"))
            .and_then(|v| v.as_str())
            .or_else(|| raw.get("Image").and_then(|v| v.as_str()))
            .unwrap_or("")
            .to_string();

        // State
        if let Some(state) = raw.get("State") {
            info.state = state
                .get("Status")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            info.status = format!(
                "{} (running: {}, pid: {})",
                info.state,
                state
                    .get("Running")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false),
                state
                    .get("Pid")
                    .and_then(serde_json::Value::as_i64)
                    .unwrap_or(0),
            );
        }

        // Config section
        if let Some(config) = raw.get("Config") {
            info.env = config
                .get("Env")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            info.cmd = config
                .get("Cmd")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            info.entrypoint = config
                .get("Entrypoint")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            info.working_dir = config
                .get("WorkingDir")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            info.user = config
                .get("User")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
        }

        // Network settings
        if let Some(net) = raw.get("NetworkSettings") {
            info.ip_address = net
                .get("IPAddress")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            info.mac_address = net
                .get("MacAddress")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            info.gateway = net
                .get("Gateway")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            // Networks
            if let Some(nets) = net.get("Networks").and_then(|v| v.as_object()) {
                info.networks = nets.keys().cloned().collect();
            }

            // Ports: Docker format (object with nested arrays)
            if let Some(ports_obj) = net.get("Ports").and_then(|v| v.as_object()) {
                for (container_port_proto, bindings) in ports_obj {
                    let parts: Vec<&str> = container_port_proto.splitn(2, '/').collect();
                    let container_port: u16 =
                        parts.first().and_then(|p| p.parse().ok()).unwrap_or(0);
                    let protocol = parts.get(1).unwrap_or(&"tcp").to_string();

                    if let Some(arr) = bindings.as_array() {
                        for binding in arr {
                            let host_ip = binding
                                .get("HostIp")
                                .and_then(|v| v.as_str())
                                .unwrap_or("0.0.0.0")
                                .to_string();
                            let host_port = binding
                                .get("HostPort")
                                .and_then(|v| v.as_str().and_then(|s| s.parse().ok()))
                                .unwrap_or(0);
                            info.ports.push(PortMapping {
                                host_ip,
                                host_port,
                                container_port,
                                protocol: protocol.clone(),
                            });
                        }
                    } else if bindings.is_null() {
                        // Exposed-only port (no host binding)
                        info.ports.push(PortMapping {
                            host_ip: String::new(),
                            host_port: 0,
                            container_port,
                            protocol,
                        });
                    }
                }
            }
        }

        // Mounts
        if let Some(mounts) = raw.get("Mounts").and_then(|v| v.as_array()) {
            info.mounts = mounts
                .iter()
                .map(|m| InspectMount {
                    source: m
                        .get("Source")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    destination: m
                        .get("Destination")
                        .or_else(|| m.get("Dest"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    mode: m
                        .get("Mode")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    rw: m
                        .get("RW")
                        .and_then(serde_json::Value::as_bool)
                        .unwrap_or(true),
                    mount_type: m
                        .get("Type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                })
                .collect();
        }

        // HostConfig resources
        if let Some(hc) = raw.get("HostConfig") {
            info.memory_limit = hc.get("Memory").and_then(serde_json::Value::as_u64);
            info.cpu_shares = hc.get("CpuShares").and_then(serde_json::Value::as_u64);
            info.pids_limit = hc.get("PidsLimit").and_then(serde_json::Value::as_i64);
            info.restart_policy = hc
                .get("RestartPolicy")
                .and_then(|rp| rp.get("Name"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
        }

        info
    }

    /// Format as a human-readable string for the TUI inspect popup.
    #[must_use]
    pub fn format_display(&self) -> String {
        let mut lines = Vec::new();

        lines.push(format!(
            "ID:       {}",
            &self.id[..std::cmp::min(12, self.id.len())]
        ));
        lines.push(format!("Name:     {}", self.name));
        lines.push(format!("Image:    {}", self.image));
        lines.push(format!("State:    {}", self.state));
        lines.push(format!("Created:  {}", self.created));
        lines.push(format!("Engine:   {}", self.engine));

        if !self.cmd.is_empty() {
            lines.push(format!("Command:  {}", self.cmd.join(" ")));
        }
        if !self.entrypoint.is_empty() {
            lines.push(format!("Entrypt:  {}", self.entrypoint.join(" ")));
        }
        if !self.working_dir.is_empty() {
            lines.push(format!("WorkDir:  {}", self.working_dir));
        }
        if !self.user.is_empty() {
            lines.push(format!("User:     {}", self.user));
        }
        if !self.restart_policy.is_empty() {
            lines.push(format!("Restart:  {}", self.restart_policy));
        }

        // Ports
        if !self.ports.is_empty() {
            lines.push(String::new());
            lines.push("Ports:".to_string());
            for p in &self.ports {
                lines.push(format!("  {}", p.display()));
            }
        }

        // Networks
        if !self.networks.is_empty() {
            lines.push(String::new());
            lines.push("Networks:".to_string());
            for n in &self.networks {
                lines.push(format!("  {n}"));
            }
        }
        if !self.ip_address.is_empty() {
            lines.push(format!("IP:       {}", self.ip_address));
        }

        // Mounts
        if !self.mounts.is_empty() {
            lines.push(String::new());
            lines.push("Mounts:".to_string());
            for m in &self.mounts {
                lines.push(format!(
                    "  {} -> {} ({}, {})",
                    m.source,
                    m.destination,
                    m.mount_type,
                    if m.mount_type == "bind" || m.rw {
                        "rw"
                    } else {
                        "ro"
                    }
                ));
            }
        }

        // Environment (first 10)
        if !self.env.is_empty() {
            lines.push(String::new());
            lines.push(format!("Env ({} vars):", self.env.len()));
            for e in self.env.iter().take(10) {
                lines.push(format!("  {e}"));
            }
            if self.env.len() > 10 {
                lines.push(format!("  ... and {} more", self.env.len() - 10));
            }
        }

        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("0.0.0.0:8080->80/tcp", "0.0.0.0", 8080, 80, "tcp")]
    #[case("127.0.0.1:3000->3000/udp", "127.0.0.1", 3000, 3000, "udp")]
    #[case(":::9090->9090/tcp", "0.0.0.0", 9090, 9090, "tcp")]
    #[case("80/tcp", "", 0, 80, "tcp")]
    fn test_parse_port_string(
        #[case] input: &str,
        #[case] expected_ip: &str,
        #[case] expected_host_port: u16,
        #[case] expected_container_port: u16,
        #[case] expected_protocol: &str,
    ) {
        let ports = parse_podman_ports(input);
        assert_eq!(ports.len(), 1);
        let p = &ports[0];
        assert_eq!(p.host_ip, expected_ip);
        assert_eq!(p.host_port, expected_host_port);
        assert_eq!(p.container_port, expected_container_port);
        assert_eq!(p.protocol, expected_protocol);
    }

    #[rstest]
    #[case(Some(serde_json::json!(500)), "500.00 B")]
    #[case(Some(serde_json::json!(1024)), "1.00 KB")]
    #[case(Some(serde_json::json!(1_048_576)), "1.00 MB")]
    #[case(Some(serde_json::json!(1_073_741_824)), "1.00 GB")]
    #[case(Some(serde_json::json!("77.8MB")), "77.8MB")]
    #[case(None, "0 B")]
    fn test_image_size_str(#[case] size: Option<serde_json::Value>, #[case] expected: &str) {
        let img = Image {
            id: "i1".into(),
            parent_id: None,
            repo_tags: None,
            repository: None,
            tag: None,
            names: None,
            size,
            virtual_size: None,
            created: None,
            dangling: None,
            engine: "docker".into(),
        };
        assert_eq!(img.get_size_str(), expected);
    }

    #[test]
    fn test_inspect_info_from_json() {
        let raw = serde_json::json!({
            "Id": "abc123def456",
            "Name": "/my_service",
            "Created": "2026-01-01T00:00:00Z",
            "Config": {
                "Image": "nginx:alpine",
                "Env": ["PORT=80", "MODE=prod"],
                "Cmd": ["nginx", "-g", "daemon off;"]
            },
            "State": {
                "Status": "running",
                "Running": true,
                "Pid": 1234
            },
            "NetworkSettings": {
                "IPAddress": "172.17.0.2",
                "Gateway": "172.17.0.1",
                "Ports": {
                    "80/tcp": [{
                        "HostIp": "0.0.0.0",
                        "HostPort": "8080"
                    }]
                }
            }
        });

        let info = InspectInfo::from_inspect_json(&raw, "docker");
        assert_eq!(info.id, "abc123def456");
        assert_eq!(info.name, "my_service");
        assert_eq!(info.image, "nginx:alpine");
        assert_eq!(info.state, "running");
        assert_eq!(info.ip_address, "172.17.0.2");
        assert_eq!(info.ports.len(), 1);
        assert_eq!(info.ports[0].host_port, 8080);
        assert_eq!(info.ports[0].container_port, 80);

        let rendered = info.format_display();
        assert!(rendered.contains("my_service"));
        assert!(rendered.contains("abc123def456"));
        assert!(rendered.contains("0.0.0.0:8080->80/tcp"));
    }

    #[test]
    fn test_image_is_dangling() {
        let tagged_image = Image {
            id: "img1".into(),
            parent_id: None,
            repo_tags: Some(serde_json::json!(["alpine:latest"])),
            repository: Some("alpine".into()),
            tag: Some("latest".into()),
            names: None,
            size: None,
            virtual_size: None,
            created: None,
            dangling: None,
            engine: "podman".into(),
        };
        assert!(!tagged_image.is_dangling());

        let dangling_none_names = Image {
            id: "img2".into(),
            parent_id: None,
            repo_tags: None,
            repository: None,
            tag: None,
            names: Some(serde_json::json!(["<none>:<none>"])),
            size: None,
            virtual_size: None,
            created: None,
            dangling: None,
            engine: "podman".into(),
        };
        assert!(dangling_none_names.is_dangling());

        let empty_names_image = Image {
            id: "img3".into(),
            parent_id: None,
            repo_tags: None,
            repository: None,
            tag: None,
            names: None,
            size: None,
            virtual_size: None,
            created: None,
            dangling: None,
            engine: "docker".into(),
        };
        assert!(empty_names_image.is_dangling());

        let explicit_dangling = Image {
            id: "img4".into(),
            parent_id: None,
            repo_tags: None,
            repository: None,
            tag: None,
            names: Some(serde_json::json!(["some_name"])),
            size: None,
            virtual_size: None,
            created: None,
            dangling: Some(true),
            engine: "podman".into(),
        };
        assert!(explicit_dangling.is_dangling());
    }

    #[test]
    fn test_deserialize_real_image_json() {
        let json_str = r#"{
  "repository": "docker.io/library/academisource-ui",
  "tag": "latest",
  "Id": "13924a622062b704f1a5975c3bf1dac972bb9eefbb08c1ee6a85ffcb44a5a5f8",
  "ParentId": "c7bdf8a717c63f6649615ba2aa62891d44b7d1b0e9cc399e55fe7a5a86878b48",
  "RepoTags": [
    "docker.io/library/academisource-ui:latest"
  ],
  "RepoDigests": [
    "docker.io/library/academisource-ui@sha256:4c21acfc632119ed6e0eef12bcb45afaa9c0978fd3f2b18bd289cda69974aa5a"
  ],
  "Created": 1786971661,
  "Size": 221929507,
  "SharedSize": 0,
  "VirtualSize": 221929507,
  "Labels": {
    "com.docker.compose.image.builder": "classic",
    "io.buildah.version": "1.45.0",
    "maintainer": "NGINX Docker Maintainers <docker-maint@nginx.com>"
  },
  "Containers": 1,
  "Digest": "sha256:4c21acfc632119ed6e0eef12bcb45afaa9c0978fd3f2b18bd289cda69974aa5a",
  "History": [
    "docker.io/library/academisource-ui:latest"
  ],
  "Names": [
    "docker.io/library/academisource-ui:latest"
  ]
}"#;
        let res: Result<Image, _> = serde_json::from_str(json_str);
        println!("Deserialization result: {res:?}");
        assert!(res.is_ok());
        let img = res.unwrap();
        assert_eq!(
            img.id,
            "13924a622062b704f1a5975c3bf1dac972bb9eefbb08c1ee6a85ffcb44a5a5f8"
        );
    }
}
