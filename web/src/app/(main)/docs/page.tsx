import type { Metadata } from "next";
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
import { LOCALE_COOKIE, parseLocale, t, type Locale, type TranslationKey } from "@/lib/i18n";
import { cookies } from "next/headers";
import { cn } from "@/lib/utils";
import { SITE_URL } from "@/lib/site";


export const metadata: Metadata = {
  title: "Docs - Tokens",
  description: "Install the Tokens CLI on macOS, Linux or Windows.",
  openGraph: {
    title: "Docs — Tokens",
    description: "Install the Tokens CLI.",
    url: `${SITE_URL}/docs`,
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

// Commands stay literal; the notes beside them are dictionary keys resolved
// at render, so the arrays keep working as plain module constants.
const MACOS = [
  { command: BREW_INSTALL_COMMAND, note: "docs.note.install" },
  { command: "tokens login", note: "docs.note.signIn" },
  // brew services start tokens would bypass the shell alias and submit
  // upstream; serve in the foreground honors TOKENS_API_URL.
  { command: "tokens serve", note: "docs.note.submitAuto" },
] as const;

// URL-bearing rows are computed in the component from the site origin so the
// docs page always points at the deployment it is served from.
const LINUX = [
  { command: "tokens login", note: "docs.note.signIn" },
  { command: "tokens serve", note: "docs.note.submitAuto" },
] as const;


const WINDOWS = [
  { command: "tokens login", note: "docs.note.signIn" },
  { command: "tokens submit", note: "docs.note.submitUsage" },
] as const;

const EVERYDAY = [
  { command: "tokens login", note: "docs.note.authenticate" },
  { command: "tokens submit", note: "docs.note.sendNow" },
  { command: "tokens serve", note: "docs.note.keepSubmitting" },
  { command: "tokens status", note: "docs.note.whatSubmitted" },
  { command: "tokens help", note: "docs.note.everythingElse" },
] as const;

function localize(
  locale: Locale,
  commands: ReadonlyArray<{ command: string; note: TranslationKey }>
): DocCommand[] {
  return commands.map(({ command, note }) => ({
    command,
    note: t(locale, note),
  }));
}

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

export default async function DocsPage() {
  const locale = parseLocale((await cookies()).get(LOCALE_COOKIE)?.value);
  // The install commands must point at the site serving this page.
  const siteUrl = SITE_URL;
  // Pre-install scripts execute straight from this repo's public GitHub root,
  // pinned to the immutable commit where they were reviewed — never a moving
  // branch. Bump the SHA deliberately when the scripts change.
  const preinstallRaw = "https://raw.githubusercontent.com/KunoLu/tokens/4921ccbed1f4286e75c35f676c400ec8f83012a6";
  const preinstallSh = `curl -fsSL ${preinstallRaw}/pre-install-tokens.sh | bash -s -- ${siteUrl}`;
  const preinstallPs1 = `iex "& { $(irm ${preinstallRaw}/pre-install-tokens.ps1) } -Site ${siteUrl}"`;

  return (
    <main
      className={cn(CONTAINER, "pb-24 pt-10 sm:pt-14")}
      id="main-content"
    >
      <div className="mx-auto w-full max-w-[860px]">
      <PageHeader
        title={t(locale, "nav.docs")}
        description={t(locale, "docs.desc")}
      />

        <div className="flex flex-col gap-12">
        <Section
          id="cli"
          title={t(locale, "docs.cliTitle")}
          description={t(locale, "docs.cliDesc")}
        >
          <Tabs defaultValue="macos">
            <TabsList>
              {/* Icons are checked in rather than hotlinked; currentColor keeps
                  the monochrome marks legible in both themes. */}
              <TabsTrigger value="macos">
                <OsIcon name="macos" />
                {t(locale, "docs.os.macos")}
              </TabsTrigger>
              <TabsTrigger value="linux">
                <OsIcon name="linux" />
                {t(locale, "docs.os.linux")}
              </TabsTrigger>
              <TabsTrigger value="windows">
                <OsIcon name="windows" />
                {t(locale, "docs.os.windows")}
              </TabsTrigger>
            </TabsList>

            <TabsContent value="macos" className="mt-4 flex flex-col gap-3">
              <CommandBlock
                commands={[
                  ...localize(locale, [MACOS[0]]),
                  { command: preinstallSh, note: t(locale, "docs.note.preinstall") },
                  ...localize(locale, MACOS.slice(1)),
                ]}
              />
              <p className="text-sm leading-relaxed text-muted-foreground">
                {t(locale, "docs.reloadNote")}
              </p>
              <p className="text-sm leading-relaxed text-muted-foreground">
                <code className="font-mono text-[13px]">tokens serve</code>{" "}
                {t(locale, "docs.macosNote", { site: siteUrl })}{" "}
                <a
                  href="https://github.com/KunoLu/tokens/blob/4921ccbed1f4286e75c35f676c400ec8f83012a6/docs/deploy/tokens-cli-usage.md"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-foreground underline underline-offset-4"
                >
                  {t(locale, "docs.cliGuideLink")}
                </a>
              </p>
            </TabsContent>

            <TabsContent value="linux" className="mt-4 flex flex-col gap-3">
              <CommandBlock
                commands={[
                  { command: `curl -fsSL ${siteUrl}/install.sh | sh`, note: t(locale, "docs.note.install") },
                  { command: preinstallSh, note: t(locale, "docs.note.preinstall") },
                  ...localize(locale, LINUX),
                ]}
              />
              <p className="text-sm leading-relaxed text-muted-foreground">
                {t(locale, "docs.reloadNote")}
              </p>
              <p className="text-sm leading-relaxed text-muted-foreground">
                <code className="font-mono text-[13px]">tokens serve</code>{" "}
                {t(locale, "docs.linuxNote")}
              </p>
            </TabsContent>

            <TabsContent value="windows" className="mt-4 flex flex-col gap-3">
              <CommandBlock
                commands={[
                  { command: preinstallPs1, note: t(locale, "docs.note.preinstall") },
                  ...localize(locale, WINDOWS),
                ]}
              />
              <p className="text-sm leading-relaxed text-muted-foreground">
                {t(locale, "docs.reloadNote")}
              </p>
              <p className="text-sm leading-relaxed text-muted-foreground">
                {t(locale, "docs.windowsNote")}
              </p>
            </TabsContent>
          </Tabs>
        </Section>


        <Section
          id="usage"
          title={t(locale, "docs.usageTitle")}
          description={t(locale, "docs.usageDesc")}
        >
          <CommandBlock commands={localize(locale, EVERYDAY)} />
        </Section>


        <Section
          id="clients"
          title={t(locale, "docs.clientsTitle")}
          description={t(locale, "docs.clientsDesc")}
        >
          <div className="flex flex-col gap-4">
            <p className="text-sm leading-relaxed text-muted-foreground">
              {t(locale, "docs.clientsDetected", { n: CLIENT_GRID.length })}
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
