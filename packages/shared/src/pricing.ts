import { BPS_DENOMINATOR } from './constants.js';

export function maxResalePrice(facePriceBaseUnits: bigint, capBps: number): bigint {
  if (facePriceBaseUnits <= 0n) throw new Error('face price must be positive');
  if (!Number.isInteger(capBps) || capBps < 10_000 || capBps > 65_535) throw new Error('invalid cap bps');
  return facePriceBaseUnits * BigInt(capBps) / BPS_DENOMINATOR;
}

export function isResaleAllowed(facePriceBaseUnits: bigint, askBaseUnits: bigint, capBps: number): boolean {
  return askBaseUnits > 0n && askBaseUnits <= maxResalePrice(facePriceBaseUnits, capBps);
}
