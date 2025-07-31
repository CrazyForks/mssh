mod app;
mod config;
mod events;
mod form_manager;
mod forms;
mod message_manager;
mod navigation_manager;
mod proxy;
mod ssh;
mod ui;

use app::App;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use events::EventHandler;
use ratatui::{prelude::CrosstermBackend, Terminal};
use std::io;

/// SSH Manager 主程序入口
///
/// # 返回
/// 返回 Result，成功为 Ok(())，失败为 Err
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 解析命令行参数
    let args: Vec<String> = std::env::args().collect();
    let mut config_path = None;
    let mut import_ssh = false;
    let mut quick_connect = None;

    for (i, arg) in args.iter().enumerate() {
        if arg == "--config" || arg == "-c" {
            if i + 1 < args.len() {
                config_path = Some(args[i + 1].clone());
            } else {
                eprintln!("错误: --config 参数需要指定配置文件路径");
                std::process::exit(1);
            }
        } else if arg == "--import-ssh" {
            import_ssh = true;
        } else if arg == "-C" {
            if i + 1 < args.len() {
                quick_connect = Some(args[i + 1].clone());
            } else {
                eprintln!("错误: -C 参数需要指定编号或 Host 别名");
                std::process::exit(1);
            }
        } else if arg == "--help" || arg == "-h" {
            println!("SSH Manager - SSH 配置管理工具");
            println!();
            println!("用法: mssh [选项] [编号或别名]");
            println!();
            println!("选项:");
            println!("  -c, --config <路径>     指定配置文件路径");
            println!("  --import-ssh           显示 SSH 配置导入界面");
            println!("  -C, <目标>              快速连接到指定配置");
            println!("  -h, --help             显示帮助信息");
            println!();
            println!("快速连接示例:");
            println!("  mssh 1                    # 连接到编号为 1 的配置");
            println!("  mssh test-server          # 连接到别名为 test-server 的配置");
            println!("  mssh -C 1                 # 连接到编号为 1 的配置");
            println!("  mssh -C test-server       # 连接到别名为 test-server 的配置");
            println!();
            println!("其他示例:");
            println!("  mssh                      # 启动 TUI 界面");
            println!("  mssh -c ~/my-config.toml  # 使用指定配置文件");
            println!("  mssh --import-ssh         # 启动时导入 SSH 配置");
            std::process::exit(0);
        }
    }

    // 设置终端
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;

    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        execute!(io::stdout(), LeaveAlternateScreen).unwrap();
        disable_raw_mode().unwrap();
        original_hook(panic_info);
    }));

    // 创建应用
    let mut app = App::new(config_path)?;

    // 处理快速连接
    if let Some(target) = quick_connect {
        if let Err(e) = app.quick_connect(&target) {
            eprintln!("连接失败: {e}");
            std::process::exit(1);
        }
        return Ok(());
    }

    // 处理无参数快速连接（第一个参数作为目标）
    if args.len() > 1 && !args[1].starts_with('-') {
        let target = &args[1];
        if let Err(e) = app.quick_connect(target) {
            eprintln!("连接失败: {e}");
            std::process::exit(1);
        }
        return Ok(());
    }

    if import_ssh {
        if let Err(e) = app.show_import_selection() {
            eprintln!("显示导入选择失败: {e}");
            std::process::exit(1);
        }
        // return Ok(());
    }

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal, &mut app);

    // 恢复终端
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

/// 运行应用程序主循环
///
/// # 参数
/// - `terminal`: 终端实例
/// - `app`: 应用状态
///
/// # 返回
/// 返回 io::Result，成功为 Ok(())，失败为 Err
fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        // 检查并清理过期消息
        app.check_message();

        terminal.draw(|f| ui::ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            match EventHandler::handle_key_event(app, key) {
                Ok(true) => return Ok(()), // 退出信号
                Ok(false) => continue,     // 继续处理
                Err(e) => {
                    app.message_manager
                        .set_error_message(format!("事件处理错误: {e}"));
                }
            }
        }
    }
}
