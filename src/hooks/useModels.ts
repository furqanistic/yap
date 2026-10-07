import { useCallback, useEffect, useSyncExternalStore } from "react";
import { useSettings } from "@/hooks/useSettings";
import { modelsApi, onEvent } from "@/lib/tauri";
import type { ModelEntry, ModelError, ModelId, ModelState } from "@/types/models";

/*
 * One shared copy of the model list for the whole window. It loads once and
 * follows `models://progress` and `models://state`, so every view (the Model
 * page, the download modal, the General page) shows the same live state.
 */
interface Snapshot {
  models: ModelEntry[];
  /** Latest download speed per model, while downloading. */
  speeds: Partial<Record<ModelId, number>>;
  dir: string;
  loaded: boolean;
}

let snapshot: Snapshot = { models: [], speeds: {}, dir: "", loaded: false };
let started = false;
const subscribers = new Set<() => void>();

function publish(next: Partial<Snapshot>) {
  snapshot = { ...snapshot, ...next };
  subscribers.forEach((notify) => notify());
}

function setState(id: ModelId, state: ModelState) {
  publish({ models: snapshot.models.map((model) => (model.id === id ? { ...model, state } : model)) });
}

function refresh() {
  return Promise.all([modelsApi.list(), modelsApi.dir()])
    .then(([models, dir]) => publish({ models, dir, loaded: true }))
    .catch((error) => console.error("Could not load models", error));
}

function ensureStarted() {
  if (started) return;
  started = true;

  onEvent("models://progress", ({ id, downloaded, total, bytesPerSec }) => {
    setState(id, { status: "downloading", downloaded, total });
    publish({ speeds: { ...snapshot.speeds, [id]: bytesPerSec } });
  });

  onEvent("models://state", ({ id, state }) => {
    setState(id, state);
    if (state.status !== "downloading") {
      const speeds = { ...snapshot.speeds };
      delete speeds[id];
      publish({ speeds });
    }
  });

  refresh();
}

function subscribe(notify: () => void) {
  subscribers.add(notify);
  return () => subscribers.delete(notify);
}

function toModelError(error: unknown): ModelError {
  if (error && typeof error === "object" && "message" in error) return error as ModelError;
  return { kind: "io", message: String(error) };
}

export interface UseModels extends Snapshot {
  /** The model chosen in settings, if it's in the catalog. */
  activeId: ModelId | null;
  /** Models that are downloaded and ready to use. */
  downloaded: ModelEntry[];
  /** Bytes used on disk by finished and partial downloads. */
  usedBytes: number;
  find: (id: ModelId) => ModelEntry | undefined;
  /** Starts a download. Resolves with an error if it couldn't start. */
  download: (id: ModelId) => Promise<ModelError | null>;
  cancel: (id: ModelId) => Promise<void>;
  remove: (id: ModelId) => Promise<ModelError | null>;
  /** Makes this the model used for transcription. */
  use: (id: ModelId) => Promise<void>;
  openFolder: () => Promise<void>;
}

export function useModels(): UseModels {
  const current = useSyncExternalStore(subscribe, () => snapshot);
  const { settings, update } = useSettings();

  useEffect(ensureStarted, []);

  const download = useCallback(async (id: ModelId) => {
    try {
      await modelsApi.download(id);
      return null;
    } catch (error) {
      return toModelError(error);
    }
  }, []);

  const cancel = useCallback((id: ModelId) => modelsApi.cancel(id), []);

  const remove = useCallback(async (id: ModelId) => {
    try {
      await modelsApi.remove(id);
      return null;
    } catch (error) {
      return toModelError(error);
    }
  }, []);

  const use = useCallback((id: ModelId) => update({ model: id }), [update]);

  const openFolder = useCallback(
    () => modelsApi.openFolder().catch((error) => console.error("Could not open the models folder", error)),
    [],
  );

  const find = (id: ModelId) => current.models.find((model) => model.id === id);
  const downloaded = current.models.filter((model) => model.state.status === "ready");
  const usedBytes = current.models.reduce((sum, { state, sizeBytes }) => {
    if (state.status === "ready") return sum + sizeBytes;
    if (state.status === "downloading") return sum + state.downloaded;
    if (state.status === "notDownloaded") return sum + state.partialBytes;
    return sum;
  }, 0);
  const activeId = settings && current.models.some((m) => m.id === settings.model) ? settings.model : null;

  return { ...current, activeId, downloaded, usedBytes, find, download, cancel, remove, use, openFolder };
}
