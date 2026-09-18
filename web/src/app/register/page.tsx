import type { Metadata } from "next";
import { sanitizeAuthReturnTo } from "@/lib/auth/returnTo";
import RegisterForm from "./RegisterForm";

export const metadata: Metadata = {
  title: "Create account - Tokens",
  description: "Register with email and password to track your AI token usage.",
};

interface PageProps {
  searchParams: Promise<{ [key: string]: string | string[] | undefined }>;
}

export default async function RegisterPage({ searchParams }: PageProps) {
  const params = await searchParams;
  const returnTo = sanitizeAuthReturnTo(
    typeof params.returnTo === "string" ? params.returnTo : null
  );
  return <RegisterForm returnTo={returnTo} />;
}
