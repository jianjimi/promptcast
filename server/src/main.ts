import { NestFactory } from '@nestjs/core';
import { ValidationPipe } from '@nestjs/common';
import { AppModule } from './app.module';

const DEV_SECRET = 'dev-access-secret-change-me';

/**
 * 安全闸：生产环境(NODE_ENV=production)必须显式设置一个强 JWT_SECRET，
 * 否则拒绝启动。HS256 是对称密钥 —— 用公开的 dev 默认值等于任何人都能伪造任意用户的
 * access token、越权读写他人数据。开发环境允许用 dev 默认值但打印告警。
 */
function assertSecrets() {
  const secret = process.env.JWT_SECRET;
  const isProd = process.env.NODE_ENV === 'production';
  if (isProd && (!secret || secret === DEV_SECRET)) {
    throw new Error(
      'JWT_SECRET 未设置或仍是 dev 默认值；生产环境必须设置一个强随机密钥后再启动。',
    );
  }
  if (!secret || secret === DEV_SECRET) {
    // eslint-disable-next-line no-console
    console.warn(
      '[security] 正在使用公开的 dev JWT_SECRET —— 仅限本地开发，切勿用于生产。',
    );
  }
}

async function bootstrap() {
  assertSecrets();
  const app = await NestFactory.create(AppModule);
  app.useGlobalPipes(new ValidationPipe({ whitelist: true, transform: true }));
  const port = Number(process.env.PORT ?? 3000);
  await app.listen(port, '0.0.0.0');
  // eslint-disable-next-line no-console
  console.log(`PromptCast sync server listening on :${port}`);
}
void bootstrap();
