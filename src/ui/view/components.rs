//! 通用 UI 组件
//!
//! 对话框、输入框等通用组件

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

/// [组件] 弹窗基础框架
pub fn render_dialog_framework(frame: &mut Frame, area: Rect, title: &str) -> Rect {
    frame.render_widget(Clear, area);
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    inner
}

/// [组件] 带有标题和样式的输入框
pub fn render_input_widget(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    value: &str,
    is_focused: bool,
    active_color: Color,
) {
    let style = if is_focused {
        Style::default()
            .fg(active_color)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };

    let input = Paragraph::new(value)
        .style(style)
        .wrap(Wrap { trim: false })
        .block(Block::default().title(title).borders(Borders::ALL));
    frame.render_widget(input, area);
}
