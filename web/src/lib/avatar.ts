/**
 * Avatar URL for a user: their stored avatar when present, otherwise an
 * inline SVG of the username's two-letter initials — the same letters the
 * AvatarFallback beneath the image renders, so a broken or missing avatar
 * never changes the page's shape.
 */
export function avatarUrlFor(user: {
  username: string;
  avatarUrl: string | null;
}): string {
  if (user.avatarUrl) {
    return user.avatarUrl;
  }
  const initials = user.username.slice(0, 2).toUpperCase();
  const svg =
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64">` +
    `<rect width="64" height="64" fill="#64748b"/>` +
    `<text x="32" y="42" font-family="sans-serif" font-size="26" font-weight="600" fill="#ffffff" text-anchor="middle">${initials}</text>` +
    `</svg>`;
  return `data:image/svg+xml;utf8,${encodeURIComponent(svg)}`;
}
