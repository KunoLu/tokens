import type { Metadata } from "next";
import { sanitizeAuthReturnTo } from "@/lib/auth/returnTo";
import LoginForm from "./LoginForm";

export const metadata: Metadata = {
  title: "Sign in - Tokens",
  description: "Sign in with your email and password.",
};

interface PageProps {
  searchParams: Promise<{ [key: string]: string | string[] | undefined }>;
}

export default async function LoginPage({ searchParams }: PageProps) {
  const params = await searchParams;
  const returnTo = sanitizeAuthReturnTo(
    typeof params.returnTo === "string" ? params.returnTo : null
  );
  return <LoginForm returnTo={returnTo} />;
}
