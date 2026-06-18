import {
  ConflictException,
  Injectable,
  UnauthorizedException,
} from '@nestjs/common';
import { JwtService } from '@nestjs/jwt';
import * as bcrypt from 'bcrypt';
import { randomBytes } from 'crypto';
import { PrismaService } from '../prisma/prisma.service';

export interface Tokens {
  access: string;
  refresh: string;
}

@Injectable()
export class AuthService {
  private readonly accessSecret =
    process.env.JWT_SECRET ?? 'dev-access-secret-change-me';
  private readonly accessTtl = process.env.ACCESS_TOKEN_TTL ?? '900s';
  private readonly refreshTtlDays = Number(
    process.env.REFRESH_TOKEN_TTL_DAYS ?? '30',
  );

  constructor(
    private readonly prisma: PrismaService,
    private readonly jwt: JwtService,
  ) {}

  async register(
    email: string,
    password: string,
  ): Promise<Tokens & { userId: string }> {
    const existing = await this.prisma.user.findUnique({ where: { email } });
    if (existing) throw new ConflictException('email already registered');
    const passwordHash = await bcrypt.hash(password, 10);
    const user = await this.prisma.user.create({
      data: { email, passwordHash },
    });
    return { userId: user.id, ...(await this.issue(user.id, user.email)) };
  }

  async login(
    email: string,
    password: string,
  ): Promise<Tokens & { userId: string }> {
    const user = await this.prisma.user.findUnique({ where: { email } });
    if (!user) throw new UnauthorizedException('invalid credentials');
    const ok = await bcrypt.compare(password, user.passwordHash);
    if (!ok) throw new UnauthorizedException('invalid credentials');
    return { userId: user.id, ...(await this.issue(user.id, user.email)) };
  }

  async refresh(refreshToken: string): Promise<Tokens> {
    const { row, user } = await this.verifyRefresh(refreshToken);
    // 轮换：吊销旧 refresh，发新对。
    await this.prisma.refreshToken.update({
      where: { id: row.id },
      data: { revoked: true },
    });
    return this.issue(user.id, user.email);
  }

  async logout(refreshToken: string): Promise<void> {
    // 幂等：无效 token 也当登出成功，不报错。
    try {
      const { row } = await this.verifyRefresh(refreshToken);
      await this.prisma.refreshToken.update({
        where: { id: row.id },
        data: { revoked: true },
      });
    } catch {
      /* ignore */
    }
  }

  async changePassword(
    userId: string,
    oldPassword: string,
    newPassword: string,
  ): Promise<void> {
    const user = await this.prisma.user.findUnique({ where: { id: userId } });
    if (!user) throw new UnauthorizedException();
    const ok = await bcrypt.compare(oldPassword, user.passwordHash);
    if (!ok) throw new UnauthorizedException('旧密码不正确');
    const passwordHash = await bcrypt.hash(newPassword, 10);
    await this.prisma.$transaction([
      this.prisma.user.update({ where: { id: userId }, data: { passwordHash } }),
      // 改密 → 吊销所有 refresh，强制其它设备重新登录。
      this.prisma.refreshToken.updateMany({
        where: { userId },
        data: { revoked: true },
      }),
    ]);
  }

  async deleteAccount(userId: string, password: string): Promise<void> {
    const user = await this.prisma.user.findUnique({ where: { id: userId } });
    if (!user) throw new UnauthorizedException();
    const ok = await bcrypt.compare(password, user.passwordHash);
    if (!ok) throw new UnauthorizedException('密码不正确');
    // 级联删除 refresh_tokens / password_resets / sync_records（onDelete: Cascade）。
    await this.prisma.user.delete({ where: { id: userId } });
  }

  /** 发起找回密码。不泄漏账号是否存在：无论存不存在都「成功」；仅存在时真的签发 token。
   *  生产应把 token 邮件给用户；本地开发(NODE_ENV!=production)直接返回 + 打日志便于测试。 */
  async forgotPassword(email: string): Promise<{ devToken?: string }> {
    const user = await this.prisma.user.findUnique({ where: { email } });
    if (!user) {
      // 等时处理：未注册也做一次等价开销的 bcrypt，避免「已注册 vs 未注册」的响应时间侧信道。
      await bcrypt.hash(randomBytes(32).toString('hex'), 10);
      return {};
    }
    const raw = randomBytes(32).toString('hex');
    const tokenHash = await bcrypt.hash(raw, 10);
    const expiresAt = new Date(Date.now() + 30 * 60 * 1000);
    const row = await this.prisma.passwordReset.create({
      data: { userId: user.id, tokenHash, expiresAt },
    });
    const token = `${row.id}.${raw}`;
    if (process.env.NODE_ENV === 'production') {
      // TODO(上线): 接邮件服务把 token 发到 email；现在生产不回显。
      return {};
    }
    // eslint-disable-next-line no-console
    console.log(`[dev] password reset token for ${email}: ${token}`);
    return { devToken: token };
  }

  async resetPassword(token: string, newPassword: string): Promise<void> {
    const dot = token.indexOf('.');
    if (dot < 0) throw new UnauthorizedException('无效重置令牌');
    const id = token.slice(0, dot);
    const raw = token.slice(dot + 1);
    let row: Awaited<
      ReturnType<typeof this.prisma.passwordReset.findUnique>
    > = null;
    try {
      row = await this.prisma.passwordReset.findUnique({ where: { id } });
    } catch {
      throw new UnauthorizedException('无效重置令牌');
    }
    if (!row || row.used || row.expiresAt.getTime() < Date.now()) {
      throw new UnauthorizedException('重置令牌无效或已过期');
    }
    const ok = await bcrypt.compare(raw, row.tokenHash);
    if (!ok) throw new UnauthorizedException('重置令牌不匹配');
    const passwordHash = await bcrypt.hash(newPassword, 10);
    // 原子消费令牌：仅当 used=false 时置位；并发第二次 count=0 被拒（防 TOCTOU 复用）。
    const consumed = await this.prisma.passwordReset.updateMany({
      where: { id, used: false },
      data: { used: true },
    });
    if (consumed.count !== 1) {
      throw new UnauthorizedException('重置令牌已被使用');
    }
    await this.prisma.$transaction([
      this.prisma.user.update({
        where: { id: row.userId },
        data: { passwordHash },
      }),
      this.prisma.refreshToken.updateMany({
        where: { userId: row.userId },
        data: { revoked: true },
      }),
    ]);
  }

  private async issue(userId: string, email: string): Promise<Tokens> {
    const access = await this.jwt.signAsync(
      { sub: userId, email },
      { secret: this.accessSecret, expiresIn: this.accessTtl },
    );
    // refresh token 用随机串，哈希入库；返回 `<rowId>.<raw>` 便于刷新时按 id 直接定位。
    const raw = randomBytes(32).toString('hex');
    const tokenHash = await bcrypt.hash(raw, 10);
    const expiresAt = new Date(
      Date.now() + this.refreshTtlDays * 24 * 3600 * 1000,
    );
    const row = await this.prisma.refreshToken.create({
      data: { userId, tokenHash, expiresAt },
    });
    return { access, refresh: `${row.id}.${raw}` };
  }

  private async verifyRefresh(token: string) {
    const dot = token.indexOf('.');
    if (dot < 0) throw new UnauthorizedException('bad refresh token');
    const id = token.slice(0, dot);
    const raw = token.slice(dot + 1);
    // id 是 @db.Uuid 列：非法 UUID 会让 findUnique 抛 Prisma 错（500）。统一吞成 401。
    let row: Awaited<
      ReturnType<typeof this.prisma.refreshToken.findUnique>
    > = null;
    try {
      row = await this.prisma.refreshToken.findUnique({ where: { id } });
    } catch {
      throw new UnauthorizedException('refresh token invalid');
    }
    if (!row || row.revoked || row.expiresAt.getTime() < Date.now()) {
      throw new UnauthorizedException('refresh token invalid/expired');
    }
    const ok = await bcrypt.compare(raw, row.tokenHash);
    if (!ok) throw new UnauthorizedException('refresh token mismatch');
    const user = await this.prisma.user.findUnique({
      where: { id: row.userId },
    });
    if (!user) throw new UnauthorizedException('user gone');
    return { row, user };
  }
}
