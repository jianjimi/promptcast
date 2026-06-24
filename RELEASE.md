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

## macOS 代码签名（自签名证书）

### 为什么要签名
macOS 的辅助功能(TCC)授权按 App 的**代码签名身份**记。ad-hoc 未签名的包每次构建 cdhash 都变，
更新替换后系统当成"另一个 App"，授权作废、需重授。用**同一张证书**签名后，签名身份（指定要求 DR）
跨版本稳定，更新后授权保留。自签名非 Apple 公证，Gatekeeper 首次打开仍需右键「打开」，但不影响授权保留。

### 已有的配置
- **证书**：自签名代码签名证书，CN=`PromptCast Self-Signed`，有效期 100 年（`extendedKeyUsage=codeSigning`）。
- **GitHub Secrets**（已用 `gh secret set` 配在 `jianjimi/promptcast`）：
  - `APPLE_CERTIFICATE`：`.p12` 的 base64
  - `APPLE_CERTIFICATE_PASSWORD`：`.p12` 密码
  - `APPLE_SIGNING_IDENTITY`：`PromptCast Self-Signed`
- **CI**：`release.yml` 的 “Setup macOS signing keychain” 步导入 `.p12` 到临时钥匙串、
  `sudo security add-trusted-cert ... -p codeSign` 信任为代码签名根（自签名必须这步，否则 codesign 报
  no identity found）；构建步通过环境变量 `APPLE_SIGNING_IDENTITY` 让 tauri 用该身份签名。Windows 忽略这些。

### 证书文件与备份（重要）
私钥证书在本机 **`~/Desktop/LXG/promptcast-signing/`**（不在 git 里）：
- `promptcast-codesign.p12`（含私钥）、`p12-password.txt`（密码）。
**务必备份到安全处**（云盘/密码管理器）。丢失只能重新生成新证书，等于换了身份、需再重授一次。

### 验证某个发版是否签名成功
```bash
gh release download vX.Y.Z --repo jianjimi/promptcast --pattern "*universal.dmg" -D /tmp/v && \
hdiutil attach /tmp/v/*.dmg -nobrowse -mountpoint /tmp/vm >/dev/null && \
codesign -dvv /tmp/vm/*.app 2>&1 | grep Authority ; hdiutil detach /tmp/vm >/dev/null
# 期望看到：Authority=PromptCast Self-Signed
```

### 重新生成 / 轮换证书（丢失或过期时）
```bash
DIR=~/Desktop/LXG/promptcast-signing; mkdir -p "$DIR"; cd "$DIR"
PASS=$(openssl rand -hex 16); echo "$PASS" > p12-password.txt
openssl req -x509 -newkey rsa:2048 -keyout key.pem -out cert.pem -days 36500 -nodes \
  -subj "/CN=PromptCast Self-Signed/O=PromptCast/C=CN" \
  -addext "basicConstraints=critical,CA:FALSE" \
  -addext "keyUsage=critical,digitalSignature" \
  -addext "extendedKeyUsage=critical,codeSigning"
openssl pkcs12 -export -inkey key.pem -in cert.pem -out promptcast-codesign.p12 \
  -name "PromptCast Self-Signed" -passout pass:"$PASS"   # macOS 自带 LibreSSL：不要加 -legacy
# 重新写入 Secret：
base64 -i promptcast-codesign.p12 | gh secret set APPLE_CERTIFICATE --repo jianjimi/promptcast
printf '%s' "$PASS" | gh secret set APPLE_CERTIFICATE_PASSWORD --repo jianjimi/promptcast
printf '%s' "PromptCast Self-Signed" | gh secret set APPLE_SIGNING_IDENTITY --repo jianjimi/promptcast
```
> 换证书 = 新身份，下一次更新会再重授一次辅助功能，之后恢复保留。有效期到期前签新版会失败，
> 届时按上面重新生成即可（设了 100 年，正常不会遇到）。

### 一次性的过渡（0.0.6 未签名 → 0.0.7 签名）
身份从"无签名"变"有签名"，**0.0.6→0.0.7 这一次仍需重授一次**（移除系统设置里旧 PromptCast 条目、
重开触发授权、再重开）。从 0.0.7 起（同证书签名）授权一直保留。

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
- macOS 包用自签名证书签名（非 Apple 公证），从别处（浏览器）下载首次打开仍需右键「打开」或
  `xattr -dr com.apple.quarantine <App>`；应用内更新走脚本替换并去隔离，relaunch 不受 Gatekeeper 阻拦。详见上文「macOS 代码签名」。
- keyring 在 macOS 走 Keychain，首次同步登录可能弹钥匙串授权，正常。
- 国内若 `raw.githubusercontent.com` 访问不稳，后续把清单与安装包镜像到 CNB 公开仓库，改 `DEFAULT_MANIFEST_URL` 重新发版即可。
