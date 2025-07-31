use crate::proxy::{GlobalConfig, ProxyConfig};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortForward {
    pub enabled: bool,
    pub local: String,  // "0.0.0.0:4422"
    pub remote: String, // "127.0.0.1:22"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SSHConfig {
    pub alias: String,   // 主机别名
    pub address: String, // 实际连接地址
    pub port: Option<u16>,
    pub user: Option<String>,
    pub key: Option<String>,
    pub port_forward: Option<PortForward>,
    #[serde(default)]
    pub proxy: Option<ProxyConfig>, // 代理配置
    #[serde(default)]
    pub use_global_proxy: bool, // 是否使用全局代理
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    #[serde(default)]
    pub global: GlobalConfig,
    #[serde(default)]
    pub servers: Vec<SSHConfig>,
}

#[derive(Debug, Clone)]
pub struct ConfigManager {
    pub configs: Vec<SSHConfig>,
    pub global_config: GlobalConfig,
}

impl ConfigManager {
    /// 获取配置目录路径
    ///
    /// # 返回
    /// 返回配置目录路径，如果无法获取则返回 None
    fn get_config_dir() -> Option<PathBuf> {
        dirs::home_dir().map(|home| home.join(".mssh"))
    }

    /// 创建新的配置管理器
    ///
    /// # 参数
    /// - `config_path`: 配置文件路径，可选
    ///
    /// # 返回
    /// 返回 Result，成功为 ConfigManager 实例，失败为 Err
    pub fn new(config_path: Option<String>) -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = config_path.map(PathBuf::from).unwrap_or_else(|| {
            if let Some(config_dir) = Self::get_config_dir() {
                config_dir.join("config.toml")
            } else {
                PathBuf::from("config.toml")
            }
        });

        let config_content = fs::read_to_string(&config_path).unwrap_or_default();
        let config_file: ConfigFile =
            toml::from_str(&config_content).unwrap_or_else(|_| ConfigFile {
                global: GlobalConfig::default(),
                servers: Vec::new(),
            });

        Ok(Self {
            configs: config_file.servers,
            global_config: config_file.global,
        })
    }

    /// 保存配置到文件
    ///
    /// # 返回
    /// 返回 Result，成功为 Ok(())，失败为 Err
    pub fn save_configs(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = if let Some(config_dir) = Self::get_config_dir() {
            config_dir.join("config.toml")
        } else {
            PathBuf::from("config.toml")
        };

        // 确保配置目录存在
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let config_file = ConfigFile {
            global: self.global_config.clone(),
            servers: self.configs.clone(),
        };

        let toml_string = toml::to_string_pretty(&config_file)?;
        fs::write(&config_path, toml_string)?;

        Ok(())
    }

    /// 添加新配置
    ///
    /// # 参数
    /// - `config`: 要添加的 SSH 配置
    ///
    /// # 返回
    /// 返回 Result，成功为 Ok(())，失败为 Err
    pub fn add_config(&mut self, config: SSHConfig) -> Result<(), Box<dyn std::error::Error>> {
        // 检查是否已存在
        if self.configs.iter().any(|c| c.alias == config.alias) {
            return Err("主机别名已存在".into());
        }

        self.configs.push(config);
        self.save_configs()?;
        Ok(())
    }

    /// 更新配置
    ///
    /// # 参数
    /// - `host`: 主机别名
    /// - `config`: 新的配置
    ///
    /// # 返回
    /// 返回 Result，成功为 Ok(())，失败为 Err
    pub fn update_config(
        &mut self,
        host: &str,
        config: SSHConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(index) = self.configs.iter().position(|c| c.alias == host) {
            self.configs[index] = config;
            self.save_configs()?;
            Ok(())
        } else {
            Err("配置不存在".into())
        }
    }

    /// 删除配置
    ///
    /// # 参数
    /// - `host`: 主机别名
    ///
    /// # 返回
    /// 返回 Result，成功为 Ok(())，失败为 Err
    pub fn remove_config(&mut self, host: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.configs.retain(|c| c.alias != host);
        self.save_configs()?;
        Ok(())
    }

    /// 解析 SSH 配置文件中的 Host 配置
    ///
    /// # 参数
    /// - `content`: SSH 配置文件内容
    /// - `host_line`: Host 行内容
    ///
    /// # 返回
    /// 返回解析后的 SSH 配置，如果解析失败则返回 None
    pub fn parse_ssh_host_config(&self, content: &str, host_line: &str) -> Option<SSHConfig> {
        let host_name = host_line.split_whitespace().nth(1)?;
        let mut config = SSHConfig {
            alias: host_name.to_string(),
            address: host_name.to_string(),
            port: None,
            user: None,
            key: None,
            port_forward: None,
            proxy: None,
            use_global_proxy: false,
        };

        // 查找该 Host 下的配置
        let lines: Vec<&str> = content.lines().collect();
        let mut in_host_block = false;

        for line in lines {
            let line = line.trim();

            if line.starts_with("Host ") {
                in_host_block = line == host_line;
                continue;
            }

            if !in_host_block {
                continue;
            }

            // 解析各种配置项
            if line.starts_with("HostName ") {
                config.address = line.split_whitespace().nth(1)?.to_string();
            } else if line.starts_with("Port ") {
                if let Ok(port) = line.split_whitespace().nth(1)?.parse::<u16>() {
                    config.port = Some(port);
                }
            } else if line.starts_with("User ") {
                config.user = Some(line.split_whitespace().nth(1)?.to_string());
            } else if line.starts_with("IdentityFile ") {
                config.key = Some(line.split_whitespace().nth(1)?.to_string());
            } else if line.starts_with("LocalForward ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let local = parts[1].to_string();
                    let remote = parts[2].to_string();
                    config.port_forward = Some(PortForward {
                        enabled: true,
                        local,
                        remote,
                    });
                }
            }
        }

        Some(config)
    }
}

impl TryFrom<toml::Value> for SSHConfig {
    type Error = Box<dyn std::error::Error>;

    /// 从 TOML 值转换为 SSH 配置
    ///
    /// # 参数
    /// - `value`: TOML 值
    ///
    /// # 返回
    /// 返回 Result，成功为 SSHConfig，失败为 Err
    fn try_from(value: toml::Value) -> Result<Self, Self::Error> {
        let table = value.as_table().ok_or("Expected table")?;

        let alias = table
            .get("alias")
            .and_then(|v| v.as_str())
            .ok_or("缺少主机别名")?
            .to_string();

        let address = table
            .get("address")
            .and_then(|v| v.as_str())
            .ok_or("缺少实际连接地址")?
            .to_string();

        let port = table
            .get("port")
            .and_then(|v| v.as_integer())
            .map(|p| p as u16);

        let user = table
            .get("user")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let key = table
            .get("key")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let port_forward = if let Some(pf_value) = table.get("port_forward") {
            if let Some(pf_table) = pf_value.as_table() {
                let enabled = pf_table
                    .get("enabled")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let local = pf_table
                    .get("local")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let remote = pf_table
                    .get("remote")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                Some(PortForward {
                    enabled,
                    local,
                    remote,
                })
            } else {
                None
            }
        } else {
            None
        };

        Ok(SSHConfig {
            alias,
            address,
            port,
            user,
            key,
            port_forward,
            proxy: None,
            use_global_proxy: false,
        })
    }
}
