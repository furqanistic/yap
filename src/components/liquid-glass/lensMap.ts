interface LensMapSize {
  width: number;
  height: number;
  radius: number;
}

interface CanvasResources {
  canvas: HTMLCanvasElement;
  context: CanvasRenderingContext2D;
  image: ImageData;
  width: number;
  height: number;
}

const MAX_MAP_EDGE = 512;
const ROW_SLICE = 18;

/**
 * Draws a compact rounded-rectangle displacement map. The canvas, context, and
 * ImageData are reused between resizes; work is split into short row slices.
 */
export class RoundedRectLensMap {
  private resources: CanvasResources | null = null;
  private generation = 0;
  private idleHandle: number | null = null;
  private idleMode = false;

  generate(size: LensMapSize, onReady: (blob: Blob) => void) {
    const generation = ++this.generation;
    this.cancelScheduledWork();

    const ratio = Math.min(1, MAX_MAP_EDGE / Math.max(size.width, size.height));
    const width = Math.max(1, Math.round(size.width * ratio));
    const height = Math.max(1, Math.round(size.height * ratio));
    const resources = this.getResources(width, height);
    const { data } = resources.image;
    const radius = Math.min(size.radius, size.width / 2, size.height / 2);
    const bevel = Math.max(1, radius * 0.5);
    const scaleX = size.width / width;
    const scaleY = size.height / height;

    // Neutral displacement outside the bevel; alpha stays fully opaque.
    for (let i = 0; i < data.length; i += 4) {
      data[i] = 128;
      data[i + 1] = 128;
      data[i + 2] = 128;
      data[i + 3] = 255;
    }

    let row = 0;
    const drawSlice = () => {
      if (generation !== this.generation) return;

      const maxRow = Math.min(row + ROW_SLICE, height);
      for (; row < maxRow; row += 1) {
        const y = (row + 0.5) * scaleY;
        for (let xIndex = 0; xIndex < width; xIndex += 1) {
          const x = (xIndex + 0.5) * scaleX;
          if (Math.min(x, y, size.width - x, size.height - y) > bevel + radius) continue;
          const distance = -signedDistanceToRoundRect(x, y, size.width, size.height, radius);
          if (distance < 0 || distance >= bevel) continue;

          // A half-CSS-pixel central difference gives a stable edge normal even
          // when the map is downsampled for a large resizable panel.
          const gx =
            signedDistanceToRoundRect(x + 0.5, y, size.width, size.height, radius) -
            signedDistanceToRoundRect(x - 0.5, y, size.width, size.height, radius);
          const gy =
            signedDistanceToRoundRect(x, y + 0.5, size.width, size.height, radius) -
            signedDistanceToRoundRect(x, y - 0.5, size.width, size.height, radius);
          const length = Math.hypot(gx, gy) || 1;
          const k = Math.pow(1 - distance / bevel, 1.5) * 0.5;
          const index = (row * width + xIndex) * 4;

          // Red and blue carry the horizontal and vertical lens gradients.
          data[index] = clampByte(128 - (gx / length) * k * 255);
          data[index + 2] = clampByte(128 - (gy / length) * k * 255);
        }
      }

      if (row < height) {
        this.schedule(drawSlice);
        return;
      }

      if (generation !== this.generation) return;
      resources.context.putImageData(resources.image, 0, 0);
      resources.canvas.toBlob((blob) => {
        if (blob && generation === this.generation) onReady(blob);
      }, "image/png");
    };

    // Start promptly, then yield between slices. Idle callbacks keep the
    // resize path responsive on Chromium; setTimeout is the safe fallback.
    this.schedule(drawSlice);
  }

  cancel() {
    this.generation += 1;
    this.cancelScheduledWork();
  }

  private getResources(width: number, height: number): CanvasResources {
    if (this.resources?.width === width && this.resources.height === height) return this.resources;

    const canvas = this.resources?.canvas ?? document.createElement("canvas");
    canvas.width = width;
    canvas.height = height;
    const context = canvas.getContext("2d", { willReadFrequently: true });
    if (!context) throw new Error("Canvas 2D is unavailable for Liquid Glass refraction.");

    this.resources = {
      canvas,
      context,
      image: context.createImageData(width, height),
      width,
      height,
    };
    return this.resources;
  }

  private schedule(callback: (deadline?: IdleDeadline) => void) {
    const idleWindow = window as Window & {
      requestIdleCallback?: (callback: IdleRequestCallback, options?: IdleRequestOptions) => number;
      cancelIdleCallback?: (handle: number) => void;
    };

    if (idleWindow.requestIdleCallback) {
      this.idleMode = true;
      this.idleHandle = idleWindow.requestIdleCallback(callback, { timeout: 40 });
    } else {
      this.idleMode = false;
      this.idleHandle = window.setTimeout(() => callback(), 0);
    }
  }

  private cancelScheduledWork() {
    if (this.idleHandle === null) return;
    const idleWindow = window as Window & { cancelIdleCallback?: (handle: number) => void };
    if (this.idleMode) idleWindow.cancelIdleCallback?.(this.idleHandle);
    else window.clearTimeout(this.idleHandle);
    this.idleHandle = null;
  }
}

function signedDistanceToRoundRect(
  x: number,
  y: number,
  width: number,
  height: number,
  radius: number,
) {
  const qx = Math.abs(x - width / 2) - (width / 2 - radius);
  const qy = Math.abs(y - height / 2) - (height / 2 - radius);
  const outside = Math.hypot(Math.max(qx, 0), Math.max(qy, 0));
  return outside + Math.min(Math.max(qx, qy), 0) - radius;
}

function clampByte(value: number) {
  return Math.max(0, Math.min(255, Math.round(value)));
}
