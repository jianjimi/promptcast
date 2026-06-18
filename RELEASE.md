# 发版与更新（PromptCast）

多平台打包 + 应用内更新。源码主仓库 `liuxiaogang/promptcast`（CNB）保持**私有**；
打包/发布/托管用公开的 GitHub 仓库 `jianjimi/promptcast`。

## 角色分工

| 用途 | 平台 | 仓库 | 可见性 |
|------|------|------|--------|
| 源码主仓库 | CNB | `liuxiaogang/promptcast` | 私有 |
| 测试 / lint CI | CNB | 同上（`.cnb.yml`） | — |
| 多平台打包 + Release + 更新清单托管 | GitHub | `jianjimi/promptcast` | **公开** |
| 国内可靠下载镜像（计划中） | CNB | `<公开仓库>` | 公开 |

> 为什么不用 CNB 打包：CNB 构建节点只有 Linux Docker，无法产出 Windows `.exe/.msi`、macOS `.dmg`。
> 为什么托管要公开：更新器匿名 GET 下载，私有仓库匿名访问 404。

## 更新清单地址（已预制，不可改）

客户端硬编码读取（见 `src-tauri/src/update/mod.rs` 的 `DEFAULT_MANIFEST_URL`）：

```
https://raw.githubusercontent.com/jianjimi/promptcast/main/update.json
```

`update.json` 由 CI **自动生成**（版本、下载直链、sha256、发布说明），**请勿手改**。
关于页不再有输入框；启动时 + 每 30 分钟静默查一次，发现新版才弹窗（可「跳过当前版本/今天忽略」）。

## 每次发版（全自动）

只需打一个**注释 tag** 并推到 GitHub，CI 全包：

```bash
# 1) 同步升版本号：src-tauri/Cargo.toml、package.json、src-tauri/tauri.conf.json（三处一致）
# 2) 打注释 tag（首段 -m 作更新标题，其余 -m 作说明，可多行）
git tag -a v0.0.3 -m "标题：本次更新要点" -m "· 改动一\n· 改动二"
git push github v0.0.3
```

CI（`.github/workflows/release.yml`）随后：

1. **build**：macOS（通用包）+ Windows 并行编译 → 产物传到**草稿** Release。
2. **publish**（两平台都成功后）：下载产物算 sha256 → 生成 `update.json` →
   **先发布 Release**（资产此刻才公开可下载）→ **再把 `update.json` 提交回 main**。
   这个顺序消除了「`update.json` 抢在资产前上线导致客户端下到 404」的竞态。

发完即生效：旧版客户端下次静默查（或手动「检查更新」）就会命中新版。

> ⚠️ 版本号三处务必一致：`Cargo.toml` 的 version 决定更新器「当前版本」（`CARGO_PKG_VERSION`），
> `tauri.conf.json` 决定安装包版本，CI 用 git tag 决定 `update.json` 的 version。

## 构建提速：main 缓存预热

GitHub Actions 缓存按 git ref 隔离 —— tag 构建彼此看不到缓存，但都能读 **main** 的缓存。
`.github/workflows/warm-cache.yml` 在 `src-tauri/**` 或依赖变动时于 main 上跑一次 `tauri build --no-bundle`，
把 ~500 个依赖的编译产物预热进 main 作用域（与 release.yml 共用 `shared-key: tauri-release`）。
之后发版 tag 构建命中缓存：冷构建 ~10 分钟 → 热构建约 2-4 分钟。公开仓库 Actions 免费。

## 本地端到端演练（不依赖外网 / 不发版）

清单地址虽预制，但后端保留了「DB 覆盖」后门（UI 不暴露）：DB 的 `update_manifest_url` 非空时优先用它。
更新器放行 `http://localhost` / `127.0.0.1` 明文，可本机全链路测：

```bash
# 1) 本机静态服务托管 update.json + 安装包
cd src-tauri/target/release/bundle && python3 -m http.server 8000
# 2) 写一个 update.json 放到该目录，version 调高（如 9.9.9），url 指向 http://localhost:8000/dmg/<dmg>
# 3) 临时设置后门地址（开发期可在前端临时调用，或直接改 settings 表 update_manifest_url）
#    → 启动/手动检查 → 应弹窗（含 立即更新/今天忽略/跳过当前版本）→ 下载 → open 挂载 dmg
```

## 注意点

- 自建更新器（托管 JSON 清单），非 Tauri 官方 updater，**无需** minisign 签名。
- 下载链接强制 https（防 MITM 注入安装器）；https→http 降级跳转被拒。
- macOS 通用包：`update.json` 里 `darwin-aarch64` 与 `darwin-x86_64` 指向同一 `*_universal.dmg`。
- macOS 包 ad-hoc 未签名，拷到别的 Mac 首次打开需右键「打开」或 `xattr -dr com.apple.quarantine <App>`。
- keyring 在 macOS 走 Keychain，首次同步登录可能弹钥匙串授权，正常。
- 国内若 `raw.githubusercontent.com` 访问不稳，后续把清单与安装包镜像到 CNB 公开仓库，改 `DEFAULT_MANIFEST_URL` 重新发版即可。
