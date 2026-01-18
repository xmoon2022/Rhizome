use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// 节点状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum NodeStatus {
    #[default]
    Active, // 活跃状态
    Failed,    // 失败状态
    Completed, // 已完成（内化为习惯）
}

/// 国策节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusNode {
    pub id: String,
    #[serde(default)]
    pub parent_id: String, // 空字符串表示根节点
    pub title: String,
    #[serde(default)]
    pub content: String,
    pub created_at: DateTime<Local>,
    #[serde(default)]
    pub status: NodeStatus,
    #[serde(default)]
    pub streak_days: u32,
}

impl FocusNode {
    pub fn new(title: String, content: String, parent_id: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            parent_id: parent_id.unwrap_or_default(),
            title,
            content,
            created_at: Local::now(),
            status: NodeStatus::Active,
            streak_days: 0,
        }
    }

    pub fn is_root(&self) -> bool {
        self.parent_id.is_empty()
    }

    pub fn days_active(&self) -> i64 {
        let duration = Local::now() - self.created_at;
        duration.num_days().max(0)
    }
}

/// TOML文件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusTreeData {
    pub meta: TreeMeta,
    pub nodes: Vec<FocusNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeMeta {
    pub version: String,
    pub created_at: DateTime<Local>,
    pub last_modified: DateTime<Local>,
}

impl Default for FocusTreeData {
    fn default() -> Self {
        let now = Local::now();
        Self {
            meta: TreeMeta {
                version: "1.0".to_string(),
                created_at: now,
                last_modified: now,
            },
            nodes: Vec::new(),
        }
    }
}

/// 运行时树结构（用于高效操作）
#[derive(Debug, Clone)]
pub struct FocusTree {
    pub nodes: HashMap<String, FocusNode>,
    pub root_ids: Vec<String>,
    pub children_map: HashMap<String, Vec<String>>, // parent_id -> child_ids
}

impl FocusTree {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            root_ids: Vec::new(),
            children_map: HashMap::new(),
        }
    }

    pub fn from_data(data: FocusTreeData) -> Self {
        let mut tree = Self::new();
        for node in data.nodes {
            tree.insert_node(node);
        }
        tree
    }

    pub fn to_data(&self) -> FocusTreeData {
        let nodes: Vec<FocusNode> = self.nodes.values().cloned().collect();
        let now = Local::now();
        FocusTreeData {
            meta: TreeMeta {
                version: "1.0".to_string(),
                created_at: now, // TODO: preserve original
                last_modified: now,
            },
            nodes,
        }
    }

    fn insert_node(&mut self, node: FocusNode) {
        let id = node.id.clone();
        let parent_id = node.parent_id.clone();

        if node.is_root() {
            self.root_ids.push(id.clone());
        } else {
            self.children_map
                .entry(parent_id)
                .or_default()
                .push(id.clone());
        }

        self.nodes.insert(id, node);
    }

    /// 添加新节点
    pub fn add_node(
        &mut self,
        title: String,
        content: String,
        parent_id: Option<String>,
    ) -> String {
        let node = FocusNode::new(title, content, parent_id);
        let id = node.id.clone();
        self.insert_node(node);
        id
    }

    /// 获取节点的所有子节点ID（递归）
    pub fn get_all_descendants(&self, node_id: &str) -> Vec<String> {
        let mut descendants = Vec::new();
        let mut stack = vec![node_id.to_string()];

        while let Some(current_id) = stack.pop() {
            if let Some(children) = self.children_map.get(&current_id) {
                for child_id in children {
                    descendants.push(child_id.clone());
                    stack.push(child_id.clone());
                }
            }
        }

        descendants
    }

    /// 删除节点及其所有子节点（堆栈式删除）
    pub fn delete_node(&mut self, node_id: &str) -> Vec<String> {
        let mut deleted = vec![node_id.to_string()];
        deleted.extend(self.get_all_descendants(node_id));

        for id in &deleted {
            if let Some(node) = self.nodes.remove(id) {
                // 从父节点的children_map中移除
                if node.is_root() {
                    self.root_ids.retain(|x| x != id);
                } else if let Some(siblings) = self.children_map.get_mut(&node.parent_id) {
                    siblings.retain(|x| x != id);
                }
                // 移除自己的children_map条目
                self.children_map.remove(id);
            }
        }

        deleted
    }

    /// 标记节点失败并级联删除所有子节点
    pub fn fail_node(&mut self, node_id: &str) -> Vec<String> {
        // 标记为失败
        if let Some(node) = self.nodes.get_mut(node_id) {
            node.status = NodeStatus::Failed;
        }
        // 删除所有子节点
        self.get_all_descendants(node_id).iter().for_each(|id| {
            self.nodes.remove(id);
        });

        // 返回被删除的子节点
        let deleted = self.get_all_descendants(node_id);
        for id in &deleted {
            self.children_map.remove(id);
        }
        deleted
    }

    /// 获取直接子节点
    #[allow(dead_code)]
    pub fn get_children(&self, node_id: &str) -> Vec<&FocusNode> {
        self.children_map
            .get(node_id)
            .map(|ids| ids.iter().filter_map(|id| self.nodes.get(id)).collect())
            .unwrap_or_default()
    }

    /// 获取根节点
    #[allow(dead_code)]
    pub fn get_roots(&self) -> Vec<&FocusNode> {
        self.root_ids
            .iter()
            .filter_map(|id| self.nodes.get(id))
            .collect()
    }

    /// 生成展开的节点列表（用于TUI显示）
    pub fn flatten_for_display(&self) -> Vec<(usize, &FocusNode)> {
        let mut result = Vec::new();

        fn traverse<'a>(
            tree: &'a FocusTree,
            node_id: &str,
            depth: usize,
            result: &mut Vec<(usize, &'a FocusNode)>,
        ) {
            if let Some(node) = tree.nodes.get(node_id) {
                result.push((depth, node));
                if let Some(children) = tree.children_map.get(node_id) {
                    for child_id in children {
                        traverse(tree, child_id, depth + 1, result);
                    }
                }
            }
        }

        for root_id in &self.root_ids {
            traverse(self, root_id, 0, &mut result);
        }

        result
    }
}

impl Default for FocusTree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_node() {
        let mut tree = FocusTree::new();
        let root_id = tree.add_node("Root".to_string(), "Root content".to_string(), None);
        let _child_id = tree.add_node(
            "Child".to_string(),
            "Child content".to_string(),
            Some(root_id.clone()),
        );

        assert_eq!(tree.nodes.len(), 2);
        assert_eq!(tree.root_ids.len(), 1);
        assert_eq!(tree.get_children(&root_id).len(), 1);
        assert_eq!(tree.get_children(&root_id)[0].title, "Child");
    }

    #[test]
    fn test_delete_cascade() {
        let mut tree = FocusTree::new();
        let root_id = tree.add_node("Root".to_string(), "".to_string(), None);
        let child_id = tree.add_node("Child".to_string(), "".to_string(), Some(root_id.clone()));
        let _grandchild_id = tree.add_node(
            "Grandchild".to_string(),
            "".to_string(),
            Some(child_id.clone()),
        );

        assert_eq!(tree.nodes.len(), 3);

        let deleted = tree.delete_node(&child_id);
        assert_eq!(deleted.len(), 2); // child + grandchild
        assert_eq!(tree.nodes.len(), 1); // only root remains
        assert!(tree.nodes.contains_key(&root_id));
    }

    #[test]
    fn test_days_active() {
        use chrono::Duration;

        let mut node = FocusNode::new("Test".to_string(), "".to_string(), None);
        assert_eq!(node.days_active(), 0);

        node.created_at = Local::now() - Duration::days(5);
        assert_eq!(node.days_active(), 5);
    }
}
