//! Action 枚举定义 (Intent)
//!
//! 用户交互转化为明确的语义化 Action

/// 用户操作枚举
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
