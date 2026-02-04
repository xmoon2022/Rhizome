# UI 模块重构计划文档

## 1. 🎯 需求解构与架构设计 (Blueprint)

*   **核心目标**：将单体 `src/ui.rs` 拆解为高内聚、低耦合的模块化结构，分离**状态管理 (State)**、**事件处理 (Event Handling)** 和 **视图渲染 (Rendering)**，以提升代码可读性和可维护性。
*   **设计模式选择**：采用 **MVI (Model-View-Intent)** 或类似 **ELM** 的架构思想。
    *   **Model (State)**: `App` 结构体及其状态数据。
    *   **View (Render)**: 纯函数，将 State 映射为 UI。
    *   **Intent (Action)**: 用户交互转化为明确的语义化 Action。
*   **接口契约**：
    *   **Directory Structure**:
        ```text
        src/ui/
        ├── mod.rs          // 统一导出
        ├── state.rs        // App 状态定义 (Model)
        ├── actions.rs      // Action 枚举定义 (Intent)
        ├── logic.rs        // 业务逻辑处理 (Update/Dispatch)
        ├── input.rs        // 键盘事件映射 (Input -> Action)
        └── view/           // 视图层
            ├── mod.rs      // 主渲染入口
            ├── components.rs // 通用组件 (Dialogs, Input fields)
            └── layouts.rs  // 布局逻辑
        ```

## 2. 🗺️ 变更影响范围 (Impact Analysis)

*   **现有代码修改点**：
    *   **`src/main.rs`**: 更新模块引用路径。
        *   `mod ui;` -> `mod ui;` (无需变动，但 `ui` 变成了文件夹)
        *   `use crate::ui::App;` -> `use crate::ui::App;` (通过 `ui/mod.rs` 重新导出，保持 `main.rs` 变动最小)。
    *   **`src/ui.rs`**: **删除**该文件，替换为 `src/ui/` 目录。
*   **新增模块**：
    *   `src/ui/state.rs`: 包含 `App`, `AppMode`, `InputField`, `ConfirmAction`。
    *   `src/ui/actions.rs`: 包含 `Action` 枚举。
    *   `src/ui/logic.rs`: 包含 `impl App` 中的 `dispatch` 及其拆分后的处理函数。
    *   `src/ui/input.rs`: 包含 `handle_key_event`, `get_action`。
    *   `src/ui/view/*.rs`: 包含 `render` 及其辅助函数。
*   **依赖变更**：无新增第三方库依赖。

## 3. 💻 核心实现指南 (Implementation Steps)

### Step 1: 基础类型拆分 (Types & State)

创建 `src/ui/actions.rs` 和 `src/ui/state.rs`。

```rust
// src/ui/actions.rs
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Quit,
    MoveSelectionUp,
    // ... 其他动作
}

// src/ui/state.rs
use crate::models::{FocusTree, FocusNode};
use super::actions::Action; // 如果 App 需要引用 Action (虽然通常是在 logic 中引用)

pub struct App {
    pub tree: FocusTree,
    // ... 字段
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode { /* ... */ }
```

### Step 2: 逻辑与输入处理 (Logic & Input)

将庞大的 `dispatch` 逻辑移动到 `logic.rs`，按功能块拆分 `impl App`。

```rust
// src/ui/logic.rs
use super::state::{App, AppMode};
use super::actions::Action;

impl App {
    pub fn dispatch(&mut self, action: Action) -> bool {
        match action {
            Action::Quit => return true,
            Action::StartAddNode => self.start_add_node(),
            // ...
        }
        false
    }
    
    // 将原有的 helper 方法如 start_add_node, move_up 等移至此处
    // 建议进一步拆分：
    // fn handle_navigation(&mut self, action: Action)
    // fn handle_editing(&mut self, action: Action)
}
```

```rust
// src/ui/input.rs
use crossterm::event::KeyCode;
use super::actions::Action;
use super::state::{App, AppMode};

pub fn handle_key_event(app: &mut App, key: KeyCode) -> std::io::Result<bool> {
    // ... 原有的 get_action 和 dispatch 调用逻辑
}
```

### Step 3: 视图层重构 (View)

将渲染逻辑按“页面”或“组件”拆分。

```rust
// src/ui/view/mod.rs
use ratatui::Frame;
use super::state::App;
mod components;

pub fn render(frame: &mut Frame, app: &mut App) {
    // ... 原有的 render 逻辑，调用 components 中的函数
    // components::render_tree(frame, app, area);
}
```

### Step 4: 胶水代码 (Integration)

在 `src/ui/mod.rs` 中重新导出，确保外部调用的兼容性。

```rust
// src/ui/mod.rs
pub mod state;
pub mod actions;
pub mod logic;
pub mod input;
pub mod view;

// Re-export for convenience
pub use state::App;
pub use view::render;
pub use input::handle_key_event;
```

## 4. 🛡️ 防御式编程与潜在坑点 (Safety & Edge Cases)

*   **可见性陷阱 (Visibility)**:
    *   拆分模块后，`App` 的字段可能需要从 `pub` 改为 `pub(crate)` 或者保持 `pub` 但仅限于 `ui` 模块内部使用。注意 `main.rs` 是否直接访问了字段。
    *   **检查**: `main.rs` 使用了 `app.tree` 进行保存，所以 `tree` 字段必须是 `pub`。
*   **循环依赖 (Circular Dependencies)**:
    *   避免 `state.rs` 引用 `logic.rs`。逻辑应该依赖于状态，而不是反过来。
    *   `view` 依赖 `state`，但不应修改 `state`（只读引用）。
*   **代码遗漏**:
    *   在移动代码时，容易遗漏某些 `impl` 块中的私有辅助函数。建议先复制粘贴，再修剪。

## 5. ✅ 测试与验收策略 (Verification)

*   **编译检查**:
    *   重构过程中频繁运行 `cargo check`。
*   **功能回归测试 (Manual Regression)**:
    *   启动应用，测试所有快捷键：
        *   `j`/`k` 导航是否正常？
        *   `a` 添加节点流程是否完整（标题 -> 内容 -> 确认）？
        *   `m` 移动节点逻辑是否正确？
        *   `d` 删除确认弹窗是否显示？
*   **单元测试 (推荐新增)**:
    *   为 `logic.rs` 中的 `dispatch` 编写纯逻辑测试：
        ```rust
        #[test]
        fn test_move_selection() {
            let mut app = App::new(mock_tree());
            app.dispatch(Action::MoveSelectionDown);
            assert_eq!(app.selected_index, 1);
        }
        ```
