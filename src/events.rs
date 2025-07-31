use crate::app::App;
use crate::navigation_manager::AppMode;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub struct EventHandler;

impl EventHandler {
    /// 处理键盘事件
    ///
    /// # 参数
    /// - `app`: 应用状态
    /// - `key`: 键盘事件
    ///
    /// # 返回
    /// 返回 Result，true 表示退出，false 表示继续，Err 表示错误
    pub fn handle_key_event(
        app: &mut App,
        key: KeyEvent,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        match key {
            KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => return Ok(true), // 退出信号

            KeyEvent {
                code: KeyCode::Char('n'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                if matches!(*app.mode(), AppMode::List) {
                    app.show_add_form();
                }
            }

            KeyEvent {
                code: KeyCode::Char('e'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                if matches!(*app.mode(), AppMode::List) {
                    app.show_edit_form();
                }
            }

            KeyEvent {
                code: KeyCode::Char('d'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                if matches!(*app.mode(), AppMode::List) {
                    app.show_delete_dialog();
                }
            }

            KeyEvent {
                code:
                    KeyCode::Char('l') | KeyCode::Char('L') | KeyCode::Char('o') | KeyCode::Char('O'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                if matches!(*app.mode(), AppMode::List) {
                    if let Err(e) = app.show_import_selection() {
                        app.message_manager
                            .set_error_message(format!("显示导入选择失败: {e}"));
                    }
                }
            }

            KeyEvent {
                code: KeyCode::Char('p'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                if matches!(*app.mode(), AppMode::List) {
                    app.show_proxy_config();
                }
            }

            KeyEvent {
                code: KeyCode::Enter,
                ..
            } => {
                Self::handle_enter_key(app)?;
            }

            KeyEvent {
                code: KeyCode::Up, ..
            } => {
                Self::handle_up_key(app);
            }

            KeyEvent {
                code: KeyCode::Down,
                ..
            } => {
                Self::handle_down_key(app);
            }

            KeyEvent {
                code: KeyCode::Tab, ..
            } => {
                Self::handle_tab_key(app);
            }

            KeyEvent {
                code: KeyCode::BackTab,
                ..
            } => {
                Self::handle_back_tab_key(app);
            }

            KeyEvent {
                code: KeyCode::Esc, ..
            } => {
                Self::handle_escape_key(app);
            }

            KeyEvent {
                code: KeyCode::Char(' '),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                Self::handle_space_key(app);
            }

            KeyEvent {
                code: KeyCode::Char(ch),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                if ch.is_ascii_graphic() || ch.is_ascii_whitespace() {
                    Self::handle_text_input(app, ch);
                }
            }

            KeyEvent {
                code: KeyCode::Backspace,
                ..
            } => {
                Self::handle_backspace(app);
            }

            KeyEvent {
                code: KeyCode::Delete,
                ..
            } => {
                Self::handle_delete(app);
            }

            KeyEvent {
                code: KeyCode::Left,
                ..
            } => {
                Self::handle_left_key(app);
            }

            KeyEvent {
                code: KeyCode::Right,
                ..
            } => {
                Self::handle_right_key(app);
            }

            KeyEvent {
                code: KeyCode::Home,
                ..
            } => {
                Self::handle_home_key(app);
            }

            KeyEvent {
                code: KeyCode::End, ..
            } => {
                Self::handle_end_key(app);
            }

            KeyEvent {
                code: KeyCode::Char('u'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                Self::handle_ctrl_u(app);
            }

            KeyEvent {
                code: KeyCode::Char('a') | KeyCode::Char('A'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                Self::handle_ctrl_a(app);
            }

            _ => {}
        }

        Ok(false)
    }

    /// 处理回车键事件
    ///
    /// # 参数
    /// - `app`: 应用状态
    ///
    /// # 返回
    /// 返回 Result，成功为 Ok(())，失败为 Err
    fn handle_enter_key(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
        // 辅助函数：执行操作并处理结果
        fn execute_and_handle_error<F>(app: &mut App, operation: F, success_msg: &str)
        where
            F: FnOnce(&mut App) -> Result<(), Box<dyn std::error::Error>>,
        {
            match operation(app) {
                Ok(_) => {
                    app.message_manager
                        .set_success_message(success_msg.to_string());
                }
                Err(e) => {
                    app.message_manager
                        .set_error_message(format!("操作失败: {e}"));
                }
            }
        }

        match *app.mode() {
            AppMode::List => {
                execute_and_handle_error(app, |a| a.connect_selected(), "连接成功");
            }
            AppMode::AddForm | AppMode::EditForm => {
                execute_and_handle_error(app, |a| a.save_config(), "配置保存成功");
            }
            AppMode::DeleteDialog => {
                execute_and_handle_error(app, |a| a.delete_config(), "配置删除成功");
            }
            AppMode::SelectImport => {
                execute_and_handle_error(app, |a| a.confirm_import(), "导入成功");
            }
            AppMode::ProxyConfig => {
                execute_and_handle_error(app, |a| a.save_proxy_config(), "代理配置保存成功");
            }
        }

        Ok(())
    }
    /// 处理上箭头键事件
    ///
    /// # 参数
    /// - `app`: 应用状态
    fn handle_up_key(app: &mut App) {
        match *app.mode() {
            AppMode::List => app.previous(),
            AppMode::AddForm | AppMode::EditForm => app.previous_field(),
            AppMode::SelectImport => app.import_previous(),
            AppMode::ProxyConfig => app.previous_field(),
            _ => {}
        }
    }

    /// 处理下箭头键事件
    ///
    /// # 参数
    /// - `app`: 应用状态
    fn handle_down_key(app: &mut App) {
        match *app.mode() {
            AppMode::List => app.next(),
            AppMode::AddForm | AppMode::EditForm => app.next_field(),
            AppMode::SelectImport => app.import_next(),
            AppMode::ProxyConfig => app.next_field(),
            _ => {}
        }
    }

    /// 处理 Tab 键事件
    ///
    /// # 参数
    /// - `app`: 应用状态
    fn handle_tab_key(app: &mut App) {
        match *app.mode() {
            AppMode::List => app.toggle_focus(),
            AppMode::AddForm | AppMode::EditForm | AppMode::ProxyConfig => app.next_field(),
            _ => {}
        }
    }

    /// 处理 Shift+Tab 键事件
    ///
    /// # 参数
    /// - `app`: 应用状态
    fn handle_back_tab_key(app: &mut App) {
        match *app.mode() {
            AppMode::AddForm | AppMode::EditForm | AppMode::ProxyConfig => app.previous_field(),
            _ => {}
        }
    }

    /// 处理 Escape 键事件
    ///
    /// # 参数
    /// - `app`: 应用状态
    fn handle_escape_key(app: &mut App) {
        match *app.mode() {
            AppMode::SelectImport => app.cancel_import(),
            _ => app.cancel_action(),
        }
    }

    /// 处理空格键事件
    ///
    /// # 参数
    /// - `app`: 应用状态
    fn handle_space_key(app: &mut App) {
        match *app.mode() {
            AppMode::SelectImport => app.toggle_import_selection(),
            AppMode::AddForm | AppMode::EditForm => {
                match app.current_field() {
                    5 => app.toggle_checkbox(),     // 端口转发
                    8 => app.toggle_proxy_option(), // 代理选项
                    _ => {}
                }
            }
            AppMode::ProxyConfig => {
                if app.current_field() == 0 {
                    // 代理类型字段
                    // 通过 FormManager 切换代理类型
                    let current_type = app
                        .form_data()
                        .get("global_proxy_type")
                        .map(|t| match t.as_str() {
                            "Socks5" => "Http",
                            "Http" => "None",
                            _ => "Socks5",
                        })
                        .unwrap_or("Socks5");
                    app.form_manager
                        .form_data
                        .data
                        .insert("global_proxy_type".to_string(), current_type.to_string());
                }
            }
            _ => {}
        }
    }

    /// 处理文本输入事件
    ///
    /// # 参数
    /// - `app`: 应用状态
    /// - `ch`: 输入的字符
    fn handle_text_input(app: &mut App, ch: char) {
        match *app.mode() {
            AppMode::AddForm | AppMode::EditForm => app.insert_char(ch),
            AppMode::ProxyConfig => app.insert_char(ch),
            _ => {}
        }
    }

    /// 处理退格键事件
    ///
    /// # 参数
    /// - `app`: 应用状态
    fn handle_backspace(app: &mut App) {
        match *app.mode() {
            AppMode::AddForm | AppMode::EditForm => app.delete_char(),
            AppMode::ProxyConfig => app.delete_char(),
            _ => {}
        }
    }

    /// 处理删除键事件
    ///
    /// # 参数
    /// - `app`: 应用状态
    fn handle_delete(app: &mut App) {
        match *app.mode() {
            AppMode::AddForm | AppMode::EditForm => app.delete_char_forward(),
            AppMode::ProxyConfig => app.delete_char_forward(),
            _ => {}
        }
    }

    /// 处理左箭头键事件
    ///
    /// # 参数
    /// - `app`: 应用状态
    fn handle_left_key(app: &mut App) {
        match *app.mode() {
            AppMode::AddForm | AppMode::EditForm => app.move_cursor_left(),
            AppMode::ProxyConfig => app.move_cursor_left(),
            _ => {}
        }
    }

    /// 处理右箭头键事件
    ///
    /// # 参数
    /// - `app`: 应用状态
    fn handle_right_key(app: &mut App) {
        match *app.mode() {
            AppMode::AddForm | AppMode::EditForm => app.move_cursor_right(),
            AppMode::ProxyConfig => app.move_cursor_right(),
            _ => {}
        }
    }

    /// 处理 Home 键事件
    ///
    /// # 参数
    /// - `app`: 应用状态
    fn handle_home_key(app: &mut App) {
        match *app.mode() {
            AppMode::AddForm | AppMode::EditForm => app.move_cursor_to_start(),
            AppMode::ProxyConfig => app.move_cursor_to_start(),
            _ => {}
        }
    }

    /// 处理 End 键事件
    ///
    /// # 参数
    /// - `app`: 应用状态
    fn handle_end_key(app: &mut App) {
        match *app.mode() {
            AppMode::AddForm | AppMode::EditForm => app.move_cursor_to_end(),
            AppMode::ProxyConfig => app.move_cursor_to_end(),
            _ => {}
        }
    }

    /// 处理 Ctrl+U 事件（清空当前字段）
    ///
    /// # 参数
    /// - `app`: 应用状态
    fn handle_ctrl_u(app: &mut App) {
        match *app.mode() {
            AppMode::AddForm | AppMode::EditForm => app.clear_current_field(),
            AppMode::ProxyConfig => app.clear_current_field(),
            _ => {}
        }
    }

    /// 处理 Ctrl+A 事件（全选）
    ///
    /// # 参数
    /// - `app`: 应用状态
    fn handle_ctrl_a(app: &mut App) {
        match *app.mode() {
            AppMode::AddForm | AppMode::EditForm => app.move_cursor_to_start(),
            AppMode::SelectImport => app.toggle_all_import_selection(),
            _ => {}
        }
    }
}
