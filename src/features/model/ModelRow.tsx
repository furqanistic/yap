import { useId, useState } from "react";
import { Button, ConfirmDialog, ProgressBar, Tag, useToast } from "@/components/ui";
import { useModels } from "@/hooks/useModels";
import { formatBytes, formatPercent } from "@/lib/format";
import type { ModelEntry } from "@/types/models";
import { LANGUAGE_TAGS } from "./options";
import { useDownloadModal } from "./useDownloadModal";

interface ModelRowProps {
  model: ModelEntry;
}

/** One catalog model, with the action that fits its download state. */
export function ModelRow({ model }: ModelRowProps) {
  const { activeId, download, cancel, remove, use } = useModels();
  const openDownload = useDownloadModal();
  const toast = useToast();
  const [confirmingDelete, setConfirmingDelete] = useState(false);
  const titleId = useId();
  const { state } = model;

  const retry = async () => {
    const error = await download(model.id);
    if (error) toast(error.message);
  };

  const deleteModel = async () => {
    setConfirmingDelete(false);
    const error = await remove(model.id);
    if (error) toast(error.message);
  };

  let actions;
  switch (state.status) {
    case "notDownloaded":
      actions = (
        <Button size="small" onClick={() => openDownload(model.id)} aria-describedby={titleId}>
          {state.partialBytes > 0 ? "Resume" : "Download"}
        </Button>
      );
      break;

    case "downloading":
      actions = (
        <>
          <ProgressBar
            className="model-row__progress"
            value={state.downloaded}
            max={state.total}
            aria-labelledby={titleId}
          />
          <span className="model-row__percent">{formatPercent(state.downloaded, state.total)}</span>
          <Button size="small" variant="quiet" onClick={() => cancel(model.id)} aria-describedby={titleId}>
            Cancel
          </Button>
        </>
      );
      break;

    case "verifying":
      actions = (
        <>
          <ProgressBar className="model-row__progress" aria-labelledby={titleId} />
          <span className="model-row__percent">Checking</span>
        </>
      );
      break;

    case "ready":
      actions = (
        <>
          <Button
            size="small"
            variant="quiet"
            onClick={() => setConfirmingDelete(true)}
            aria-describedby={titleId}
          >
            Delete
          </Button>
          {activeId === model.id ? (
            <Tag selected>In use</Tag>
          ) : (
            <Button size="small" onClick={() => use(model.id)} aria-describedby={titleId}>
              Use
            </Button>
          )}
        </>
      );
      break;

    case "failed":
      actions = (
        <Button size="small" onClick={retry} aria-describedby={titleId}>
          Retry
        </Button>
      );
      break;
  }

  return (
    <div className="setting-row model-row">
      <div className="setting-row__text">
        <div className="model-row__title">
          <span id={titleId} className="setting-row__title">
            {model.label}
          </span>
          <Tag>{LANGUAGE_TAGS[model.languages]}</Tag>
          {model.recommended && <Tag selected>Recommended</Tag>}
        </div>
        <p className="setting-row__description">
          {model.description} {formatBytes(model.sizeBytes)}.
        </p>
        {state.status === "failed" && <p className="model-row__error">{state.error.message}</p>}
      </div>

      <div className="model-row__actions">{actions}</div>

      <ConfirmDialog
        open={confirmingDelete}
        title={`Delete ${model.label}?`}
        message={
          activeId === model.id
            ? "Yap uses this model for transcription. You'll need to pick or download another one."
            : "You can download it again at any time."
        }
        confirmLabel="Delete"
        destructive
        onConfirm={deleteModel}
        onCancel={() => setConfirmingDelete(false)}
      />
    </div>
  );
}
