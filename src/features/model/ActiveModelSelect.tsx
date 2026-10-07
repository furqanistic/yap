import { IconDownloadMinimalistic } from "@devigner-ui/icons/DownloadMinimalistic";
import { Button, Select, type SelectOption } from "@/components/ui";
import type { SettingControlA11y } from "@/components/settings";
import { useModels } from "@/hooks/useModels";
import type { ModelId } from "@/types/models";
import { RECOMMENDED_MODEL } from "./options";
import { useDownloadModal } from "./useDownloadModal";

/**
 * Picks the model used for transcription from the downloaded ones. Shared by
 * the General and Model pages so there's one source of truth.
 */
export function ActiveModelSelect(a11y: SettingControlA11y) {
  const { downloaded, activeId, use } = useModels();
  const openDownload = useDownloadModal();

  if (downloaded.length === 0) {
    return (
      <Button icon={IconDownloadMinimalistic} onClick={() => openDownload(RECOMMENDED_MODEL)}>
        Download a model
      </Button>
    );
  }

  const options: SelectOption<ModelId>[] = downloaded.map((model) => ({ value: model.id, label: model.label }));

  return (
    <Select
      value={activeId ?? RECOMMENDED_MODEL}
      options={options}
      placeholder="Choose a model"
      onChange={use}
      {...a11y}
    />
  );
}
