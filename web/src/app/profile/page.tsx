import { redirect } from "next/navigation";
import { getSession } from "@/lib/auth/session";
import { getTranslator } from "@/lib/i18n/server";
import { localizeHref } from "@/lib/i18n/config";

export default async function ProfilePage() {
  const [{ locale }, session] = await Promise.all([getTranslator(), getSession()]);

  if (session) {
    redirect(localizeHref(`/u/${session.username}`, locale));
  } else {
    // Comes back here after sign-in, which then forwards to /u/<name>.
    // Without returnTo the login page falls back to /leaderboard and the user
    // never reaches the profile they asked for. The return path keeps the
    // locale prefix, so someone who signed in from `/zh-CN/profile` is not
    // dropped on the English board afterwards.
    const returnTo = localizeHref("/profile", locale);
    redirect(`/login?returnTo=${encodeURIComponent(returnTo)}`);
  }
}
