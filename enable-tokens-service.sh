#!/usr/bin/env bash
#
# enable-tokens-service.sh — 在已指向自建站的 Linux 机器上启用常驻自动上报（systemd）。
#
# 做的事：
#   1. 确保 systemd 用户服务 tokens.service 存在（没有则跑官方 install.sh 生成）；
#   2. 写 drop-in override.conf 把 TOKENS_API_URL 指向本站——不改 base unit
#      （install.sh 重装会整体重写 tokens.service，drop-in 才能存活）；
#   3. 先配好 Environment 再 daemon-reload && enable --now——服务不会带着
#      上游地址开报；
#   4. 凭据检查：找不到 credentials.json 就提示先 login。
#
# 不做的事：不登录（device flow 必须用户本人授权）、不 logout——本脚本假设
# 你已按使用手册完成 preinstall + login，凭据直接复用。
#
# 用法（站点地址必须显式传，脚本没有默认域名）：
#   bash enable-tokens-service.sh https://tokens.example.com
#   TOKENS_SITE=https://tokens.example.com bash enable-tokens-service.sh
# 先看会做什么再实跑：bash enable-tokens-service.sh --dry-run https://...
# 卸载：systemctl --user disable --now tokens
#       rm ~/.config/systemd/user/tokens.service.d/override.conf

set -euo pipefail

info() { printf "==> %s\n" "$*"; }
ok()   { printf "✓ %s\n" "$*"; }
die()  { printf "✗ %s\n" "$*" >&2; exit 1; }

DRY_RUN=0
if [ "${1:-}" = "--dry-run" ]; then DRY_RUN=1; shift; fi

# --- 站点地址：与 pre-install-tokens.sh 同款校验（纯 origin 或精确 loopback）。
TOKENS_SITE="${1:-${TOKENS_SITE:-}}"
TOKENS_SITE="${TOKENS_SITE%/}"

if [ -z "$TOKENS_SITE" ] || [ "$TOKENS_SITE" = "https://tokens.example.com" ]; then
  die "缺少站点地址。用法：bash enable-tokens-service.sh https://<线上域名>"
fi
if ! printf '%s' "$TOKENS_SITE" | grep -Eq '^https://[A-Za-z0-9.-]+(:[0-9]+)?$' \
  && ! printf '%s' "$TOKENS_SITE" | grep -Eq '^http://(localhost|127\.0\.0\.1|\[::1\])(:[0-9]+)?$'; then
  die "站点地址只接受 https://<域名>[:端口]（本地联调仅 http://localhost / 127.0.0.1 / [::1]），收到：$TOKENS_SITE"
fi

if [ "$DRY_RUN" = 1 ]; then
  info "[dry-run] 跳过平台与依赖检查（实跑要求 Linux + systemctl + tokens CLI）"
else
  [ "$(uname -s)" = "Linux" ] || die "本脚本只支持 Linux（systemd）；macOS 常驻方式见使用手册"
  command -v systemctl >/dev/null 2>&1 || die "未找到 systemctl；可在前台跑 tokens serve 替代常驻"
  command -v tokens   >/dev/null 2>&1 || die "未找到 tokens CLI，先跑 pre-install-tokens.sh"
fi


info "目标站点：$TOKENS_SITE"

UNIT_DIR="$HOME/.config/systemd/user"
UNIT="$UNIT_DIR/tokens.service"
OVERRIDE_DIR="$UNIT_DIR/tokens.service.d"
OVERRIDE="$OVERRIDE_DIR/override.conf"

# --- 1. unit 不存在时用本站的 install.sh 生成（它只写 unit，不自动启动）。 ---
# 不用上游 s.ee 移动目标：脚本 URL 必须和 docs 页一样钉在本站部署上。
if [ -f "$UNIT" ]; then
  ok "tokens.service 已存在"
elif [ "$DRY_RUN" = 1 ]; then
  info "[dry-run] 将运行 $TOKENS_SITE/install.sh 生成 $UNIT"
else
  info "tokens.service 不存在，用本站 install.sh 生成"
  curl -fsSL "$TOKENS_SITE/install.sh" | bash
  [ -f "$UNIT" ] || die "install.sh 未生成 $UNIT"
fi

# --- 2. drop-in 指向本站（幂等：重跑只改写这一行 Environment）。 ---
if [ "$DRY_RUN" = 1 ]; then
  info "[dry-run] 将写入 ${OVERRIDE}：[Service] Environment=TOKENS_API_URL=$TOKENS_SITE"
else
  mkdir -p "$OVERRIDE_DIR"
  printf '[Service]\nEnvironment=TOKENS_API_URL=%s\n' "$TOKENS_SITE" > "$OVERRIDE"
  ok "已写入 $OVERRIDE"
fi

# --- 3. 先配环境再启动。 ---
if [ "$DRY_RUN" = 1 ]; then
  info "[dry-run] 将执行 systemctl --user daemon-reload && systemctl --user enable --now tokens"
else
  systemctl --user daemon-reload
  systemctl --user enable --now tokens
  ok "tokens.service 已启用并启动"
fi

# --- 4. 凭据检查：没有凭据，服务起了也只会 401。 ---
CRED_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/tokens"
if [ -f "$CRED_DIR/credentials.json" ]; then
  ok "找到登录凭据"
else
  info "未找到登录凭据（$CRED_DIR/credentials.json）——服务已启用，但请先重开终端跑 tokens login，否则上报会 401"
fi

if [ "$DRY_RUN" = 0 ]; then
  if [ "$(loginctl show-user "$USER" -p Linger 2>/dev/null | cut -d= -f2)" != "yes" ]; then
    info "如需注销后常驻：sudo loginctl enable-linger \"$USER\""
  fi
  info "日志：journalctl --user -u tokens -f"
fi

ok "完成"
