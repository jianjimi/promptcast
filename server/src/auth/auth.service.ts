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
    const row = await this.prisma.refreshToken.findUnique({ where: { id } });
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
