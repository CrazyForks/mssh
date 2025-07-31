use crate::config::{PortForward, SSHConfig};
use crate::proxy::{ProxyConfig, ProxyType};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum FormField {
    Alias,
    Address,
    Port,
    User,
    Key,
    PortForwardEnabled,
    PortForwardLocal,
    PortForwardRemote,
    UseGlobalProxy,
    ProxyHost,
    ProxyPort,
    ProxyUsername,
    ProxyPassword,
    // 全局代理配置字段
    GlobalProxyType,
    GlobalProxyHost,
    GlobalProxyPort,
    GlobalProxyUsername,
    GlobalProxyPassword,
}

impl FormField {
    /// 获取字段对应的字符串标识符
    ///
    /// # 返回
    /// 返回字段的字符串标识符
    pub fn as_str(&self) -> &'static str {
        match self {
            FormField::Alias => "alias",
            FormField::Address => "address",
            FormField::Port => "port",
            FormField::User => "user",
            FormField::Key => "key",
            FormField::PortForwardEnabled => "pf_enabled",
            FormField::PortForwardLocal => "pf_local",
            FormField::PortForwardRemote => "pf_remote",
            FormField::UseGlobalProxy => "use_global_proxy",
            FormField::ProxyHost => "proxy_host",
            FormField::ProxyPort => "proxy_port",
            FormField::ProxyUsername => "proxy_username",
            FormField::ProxyPassword => "proxy_password",
            FormField::GlobalProxyType => "global_proxy_type",
            FormField::GlobalProxyHost => "global_proxy_host",
            FormField::GlobalProxyPort => "global_proxy_port",
            FormField::GlobalProxyUsername => "global_proxy_username",
            FormField::GlobalProxyPassword => "global_proxy_password",
        }
    }

    /// 判断字段是否为文本输入类型
    ///
    /// # 返回
    /// 返回 true 表示是文本输入，false 表示是特殊控件
    pub fn is_text_input(&self) -> bool {
        !matches!(
            self,
            FormField::PortForwardEnabled | FormField::UseGlobalProxy | FormField::GlobalProxyType
        )
    }

    /// 获取 SSH 配置编辑字段列表
    ///
    /// # 返回
    /// 返回 SSH 配置编辑时使用的字段列表（不包含全局代理字段）
    pub fn ssh_config_fields() -> Vec<FormField> {
        vec![
            FormField::Alias,
            FormField::Address,
            FormField::Port,
            FormField::User,
            FormField::Key,
            FormField::PortForwardEnabled,
            FormField::PortForwardLocal,
            FormField::PortForwardRemote,
            FormField::UseGlobalProxy,
            FormField::ProxyHost,
            FormField::ProxyPort,
            FormField::ProxyUsername,
            FormField::ProxyPassword,
        ]
    }

    /// 获取全局代理配置字段列表
    ///
    /// # 返回
    /// 返回全局代理配置时使用的字段列表
    pub fn global_proxy_fields() -> Vec<FormField> {
        vec![
            FormField::GlobalProxyType,
            FormField::GlobalProxyHost,
            FormField::GlobalProxyPort,
            FormField::GlobalProxyUsername,
            FormField::GlobalProxyPassword,
        ]
    }
}

#[derive(Debug, Clone)]
pub struct FormData {
    pub data: HashMap<String, String>,
    pub current_field: usize,
    pub cursor_position: usize,
}

impl FormData {
    /// 创建新的表单数据实例
    ///
    /// # 返回
    /// 返回初始化的表单数据
    pub fn new() -> Self {
        let mut data = HashMap::new();
        data.insert("pf_enabled".to_string(), "false".to_string());
        data.insert("use_global_proxy".to_string(), "true".to_string());
        data.insert("proxy_enabled".to_string(), "false".to_string());

        let mut form_data = Self {
            data,
            current_field: 0,
            cursor_position: 0,
        };

        // 确保字段索引在有效范围内
        form_data.ensure_field_index_valid();

        form_data
    }

    /// 根据 SSH 配置创建表单数据
    ///
    /// # 参数
    /// - `config`: SSH 配置
    ///
    /// # 返回
    /// 返回填充了配置数据的表单数据
    pub fn from_config(config: &SSHConfig) -> Self {
        let mut form_data = Self::new();

        form_data
            .data
            .insert("alias".to_string(), config.alias.clone());
        form_data
            .data
            .insert("address".to_string(), config.address.clone());

        if let Some(port) = config.port {
            form_data.data.insert("port".to_string(), port.to_string());
        }

        if let Some(user) = &config.user {
            form_data.data.insert("user".to_string(), user.clone());
        }

        if let Some(key) = &config.key {
            form_data.data.insert("key".to_string(), key.clone());
        }

        if let Some(pf) = &config.port_forward {
            form_data
                .data
                .insert("pf_enabled".to_string(), pf.enabled.to_string());
            form_data
                .data
                .insert("pf_local".to_string(), pf.local.clone());
            form_data
                .data
                .insert("pf_remote".to_string(), pf.remote.clone());
        }

        form_data.data.insert(
            "use_global_proxy".to_string(),
            config.use_global_proxy.to_string(),
        );

        if let Some(proxy) = &config.proxy {
            form_data
                .data
                .insert("proxy_enabled".to_string(), "true".to_string());
            form_data
                .data
                .insert("proxy_type".to_string(), format!("{:?}", proxy.proxy_type));
            form_data
                .data
                .insert("proxy_host".to_string(), proxy.host.clone());

            if let Some(port) = proxy.port {
                form_data
                    .data
                    .insert("proxy_port".to_string(), port.to_string());
            }

            if let Some(username) = &proxy.username {
                form_data
                    .data
                    .insert("proxy_username".to_string(), username.clone());
            }

            if let Some(password) = &proxy.password {
                form_data
                    .data
                    .insert("proxy_password".to_string(), password.clone());
            }
        }

        // 确保字段索引在有效范围内
        form_data.ensure_field_index_valid();

        form_data
    }

    /// 获取指定字段的值
    ///
    /// # 参数
    /// - `field`: 字段枚举
    ///
    /// # 返回
    /// 返回字段值，如果不存在则返回空字符串
    pub fn get(&self, field: &FormField) -> String {
        self.data.get(field.as_str()).cloned().unwrap_or_default()
    }

    /// 设置指定字段的值
    ///
    /// # 参数
    /// - `field`: 字段枚举
    /// - `value`: 要设置的值
    pub fn set(&mut self, field: &FormField, value: String) {
        self.data.insert(field.as_str().to_string(), value);
    }

    /// 获取当前字段
    ///
    /// # 返回
    /// 返回当前字段枚举
    pub fn get_current_field(&self) -> FormField {
        if self.is_global_proxy_mode() {
            FormField::global_proxy_fields()
                .get(self.current_field)
                .cloned()
                .unwrap_or(FormField::GlobalProxyType)
        } else {
            FormField::ssh_config_fields()
                .get(self.current_field)
                .cloned()
                .unwrap_or(FormField::Alias)
        }
    }

    /// 获取当前字段的值
    ///
    /// # 返回
    /// 返回当前字段的值
    pub fn get_current_value(&self) -> String {
        let field = self.get_current_field();
        self.get(&field)
    }

    /// 设置当前字段的值
    ///
    /// # 参数
    /// - `value`: 要设置的值
    pub fn set_current_value(&mut self, value: String) {
        let field = self.get_current_field();
        self.set(&field, value);
    }

    /// 移动到下一个字段
    pub fn next_field(&mut self) {
        let max_field = if self.is_global_proxy_mode() {
            FormField::global_proxy_fields().len()
        } else {
            FormField::ssh_config_fields().len()
        };
        self.current_field = (self.current_field + 1) % max_field;
        self.cursor_position = self.get_current_value().len();
    }

    /// 移动到上一个字段
    pub fn previous_field(&mut self) {
        let max_field = if self.is_global_proxy_mode() {
            FormField::global_proxy_fields().len()
        } else {
            FormField::ssh_config_fields().len()
        };
        self.current_field = if self.current_field == 0 {
            max_field - 1
        } else {
            self.current_field - 1
        };
        self.cursor_position = self.get_current_value().len();
    }

    /// 光标左移
    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    /// 光标右移
    pub fn move_cursor_right(&mut self) {
        let value = self.get_current_value();
        if self.cursor_position < value.len() {
            self.cursor_position += 1;
        }
    }

    /// 光标移动到开头
    pub fn move_cursor_to_start(&mut self) {
        self.cursor_position = 0;
    }

    /// 光标移动到结尾
    pub fn move_cursor_to_end(&mut self) {
        let value = self.get_current_value();
        self.cursor_position = value.len();
    }

    /// 在当前光标位置插入字符
    ///
    /// # 参数
    /// - `c`: 要插入的字符
    pub fn insert_char(&mut self, c: char) {
        let field = self.get_current_field();
        if field.is_text_input() {
            let mut value = self.get_current_value();
            if self.cursor_position <= value.len() {
                value.insert(self.cursor_position, c);
                self.cursor_position += 1;
                self.set_current_value(value);
            }
        }
    }

    /// 删除光标前的字符
    pub fn delete_char(&mut self) {
        let field = self.get_current_field();
        if field.is_text_input() {
            let mut value = self.get_current_value();
            if self.cursor_position > 0 {
                value.remove(self.cursor_position - 1);
                self.cursor_position -= 1;
                self.set_current_value(value);
            }
        }
    }

    /// 删除光标后的字符
    pub fn delete_char_forward(&mut self) {
        let field = self.get_current_field();
        if field.is_text_input() {
            let mut value = self.get_current_value();
            if self.cursor_position < value.len() {
                value.remove(self.cursor_position);
                self.set_current_value(value);
            }
        }
    }

    /// 清空当前字段
    pub fn clear_current_field(&mut self) {
        let field = self.get_current_field();
        if field.is_text_input() {
            self.set_current_value(String::new());
            self.cursor_position = 0;
        }
    }

    /// 切换复选框状态（端口转发启用/禁用）
    pub fn toggle_checkbox(&mut self) {
        let field = self.get_current_field();
        if field == FormField::PortForwardEnabled {
            let current = self.get(&field).to_lowercase() == "true";
            self.set(&field, (!current).to_string());
        }
    }

    /// 切换代理选项（全局代理/不使用代理/SOCKS5/HTTP）
    pub fn toggle_proxy_option(&mut self) {
        let field = self.get_current_field();
        if matches!(field, FormField::UseGlobalProxy) {
            let use_global_proxy = self.get(&FormField::UseGlobalProxy).to_lowercase() == "true";
            let proxy_enabled = self
                .data
                .get("proxy_enabled")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(false);
            let proxy_type = self
                .data
                .get("proxy_type")
                .map(|t| t.as_str())
                .unwrap_or("None");

            // 状态循环: 全局代理 -> 不使用代理 -> SOCKS5代理 -> HTTP代理 -> 全局代理
            if use_global_proxy {
                // 全局代理 -> 不使用代理
                self.set(&FormField::UseGlobalProxy, "false".to_string());
                self.data
                    .insert("proxy_enabled".to_string(), "false".to_string());
                self.data
                    .insert("proxy_type".to_string(), "None".to_string());
            } else if !proxy_enabled || proxy_type == "None" {
                // 不使用代理 -> SOCKS5代理
                self.data
                    .insert("proxy_enabled".to_string(), "true".to_string());
                self.data
                    .insert("proxy_type".to_string(), "Socks5".to_string());
            } else if proxy_type == "Socks5" {
                // SOCKS5代理 -> HTTP代理
                self.data
                    .insert("proxy_type".to_string(), "Http".to_string());
            } else if proxy_type == "Http" {
                // HTTP代理 -> 全局代理
                self.set(&FormField::UseGlobalProxy, "true".to_string());
                self.data
                    .insert("proxy_enabled".to_string(), "false".to_string());
                self.data
                    .insert("proxy_type".to_string(), "None".to_string());
            } else {
                // 默认回到全局代理
                self.set(&FormField::UseGlobalProxy, "true".to_string());
                self.data
                    .insert("proxy_enabled".to_string(), "false".to_string());
                self.data
                    .insert("proxy_type".to_string(), "None".to_string());
            }
        }
    }

    /// 验证表单数据
    ///
    /// # 返回
    /// 返回 Result，成功为 Ok(())，失败为包含错误信息的 Err
    pub fn validate(&self) -> Result<(), String> {
        let alias = self.get(&FormField::Alias);
        let address = self.get(&FormField::Address);
        let port_str = self.get(&FormField::Port);

        // 基本信息验证
        if alias.is_empty() {
            return Err("主机别名不能为空".to_string());
        }

        if address.is_empty() {
            return Err("连接地址不能为空".to_string());
        }

        if !port_str.is_empty() {
            let port: u16 = port_str
                .parse()
                .map_err(|_| "端口必须是1-65535之间的有效数字".to_string())?;
            if port == 0 {
                return Err("端口必须是1-65535之间的有效数字".to_string());
            }
        }

        // 端口转发验证
        let pf_enabled = self.get(&FormField::PortForwardEnabled).to_lowercase() == "true";
        if pf_enabled {
            let local = self.get(&FormField::PortForwardLocal);
            let remote = self.get(&FormField::PortForwardRemote);

            if local.is_empty() {
                return Err("启用端口转发时，本地端口不能为空".to_string());
            }

            if remote.is_empty() {
                return Err("启用端口转发时，远程端口不能为空".to_string());
            }
        }

        // 代理配置验证
        let use_global_proxy = self.get(&FormField::UseGlobalProxy).to_lowercase() == "true";
        if !use_global_proxy {
            let proxy_enabled = self
                .data
                .get("proxy_enabled")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(false);
            if proxy_enabled {
                let proxy_host = self.get(&FormField::ProxyHost);
                let proxy_port_str = self.get(&FormField::ProxyPort);

                if proxy_host.is_empty() {
                    return Err("代理主机不能为空".to_string());
                }

                if !proxy_port_str.is_empty() {
                    let proxy_port: u16 = proxy_port_str
                        .parse()
                        .map_err(|_| "代理端口必须是1-65535之间的有效数字".to_string())?;
                    if proxy_port == 0 {
                        return Err("代理端口必须是1-65535之间的有效数字".to_string());
                    }
                } else {
                    return Err("代理端口不能为空".to_string());
                }
            }
        }

        Ok(())
    }

    /// 验证全局代理配置数据
    ///
    /// # 返回
    /// 返回 Result，成功为 Ok(())，失败为包含错误信息的 Err
    pub fn validate_global_proxy(&self) -> Result<(), String> {
        let proxy_type = self.get(&FormField::GlobalProxyType);
        let proxy_host = self.get(&FormField::GlobalProxyHost);
        let proxy_port_str = self.get(&FormField::GlobalProxyPort);

        // 代理类型验证
        if proxy_type.is_empty() {
            return Err("代理类型不能为空".to_string());
        }

        // 如果选择了代理类型，则主机和端口必填
        if proxy_type != "None" {
            if proxy_host.is_empty() {
                return Err("代理主机不能为空".to_string());
            }

            if proxy_port_str.is_empty() {
                return Err("代理端口不能为空".to_string());
            }

            let proxy_port: u16 = proxy_port_str
                .parse()
                .map_err(|_| "代理端口必须是1-65535之间的有效数字".to_string())?;
            if proxy_port == 0 {
                return Err("代理端口必须是1-65535之间的有效数字".to_string());
            }
        }

        Ok(())
    }

    /// 检查是否在全局代理配置模式
    ///
    /// # 返回
    /// 返回 true 表示在全局代理配置模式，false 表示在 SSH 配置编辑模式
    pub fn is_global_proxy_mode(&self) -> bool {
        // 检查是否存在全局代理字段的数据
        self.data.contains_key("global_proxy_type")
    }

    /// 确保字段索引在有效范围内
    fn ensure_field_index_valid(&mut self) {
        let max_field = if self.is_global_proxy_mode() {
            FormField::global_proxy_fields().len()
        } else {
            FormField::ssh_config_fields().len()
        };
        if self.current_field >= max_field {
            self.current_field = 0;
        }
    }

    /// 将表单数据转换为 SSH 配置
    ///
    /// # 返回
    /// 返回 Result，成功为 SSHConfig，失败为包含错误信息的 Err
    pub fn to_ssh_config(&self) -> Result<SSHConfig, String> {
        self.validate()?;

        let alias = self.get(&FormField::Alias);
        let address = self.get(&FormField::Address);
        let port = self.get(&FormField::Port).parse::<u16>().ok();
        let user = if self.get(&FormField::User).is_empty() {
            None
        } else {
            Some(self.get(&FormField::User))
        };
        let key = if self.get(&FormField::Key).is_empty() {
            None
        } else {
            Some(self.get(&FormField::Key))
        };

        let pf_enabled = self.get(&FormField::PortForwardEnabled).to_lowercase() == "true";
        let port_forward = if pf_enabled {
            let local = self.get(&FormField::PortForwardLocal);
            let remote = self.get(&FormField::PortForwardRemote);
            if !local.is_empty() && !remote.is_empty() {
                Some(PortForward {
                    enabled: true,
                    local,
                    remote,
                })
            } else {
                None
            }
        } else {
            None
        };

        let use_global_proxy = self.get(&FormField::UseGlobalProxy).to_lowercase() == "true";
        let proxy = if !use_global_proxy {
            let proxy_enabled = self
                .data
                .get("proxy_enabled")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(false);

            if proxy_enabled {
                let proxy_type = self
                    .data
                    .get("proxy_type")
                    .map(|t| match t.as_str() {
                        "Socks5" => ProxyType::Socks5,
                        "Http" => ProxyType::Http,
                        _ => ProxyType::None,
                    })
                    .unwrap_or(ProxyType::None);

                if proxy_type != ProxyType::None {
                    let host = self.get(&FormField::ProxyHost);
                    let port = self.get(&FormField::ProxyPort).parse::<u16>().ok();
                    let username = if self.get(&FormField::ProxyUsername).is_empty() {
                        None
                    } else {
                        Some(self.get(&FormField::ProxyUsername))
                    };
                    let password = if self.get(&FormField::ProxyPassword).is_empty() {
                        None
                    } else {
                        Some(self.get(&FormField::ProxyPassword))
                    };

                    Some(ProxyConfig {
                        proxy_type,
                        host,
                        port,
                        username,
                        password,
                    })
                } else {
                    None
                }
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
            proxy,
            use_global_proxy,
        })
    }
}
