import type { Metadata } from "next";
import { sanitizeAuthReturnTo } from "@/lib/auth/returnTo";
import ForgotPasswordForm from "./ForgotPasswordForm";

export const metadata: Metadata = {
  title: "Forgot password - Tokens",
  description: "Request a password reset link.",
};

interface PageProps {
  searchParams: Promise<{ [key: string]: string | string[] | undefined }>;
}

export default async function ForgotPasswordPage({ searchParams }: PageProps) {
  const params = await searchParams;
  const returnTo = sanitizeAuthReturnTo(
    typeof params.returnTo === "string" ? params.returnTo : null
  );
  return <ForgotPasswordForm returnTo={returnTo} />;
}
