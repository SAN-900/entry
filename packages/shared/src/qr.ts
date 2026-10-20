export type EntryQrPayload = {
  version: 1;
  event: string;
  assetId: string;
  issuedAt: number;
  expiresAt: number;
  nonce: string;
  signature: string;
};

export function isFreshQr(payload: EntryQrPayload, nowMs = Date.now()): boolean {
  return payload.version === 1 && payload.issuedAt <= nowMs && payload.expiresAt >= nowMs;
}
