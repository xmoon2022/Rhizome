//! 业务逻辑处理 (Update/Dispatch)
//!
//! 包含核心的 dispatch 逻辑和各种业务处理方法

use super::actions::Action;
use super::state::{App, AppMode, ConfirmAction, InputField};
use crate::models::NodeStatus;

impl App {
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

    // ============ 导航相关 ============

    /// 向上移动选择
    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// 向下移动选择
    pub fn move_down(&mut self) {
        if self.selected_index + 1 < self.display_list.len() {
            self.selected_index += 1;
        }
    }

    // ============ 添加节点相关 ============

    /// 开始添加节点
    pub fn start_add_node(&mut self) {
        self.mode = AppMode::AddingNode;
        self.input_buffer.clear();
        self.input_field = InputField::Title;
        self.temp_title.clear();
    }

    /// 切换到内容输入
    pub fn move_to_content_input(&mut self) {
        self.temp_title = self.input_buffer.clone();
        self.input_buffer.clear();
        self.input_field = InputField::Content;
    }

    /// 确认添加节点
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

    // ============ 编辑内容相关 ============

    /// 开始编辑内容
    pub fn start_edit_content(&mut self) {
        if let Some(node) = self.selected_node() {
            let id = node.id.clone();
            let content = node.content.clone();
            self.mode = AppMode::EditingContent(id);
            self.input_buffer = content;
        }
    }

    /// 确认编辑内容
    pub fn confirm_edit_content(&mut self, node_id: String) {
        if let Some(node) = self.tree.nodes.get_mut(&node_id) {
            node.content = self.input_buffer.clone();
        }
        self.mode = AppMode::Normal;
        self.input_buffer.clear();
        self.message = Some("内容已更新".to_string());
    }

    // ============ 编辑标题相关 ============

    /// 开始编辑标题
    pub fn start_edit_title(&mut self) {
        if let Some(node) = self.selected_node() {
            let id = node.id.clone();
            let title = node.title.clone();
            self.mode = AppMode::EditingTitle(id);
            self.input_buffer = title;
        }
    }

    /// 确认编辑标题
    pub fn confirm_edit_title(&mut self, node_id: String) {
        if let Some(node) = self.tree.nodes.get_mut(&node_id) {
            node.title = self.input_buffer.clone();
        }
        self.mode = AppMode::Normal;
        self.input_buffer.clear();
        self.message = Some("标题已更新".to_string());
    }

    // ============ 移动节点相关 ============

    /// 开始移动节点
    pub fn start_move_node(&mut self) {
        if let Some(id) = self.selected_node_id() {
            self.mode = AppMode::MovingNode(id);
            self.message = Some("请选择新的父节点（或根节点），按 'm' 确认移动".to_string());
        }
    }

    /// 确认移动节点
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

    // ============ 删除/失败节点相关 ============

    /// 开始删除节点
    pub fn start_delete_node(&mut self) {
        if let Some(id) = self.selected_node_id() {
            self.mode = AppMode::Confirm(ConfirmAction::Delete(id));
        }
    }

    /// 开始标记节点失败
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

    /// 执行确认操作
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

    // ============ 通用操作 ============

    /// 取消当前操作
    pub fn cancel(&mut self) {
        self.mode = AppMode::Normal;
        self.input_buffer.clear();
        self.message = None;
    }
}
