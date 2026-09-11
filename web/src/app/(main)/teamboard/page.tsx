import type { Metadata } from "next";
import { CONTAINER } from "@/components/layout/Container";
import { PageHeader } from "@/components/layout/PageHeader";
import { cn } from "@/lib/utils";

export const metadata: Metadata = {
  title: "Teamboard - Tokens",
  description: "Team rankings on Tokens.",
};

export default function TeamboardPage() {
  return (
    <main
      className={cn(CONTAINER, "pb-24 pt-10 sm:pt-14")}
      id="main-content"
    >
      <div className="mx-auto w-full max-w-[860px]">
        <PageHeader
          title="Teamboard"
          description="Team rankings will live here. Select a team to see its members."
        />
      </div>
    </main>
  );
}
