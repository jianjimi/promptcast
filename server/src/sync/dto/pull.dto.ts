import { IsInt, IsOptional, Max, Min } from 'class-validator';

export class PullDto {
  @IsOptional()
  @IsInt()
  @Min(0)
  since_cursor?: number;

  @IsOptional()
  @IsInt()
  @Min(1)
  @Max(1000)
  limit?: number;
}
