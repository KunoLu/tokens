import { redirect } from "next/navigation";
import { getSession } from "@/lib/auth/session";

export default async function ProfilePage() {
  const session = await getSession();

  if (session) {
    redirect(`/u/${session.username}`);
  } else {
    // Comes back here after sign-in, which then forwards to /u/<name>.
    // Without returnTo the login page falls back to /leaderboard and the user
    // never reaches the profile they asked for.
    redirect("/login?returnTo=/profile");
  }
}
