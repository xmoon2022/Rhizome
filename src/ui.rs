use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::models::{FocusNode, FocusTree, NodeStatus};

/// 应用状态
pub struct App {
    pub tree: FocusTree,
    pub selected_index: usize,
    pub display_list: Vec<(usize, String)>, // (depth, node_id)
    pub mode: AppMode,
    pub input_buffer: String,
    pub input_field: InputField,
    pub message: Option<String>,
    pub temp_title: String, // Store title when moving to content input
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Normal,
    AddingNode,
    EditingContent(String), // String is the node ID being edited
    EditingTitle(String),   // String is the node ID being edited
    MovingNode(String),     // String is the node ID to move
    Confirm(ConfirmAction),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfirmAction {
    Delete(String),
    Fail(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputField {
    Title,
    Content,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Quit,
    MoveSelectionUp,
    MoveSelectionDown,

    // 触发特定功能
    StartAddNode,
    StartEditContent,
    StartEditTitle,
    StartMoveNode,
    StartDeleteNode,
    StartFailNode,

    // 表单/通用交互
    Cancel,      // Esc / n
    Submit,      // Enter / y / m
    Input(char), // 输入字符
    DeleteChar,  // Backspace
}

impl App {
    pub fn new(tree: FocusTree) -> Self {
        let mut app = Self {
            tree,
            selected_index: 0,
            display_list: Vec::new(),
            mode: AppMode::Normal,
            input_buffer: String::new(),
            input_field: InputField::Title,
            message: None,
            temp_title: String::new(),
        };
        app.refresh_display_list();
        app
    }

    /// 核心逻辑分发
    pub fn dispatch(&mut self, action: Action) -> bool {
        match action {
            Action::Quit => return true,
            Action::MoveSelectionUp => self.move_up(),
            Action::MoveSelectionDown => self.move_down(),

            Action::StartAddNode => self.start_add_node(),
            Action::StartEditContent => self.start_edit_content(),
            Action::StartEditTitle => self.start_edit_title(),
            Action::StartMoveNode => self.start_move_node(),
            Action::StartDeleteNode => self.start_delete_node(),
            Action::StartFailNode => self.start_fail_node(),

            Action::Cancel => self.cancel(),

            Action::Submit => match &self.mode {
                AppMode::AddingNode => match self.input_field {
                    InputField::Title => {
                        if !self.input_buffer.is_empty() {
                            self.move_to_content_input();
                        }
                    }
                    InputField::Content => self.confirm_add_node(),
                },
                AppMode::EditingContent(id) => {
                    let id = id.clone();
                    self.confirm_edit_content(id);
                }
                AppMode::EditingTitle(id) => {
                    let id = id.clone();
                    self.confirm_edit_title(id);
                }
                AppMode::MovingNode(id) => {
                    let id = id.clone();
                    self.confirm_move_node(id);
                }
                AppMode::Confirm(_) => self.execute_confirm(),
                AppMode::Normal => {}
            },

            Action::Input(c) => {
                if matches!(
                    self.mode,
                    AppMode::AddingNode | AppMode::EditingContent(_) | AppMode::EditingTitle(_)
                ) {
                    self.input_buffer.push(c);
                }
            }

            Action::DeleteChar => {
                if matches!(
                    self.mode,
                    AppMode::AddingNode | AppMode::EditingContent(_) | AppMode::EditingTitle(_)
                ) {
                    self.input_buffer.pop();
                }
            }
        }
        false
    }

    pub fn refresh_display_list(&mut self) {
        self.display_list = self
            .tree
            .flatten_for_display()
            .iter()
            .map(|(depth, node)| (*depth, node.id.clone()))
            .collect();

        // 确保选中索引有效
        if self.display_list.is_empty() {
            self.selected_index = 0;
        } else if self.selected_index >= self.display_list.len() {
            self.selected_index = self.display_list.len() - 1;
        }
    }

    pub fn selected_node(&self) -> Option<&FocusNode> {
        self.display_list
            .get(self.selected_index)
            .and_then(|(_, id)| self.tree.nodes.get(id))
    }

    pub fn selected_node_id(&self) -> Option<String> {
        self.display_list
            .get(self.selected_index)
            .map(|(_, id)| id.clone())
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.selected_index + 1 < self.display_list.len() {
            self.selected_index += 1;
        }
    }

    pub fn start_add_node(&mut self) {
        self.mode = AppMode::AddingNode;
        self.input_buffer.clear();
        self.input_field = InputField::Title;
        self.temp_title.clear();
    }

    pub fn move_to_content_input(&mut self) {
        self.temp_title = self.input_buffer.clone();
        self.input_buffer.clear();
        self.input_field = InputField::Content;
    }

    pub fn confirm_add_node(&mut self) {
        let title = self.temp_title.clone();
        let content = self.input_buffer.clone();
        let parent_id = self.selected_node_id();
        self.tree.add_node(title, content, parent_id);
        self.refresh_display_list();
        self.mode = AppMode::Normal;
        self.temp_title.clear();
        self.message = Some("节点已添加".to_string());
    }

    pub fn start_edit_content(&mut self) {
        if let Some(node) = self.selected_node() {
            let id = node.id.clone();
            let content = node.content.clone();
            self.mode = AppMode::EditingContent(id);
            self.input_buffer = content;
        }
    }

    pub fn confirm_edit_content(&mut self, node_id: String) {
        if let Some(node) = self.tree.nodes.get_mut(&node_id) {
            node.content = self.input_buffer.clone();
        }
        self.mode = AppMode::Normal;
        self.input_buffer.clear();
        self.message = Some("内容已更新".to_string());
    }

    pub fn start_edit_title(&mut self) {
        if let Some(node) = self.selected_node() {
            let id = node.id.clone();
            let title = node.title.clone();
            self.mode = AppMode::EditingTitle(id);
            self.input_buffer = title;
        }
    }

    pub fn confirm_edit_title(&mut self, node_id: String) {
        if let Some(node) = self.tree.nodes.get_mut(&node_id) {
            node.title = self.input_buffer.clone();
        }
        self.mode = AppMode::Normal;
        self.input_buffer.clear();
        self.message = Some("标题已更新".to_string());
    }

    pub fn start_move_node(&mut self) {
        if let Some(id) = self.selected_node_id() {
            self.mode = AppMode::MovingNode(id);
            self.message = Some("请选择新的父节点（或根节点），按 'm' 确认移动".to_string());
        }
    }

    pub fn confirm_move_node(&mut self, node_id: String) {
        let new_parent_id = self.selected_node_id();

        // 防止将节点移动到自己或自己的子节点下
        if let Some(new_parent) = &new_parent_id {
            if new_parent == &node_id {
                self.message = Some("不能将节点移动到自己下面".to_string());
                self.mode = AppMode::Normal;
                return;
            }
            // 检查是否是移动到自己的后代
            let descendants = self.tree.get_all_descendants(&node_id);
            if descendants.contains(new_parent) {
                self.message = Some("不能将节点移动到其子节点下".to_string());
                self.mode = AppMode::Normal;
                return;
            }
        }

        // 执行移动
        if let Some(node) = self.tree.nodes.get_mut(&node_id) {
            // 从旧父节点中移除
            if node.is_root() {
                self.tree.root_ids.retain(|id| id != &node_id);
            } else if let Some(siblings) = self.tree.children_map.get_mut(&node.parent_id) {
                siblings.retain(|id| id != &node_id);
            }

            // 更新父节点
            node.parent_id = new_parent_id.clone().unwrap_or_default();

            // 添加到新父节点
            if node.is_root() {
                self.tree.root_ids.push(node_id.clone());
            } else {
                self.tree
                    .children_map
                    .entry(node.parent_id.clone())
                    .or_default()
                    .push(node_id.clone());
            }
        }

        self.refresh_display_list();
        self.mode = AppMode::Normal;
        self.message = Some("节点已移动".to_string());
    }

    pub fn start_delete_node(&mut self) {
        if let Some(id) = self.selected_node_id() {
            self.mode = AppMode::Confirm(ConfirmAction::Delete(id));
        }
    }

    pub fn start_fail_node(&mut self) {
        if let Some(node) = self.selected_node() {
            match node.status {
                NodeStatus::Active => {
                    let id = node.id.clone();
                    self.mode = AppMode::Confirm(ConfirmAction::Fail(id));
                }
                NodeStatus::Failed => {
                    let id = node.id.clone();
                    self.tree.recover_node(&id);
                    self.message = Some("节点已恢复为活跃状态".to_string());
                }
            }
        }
    }

    pub fn execute_confirm(&mut self) {
        match &self.mode {
            AppMode::Confirm(ConfirmAction::Delete(id)) => {
                let id = id.clone();
                let deleted = self.tree.delete_node(&id);
                self.message = Some(format!("已删除 {} 个节点", deleted.len()));
            }
            AppMode::Confirm(ConfirmAction::Fail(id)) => {
                let id = id.clone();
                let deleted = self.tree.fail_node(&id);
                self.message = Some(format!("节点已标记失败，删除了 {} 个子节点", deleted.len()));
            }
            _ => {}
        }
        self.refresh_display_list();
        self.mode = AppMode::Normal;
    }

    pub fn cancel(&mut self) {
        self.mode = AppMode::Normal;
        self.input_buffer.clear();
        self.message = None;
    }
}

/// 渲染UI
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

/// [组件] 弹窗基础框架
fn render_dialog_framework(frame: &mut Frame, area: Rect, title: &str) -> Rect {
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
fn render_input_widget(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    value: &str,
    is_focused: bool,
    active_color: Color,
) {
    let style = if is_focused {
        Style::default().fg(active_color).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };

    let input = Paragraph::new(value)
        .style(style)
        .wrap(Wrap { trim: false })
        .block(Block::default().title(title).borders(Borders::ALL));
    frame.render_widget(input, area);
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
    let title_val = if is_title_active { &app.input_buffer } else { &app.temp_title };
    render_input_widget(frame, chunks[0], "标题", title_val, is_title_active, Color::Yellow);

    // 内容输入
    let is_content_active = app.input_field == InputField::Content;
    let content_val = if is_content_active { &app.input_buffer } else { "" };
    render_input_widget(frame, chunks[1], "内容 (可选)", content_val, is_content_active, Color::Yellow);

    let hint = match app.input_field {
        InputField::Title => "输入标题后按 Enter 继续",
        InputField::Content => "输入内容后按 Enter 完成（可留空）",
    };
    frame.render_widget(Paragraph::new(hint).style(Style::default().fg(Color::Gray)), chunks[2]);
}

fn render_edit_content_dialog(frame: &mut Frame, app: &App) {
    let area = centered_rect(70, 30, frame.area());
    let inner = render_dialog_framework(frame, area, "编辑内容");

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(inner);

    render_input_widget(frame, chunks[0], "内容", &app.input_buffer, true, Color::Yellow);
    
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

    render_input_widget(frame, chunks[0], "标题", &app.input_buffer, true, Color::Yellow);

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

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// 根据当前模式和按键获取对应的 Action
fn get_action(mode: &AppMode, key: KeyCode) -> Option<Action> {
    match mode {
        AppMode::Normal => match key {
            KeyCode::Char('q') => Some(Action::Quit),
            KeyCode::Char('j') | KeyCode::Down => Some(Action::MoveSelectionDown),
            KeyCode::Char('k') | KeyCode::Up => Some(Action::MoveSelectionUp),
            KeyCode::Char('a') => Some(Action::StartAddNode),
            KeyCode::Char('e') => Some(Action::StartEditContent),
            KeyCode::Char('r') => Some(Action::StartEditTitle),
            KeyCode::Char('m') => Some(Action::StartMoveNode),
            KeyCode::Char('d') => Some(Action::StartDeleteNode),
            KeyCode::Char('f') => Some(Action::StartFailNode),
            _ => None,
        },
        AppMode::AddingNode | AppMode::EditingContent(_) | AppMode::EditingTitle(_) => match key {
            KeyCode::Esc => Some(Action::Cancel),
            KeyCode::Enter => Some(Action::Submit),
            KeyCode::Backspace => Some(Action::DeleteChar),
            KeyCode::Char(c) => Some(Action::Input(c)),
            _ => None,
        },
        AppMode::MovingNode(_) => match key {
            KeyCode::Esc => Some(Action::Cancel),
            KeyCode::Char('m') | KeyCode::Char('M') => Some(Action::Submit),
            KeyCode::Char('j') | KeyCode::Down => Some(Action::MoveSelectionDown),
            KeyCode::Char('k') | KeyCode::Up => Some(Action::MoveSelectionUp),
            _ => None,
        },
        AppMode::Confirm(_) => match key {
            KeyCode::Char('y') | KeyCode::Char('Y') => Some(Action::Submit),
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Some(Action::Cancel),
            _ => None,
        },
    }
}

/// 处理按键事件
pub fn handle_key_event(app: &mut App, key: KeyCode) -> io::Result<bool> {
    if let Some(action) = get_action(&app.mode, key) {
        Ok(app.dispatch(action))
    } else {
        Ok(false)
    }
}

/// 运行事件循环
#[allow(dead_code)]
pub fn run_event_loop(app: &mut App) -> io::Result<()> {
    loop {
        if let Event::Key(event::KeyEvent {
            kind: KeyEventKind::Press,
            code,
            ..
        }) = event::read()?
            && handle_key_event(app, code)?
        {
            break;
        }
    }
    Ok(())
}
