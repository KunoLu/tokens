import type { Metadata } from "next";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { CommandBlock, type DocCommand } from "@/components/docs/CommandBlock";
import { BrandGlyph } from "@/components/profile/ModelIcon";
import { CONTAINER } from "@/components/layout/Container";
import {
  BREW_INSTALL_COMMAND,
  SOURCE_DISPLAY_NAMES,
  SOURCE_LOGOS,
  SUPPORTED_CLIENTS,
} from "@/lib/constants";
import { PageHeader } from "@/components/layout/PageHeader";
import { cn } from "@/lib/utils";

export const metadata: Metadata = {
  title: "Docs - Tokens",
  description: "Install the Tokens CLI on macOS, Linux or Windows.",
  openGraph: {
    title: "Docs — Tokens",
    description: "Install the Tokens CLI.",
    url: "https://tokens.ci",
    siteName: "Tokens",
    images: [
      {
        url: `/api/og?title=Docs&subtitle=Install+the+Tokens+CLI.`,
        width: 1200,
        height: 630,
      },
    ],
  },
  twitter: { card: "summary_large_image" },
};

/**
 * What the Supported clients grid renders.
 *
 * The scanned clients, plus Orca. Orca is not a client of its own — it runs
 * Codex against an isolated runtime home, and the scanner finds that directory
 * and counts what is there under Codex. It still belongs in this list, because
 * the question a reader has is "will my setup be picked up", and for Orca the
 * answer is yes.
 */
const CLIENT_GRID: ReadonlyArray<{ id: string; name: string; logo: string }> = [
  ...SUPPORTED_CLIENTS.map((client) => ({
    id: client,
    name: SOURCE_DISPLAY_NAMES[client],
    logo: SOURCE_LOGOS[client],
  })),
  { id: "orca", name: "Orca", logo: "/clients/client-orca.png" },
].sort((a, b) => a.name.toLowerCase().localeCompare(b.name.toLowerCase()));

const MACOS: readonly DocCommand[] = [
  { command: BREW_INSTALL_COMMAND, note: "install" },
  { command: "tokens login", note: "link your GitHub account" },
  { command: "brew services start tokens", note: "submit automatically" },
];

const LINUX: readonly DocCommand[] = [
  { command: "curl -fsSL https://tokens.ci/install.sh | sh", note: "install" },
  { command: "tokens login", note: "link your GitHub account" },
  { command: "tokens serve", note: "submit automatically" },
];

const WINDOWS: readonly DocCommand[] = [
  { command: "bunx tokens-cli@latest login", note: "link your GitHub account" },
  { command: "bunx tokens-cli@latest submit", note: "submit your usage" },
];

/**
 * Platform marks. Apple and Microsoft come from the shared brand set so they
 * follow the text colour; Linux has no entry there, so Tux is drawn inline.
 */
function OsIcon({ name }: { name: "macos" | "linux" | "windows" }) {
  if (name === "linux") {
    return (
      <svg
        viewBox="0 0 24 24"
        width={14}
        height={14}
        fill="currentColor"
        aria-hidden="true"
        className="shrink-0"
        data-icon="inline-start"
      >
        <path d="M12 2c-2.4 0-3.7 1.9-3.7 4.3 0 .9.1 1.7.1 2.4 0 .8-.5 1.5-1.1 2.4C6.4 12.4 5 14.3 5 16.4c0 1 .3 1.8.9 2.4-.3.4-.5.9-.5 1.4 0 1.1 1 1.8 2.4 1.8 1 0 1.8-.3 2.4-.8.5.1 1.1.2 1.8.2s1.3-.1 1.8-.2c.6.5 1.4.8 2.4.8 1.4 0 2.4-.7 2.4-1.8 0-.5-.2-1-.5-1.4.6-.6.9-1.4.9-2.4 0-2.1-1.4-4-2.3-5.3-.6-.9-1.1-1.6-1.1-2.4 0-.7.1-1.5.1-2.4C15.7 3.9 14.4 2 12 2zm-1.5 3.3c.4 0 .8.5.8 1.1s-.4 1.1-.8 1.1-.8-.5-.8-1.1.4-1.1.8-1.1zm3 0c.4 0 .8.5.8 1.1s-.4 1.1-.8 1.1-.8-.5-.8-1.1.4-1.1.8-1.1zM12 8.4c.9 0 1.7.4 1.7.8 0 .2-.2.4-.5.6l-1 .6c-.1.1-.3.1-.4 0l-1-.6c-.3-.2-.5-.4-.5-.6 0-.4.8-.8 1.7-.8z" />
      </svg>
    );
  }
  return (
    <BrandGlyph
      slug={name === "macos" ? "apple" : "microsoft"}
      size={14}
      className="fill-current"
    />
  );
}

function Section({
  id,
  title,
  description,
  children,
}: {
  id: string;
  title: string;
  description?: string;
  children: React.ReactNode;
}) {
  return (
    <section id={id} className="scroll-mt-20">
      <h2 className="text-lg font-semibold tracking-tight">{title}</h2>
      {description && (
        <p className="mt-1.5 text-sm leading-relaxed text-muted-foreground">
          {description}
        </p>
      )}
      <div className="mt-4">{children}</div>
    </section>
  );
}

export default function DocsPage() {
  return (
    <main
      className={cn(CONTAINER, "pb-24 pt-10 sm:pt-14")}
      id="main-content"
    >
      <div className="mx-auto w-full max-w-[860px]">
      <PageHeader
        title="Docs"
        description="Get your AI coding usage onto the leaderboard from the terminal."
      />

        <div className="flex flex-col gap-12">
        <Section
          id="cli"
          title="Install the CLI"
          description="The CLI scans the AI coding clients already installed on your machine, totals the usage locally, and submits only the totals."
        >
          <Tabs defaultValue="macos">
            <TabsList>
              {/* Icons are checked in rather than hotlinked; currentColor keeps
                  the monochrome marks legible in both themes. */}
              <TabsTrigger value="macos">
                <OsIcon name="macos" />
                macOS
              </TabsTrigger>
              <TabsTrigger value="linux">
                <OsIcon name="linux" />
                Linux
              </TabsTrigger>
              <TabsTrigger value="windows">
                <OsIcon name="windows" />
                Windows
              </TabsTrigger>
            </TabsList>

            <TabsContent value="macos" className="mt-4 flex flex-col gap-3">
              <CommandBlock commands={MACOS} />
              <p className="text-sm leading-relaxed text-muted-foreground">
                <code className="font-mono text-[13px]">brew services</code>{" "}
                keeps a background agent running, so your usage stays current
                without you thinking about it.
              </p>
            </TabsContent>

            <TabsContent value="linux" className="mt-4 flex flex-col gap-3">
              <CommandBlock commands={LINUX} />
              <p className="text-sm leading-relaxed text-muted-foreground">
                <code className="font-mono text-[13px]">tokens serve</code> runs
                the submitter in the foreground; pair it with a systemd unit to
                keep it alive across reboots.
              </p>
            </TabsContent>

            <TabsContent value="windows" className="mt-4 flex flex-col gap-3">
              <CommandBlock commands={WINDOWS} />
              <p className="text-sm leading-relaxed text-muted-foreground">
                Runs straight from npm, so nothing is installed globally. Use a
                Scheduled Task to submit on a timer.
              </p>
            </TabsContent>
          </Tabs>
        </Section>


        <Section
          id="usage"
          title="Everyday use"
          description="Five commands cover the whole workflow."
        >
          <CommandBlock
            commands={[
              { command: "tokens login", note: "authenticate" },
              { command: "tokens submit", note: "send usage now" },
              { command: "tokens serve", note: "keep submitting in the background" },
              { command: "tokens status", note: "what has been submitted" },
              { command: "tokens help", note: "everything else" },
            ]}
          />
        </Section>

        <Section
          id="verified"
          title="The verified badge"
          description="A small check next to a name on the leaderboard. It says the account is a real, findable person — nothing more."
        >
          <div className="flex flex-col gap-4">
            <Card size="sm">
              <CardHeader>
                <CardTitle className="text-sm">How to get it</CardTitle>
              </CardHeader>
              <CardContent className="text-sm leading-relaxed text-muted-foreground">
                Add at least <strong className="font-medium text-foreground">two social
                links</strong> to your GitHub profile — the &ldquo;Social accounts&rdquo;
                fields in{" "}
                <a
                  href="https://github.com/settings/profile"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="underline underline-offset-4 hover:text-foreground"
                >
                  GitHub profile settings
                </a>
                . Any two count: a personal site, X, LinkedIn, Mastodon, YouTube.
                That is the whole rule.
              </CardContent>
            </Card>

            <Card size="sm">
              <CardHeader>
                <CardTitle className="text-sm">When it appears</CardTitle>
              </CardHeader>
              <CardContent className="text-sm leading-relaxed text-muted-foreground">
                Links are re-read once a day, at 03:20 UTC. Adding them now means
                the badge appears on the next run rather than immediately —
                signing out and back in does not speed it up. Dropping below two
                links removes it on the same schedule.
              </CardContent>
            </Card>

            <Card size="sm">
              <CardHeader>
                <CardTitle className="text-sm">Why two links</CardTitle>
              </CardHeader>
              <CardContent className="text-sm leading-relaxed text-muted-foreground">
                A leaderboard attracts throwaway accounts. Filling in two social
                fields is trivial for someone who already exists online and
                tedious to fake at scale, which is all the badge claims. It is
                not an identity check, and it has no effect on ranking —
                inflated numbers are handled separately, by the submission
                checks.
              </CardContent>
            </Card>
          </div>
        </Section>

        <Section
          id="clients"
          title="Supported clients"
          description="The CLI scans whatever is already on your machine — nothing to configure per client."
        >
          <div className="flex flex-col gap-4">
            <p className="text-sm leading-relaxed text-muted-foreground">
              All {CLIENT_GRID.length} of these are detected automatically — if
              it is installed and has written sessions, it is counted.
            </p>

            {/* A plain responsive grid rather than a table: these are names,
                not tabular data, and at 39 entries a table would force either
                a very long single column or horizontal scrolling on a phone. */}
            <ul className="grid grid-cols-2 gap-x-4 gap-y-2.5 sm:grid-cols-3 lg:grid-cols-4">
              {CLIENT_GRID.map(({ id, name, logo }) => (
                <li key={id} className="flex min-w-0 items-center gap-2.5">
                  {/* Plain <img>: these are small third-party marks in a mix of
                      png/jpg/webp/svg, already served from our own origin, so
                      the optimizer would add requests without adding much. */}
                  <img
                    src={logo}
                    alt=""
                    width={20}
                    height={20}
                    loading="lazy"
                    className="size-5 shrink-0 rounded-[4px] object-contain"
                  />
                  <span className="truncate text-sm text-muted-foreground">
                    {name}
                  </span>
                </li>
              ))}
            </ul>
          </div>
        </Section>


        </div>
      </div>
    </main>
  );
}
