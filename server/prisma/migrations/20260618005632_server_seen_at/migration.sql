-- AlterTable
ALTER TABLE "sync_records" ADD COLUMN     "server_seen_at" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP;
