import { INestApplication, ValidationPipe } from '@nestjs/common';
import { Test } from '@nestjs/testing';
import { ThrottlerGuard } from '@nestjs/throttler';
import request from 'supertest';
import { AppModule } from '../src/app.module';
import { PrismaService } from '../src/prisma/prisma.service';

// 需要一个可连的 Postgres（DATABASE_URL）且已 migrate。
//   docker compose up -d postgres
//   npx prisma migrate deploy   (或 prisma db push)
//   npm run test:e2e
describe('auth + sync e2e', () => {
  let app: INestApplication;
  const stamp = `${Math.floor(Math.random() * 1e9)}`;
  const emailA = `a${stamp}@t.io`;
  const emailB = `b${stamp}@t.io`;

  beforeAll(async () => {
    const mod = await Test.createTestingModule({
      imports: [AppModule],
    })
      // 关掉限流：e2e 测的是功能，单测里大量连续 auth 调用否则会被 429。
      .overrideGuard(ThrottlerGuard)
      .useValue({ canActivate: () => true })
      .compile();
    app = mod.createNestApplication();
    app.useGlobalPipes(new ValidationPipe({ whitelist: true, transform: true }));
    await app.init();
  });

  afterAll(async () => {
    await app.get(PrismaService).$disconnect();
    await app.close();
  });

  const change = (
    uuid: string,
    updated_at: number,
    data: Record<string, unknown>,
    deleted_at: number | null = null,
  ) => ({ entity: 'prompt', uuid, updated_at, deleted_at, data });

  const auth = (t: string) => ({ Authorization: `Bearer ${t}` });

  it('register → push → pull → LWW → tombstone → user isolation', async () => {
    const http = () => request(app.getHttpServer());

    const reg = await http()
      .post('/auth/register')
      .send({ email: emailA, password: 'password123' })
      .expect(201);
    const tokenA: string = reg.body.access;
    expect(typeof tokenA).toBe('string');
    expect(typeof reg.body.refresh).toBe('string');

    const u1 = '11111111-1111-4111-8111-111111111111'; // 合法 v4 UUID

    // push 一条
    const p1 = await http()
      .post('/sync/push')
      .set(auth(tokenA))
      .send({ changes: [change(u1, 100, { title: 'v1' })] })
      .expect(200);
    expect(p1.body.results[0].applied).toBe(true);

    // 从 0 拉到它
    const pull1 = await http()
      .post('/sync/pull')
      .set(auth(tokenA))
      .send({ since_cursor: 0 })
      .expect(200);
    expect(pull1.body.changes).toHaveLength(1);
    expect(pull1.body.changes[0].data.title).toBe('v1');
    const cursor1: number = pull1.body.next_cursor;

    // push 更新版本
    await http()
      .post('/sync/push')
      .set(auth(tokenA))
      .send({ changes: [change(u1, 200, { title: 'v2' })] })
      .expect(200);

    // 从 cursor1 只拉到改动的那条、且 seq 更大
    const pull2 = await http()
      .post('/sync/pull')
      .set(auth(tokenA))
      .send({ since_cursor: cursor1 })
      .expect(200);
    expect(pull2.body.changes).toHaveLength(1);
    expect(pull2.body.changes[0].data.title).toBe('v2');
    expect(pull2.body.next_cursor).toBeGreaterThan(cursor1);

    // push 过期更新 → LWW 拒
    const stale = await http()
      .post('/sync/push')
      .set(auth(tokenA))
      .send({ changes: [change(u1, 150, { title: 'old' })] })
      .expect(200);
    expect(stale.body.results[0].applied).toBe(false);

    // push 墓碑 → pull 能拿到 deleted_at
    await http()
      .post('/sync/push')
      .set(auth(tokenA))
      .send({ changes: [change(u1, 300, { title: 'v2' }, 300)] })
      .expect(200);
    const pullDel = await http()
      .post('/sync/pull')
      .set(auth(tokenA))
      .send({ since_cursor: cursor1 })
      .expect(200);
    const rec = pullDel.body.changes.find(
      (c: { uuid: string }) => c.uuid === u1,
    );
    expect(rec.deleted_at).toBe(300);

    // 用户隔离：B 拉不到 A 的任何东西
    const regB = await http()
      .post('/auth/register')
      .send({ email: emailB, password: 'password123' })
      .expect(201);
    const pullB = await http()
      .post('/sync/pull')
      .set(auth(regB.body.access))
      .send({ since_cursor: 0 })
      .expect(200);
    expect(pullB.body.changes).toHaveLength(0);
  });

  it('rejects unauthenticated sync', async () => {
    await request(app.getHttpServer())
      .post('/sync/pull')
      .send({ since_cursor: 0 })
      .expect(401);
  });

  it('refresh rotates tokens; login works', async () => {
    const http = () => request(app.getHttpServer());
    const login = await http()
      .post('/auth/login')
      .send({ email: emailA, password: 'password123' })
      .expect(200);
    const refreshed = await http()
      .post('/auth/refresh')
      .send({ refresh: login.body.refresh })
      .expect(200);
    expect(typeof refreshed.body.access).toBe('string');
    // 旧 refresh 已被轮换吊销 → 再用应 401
    await http()
      .post('/auth/refresh')
      .send({ refresh: login.body.refresh })
      .expect(401);
  });

  it('change password, forgot/reset flow, delete account', async () => {
    const http = () => request(app.getHttpServer());
    const email = `acct${Math.floor(Math.random() * 1e9)}@t.io`;
    const reg = await http()
      .post('/auth/register')
      .send({ email, password: 'password123' })
      .expect(201);

    // 改密 → 旧密码失效、新密码可登。
    await http()
      .post('/auth/change-password')
      .set(auth(reg.body.access))
      .send({ oldPassword: 'password123', newPassword: 'newpass456' })
      .expect(204);
    await http()
      .post('/auth/login')
      .send({ email, password: 'password123' })
      .expect(401);
    await http()
      .post('/auth/login')
      .send({ email, password: 'newpass456' })
      .expect(200);

    // 找回密码：dev 回显 token → reset → 新密码可登。
    const forgot = await http()
      .post('/auth/forgot-password')
      .send({ email })
      .expect(200);
    expect(typeof forgot.body.devToken).toBe('string');
    await http()
      .post('/auth/reset-password')
      .send({ token: forgot.body.devToken, newPassword: 'reset789x' })
      .expect(204);
    // 同一 token 不能复用。
    await http()
      .post('/auth/reset-password')
      .send({ token: forgot.body.devToken, newPassword: 'again000x' })
      .expect(401);
    const relog = await http()
      .post('/auth/login')
      .send({ email, password: 'reset789x' })
      .expect(200);

    // 不存在的邮箱也返回 200，但不发 token（不泄漏账号存在）。
    const f2 = await http()
      .post('/auth/forgot-password')
      .send({ email: `nobody${Math.floor(Math.random() * 1e9)}@x.io` })
      .expect(200);
    expect(f2.body.devToken).toBeUndefined();

    // 删除账户：密码错拒、对了删；删后登录失败。
    await http()
      .post('/auth/delete-account')
      .set(auth(relog.body.access))
      .send({ password: 'wrong' })
      .expect(401);
    await http()
      .post('/auth/delete-account')
      .set(auth(relog.body.access))
      .send({ password: 'reset789x' })
      .expect(204);
    await http()
      .post('/auth/login')
      .send({ email, password: 'reset789x' })
      .expect(401);
  });
});
