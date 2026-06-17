import { Type } from 'class-transformer';
import {
  IsArray,
  IsIn,
  IsInt,
  IsObject,
  IsOptional,
  IsUUID,
  ValidateNested,
} from 'class-validator';

export class ChangeDto {
  @IsIn(['prompt', 'folder', 'tag', 'site'])
  entity!: string;

  @IsUUID()
  uuid!: string;

  @IsInt()
  updated_at!: number;

  @IsOptional()
  @IsInt()
  deleted_at?: number | null;

  @IsObject()
  data!: Record<string, unknown>;
}

export class PushDto {
  @IsArray()
  @ValidateNested({ each: true })
  @Type(() => ChangeDto)
  changes!: ChangeDto[];
}
