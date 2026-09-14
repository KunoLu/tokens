"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useI18n } from "@/lib/i18n";

/**
 * Shared confirmation dialog. With `confirmName` set, the confirm button
 * stays disabled until the typed text matches the name exactly (the gate
 * for destructive team/group operations); without it the dialog is a plain
 * confirm. `onConfirm` runs the mutation, surfaces any error itself, and
 * returns `null` on success — the dialog closes.
 */
export function ConfirmDialog({
  open,
  onOpenChange,
  title,
  description,
  confirmName,
  confirmLabel,
  destructive = true,
  onConfirm,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: string;
  description?: string;
  confirmName?: string;
  confirmLabel: string;
  destructive?: boolean;
  onConfirm: () => Promise<string | null>;
}) {
  const { t } = useI18n();
  const [value, setValue] = useState("");
  const [busy, setBusy] = useState(false);

  const close = (next: boolean) => {
    if (busy) return;
    if (!next) setValue("");
    onOpenChange(next);
  };

  const submit = async () => {
    setBusy(true);
    const error = await onConfirm();
    setBusy(false);
    if (error) return;
    setValue("");
    onOpenChange(false);
  };

  return (
    <Dialog open={open} onOpenChange={close}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          {description && <DialogDescription>{description}</DialogDescription>}
        </DialogHeader>
        {confirmName !== undefined && (
          <div className="flex flex-col gap-2">
            <Label htmlFor="confirm-name-input">
              {t("dlg.typeToConfirm", { name: confirmName })}
            </Label>
            <Input
              id="confirm-name-input"
              value={value}
              onChange={(e) => setValue(e.target.value)}
              placeholder={confirmName}
              autoComplete="off"
            />
          </div>
        )}
        <DialogFooter>
          <DialogClose render={<Button variant="outline" disabled={busy} />}>
            {t("dlg.cancel")}
          </DialogClose>
          <Button
            variant={destructive ? "destructive" : "default"}
            disabled={
              busy ||
              (confirmName !== undefined && value.trim() !== confirmName)
            }
            onClick={() => void submit()}
          >
            {confirmLabel}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
