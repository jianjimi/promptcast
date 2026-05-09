// models — 领域模型，与前端 src/types/ 字段一一对应。
// M0 阶段尚未在命令里使用，先全模块抑制 dead_code 警告。
#![allow(dead_code)]

pub mod prompt;
pub mod folder;
pub mod tag;
pub mod site;
pub mod settings;
