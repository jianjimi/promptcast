import { Injectable, Logger } from '@nestjs/common';
import { Cron, CronExpression } from '@nestjs/schedule';
import { PrismaService } from '../prisma/prisma.service';

// 墓碑保留 90 天：到这时所有设备早该拉过了，删掉防服务端无限增长。
// 比客户端 30 天保留更长，避免服务端先删、某台长期离线设备还没学到删除。
const RETENTION_MS = 90 * 24 * 60 * 60 * 1000;

@Injectable()
export class GcService {
  private readonly logger = new Logger(GcService.name);

  constructor(private readonly prisma: PrismaService) {}

  @Cron(CronExpression.EVERY_DAY_AT_3AM)
  async purge(): Promise<void> {
    // 用服务端时间 server_seen_at 作保留判据，而非客户端可被时钟跑飞的 deleted_at，
    // 否则一台时钟设到过去的设备删的记录会被立刻 GC、别的设备永远学不到这次删除。
    const cutoff = new Date(Date.now() - RETENTION_MS);
    const tombstones = await this.prisma.syncRecord.deleteMany({
      where: { deletedAt: { not: null }, serverSeenAt: { lt: cutoff } },
    });
    const resets = await this.prisma.passwordReset.deleteMany({
      where: { OR: [{ used: true }, { expiresAt: { lt: new Date() } }] },
    });
    if (tombstones.count || resets.count) {
      this.logger.log(
        `GC: purged ${tombstones.count} old tombstones, ${resets.count} stale password resets`,
      );
    }
  }
}
