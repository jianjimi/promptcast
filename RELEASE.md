# 发版与更新（PromptCast）

多平台打包 + 应用内更新的完整流程。源码主仓库 `liuxiaogang/promptcast` 保持**私有**；
打包与托管用两个**公开**仓库。

## 角色分工

| 用途 | 平台 | 仓库 | 可见性 |
|------|------|------|--------|
| 源码主仓库 | CNB | `liuxiaogang/promptcast` | 私有 |
| 测试 / lint CI | CNB | 同上（`.cnb.yml`） | — |
| 多平台打包 + Release | GitHub（公开仓库） | `<github用户>/<公开仓库>` | **公开** |
| 国内可靠下载镜像 | CNB（公开仓库） | `<公开组>/<公开仓库>` | **公开** |

> 为什么不用 CNB 打包：CNB 构建节点只有 Linux Docker（amd64/arm64），无法产出
> Windows `.exe/.msi`（需 Windows）和 macOS `.dmg`（需苹果机器）。CNB 只跑测试，GitHub 负责打包。
>
> 为什么托管要公开：更新器下载是匿名 GET（不带 token），私有仓库匿名访问 404。

## 一次性准备

1. 在 GitHub 建一个**公开**仓库，把源码推上去（确认 `.github/workflows/release.yml` 在内）。
   - ⚠️ 公开前自查无密钥泄露：`server/.env` 不要提交（仓库已 `.gitignore`，只保留 `.env.example`）。
2. 在 CNB 建一个**公开**仓库做下载镜像（如 `promptcast-dist`）。
3. （可选，macOS 免 Gatekeeper 警告）准备 Apple 开发者证书，后续在 GitHub Secrets 配
   `APPLE_CERTIFICATE` 等给 tauri-action；没有则为 ad-hoc 未签名，可用但首次打开要右键「打开」。

## 每次发版

1. **升版本号**：`src-tauri/tauri.conf.json` 的 `version` 和 `package.json` 的 `version` 同步改（如 `0.2.0`）。
2. **打 tag 并推到 GitHub 公开仓库**：
   ```bash
   git tag v0.2.0
   git push <github-remote> v0.2.0
   ```
   GitHub Actions 自动在 macOS/Windows/Linux 三机打包，产物传到一个**草稿 Release**。
3. **审核并发布 GitHub Release**：产物齐了（macOS `*_universal.dmg`、Windows `*_x64-setup.exe` / `*.msi`、
   Linux `*.AppImage` / `*.deb`），确认无误后点 Publish。
4. **镜像到 CNB 公开仓库**：下载这批安装包，在 CNB 公开仓库新建同名 tag 的 Release，上传同样的文件。
   在 CNB 资产上「复制下载链接」拿到匿名 https 直链。
5. **算 sha256**（建议）：
   ```bash
   shasum -a 256 PromptCast_0.2.0_universal.dmg PromptCast_0.2.0_x64-setup.exe
   ```
6. **写 update.json**：照 `update.example.json` 复制为 `update.json`：
   - `version` = 新版本号；`url` 填 **CNB** 直链（国内可靠）；`sha256` 填上一步的值。
   - GitHub 直链记到 `_backup_github`（客户端不读，仅备查）。
7. **上传 update.json** 到公开托管处（CNB 公开仓库 Release 资产，或仓库 raw），拿到它的 https 直链。
8. **客户端配置**：首次在「设置 → 关于 → 更新清单地址」填入 update.json 的直链（之后版本不用再改地址，
   只要更新清单内容）。旧版本客户端「检查更新」即命中新版 → 下载 → 拉起安装器。

## 本地端到端演练（不依赖外网）

更新器允许 `http://localhost` / `127.0.0.1` 走明文，便于本机全链路测：

```bash
# 1) 本机起静态服务托管 update.json + 安装包（端口随意）
cd src-tauri/target/release/bundle
python3 -m http.server 8000
# 2) 临时把 update.json 的 url 改成 http://localhost:8000/dmg/PromptCast_x.x.x_universal.dmg
# 3) 把 update.json 也放到该目录；设置里填 http://localhost:8000/update.json
# 4) 把清单 version 改成比当前高（如 0.9.9）→ 点检查更新 → 应弹窗 → 下载 → open 挂载 dmg
```

## 注意点

- **更新器是自建的**（托管 JSON 清单），非 Tauri 官方 updater，故**无需** minisign 签名密钥。
- 下载链接强制 https（防 MITM 注入安装器）；https→http 降级跳转会被拒。
- macOS 通用包：`darwin-aarch64` 与 `darwin-x86_64` 在 update.json 里指向同一个 `*_universal.dmg`。
- keyring 在 macOS 走 Keychain，首次同步登录可能弹钥匙串授权，正常。
