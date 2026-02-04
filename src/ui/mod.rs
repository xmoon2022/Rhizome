//! UI 模块
//!
//! 采用 MVI (Model-View-Intent) 架构：
//! - Model (state.rs): App 结构体及其状态数据
//! - View (view/): 纯函数，将 State 映射为 UI
//! - Intent (actions.rs): 用户交互转化为明确的语义化 Action

pub mod actions;
pub mod input;
pub mod logic;
pub mod state;
pub mod view;

// Re-export for convenience
pub use input::handle_key_event;
pub use state::App;
pub use view::render;
