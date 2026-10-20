import { describe, expect, it } from 'vitest';
import { isFreshQr, parseFreshQr, parseQrPayload, type EntryQrPayload } from './qr.js';

const validPayload: EntryQrPayload = {
  version: 1,
  event: '42',
  assetId: 'asset-id',
  issuedAt: 1_000,
  expiresAt: 2_000,
  nonce: 'nonce-1',
  signature: 'signature',
};

describe('ENTRY QR payload validation', () => {
  it('accepts a well-formed payload only while it is fresh', () => {
    expect(parseFreshQr(validPayload, 1_500)).toEqual(validPayload);
    expect(isFreshQr(validPayload, 2_001)).toBe(false);
    expect(parseFreshQr(validPayload, 999)).toBeNull();
  });

  it('parses a serialized payload and rejects malformed input', () => {
    expect(parseFreshQr(JSON.stringify(validPayload), 1_500)).toEqual(validPayload);
    expect(parseFreshQr('{not json', 1_500)).toBeNull();
    expect(parseQrPayload({ ...validPayload, signature: '' })).toBeNull();
    expect(parseQrPayload({ ...validPayload, issuedAt: Number.NaN })).toBeNull();
  });

  it('retains a structurally valid payload for offline queue history after expiry', () => {
    expect(parseQrPayload(validPayload)).toEqual(validPayload);
    expect(parseFreshQr(validPayload, 2_001)).toBeNull();
  });
});
