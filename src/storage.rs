use std::fs;
use std::io;
use std::path::Path;

use crate::models::{FocusTree, FocusTreeData};

/// 从TOML文件加载树
pub fn load_tree(path: &Path) -> io::Result<FocusTree> {
    if !path.exists() {
        return Ok(FocusTree::new());
    }

    let content = fs::read_to_string(path)?;
    let data: FocusTreeData =
        toml::from_str(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    Ok(FocusTree::from_data(data))
}

/// 保存树到TOML文件
pub fn save_tree(tree: &mut FocusTree, path: &Path) -> io::Result<()> {
    if !tree.dirty {
        return Ok(());
    }

    let data = tree.to_data();
    let content =
        toml::to_string_pretty(&data).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    fs::write(path, content)?;
    
    tree.dirty = false;
    Ok(())
}
