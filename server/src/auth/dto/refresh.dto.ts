import { IsString } from 'class-validator';

export class RefreshDto {
  @IsString()
  refresh!: string;
}
