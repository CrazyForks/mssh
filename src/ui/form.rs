use crate::app::App;
use crate::navigation_manager::AppMode;
use ratatui::{prelude::*, widgets::*};

/// 渲染表单界面
///
/// # 参数
/// - `f`: 绘制 Frame
/// - `area`: 绘制区域
/// - `app`: 应用状态
pub fn render_form(f: &mut Frame, area: Rect, app: &mut App) {
    let title = match *app.mode() {
        AppMode::AddForm => "添加 SSH 配置",
        AppMode::EditForm => "编辑 SSH 配置",
        _ => "表单",
    };

    let form_block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Color::Cyan));

    let inner_area = form_block.inner(area);
    f.render_widget(form_block, area);

    // 计算可视区域大小（减去标题和边框）
    let visible_height = inner_area.height.saturating_sub(2); // 减去标题和边框
    let visible_fields = if visible_height > 0 {
        visible_height as usize / 3
    } else {
        1
    }; // 每个字段大约3行高度，至少1个

    // 更新表单管理器的可视区域大小
    app.form_manager.set_visible_fields(visible_fields);

    // 获取总字段数
    let total_fields = if app.form_data().contains_key("global_proxy_type") {
        crate::forms::FormField::global_proxy_fields().len()
    } else {
        crate::forms::FormField::ssh_config_fields().len()
    };

    // 设置总字段数并更新滚动位置
    app.form_manager
        .scroll_manager
        .set_total_items(total_fields);
    app.form_manager.update_scroll_position();

    // 检查是否需要滚动
    if total_fields > visible_fields {
        render_scrollable_form_impl(f, inner_area, app, visible_fields);
    } else {
        render_full_form(f, inner_area, app);
    }
}

/// 渲染可滚动表单的实现
fn render_scrollable_form_impl(f: &mut Frame, area: Rect, app: &mut App, visible_fields: usize) {
    // 定义段落信息
    let sections = if app.form_data().contains_key("global_proxy_type") {
        // 全局代理配置只有一个段落
        vec![(0, "全局代理配置", 5)]
    } else {
        // SSH配置有三个段落
        vec![
            (0, "基本信息", 5), // 别名、地址、端口、用户、密钥
            (5, "端口转发", 3), // 启用、本地端口、远程端口
            (8, "代理配置", 5), // 代理设置、代理主机、代理端口、代理用户名、代理密码
        ]
    };

    let (scroll_offset, _, _) = app.form_manager.get_scroll_info();

    let mut current_y = area.y;
    let mut rendered_items = 0;

    // 带段落标题的滚动渲染
    for (section_start, section_title, section_item_count) in sections {
        // 检查这个段落是否在可视区域内
        let section_end = section_start + section_item_count;

        if section_end <= scroll_offset {
            // 整个段落在可视区域上方，跳过
            continue;
        }

        if section_start >= scroll_offset + visible_fields {
            // 整个段落在可视区域下方，跳过
            break;
        }

        // 计算段落在可视区域内的项目
        let visible_start = std::cmp::max(section_start, scroll_offset);
        let visible_end = std::cmp::min(section_end, scroll_offset + visible_fields);

        // 渲染段落标题（如果段落开头在可视区域内）
        if section_start >= scroll_offset {
            let title_height = 2;
            let title_area = Rect::new(area.x, current_y, area.width, title_height);

            let title_block = Block::default()
                .borders(Borders::NONE)
                .title(Span::styled(
                    section_title,
                    Style::default().fg(Color::Cyan),
                ))
                .title_alignment(Alignment::Center);

            f.render_widget(title_block, title_area);
            current_y += title_height;
        }

        // 渲染段落内的可见项目
        for item_index in visible_start..visible_end {
            if rendered_items >= visible_fields {
                break;
            }

            let item_height = 3; // 每个项目3行高度
            let item_area = Rect::new(area.x, current_y, area.width, item_height);

            let fields = if app.form_data().contains_key("global_proxy_type") {
                crate::forms::FormField::global_proxy_fields()
            } else {
                crate::forms::FormField::ssh_config_fields()
            };

            if item_index < fields.len() {
                let field = &fields[item_index];
                render_field_by_type(f, item_area, app, item_index, field.clone());
            }

            current_y += item_height;
            rendered_items += 1;
        }
    }

    // 渲染滚动条
    use crate::ui::render_scrollbar;
    render_scrollbar(f, area, &app.form_manager.scroll_manager);
}

/// 渲染基本信息区域
///
/// # 参数
/// - `f`: 绘制 Frame
/// - `area`: 区域
/// - `app`: 应用状态
fn render_basic_info_section(f: &mut Frame, area: Rect, app: &mut App) {
    let basic_block = Block::default()
        .borders(Borders::NONE)
        .title(Span::styled("基本信息", Style::default().fg(Color::Cyan)))
        .title_alignment(Alignment::Center);

    let basic_inner = basic_block.inner(area);
    f.render_widget(basic_block, area);

    let basic_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // 别名
            Constraint::Length(3), // 地址
            Constraint::Length(3), // 端口
            Constraint::Length(3), // 用户
            Constraint::Length(3), // 密钥
            Constraint::Min(0),
        ])
        .split(basic_inner);

    render_form_field(f, basic_chunks[0], "别名", "alias", app, 0);
    render_form_field(f, basic_chunks[1], "地址", "address", app, 1);
    render_form_field(f, basic_chunks[2], "端口", "port", app, 2);
    render_form_field(f, basic_chunks[3], "用户", "user", app, 3);
    render_form_field(f, basic_chunks[4], "密钥", "key", app, 4);
}

/// 渲染端口转发区域
///
/// # 参数
/// - `f`: 绘制 Frame
/// - `area`: 区域
/// - `app`: 应用状态
fn render_port_forward_section(f: &mut Frame, area: Rect, app: &mut App) {
    let pf_enabled = app
        .form_data()
        .get("pf_enabled")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false);

    let pf_block = Block::default()
        .borders(Borders::NONE)
        .title(Span::styled(
            "端口转发配置",
            Style::default().fg(Color::Cyan),
        ))
        .title_alignment(Alignment::Center);

    let pf_inner = pf_block.inner(area);
    f.render_widget(pf_block, area);

    let pf_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // 端口转发复选框
            Constraint::Length(3), // 本地端口
            Constraint::Length(3), // 远程端口
            Constraint::Min(0),
        ])
        .split(pf_inner);

    render_checkbox_field(f, pf_chunks[0], "端口转发(空格启用)", "pf_enabled", app, 5);
    render_form_field_with_enabled(
        f,
        pf_chunks[1],
        "本地端口(IP:PORT)",
        "pf_local",
        app,
        6,
        pf_enabled,
    );
    render_form_field_with_enabled(
        f,
        pf_chunks[2],
        "远程端口(IP:PORT)",
        "pf_remote",
        app,
        7,
        pf_enabled,
    );
}

/// 渲染代理配置区域
///
/// # 参数
/// - `f`: 绘制 Frame
/// - `area`: 区域
/// - `app`: 应用状态
fn render_proxy_section(f: &mut Frame, area: Rect, app: &mut App) {
    let use_global_proxy = app
        .form_data()
        .get("use_global_proxy")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(true);
    let proxy_enabled = app
        .form_data()
        .get("proxy_enabled")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false);

    let proxy_block = Block::default()
        .borders(Borders::NONE)
        .title(Span::styled("代理配置", Style::default().fg(Color::Cyan)))
        .title_alignment(Alignment::Center);

    let proxy_inner = proxy_block.inner(area);
    f.render_widget(proxy_block, area);

    if !use_global_proxy && proxy_enabled {
        // 需要显示详细的代理配置字段
        let proxy_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // 代理选项
                Constraint::Length(3), // 代理主机
                Constraint::Length(3), // 代理端口
                Constraint::Length(3), // 代理用户名
                Constraint::Length(3), // 代理密码
                Constraint::Min(0),
            ])
            .split(proxy_inner);

        render_proxy_option_field(f, proxy_chunks[0], app, 8);
        render_form_field(f, proxy_chunks[1], "代理主机", "proxy_host", app, 9);
        render_form_field(f, proxy_chunks[2], "代理端口", "proxy_port", app, 10);
        render_form_field(f, proxy_chunks[3], "代理用户", "proxy_username", app, 11);
        render_form_field(f, proxy_chunks[4], "代理密码", "proxy_password", app, 12);
    } else {
        // 只显示代理选项
        let proxy_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // 代理选项
                Constraint::Min(1),    // 剩余空间
            ])
            .split(proxy_inner);

        render_proxy_option_field(f, proxy_chunks[0], app, 8);

        // 在剩余空间显示提示信息
        let hint_text = if use_global_proxy {
            "使用全局代理配置，可在主菜单中配置"
        } else {
            "未启用代理"
        };

        let hint = Paragraph::new(hint_text)
            .style(Style::default().fg(Color::Green))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        f.render_widget(hint, proxy_chunks[1]);
    }
}

/// 渲染单个表单字段（通用文本输入）
///
/// # 参数
/// - `f`: 绘制 Frame
/// - `area`: 区域
/// - `label`: 字段标签
/// - `field_name`: 字段名
/// - `app`: 应用状态
/// - `field_index`: 字段索引
fn render_form_field(
    f: &mut Frame,
    area: Rect,
    label: &str,
    field_name: &str,
    app: &mut App,
    field_index: usize,
) {
    let is_focused = app.current_field() == field_index;
    let value = app.form_data().get(field_name).cloned().unwrap_or_default();
    // 改进密码掩码显示，聚焦时显示最后一个字符
    let display_value = if field_name.contains("password") {
        if is_focused && !value.is_empty() {
            // 显示前面的字符为星号，最后一个字符显示明文
            value
        } else {
            // 非聚焦或空字符串时全部显示为星号
            "*".repeat(value.len())
        }
    } else {
        value
    };
    let input = Paragraph::new(display_value).block(
        Block::default()
            .borders(Borders::ALL)
            .title(label)
            .border_style(if is_focused {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            }),
    );

    f.render_widget(input, area);

    if is_focused {
        f.set_cursor(area.x + app.cursor_position() as u16 + 1, area.y + 1);
    }
}

/// 渲染可禁用的表单字段（如端口转发相关字段）
///
/// # 参数
/// - `f`: 绘制 Frame
/// - `area`: 区域
/// - `label`: 字段标签
/// - `field_name`: 字段名
/// - `app`: 应用状态
/// - `field_index`: 字段索引
/// - `enabled`: 是否可用
fn render_form_field_with_enabled(
    f: &mut Frame,
    area: Rect,
    label: &str,
    field_name: &str,
    app: &mut App,
    field_index: usize,
    enabled: bool,
) {
    let is_focused = app.current_field() == field_index && enabled;
    let value = app.form_data().get(field_name).cloned().unwrap_or_default();

    let style = if enabled {
        Style::default()
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let border_style = if enabled {
        if is_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        }
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let input = Paragraph::new(value).style(style).block(
        Block::default()
            .borders(Borders::ALL)
            .title(label)
            .border_style(border_style),
    );

    f.render_widget(input, area);

    if is_focused && enabled {
        f.set_cursor(area.x + app.cursor_position() as u16 + 1, area.y + 1);
    }
}

/// 渲染复选框字段（如启用端口转发）
///
/// # 参数
/// - `f`: 绘制 Frame
/// - `area`: 区域
/// - `label`: 字段标签
/// - `field_name`: 字段名
/// - `app`: 应用状态
/// - `field_index`: 字段索引
fn render_checkbox_field(
    f: &mut Frame,
    area: Rect,
    label: &str,
    field_name: &str,
    app: &mut App,
    field_index: usize,
) {
    let is_focused = app.current_field() == field_index;
    let checked = app
        .form_data()
        .get(field_name)
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false);

    let checkbox_text = if checked { "[✓]" } else { "[ ]" };
    let display_text = format!("{checkbox_text} {label}");

    let checkbox =
        Paragraph::new(display_text).block(Block::default().borders(Borders::ALL).border_style(
            if is_focused {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            },
        ));

    f.render_widget(checkbox, area);
}

/// 渲染代理选项字段（如全局代理、SOCKS5、HTTP等）
///
/// # 参数
/// - `f`: 绘制 Frame
/// - `area`: 区域
/// - `app`: 应用状态
/// - `field_index`: 字段索引
fn render_proxy_option_field(f: &mut Frame, area: Rect, app: &mut App, field_index: usize) {
    let is_focused = app.current_field() == field_index;
    let use_global_proxy = app
        .form_data()
        .get("use_global_proxy")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(true);
    let proxy_enabled = app
        .form_data()
        .get("proxy_enabled")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false);
    let proxy_type = app
        .form_data()
        .get("proxy_type")
        .map(|t| t.as_str())
        .unwrap_or("None");

    let option_text = if use_global_proxy {
        "全局代理"
    } else if !proxy_enabled {
        "不使用代理"
    } else {
        match proxy_type {
            "Socks5" => "SOCKS5代理",
            "Http" => "HTTP代理",
            _ => "不使用代理",
        }
    };

    let proxy_field = Paragraph::new(option_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title("代理选项 (空格切换)")
            .border_style(if is_focused {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            }),
    );

    f.render_widget(proxy_field, area);
}

/// 渲染完整表单（无需滚动）
///
/// # 参数
/// - `f`: 绘制 Frame
/// - `area`: 绘制区域
/// - `app`: 应用状态
fn render_full_form(f: &mut Frame, area: Rect, app: &mut App) {
    // 分割为三个主要区域：基本信息、端口转发、代理配置
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(17), // 基本信息区域 (5个字段 + 2个空行)
            Constraint::Length(12), // 端口转发区域 (3个字段)
            Constraint::Min(25),    // 代理配置区域(动态字段)
            Constraint::Min(0),
        ])
        .split(area);

    // 渲染基本信息区域
    render_basic_info_section(f, main_chunks[0], app);

    // 渲染端口转发区域
    render_port_forward_section(f, main_chunks[1], app);

    // 渲染代理配置区域
    render_proxy_section(f, main_chunks[2], app);
}

/// 根据字段类型渲染字段
///
/// # 参数
/// - `f`: 绘制 Frame
/// - `area`: 绘制区域
/// - `app`: 应用状态
/// - `field_index`: 字段索引
/// - `field`: 字段类型
fn render_field_by_type(
    f: &mut Frame,
    area: Rect,
    app: &mut App,
    field_index: usize,
    field: crate::forms::FormField,
) {
    let field_name = field.as_str();
    let label = get_field_label(&field);

    match field {
        crate::forms::FormField::PortForwardEnabled => {
            render_checkbox_field(f, area, &label, field_name, app, field_index);
        }
        crate::forms::FormField::PortForwardLocal | crate::forms::FormField::PortForwardRemote => {
            // 端口转发字段需要根据是否启用来决定是否可用
            let pf_enabled = app
                .form_data()
                .get("pf_enabled")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(false);
            render_form_field_with_enabled(
                f,
                area,
                &label,
                field_name,
                app,
                field_index,
                pf_enabled,
            );
        }
        crate::forms::FormField::UseGlobalProxy => {
            render_proxy_option_field(f, area, app, field_index);
        }
        crate::forms::FormField::ProxyHost
        | crate::forms::FormField::ProxyPort
        | crate::forms::FormField::ProxyUsername
        | crate::forms::FormField::ProxyPassword => {
            // 代理字段只有在不使用全局代理且启用代理时才显示
            let use_global_proxy = app
                .form_data()
                .get("use_global_proxy")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(true);
            let proxy_enabled = app
                .form_data()
                .get("proxy_enabled")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(false);

            if !use_global_proxy && proxy_enabled {
                render_form_field(f, area, &label, field_name, app, field_index);
            }
        }
        _ => {
            render_form_field(f, area, &label, field_name, app, field_index);
        }
    }
}

/// 获取字段标签
///
/// # 参数
/// - `field`: 字段类型
///
/// # 返回
/// 返回字段的显示标签
fn get_field_label(field: &crate::forms::FormField) -> String {
    match field {
        crate::forms::FormField::Alias => "别名".to_string(),
        crate::forms::FormField::Address => "地址".to_string(),
        crate::forms::FormField::Port => "端口".to_string(),
        crate::forms::FormField::User => "用户".to_string(),
        crate::forms::FormField::Key => "密钥".to_string(),
        crate::forms::FormField::PortForwardEnabled => "端口转发".to_string(),
        crate::forms::FormField::PortForwardLocal => "本地端口".to_string(),
        crate::forms::FormField::PortForwardRemote => "远程端口".to_string(),
        crate::forms::FormField::UseGlobalProxy => "代理设置".to_string(),
        crate::forms::FormField::ProxyHost => "代理主机".to_string(),
        crate::forms::FormField::ProxyPort => "代理端口".to_string(),
        crate::forms::FormField::ProxyUsername => "代理用户名".to_string(),
        crate::forms::FormField::ProxyPassword => "代理密码".to_string(),
        crate::forms::FormField::GlobalProxyType => "代理类型".to_string(),
        crate::forms::FormField::GlobalProxyHost => "代理主机".to_string(),
        crate::forms::FormField::GlobalProxyPort => "代理端口".to_string(),
        crate::forms::FormField::GlobalProxyUsername => "代理用户名".to_string(),
        crate::forms::FormField::GlobalProxyPassword => "代理密码".to_string(),
    }
}
