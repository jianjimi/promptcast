import { Module } from '@nestjs/common';
import { SyncService } from './sync.service';
import { SyncController } from './sync.controller';
import { GcService } from './gc.service';

@Module({
  controllers: [SyncController],
  providers: [SyncService, GcService],
})
export class SyncModule {}
