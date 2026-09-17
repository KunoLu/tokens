/** 部署自身 origin（https://host[:port]）；任何用户可见 URL 都从它派生，不写死上游域名。 */
export const SITE_URL = (process.env.NEXT_PUBLIC_URL || "http://localhost:3000").replace(/\/+$/, "");
/** 部署 hostname，供 OG / embed / 法律页等品牌展示。 */
export const SITE_HOST = new URL(SITE_URL).host;
