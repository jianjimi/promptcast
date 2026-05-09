// commands — 所有 IPC 命令；按领域拆分子模块。
// 各子模块在 M1 起逐步实装。当前 M0 仅暴露 ping 用于自检。
//
// 注：lib.rs 中通过 `commands::ping::ping` 访问（tauri::generate_handler!
// 需要在命令所在模块解析其同名隐藏 helper 项 __cmd__xxx）。

pub mod ping;
// pub mod prompts;     // M1
// pub mod folders;     // M1
// pub mod tags;        // M1
// pub mod settings;    // M1
// pub mod sites;       // M2.8
// pub mod inject;      // M4
// pub mod window;      // M4
// pub mod data;        // M1（导入导出）
