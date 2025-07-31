use crate::config::{ConfigManager, SSHConfig};
use crate::form_manager::FormManager;
use crate::message_manager::MessageManager;
use crate::navigation_manager::{AppMode, NavigationManager};
use crate::ssh::SSHManager;

pub use crate::message_manager::Message;

#[derive(Debug, Clone)]
pub struct App {
    pub config_manager: ConfigManager,
    pub ssh_manager: SSHManager,
    pub navigation: NavigationManager,
    pub form_manager: FormManager,
    pub message_manager: MessageManager,
}

impl App {
    /// 创建新的应用实例
    ///
    /// # 参数
    /// - `config_path`: 配置文件路径，可选
    ///
    /// # 返回
    /// 返回 Result，成功为 App 实例，失败为 Err
    pub fn new(config_path: Option<String>) -> Result<Self, Box<dyn std::error::Error>> {
        let config_manager = ConfigManager::new(config_path)?;
        let ssh_manager = SSHManager::new(config_manager.global_config.clone());

        Ok(Self {
            config_manager,
            ssh_manager,
            navigation: NavigationManager::new(),
            form_manager: FormManager::new(),
            message_manager: MessageManager::new(),
        })
    }

    /// 获取当前应用模式
    ///
    /// # 返回
    /// 返回当前应用模式引用
    pub fn mode(&self) -> &AppMode {
        &self.navigation.mode
    }

    /// 获取当前选中的索引
    ///
    /// # 返回
    /// 返回当前选中的索引
    pub fn selected_index(&self) -> usize {
        self.navigation.selected_index
    }

    /// 获取当前焦点
    ///
    /// # 返回
    /// 返回当前焦点索引
    pub fn focus(&self) -> usize {
        self.navigation.focus
    }

    /// 获取当前表单字段索引
    ///
    /// # 返回
    /// 返回当前表单字段索引
    pub fn current_field(&self) -> usize {
        self.form_manager.get_current_field()
    }

    /// 获取当前光标位置
    ///
    /// # 返回
    /// 返回当前光标位置
    pub fn cursor_position(&self) -> usize {
        self.form_manager.get_cursor_position()
    }

    /// 获取表单数据
    ///
    /// # 返回
    /// 返回表单数据的引用
    pub fn form_data(&self) -> &std::collections::HashMap<String, String> {
        self.form_manager.get_form_data()
    }

    /// 获取当前消息
    ///
    /// # 返回
    /// 返回当前消息的引用，如果没有则为 None
    pub fn message(&self) -> Option<&Message> {
        self.message_manager.get_message()
    }

    /// 导航到下一个项目
    pub fn next(&mut self) {
        if self.navigation.is_list_mode() {
            let len = self.config_manager.configs.len();
            self.navigation.next_item(len);
        }
    }

    /// 导航到上一个项目
    pub fn previous(&mut self) {
        if self.navigation.is_list_mode() {
            let len = self.config_manager.configs.len();
            self.navigation.previous_item(len);
        }
    }

    /// 切换焦点
    pub fn toggle_focus(&mut self) {
        self.navigation.toggle_focus(2); // 2个焦点区域：列表和详情
    }

    /// 显示添加表单
    pub fn show_add_form(&mut self) {
        self.navigation.set_mode(AppMode::AddForm);
        self.form_manager.start_add_form();
    }

    /// 显示编辑表单
    pub fn show_edit_form(&mut self) {
        if let Some(config) = self.get_selected_config().cloned() {
            self.navigation.set_mode(AppMode::EditForm);
            self.form_manager.start_edit_form(&config);
        }
    }

    /// 显示删除对话框
    pub fn show_delete_dialog(&mut self) {
        if self.get_selected_config().is_some() {
            self.navigation.set_mode(AppMode::DeleteDialog);
        }
    }

    /// 显示代理配置
    pub fn show_proxy_config(&mut self) {
        self.navigation.set_mode(AppMode::ProxyConfig);
        let global_proxy = &self.config_manager.global_config.proxy;
        self.form_manager.start_proxy_config(global_proxy);
    }

    /// 取消当前操作
    pub fn cancel_action(&mut self) {
        self.navigation.return_to_list();
        self.form_manager.clear();
        self.message_manager.clear_message();
    }

    /// 保存配置
    ///
    /// # 返回
    /// 返回 Result，成功为 Ok(())，失败为 Err
    pub fn save_config(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let config = self.form_manager.validate_and_create_config()?;

        let result = if self.form_manager.is_editing() {
            if let Some(editing_host) = self.form_manager.get_editing_host() {
                self.config_manager.update_config(editing_host, config)
            } else {
                return Err("正在编辑的主机不存在".into());
            }
        } else {
            self.config_manager.add_config(config)
        };

        result?;
        self.navigation.return_to_list();
        self.form_manager.clear();

        Ok(())
    }

    /// 删除配置
    ///
    /// # 返回
    /// 返回 Result，成功为 Ok(())，失败为 Err
    pub fn delete_config(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(config) = self.get_selected_config() {
            let alias = config.alias.clone();
            self.config_manager.remove_config(&alias)?; // 删除失败自动返回 Err
        }
        self.navigation.return_to_list();
        Ok(())
    }

    /// 保存代理配置
    ///
    /// # 返回
    /// 返回 Result，成功为 Ok(())，失败为 Err
    pub fn save_proxy_config(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let proxy_config = self.form_manager.create_proxy_config()?;
        self.config_manager.global_config.proxy = proxy_config;
        self.config_manager.save_configs()?;
        self.navigation.return_to_list();
        Ok(())
    }
    /// 连接选中的配置
    ///
    /// # 返回
    /// 返回 Result，成功为 Ok(())，失败为 Err
    pub fn connect_selected(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let valid_index = self
            .navigation
            .get_valid_selected_index(self.config_manager.configs.len());
        if valid_index < self.config_manager.configs.len() {
            let config = self.config_manager.configs[valid_index].clone();
            self.ssh_manager.global_config = self.config_manager.global_config.clone();
            self.ssh_manager.connect(&config)?;
        }
        Ok(())
    }

    /// 获取选中的配置
    ///
    /// # 返回
    /// 返回选中的配置引用，如果没有则为 None
    pub fn get_selected_config(&self) -> Option<&SSHConfig> {
        if self.config_manager.configs.is_empty() {
            return None;
        }

        let valid_index = self
            .navigation
            .get_valid_selected_index(self.config_manager.configs.len());
        self.config_manager.configs.get(valid_index)
    }

    /// 移动到下一个表单字段
    pub fn next_field(&mut self) {
        self.form_manager.next_field();
    }

    /// 移动到上一个表单字段
    pub fn previous_field(&mut self) {
        self.form_manager.previous_field();
    }

    /// 光标左移
    pub fn move_cursor_left(&mut self) {
        self.form_manager.move_cursor_left();
    }

    /// 光标右移
    pub fn move_cursor_right(&mut self) {
        self.form_manager.move_cursor_right();
    }

    /// 光标移动到开头
    pub fn move_cursor_to_start(&mut self) {
        self.form_manager.move_cursor_to_start();
    }

    /// 光标移动到结尾
    pub fn move_cursor_to_end(&mut self) {
        self.form_manager.move_cursor_to_end();
    }

    /// 插入字符
    ///
    /// # 参数
    /// - `c`: 要插入的字符
    pub fn insert_char(&mut self, c: char) {
        self.form_manager.insert_char(c);
    }

    /// 删除字符
    pub fn delete_char(&mut self) {
        self.form_manager.delete_char();
    }

    /// 向前删除字符
    pub fn delete_char_forward(&mut self) {
        self.form_manager.delete_char_forward();
    }

    /// 清空当前字段
    pub fn clear_current_field(&mut self) {
        self.form_manager.clear_current_field();
    }

    /// 切换复选框状态
    pub fn toggle_checkbox(&mut self) {
        self.form_manager.toggle_checkbox();
    }

    /// 切换代理选项
    pub fn toggle_proxy_option(&mut self) {
        self.form_manager.toggle_proxy_option();
    }

    /// 显示导入选择界面
    ///
    /// # 返回
    /// 返回 Result，成功为 Ok(())，失败为 Err
    pub fn show_import_selection(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(format!("{}/.ssh/config", std::env::var("HOME")?))?;
        let mut candidates = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("Host ") {
                if let Some(host) = self.config_manager.parse_ssh_host_config(&content, line) {
                    candidates.push(host);
                }
            }
        }

        candidates.retain(|host| {
            !self
                .config_manager
                .configs
                .iter()
                .any(|c| c.alias == host.alias)
        });

        self.navigation.start_import(candidates);
        Ok(())
    }

    /// 导入界面下一个项目
    pub fn import_next(&mut self) {
        self.navigation.import_next();
    }

    /// 导入界面上一个项目
    pub fn import_previous(&mut self) {
        self.navigation.import_previous();
    }

    /// 切换导入选择状态
    pub fn toggle_import_selection(&mut self) {
        self.navigation.toggle_import_selection();
    }

    /// 切换所有导入选择状态
    pub fn toggle_all_import_selection(&mut self) {
        self.navigation.toggle_all_import_selection();
    }

    /// 确认导入
    ///
    /// # 返回
    /// 返回 Result，成功为 Ok(())，失败为 Err
    pub fn confirm_import(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let selected_configs = self.navigation.get_selected_imports();
        let mut imported_count = 0;

        for config in selected_configs {
            self.config_manager.add_config(config)?; // 出错自动返回 Err
            imported_count += 1;
        }

        self.navigation.return_to_list();

        if imported_count == 0 {
            return Err("未选择任何可导入的配置".into());
        }

        Ok(())
    }

    /// 取消导入
    pub fn cancel_import(&mut self) {
        self.navigation.cancel_import();
        self.message_manager.clear_message();
    }

    /// 快速连接
    ///
    /// # 参数
    /// - `target`: 目标配置（编号或别名）
    ///
    /// # 返回
    /// 返回 Result，成功为 Ok(())，失败为 Err
    pub fn quick_connect(&mut self, target: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(index) = target.parse::<usize>() {
            if index > 0 && index <= self.config_manager.configs.len() {
                let config = &self.config_manager.configs[index - 1];
                println!("正在连接到 {} (编号: {})", config.alias, index);
                self.ssh_manager.connect(config)?;
                return Ok(());
            } else {
                return Err(format!(
                    "编号 {} 超出范围 (1-{})",
                    index,
                    self.config_manager.configs.len()
                )
                .into());
            }
        }

        for (i, config) in self.config_manager.configs.iter().enumerate() {
            if config.alias == target {
                println!("正在连接到 {} (编号: {})", config.alias, i + 1);
                self.ssh_manager.connect(config)?;
                return Ok(());
            }
        }

        let mut matches = Vec::new();
        for (i, config) in self.config_manager.configs.iter().enumerate() {
            if config.alias.contains(target)
                || config.address.contains(target)
                || config.user.as_ref().is_some_and(|u| u.contains(target))
            {
                matches.push((i + 1, config));
            }
        }

        if matches.len() == 1 {
            let (index, config) = matches[0];
            println!("正在连接到 {} (编号: {})", config.alias, index);
            self.ssh_manager.connect(config)?;
            return Ok(());
        } else if matches.len() > 1 {
            println!("找到多个匹配的配置:");
            for (index, config) in matches {
                println!("  {}: {} ({})", index, config.alias, config.address);
            }
            return Err("请使用更具体的编号或别名".into());
        }

        Err(format!("未找到匹配的配置: {target}").into())
    }

    /// 检查并清理过期消息
    pub fn check_message(&mut self) {
        self.message_manager.check_and_clear_expired();
    }

    /// 导入相关访问器（向后兼容）
    /// 获取导入候选列表
    ///
    /// # 返回
    /// 返回导入候选配置列表的引用
    pub fn import_candidates(&self) -> &Vec<SSHConfig> {
        &self.navigation.import_manager.candidates
    }

    /// 获取导入选择状态
    ///
    /// # 返回
    /// 返回导入选择状态列表的引用
    pub fn import_selected(&self) -> &Vec<bool> {
        &self.navigation.import_manager.selected
    }

    /// 获取导入选择索引
    ///
    /// # 返回
    /// 返回导入选择索引
    pub fn import_selected_index(&self) -> usize {
        self.navigation.import_manager.selected_index
    }
}
