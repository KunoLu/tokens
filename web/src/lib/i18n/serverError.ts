import type { Translate } from "./I18nProvider";
import { LOCALE_COOKIE, parseLocale } from "./locale";
import { t } from "./t";
import type { TranslationKey } from "./t";

/** Exact English `{ error }` / TeamError / password-detail strings from APIs. */
const EXACT: Record<string, TranslationKey> = {
  "Invalid email or password": "auth.login.invalidCredentials",
  "Account banned": "banned.badge",
  Forbidden: "auth.forbidden",
  "Too many requests": "auth.tooMany",
  "Invalid email address": "auth.invalidEmail",
  "Username must be 1-39 characters of letters, digits or hyphens":
    "auth.usernameRules",
  "Email is already registered": "auth.emailTaken",
  "Username is already taken": "auth.usernameTaken",
  "Email or username is already taken": "auth.emailOrUsernameTaken",
  "Password does not meet the requirements": "auth.password.requirements",
  "At least 8 characters": "auth.password.min",
  "At least one uppercase letter": "auth.password.upper",
  "At least one lowercase letter": "auth.password.lower",
  "At least one special character": "auth.password.special",
  "Invalid or expired reset link": "auth.reset.invalidLink",
  "Invalid or expired verification link": "auth.verify.invalidLink",
  "Not authenticated": "auth.notAuthenticated",
  "No email on this account": "auth.noEmail",
  "Failed to log in": "auth.login.failed",
  "Failed to register": "auth.register.failed",
  "Failed to reset password": "auth.reset.failed",
  "Failed to verify email": "auth.verify.failed",
  "Failed to resend verification email": "auth.verify.couldNotResend",
  "Failed to process request": "auth.somethingWrong",
  "Network error": "auth.network",
  "Not found": "teams.err.notFound",
  "Team is not active": "teams.err.notActive",
  "Name is required (max 100 characters)": "teams.err.nameRequired",
  "Leave your current team before creating another": "teams.err.leaveBeforeCreate",
  "Could not create team": "teams.err.createFailed",
  "A group with that name already exists": "teams.err.groupNameTaken",
  "visibility must be public or private": "teams.err.visibility",
  "Disband the team before deleting it": "teams.err.disbandBeforeDelete",
  "Team still has members": "teams.err.stillHasMembers",
  "Invite at least one person": "teams.err.inviteAtLeastOne",
  "Group is not available for auto-assign": "teams.err.groupAutoAssign",
  "Each invite needs a username or email": "teams.err.inviteNeedsIdentity",
  "User is already a member": "teams.err.alreadyMember",
  "Invite failed": "teams.err.inviteFailed",
  "Invalid role": "teams.err.invalidRole",
  "Transfer admin before changing your own role": "teams.err.transferBeforeRole",
  "Member not found": "teams.err.memberNotFound",
  "Subadmin limit reached": "teams.err.subadminCap",
  "Cannot remove the admin": "teams.err.cannotRemoveAdmin",
  "Transfer admin or disband the team before leaving": "teams.err.transferOrDisband",
  "Disband the group before deleting it": "teams.err.disbandGroupFirst",
  "Group still has members": "teams.err.groupHasMembers",
  "User must be a team member first": "teams.err.mustBeMember",
  "Verify your email before accepting this invitation":
    "teams.err.verifyEmailFirst",
  "Invitation has expired": "teams.err.inviteExpired",
  "Invitation is no longer pending": "teams.err.inviteNotPending",
  "Leave your current team before accepting": "teams.err.leaveBeforeAccept",
};

const UNKNOWN_USERNAME = "Unknown username: ";

export function localizeServerError(
  translate: Translate,
  message: string
): string {
  const key = EXACT[message];
  if (key) return translate(key);
  if (message.startsWith(UNKNOWN_USERNAME)) {
    return translate("teams.err.unknownUsername", {
      username: message.slice(UNKNOWN_USERNAME.length),
    });
  }
  const http = /^HTTP (\d+)$/.exec(message);
  if (http) {
    return translate("auth.httpError", { status: http[1] });
  }
  return translate("auth.somethingWrong");
}


export function localizeServerErrorList(
  translate: Translate,
  details: string[]
): string[] {
  return details.map((item) => localizeServerError(translate, item));
}

/** Client fetch helpers that are not React hooks. */
export function localizeServerErrorFromCookie(message: string): string {
  const raw =
    typeof document === "undefined"
      ? null
      : document.cookie
          .split("; ")
          .find((part) => part.startsWith(`${LOCALE_COOKIE}=`))
          ?.slice(LOCALE_COOKIE.length + 1);
  const locale = parseLocale(raw);
  return localizeServerError((key, vars) => t(locale, key, vars), message);
}
