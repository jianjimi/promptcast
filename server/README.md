# PromptCast 同步后端（NestJS + Prisma + Postgres）

多设备同步的服务端：认证（注册/登录/刷新/登出）+ 通用增量存储（按用户隔离）。
本地是离线优先的真相源，服务端只负责认证、按用户存变更、返回「自游标以来」的增量。

## 一键起（推荐，需 Docker）

```bash
cd server
docker compose up --build
# Postgres + 后端都起来；后端启动会自动 prisma migrate deploy 建表。
# 健康检查后 API 在 http://localhost:3000
```

## 本地开发（Node 直跑，DB 用 docker）

```bash
cd server
cp .env.example .env
npm install
docker compose up -d postgres          # 只起数据库
npx prisma migrate dev --name init     # 首次：生成并应用迁移
npm run start:dev                      # 热重载起服务
```

## 端点

| 方法 | 路径 | 说明 |
|---|---|---|
| POST | `/auth/register` | `{email, password}` → `{userId, access, refresh}` |
| POST | `/auth/login` | 同上 |
| POST | `/auth/refresh` | `{refresh}` → `{access, refresh}`（轮换吊销旧的） |
| POST | `/auth/logout` | `{refresh}` → 204 |
| POST | `/sync/pull` | `{since_cursor, limit}` → `{changes[], next_cursor, has_more}`（需 Bearer） |
| POST | `/sync/push` | `{changes[]}` → `{results[], server_cursor}`（需 Bearer） |

`changes[]` 的信封：`{entity: 'prompt'|'folder'|'tag'|'site', uuid, updated_at, deleted_at, data}`。
`data` 是不透明 JSON，服务端不解析其结构。

## 设计要点

- **通用薄存储** `sync_records(user_id, entity, uuid, updated_at, deleted_at, data jsonb, seq)`：
  桌面端独占 schema，未来改字段不动服务端。
- **单调游标 `seq`**（bigint 序列）：客户端「拉取自 seq 之后」。每次 push 命中写入取 `nextval`，
  使更新过的记录总能被重新拉到。避免用墙钟时间做游标（时钟漂移）。
- **服务端 LWW**：`incoming.updated_at >= 现有` 才写入（平局 incoming 赢、幂等）；输掉的记录
  客户端下一拍 pull 拿到权威版本。
- **认证**：bcrypt 存密码；JWT access（短期）+ 随机 refresh（哈希入库、可轮换吊销）。

## 测试

```bash
docker compose up -d postgres
npx prisma migrate deploy   # 或 prisma db push
npm run test:e2e
```
