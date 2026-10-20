import { describe, expect, it } from 'vitest';
import { isResaleAllowed, maxResalePrice } from './pricing.js';

describe('resale cap is calculated from face value', () => {
  it('allows a 105% resale and rejects 150% at a 110% cap', () => {
    const face = 1_000_000n;
    expect(maxResalePrice(face, 11_000)).toBe(1_100_000n);
    expect(isResaleAllowed(face, 1_050_000n, 11_000)).toBe(true);
    expect(isResaleAllowed(face, 1_500_000n, 11_000)).toBe(false);
  });
});
