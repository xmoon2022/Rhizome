//! 视图层模块
//!
//! 包含主渲染入口和各种视图组件

pub mod components;
pub mod layouts;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};

use super::state::{App, AppMode, ConfirmAction, InputField};
use crate::models::NodeStatus;
use components::{render_dialog_framework, render_input_widget};
use layouts::centered_rect;

/// 渲染 UI
pub fn render(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // 标题
            Constraint::Min(10),   // 树
            Constraint::Length(6), // 详情
            Constraint::Length(3), // 帮助
        ])
        .split(frame.area());

    render_title(frame, chunks[0]);
    render_tree(frame, app, chunks[1]);
    render_details(frame, app, chunks[2]);
    render_help(frame, app, chunks[3]);

    // 渲染弹窗
    match &app.mode {
        AppMode::AddingNode => render_add_dialog(frame, app),
        AppMode::EditingContent(_) => render_edit_content_dialog(frame, app),
        AppMode::EditingTitle(_) => render_edit_title_dialog(frame, app),
        AppMode::MovingNode(_) => {} // 移动模式下不需要额外弹窗，使用底部提示
        AppMode::Confirm(action) => render_confirm_dialog(frame, action),
        _ => {}
    }
}

fn render_title(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new("🌳 RSIP 国策树")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, area);
}

fn render_tree(frame: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .display_list
        .iter()
        .enumerate()
        .map(|(i, (depth, id))| {
            let node = app.tree.nodes.get(id).unwrap();
            let indent = "  ".repeat(*depth);
            let prefix = if *depth == 0 { "📋 " } else { "├── " };

            let status_icon = match node.status {
                NodeStatus::Active => "●",
                NodeStatus::Failed => "✗",
            };

            let status_color = match node.status {
                NodeStatus::Active => Color::Green,
                NodeStatus::Failed => Color::Red,
            };

            let content = format!(
                "{}{}{} ({} 天) [{}]",
                indent,
                prefix,
                node.title,
                node.days_active(),
                status_icon
            );

            let style = if i == app.selected_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD | Modifier::REVERSED)
            } else {
                Style::default().fg(status_color)
            };

            ListItem::new(Line::from(vec![Span::styled(content, style)]))
        })
        .collect();

    let tree_widget = List::new(items)
        .block(Block::default().title("节点列表").borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    let mut state = ListState::default();
    state.select(Some(app.selected_index));

    frame.render_stateful_widget(tree_widget, area, &mut state);
}

fn render_details(frame: &mut Frame, app: &App, area: Rect) {
    let content = if let Some(node) = app.selected_node() {
        format!(
            "标题: {}\n创建于: {}  连续: {} 天  状态: {:?}\n规则: {}",
            node.title,
            node.created_at.format("%Y-%m-%d %H:%M"),
            node.days_active(),
            node.status,
            if node.content.is_empty() {
                "(无)"
            } else {
                &node.content
            }
        )
    } else {
        "暂无节点，按 'a' 添加第一个国策".to_string()
    };

    let details = Paragraph::new(content)
        .block(Block::default().title("详情").borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    frame.render_widget(details, area);
}

fn render_help(frame: &mut Frame, app: &App, area: Rect) {
    let help_text = match &app.mode {
        AppMode::Normal => {
            "[a] 添加  [e] 编辑  [r] 重命名  [m] 移动  [d] 删除  [f] 失败/激活  [j/k] 导航  [q] 退出"
        }
        AppMode::AddingNode => match app.input_field {
            InputField::Title => "输入标题后按 [Enter] 继续  [Esc] 取消",
            InputField::Content => "输入内容后按 [Enter] 完成  [Esc] 取消",
        },
        AppMode::EditingContent(_) => "[Enter] 保存  [Esc] 取消",
        AppMode::EditingTitle(_) => "[Enter] 保存  [Esc] 取消",
        AppMode::MovingNode(_) => "[j/k] 选择目标位置  [m] 确认移动  [Esc] 取消",
        AppMode::Confirm(_) => "[y] 确认  [n] 取消",
    };

    let message = app.message.as_deref().unwrap_or("");
    let text = if message.is_empty() {
        help_text.to_string()
    } else {
        format!("{}  |  {}", help_text, message)
    };

    let help = Paragraph::new(text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(help, area);
}

fn render_add_dialog(frame: &mut Frame, app: &App) {
    let area = centered_rect(60, 50, frame.area());
    let inner = render_dialog_framework(frame, area, "添加新国策");

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Min(1),
        ])
        .split(inner);

    // 标题输入
    let is_title_active = app.input_field == InputField::Title;
    let title_val = if is_title_active {
        &app.input_buffer
    } else {
        &app.temp_title
    };
    render_input_widget(
        frame,
        chunks[0],
        "标题",
        title_val,
        is_title_active,
        Color::Yellow,
    );

    // 内容输入
    let is_content_active = app.input_field == InputField::Content;
    let content_val = if is_content_active {
        &app.input_buffer
    } else {
        ""
    };
    render_input_widget(
        frame,
        chunks[1],
        "内容 (可选)",
        content_val,
        is_content_active,
        Color::Yellow,
    );

    let hint = match app.input_field {
        InputField::Title => "输入标题后按 Enter 继续",
        InputField::Content => "输入内容后按 Enter 完成（可留空）",
    };
    frame.render_widget(
        Paragraph::new(hint).style(Style::default().fg(Color::Gray)),
        chunks[2],
    );
}

fn render_edit_content_dialog(frame: &mut Frame, app: &App) {
    let area = centered_rect(70, 30, frame.area());
    let inner = render_dialog_framework(frame, area, "编辑内容");

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(inner);

    render_input_widget(
        frame,
        chunks[0],
        "内容",
        &app.input_buffer,
        true,
        Color::Yellow,
    );

    let hint = Paragraph::new("按 Enter 保存，Esc 取消").style(Style::default().fg(Color::Gray));
    frame.render_widget(hint, chunks[1]);
}

fn render_edit_title_dialog(frame: &mut Frame, app: &App) {
    let area = centered_rect(70, 30, frame.area());
    let inner = render_dialog_framework(frame, area, "编辑标题");

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(inner);

    render_input_widget(
        frame,
        chunks[0],
        "标题",
        &app.input_buffer,
        true,
        Color::Yellow,
    );

    let hint = Paragraph::new("按 Enter 保存，Esc 取消").style(Style::default().fg(Color::Gray));
    frame.render_widget(hint, chunks[1]);
}

fn render_confirm_dialog(frame: &mut Frame, action: &ConfirmAction) {
    let area = centered_rect(50, 20, frame.area());
    frame.render_widget(Clear, area);

    let message = match action {
        ConfirmAction::Delete(_) => "确认删除该节点及其所有子节点？",
        ConfirmAction::Fail(_) => "确认标记该节点为失败并删除所有子节点？",
    };

    let dialog = Paragraph::new(format!("{}\n\n[y] 确认  [n] 取消", message))
        .style(Style::default().fg(Color::Red))
        .block(Block::default().title("⚠️ 确认操作").borders(Borders::ALL));

    frame.render_widget(dialog, area);
}
