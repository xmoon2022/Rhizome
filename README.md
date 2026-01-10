# Rhizome TUI 软件

基于 RSIP（递归稳态迭代协议）方法论的国策树管理工具，使用 Rust + Ratatui 构建的终端用户界面。

> **灵感来源**：本项目的创意灵感来自知乎问答 [「如何提高自制力？」](https://www.zhihu.com/question/19888447/answer/1930799480401293785) 中 edmond 的回答。

## 功能特性

### 核心功能
- ✅ **树状结构管理** - 支持层级嵌套的国策节点
- ✅ **TOML 数据持久化** - 自动保存/加载 `rsip_data.toml`
- ✅ **堆栈式删除** - 删除节点时自动级联删除所有子节点
- ✅ **失败标记** - 符合RSIP方法论的失败处理

### TUI 功能
- ✅ **Vim风格导航** - `j/k` 上下移动
- ✅ **节点添加** - 两步输入（标题 + 可选内容）
- ✅ **内容编辑** - 修改现有节点内容
- ✅ **节点移动** - 调整节点的父子层级关系
- ✅ **详情显示** - 查看节点创建时间、连续天数、状态等

---

## 安装

### 使用 Cargo
```bash
cargo build --release
./target/release/rsip-tree
```

### 使用 Nix Flakes
```bash
# 直接运行
nix run .

# 安装到用户环境
nix profile add .

# 进入开发环境
nix develop
```

---

## 快捷键

| 按键 | 功能 |
|------|------|
| `j/k` | 上下导航 |
| `a` | 添加新节点 |
| `e` | 编辑选中节点内容 |
| `m` | 移动节点到新位置 |
| `d` | 删除节点（级联删除子节点） |
| `f` | 标记节点失败 |
| `q` | 退出程序 |

---

## 数据存储

数据文件存储在 `~/.local/share/rhizome/data.toml`，符合 XDG 基目录规范。

## 文件结构

```
rhizome/
├── Cargo.toml          # 项目配置
├── flake.nix           # Nix Flakes 配置
└── src/
    ├── main.rs         # 程序入口
    ├── models.rs       # 数据模型（FocusNode, FocusTree）
    ├── storage.rs      # TOML 读写
    └── ui.rs           # TUI 界面
```

---

## RSIP 方法论

RSIP（Recursive Stabilization Iteration Protocol，递归稳态迭代协议）是一种自控方法论：

- **国策/定式** - 针对特定负面状态的局部最优解
- **堆栈式删除** - 如果父节点失败，所有子节点也会被删除
- **递归回溯** - 通过不断尝试和回退，找到最优的国策组合

---

## 待实现功能

- [ ] 国策组支持（容错机制）
- [ ] 节点强化升级
- [ ] 统计与进度追踪
- [ ] 每日添加限制提醒

---

## License

[GPL-3.0-or-later](LICENSE)
