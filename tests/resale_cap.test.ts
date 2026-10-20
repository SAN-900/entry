import { describe, expect, it } from 'vitest';
import { isResaleAllowed, maxResalePrice } from '../packages/shared/src/pricing.js';

describe('ENTRY resale cap', () => {
  const face = 10_000_000n;
  it('allows 105% of face value', () => expect(isResaleAllowed(face, 10_500_000n, 11_000)).toBe(true));
  it('rejects 150% of face value', () => expect(isResaleAllowed(face, 15_000_000n, 11_000)).toBe(false));
  it('does not compound against the previous resale price', () => expect(maxResalePrice(face, 11_000)).toBe(11_000_000n));
});
