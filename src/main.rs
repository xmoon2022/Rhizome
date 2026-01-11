mod models;
mod storage;
mod ui;

use std::fs;
use std::io;
use std::path::PathBuf;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::prelude::*;

use crate::storage::{load_tree, save_tree};
use crate::ui::{App, render};

/// 获取数据目录路径 (~/.local/share/rhizome/)
fn get_data_dir() -> io::Result<PathBuf> {
    let data_dir = dirs::data_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "无法获取用户数据目录"))?
        .join("rhizome");

    fs::create_dir_all(&data_dir)?;

    Ok(data_dir)
}

fn main() -> io::Result<()> {
    // 数据文件路径 (~/.local/share/rhizome/data.toml)
    let data_path = get_data_dir()?.join("data.toml");

    // 加载树
    let tree = load_tree(&data_path)?;

    // 创建应用状态
    let mut app = App::new(tree);

    // 设置终端
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 主循环
    let result = run_app(&mut terminal, &mut app);

    // 恢复终端
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // 保存数据
    save_tree(&app.tree, &data_path)?;
    println!("数据已保存到 {}", data_path.display());

    result
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| render(f, app))?;

        if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
            if key.kind == crossterm::event::KeyEventKind::Press {
                if ui::handle_key_event(app, key.code)? {
                    break;
                }
            }
        }
    }
    Ok(())
}
