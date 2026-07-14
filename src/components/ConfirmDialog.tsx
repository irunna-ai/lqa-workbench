import { useEffect, useRef } from "react";

interface ConfirmDialogProps {
  open: boolean;
  title: string;
  message: string;
  detail?: string;
  confirmLabel?: string;
  cancelLabel?: string;
  variant?: "danger" | "warning" | "default";
  onConfirm: () => void;
  onCancel: () => void;
}

export default function ConfirmDialog({
  open,
  title,
  message,
  detail,
  confirmLabel = "Confirm",
  cancelLabel = "Cancel",
  variant = "default",
  onConfirm,
  onCancel,
}: ConfirmDialogProps) {
  const confirmRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (open) {
      setTimeout(() => confirmRef.current?.focus(), 50);
    }
  }, [open]);

  useEffect(() => {
    if (!open) return;
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onCancel();
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [open, onCancel]);

  if (!open) return null;

  const icon = variant === "danger" ? "⚠" : variant === "warning" ? "⚠" : "❓";

  return (
    <div className="dialog-overlay" onClick={onCancel}>
      <div className="dialog confirm-dialog" onClick={(e) => e.stopPropagation()} role="alertdialog" aria-labelledby="confirm-title">
        <div className="dialog-header">
          <h2 id="confirm-title">{title}</h2>
          <button className="btn-icon" onClick={onCancel} aria-label="Close">×</button>
        </div>
        <div className="dialog-body">
          <div className="confirm-icon">{icon}</div>
          <p className="confirm-message">{message}</p>
          {detail && <p className="confirm-detail">{detail}</p>}
        </div>
        <div className="dialog-footer">
          <button className="btn-secondary" onClick={onCancel}>{cancelLabel}</button>
          <button
            ref={confirmRef}
            className={variant === "danger" ? "btn-danger" : "btn-primary"}
            onClick={onConfirm}
          >
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}