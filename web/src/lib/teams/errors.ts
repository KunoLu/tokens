export class TeamError extends Error {
  readonly status: number;
  readonly details?: string[];

  constructor(message: string, status: number, details?: string[]) {
    super(message);
    this.name = "TeamError";
    this.status = status;
    this.details = details;
  }
}

export function postgresErrorCode(err: unknown): string | undefined {
  if (err && typeof err === "object" && "code" in err && typeof err.code === "string") {
    return err.code;
  }
  return undefined;
}
