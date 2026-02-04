//! 键盘事件映射 (Input -> Action)
//!
//! 将按键事件转换为 Action

use std::io;

use crossterm::event::KeyCode;

use super::actions::Action;
use super::state::{App, AppMode};

/// 根据当前模式和按键获取对应的 Action
pub fn get_action(mode: &AppMode, key: KeyCode) -> Option<Action> {
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
