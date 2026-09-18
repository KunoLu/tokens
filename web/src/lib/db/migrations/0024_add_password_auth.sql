ALTER TABLE "users" ADD COLUMN IF NOT EXISTS "password_hash" varchar(255);--> statement-breakpoint
ALTER TABLE "users" ADD COLUMN IF NOT EXISTS "email_verified_at" timestamp with time zone;--> statement-breakpoint
ALTER TABLE "users" ALTER COLUMN "github_id" DROP NOT NULL;--> statement-breakpoint
CREATE UNIQUE INDEX IF NOT EXISTS "users_email_lower_unique"
  ON "users" (lower("email")) WHERE "email" IS NOT NULL;--> statement-breakpoint
CREATE TABLE IF NOT EXISTS "email_verification_tokens" (
  "id"         uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  "user_id"    uuid NOT NULL,
  "token_hash" varchar(64) NOT NULL,
  "purpose"    varchar(20) NOT NULL,          -- 'verify_email' | 'reset_password'
  "expires_at" timestamp with time zone NOT NULL,
  "consumed_at" timestamp with time zone,
  "created_at" timestamp with time zone DEFAULT now() NOT NULL,
  CONSTRAINT "email_verification_tokens_token_hash_unique" UNIQUE("token_hash")
);--> statement-breakpoint
ALTER TABLE "email_verification_tokens"
  ADD CONSTRAINT "email_verification_tokens_user_id_users_id_fk"
  FOREIGN KEY ("user_id") REFERENCES "public"."users"("id")
  ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "idx_email_verification_tokens_user_purpose"
  ON "email_verification_tokens" ("user_id", "purpose");--> statement-breakpoint
CREATE INDEX IF NOT EXISTS "idx_email_verification_tokens_expires_at"
  ON "email_verification_tokens" ("expires_at");
