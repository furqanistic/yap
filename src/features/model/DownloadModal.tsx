import { useEffect, useRef, useState } from "react";
import { Button, ConfirmDialog, Modal, ProgressBar } from "@/components/ui";
import { useModels } from "@/hooks/useModels";
import { formatBytes, formatEta, formatPercent, formatSpeed } from "@/lib/format";
import { modelsApi } from "@/lib/tauri";
import type { DiskSpace, ModelError, ModelId } from "@/types/models";
import "./DownloadModal.css";

interface DownloadModalProps {
  modelId: ModelId;
  open: boolean;
  onClose: () => void;
}

type View = "confirm" | "downloading" | "verifying" | "done" | "error";

/**
 * Confirm, follow and finish a model download. Closing (or Esc) only hides
 * the modal: the download keeps going and the Model page shows its progress.
 */
export function DownloadModal({ modelId, open, onClose }: DownloadModalProps) {
  const { find, download, cancel, use, activeId, speeds } = useModels();
  const model = find(modelId);
  const [startError, setStartError] = useState<ModelError | null>(null);
  const [starting, setStarting] = useState(false);
  const [confirmingCancel, setConfirmingCancel] = useState(false);
  const [space, setSpace] = useState<DiskSpace | null>(null);
  const primaryRef = useRef<HTMLButtonElement>(null);

  const state = model?.state;
  let view: View = "confirm";
  if (startError || state?.status === "failed") view = "error";
  else if (state?.status === "downloading") view = "downloading";
  else if (state?.status === "verifying") view = "verifying";
  else if (state?.status === "ready") view = "done";

  // Start fresh each time the modal opens.
  useEffect(() => {
    if (!open) return;
    setStartError(null);
    setConfirmingCancel(false);
  }, [open, modelId]);

  useEffect(() => {
    if (!open || view !== "confirm") return;
    let live = true;
    setSpace(null);
    modelsApi
      .diskSpace(modelId)
      .then((result) => live && setSpace(result))
      .catch(() => live && setSpace(null));
    return () => {
      live = false;
    };
  }, [open, view, modelId]);

  // Keep keyboard focus on the main action as the modal moves between steps.
  useEffect(() => {
    if (open) primaryRef.current?.focus();
  }, [open, view]);

  if (!model || !state) return null;

  const start = async () => {
    setStartError(null);
    setStarting(true);
    const error = await download(modelId);
    setStarting(false);
    if (error) setStartError(error);
  };

  const stopDownload = () => {
    cancel(modelId);
    setConfirmingCancel(false);
    onClose();
  };

  const useModel = () => {
    use(modelId);
    onClose();
  };

  const errorMessage = startError?.message ?? (state.status === "failed" ? state.error.message : "");
  const announcement = view === "done" ? "Download complete" : view === "error" ? errorMessage : "";

  let title = `Download ${model.label}`;
  let body = null;
  let footer = null;

  switch (view) {
    case "confirm": {
      const partial = state.status === "notDownloaded" ? state.partialBytes : 0;
      const enough = !space || space.availableBytes >= space.requiredBytes;
      body = (
        <>
          <p>{model.description}</p>
          <dl className="download-modal__facts">
            <div>
              <dt>Size</dt>
              <dd>{formatBytes(model.sizeBytes)}</dd>
            </div>
            <div>
              <dt>Free space</dt>
              <dd className={enough ? undefined : "download-modal__danger"}>
                {space ? `${formatBytes(space.availableBytes)} available` : "Checking…"}
              </dd>
            </div>
          </dl>
          {!enough && space && (
            <p className="download-modal__danger">
              Not enough space. Free up {formatBytes(space.requiredBytes - space.availableBytes)} and try again.
            </p>
          )}
          {partial > 0 && (
            <p>
              {formatPercent(partial, model.sizeBytes)} is already downloaded. Yap will pick up where it left
              off.
            </p>
          )}
          <p className="download-modal__note">Downloads once from Hugging Face. Works offline afterwards.</p>
        </>
      );
      footer = (
        <>
          <Button onClick={onClose}>Cancel</Button>
          <Button ref={primaryRef} variant="primary" loading={starting} disabled={!enough} onClick={start}>
            {partial > 0 ? "Resume download" : "Download"}
          </Button>
        </>
      );
      break;
    }

    case "downloading": {
      const { downloaded, total } = state as { downloaded: number; total: number };
      const speed = speeds[modelId] ?? 0;
      title = `Downloading ${model.label}`;
      body = (
        <>
          <ProgressBar
            className="download-modal__progress"
            value={downloaded}
            max={total}
            aria-label={`Downloading ${model.label}`}
          />
          <div className="download-modal__stats">
            <span>
              {formatBytes(downloaded)} of {formatBytes(total)}
            </span>
            <span>{speed > 0 ? `${formatSpeed(speed)} · ${formatEta((total - downloaded) / speed)}` : "Starting…"}</span>
          </div>
          <p className="download-modal__note">You can hide this window. The download keeps going.</p>
        </>
      );
      footer = (
        <>
          <Button onClick={() => setConfirmingCancel(true)}>Cancel download</Button>
          <Button ref={primaryRef} variant="primary" onClick={onClose}>
            Hide
          </Button>
        </>
      );
      break;
    }

    case "verifying":
      title = `Downloading ${model.label}`;
      body = (
        <>
          <ProgressBar className="download-modal__progress" aria-label="Checking the file" />
          <p className="download-modal__note">Checking the file…</p>
        </>
      );
      break;

    case "done": {
      const inUse = activeId === modelId;
      title = "Download complete";
      body = <p>{inUse ? `${model.label} is ready and in use.` : `${model.label} is ready.`}</p>;
      footer = inUse ? (
        <Button ref={primaryRef} variant="primary" onClick={onClose}>
          Close
        </Button>
      ) : (
        <>
          <Button onClick={onClose}>Close</Button>
          <Button ref={primaryRef} variant="primary" onClick={useModel}>
            Use this model
          </Button>
        </>
      );
      break;
    }

    case "error":
      title = `Couldn't download ${model.label}`;
      body = <p>{errorMessage}</p>;
      footer = (
        <>
          <Button onClick={onClose}>Close</Button>
          <Button ref={primaryRef} variant="primary" loading={starting} onClick={start}>
            Retry
          </Button>
        </>
      );
      break;
  }

  return (
    <>
      <Modal open={open} onClose={onClose} title={title} footer={footer}>
        {body}
        <p className="sr-only" aria-live="polite">
          {announcement}
        </p>
      </Modal>
      <ConfirmDialog
        open={open && confirmingCancel}
        title="Stop the download?"
        message="Yap keeps what's downloaded so far, so you can resume later."
        confirmLabel="Stop download"
        cancelLabel="Keep downloading"
        onConfirm={stopDownload}
        onCancel={() => setConfirmingCancel(false)}
      />
    </>
  );
}
