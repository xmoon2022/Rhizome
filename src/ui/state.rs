//! App 状态定义 (Model)
//!
//! 包含应用状态结构体及相关枚举

use crate::models::{FocusNode, FocusTree};

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

/// 应用模式
#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Normal,
    AddingNode,
    EditingContent(String), // String is the node ID being edited
    EditingTitle(String),   // String is the node ID being edited
    MovingNode(String),     // String is the node ID to move
    Confirm(ConfirmAction),
}

/// 确认操作类型
#[derive(Debug, Clone, PartialEq)]
pub enum ConfirmAction {
    Delete(String),
    Fail(String),
}

/// 输入字段类型
#[derive(Debug, Clone, PartialEq)]
pub enum InputField {
    Title,
    Content,
}

impl App {
    /// 创建新的应用实例
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

    /// 刷新显示列表
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

    /// 获取当前选中的节点
    pub fn selected_node(&self) -> Option<&FocusNode> {
        self.display_list
            .get(self.selected_index)
            .and_then(|(_, id)| self.tree.nodes.get(id))
    }

    /// 获取当前选中的节点 ID
    pub fn selected_node_id(&self) -> Option<String> {
        self.display_list
            .get(self.selected_index)
            .map(|(_, id)| id.clone())
    }
}
