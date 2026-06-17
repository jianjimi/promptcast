import { Injectable } from '@nestjs/common';
import { PrismaService } from '../prisma/prisma.service';
import { ChangeDto } from './dto/push.dto';

export interface ChangeEnvelope {
  entity: string;
  uuid: string;
  updated_at: number;
  deleted_at: number | null;
  data: unknown;
  seq: number;
}

export interface PullResult {
  changes: ChangeEnvelope[];
  next_cursor: number;
  has_more: boolean;
}

export interface PushItemResult {
  uuid: string;
  applied: boolean;
  seq: number | null;
}

export interface PushResult {
  results: PushItemResult[];
  server_cursor: number;
}

@Injectable()
export class SyncService {
  constructor(private readonly prisma: PrismaService) {}

  /** 返回该用户 seq > since_cursor 的变更（含墓碑），按 seq 升序、最多 limit 条。 */
  async pull(
    userId: string,
    sinceCursor: number,
    limit: number,
  ): Promise<PullResult> {
    const rows = await this.prisma.syncRecord.findMany({
      where: { userId, seq: { gt: BigInt(sinceCursor) } },
      orderBy: { seq: 'asc' },
      take: limit,
    });
    const changes: ChangeEnvelope[] = rows.map((r) => ({
      entity: r.entity,
      uuid: r.uuid,
      updated_at: Number(r.updatedAt),
      deleted_at: r.deletedAt === null ? null : Number(r.deletedAt),
      data: r.data,
      seq: Number(r.seq),
    }));
    const next_cursor = changes.length
      ? changes[changes.length - 1].seq
      : sinceCursor;
    return { changes, next_cursor, has_more: changes.length === limit };
  }

  /**
   * 逐条 upsert + 服务端 LWW：仅当 incoming.updated_at >= 现有时才写，并取新的 seq。
   * 命中写入 → applied:true + 新 seq；被 LWW 拒（incoming 更旧）→ applied:false。
   * 客户端 push-then-pull，输掉 LWW 的记录下一拍 pull 会拿到权威版本。
   */
  async push(userId: string, changes: ChangeDto[]): Promise<PushResult> {
    const results: PushItemResult[] = [];
    for (const c of changes) {
      const deletedAt =
        c.deleted_at === undefined || c.deleted_at === null
          ? null
          : BigInt(c.deleted_at);
      const rows = await this.prisma.$queryRaw<{ seq: bigint }[]>`
        INSERT INTO sync_records (user_id, entity, uuid, updated_at, deleted_at, data, seq)
        VALUES (
          ${userId}::uuid, ${c.entity}, ${c.uuid}::uuid,
          ${BigInt(c.updated_at)}, ${deletedAt},
          ${JSON.stringify(c.data)}::jsonb, nextval('sync_records_seq')
        )
        ON CONFLICT (user_id, entity, uuid) DO UPDATE
          SET updated_at = EXCLUDED.updated_at,
              deleted_at = EXCLUDED.deleted_at,
              data       = EXCLUDED.data,
              seq        = nextval('sync_records_seq')
          WHERE EXCLUDED.updated_at >= sync_records.updated_at
        RETURNING seq
      `;
      if (rows.length > 0) {
        results.push({ uuid: c.uuid, applied: true, seq: Number(rows[0].seq) });
      } else {
        results.push({ uuid: c.uuid, applied: false, seq: null });
      }
    }
    const max = await this.prisma.syncRecord.aggregate({
      where: { userId },
      _max: { seq: true },
    });
    const server_cursor = max._max.seq ? Number(max._max.seq) : 0;
    return { results, server_cursor };
  }
}
