export type EntryQrPayload = {
  version: 1;
  event: string;
  assetId: string;
  issuedAt: number;
  expiresAt: number;
  nonce: string;
  signature: string;
};

export function parseQrPayload(value: unknown): EntryQrPayload | null {
  let candidate = value;

  if (typeof value === 'string') {
    try {
      candidate = JSON.parse(value) as unknown;
    } catch {
      return null;
    }
  }

  if (typeof candidate !== 'object' || candidate === null) return null;
  const payload = candidate as Partial<EntryQrPayload>;
  if (
    payload.version !== 1
    || typeof payload.event !== 'string'
    || payload.event.length === 0
    || payload.event.length > 64
    || typeof payload.assetId !== 'string'
    || payload.assetId.length === 0
    || payload.assetId.length > 64
    || !Number.isSafeInteger(payload.issuedAt)
    || !Number.isSafeInteger(payload.expiresAt)
    || typeof payload.nonce !== 'string'
    || payload.nonce.length === 0
    || payload.nonce.length > 128
    || typeof payload.signature !== 'string'
    || payload.signature.length === 0
    || payload.signature.length > 512
  ) {
    return null;
  }

  return payload as EntryQrPayload;
}

export function parseFreshQr(value: unknown, nowMs = Date.now()): EntryQrPayload | null {
  const payload = parseQrPayload(value);
  if (!payload) return null;

  const issuedAt = payload.issuedAt as number;
  const expiresAt = payload.expiresAt as number;
  if (issuedAt > nowMs || expiresAt < nowMs || expiresAt < issuedAt) return null;

  return payload;
}

export function isFreshQr(payload: EntryQrPayload, nowMs = Date.now()): boolean {
  return parseFreshQr(payload, nowMs) !== null;
}
