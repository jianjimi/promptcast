import { Body, Controller, HttpCode, Post, UseGuards } from '@nestjs/common';
import { JwtAuthGuard } from '../auth/jwt-auth.guard';
import { CurrentUser, AuthUser } from '../auth/current-user.decorator';
import { SyncService } from './sync.service';
import { PullDto } from './dto/pull.dto';
import { PushDto } from './dto/push.dto';

@Controller('sync')
@UseGuards(JwtAuthGuard)
export class SyncController {
  constructor(private readonly sync: SyncService) {}

  @Post('pull')
  @HttpCode(200)
  pull(@CurrentUser() user: AuthUser, @Body() dto: PullDto) {
    return this.sync.pull(user.userId, dto.since_cursor ?? 0, dto.limit ?? 500);
  }

  @Post('push')
  @HttpCode(200)
  push(@CurrentUser() user: AuthUser, @Body() dto: PushDto) {
    return this.sync.push(user.userId, dto.changes);
  }
}
