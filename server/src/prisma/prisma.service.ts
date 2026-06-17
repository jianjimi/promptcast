import {
  Injectable,
  OnModuleInit,
  OnModuleDestroy,
} from '@nestjs/common';
import { PrismaClient } from '@prisma/client';

@Injectable()
export class PrismaService
  extends PrismaClient
  implements OnModuleInit, OnModuleDestroy
{
  async onModuleInit() {
    await this.$connect();
    // 单调游标序列：push 命中写入时取 nextval，使更新过的记录总能被 seq>cursor 重新拉到。
    // 放在表迁移之外、应用启动时幂等创建（见 schema.prisma 注释 / plan D5）。
    await this.$executeRawUnsafe(
      'CREATE SEQUENCE IF NOT EXISTS sync_records_seq',
    );
  }

  async onModuleDestroy() {
    await this.$disconnect();
  }
}
