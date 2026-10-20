import { describe, expect, it } from 'vitest';
import {
  PublicKey,
  TransactionInstruction,
  TransactionMessage,
  VersionedTransaction,
} from '@solana/web3.js';
import { isPreparedTransaction, parseEventId, parsePublicKey, requestPreparedTransaction } from './helpers';

const programId = new PublicKey('EsW1tbFTY87F7k669NzJ1mdp7kVgTb2DXyANvpSoU2y9');

function serializedTransaction(feePayer: PublicKey, instructionProgram = programId): string {
  const message = new TransactionMessage({
    payerKey: feePayer,
    recentBlockhash: PublicKey.default.toBase58(),
    instructions: [new TransactionInstruction({
      programId: instructionProgram,
      keys: [],
      data: Buffer.alloc(0),
    })],
  }).compileToV0Message();
  return Buffer.from(new VersionedTransaction(message).serialize()).toString('base64');
}

function validBuilderResult(account: PublicKey) {
  return {
    transaction: serializedTransaction(account),
    message: 'Buy ENTRY ticket',
    summary: {
      eventId: '42',
      amount: '10.000000',
      token: 'USDC',
      feePayer: account.toBase58(),
      cluster: 'devnet',
      payments: [
        { recipient: KeypairLike.organizer.toBase58(), amount: '9.850000' },
        { recipient: KeypairLike.feeVault.toBase58(), amount: '0.150000' },
      ],
    },
  };
}

const KeypairLike = {
  organizer: new PublicKey('11111111111111111111111111111111'),
  feeVault: new PublicKey('SysvarRent111111111111111111111111111111111'),
};

describe('Actions API request validation', () => {
  it('accepts canonical u64 event IDs and rejects malformed or overflowing IDs', () => {
    expect(parseEventId('42')).toBe('42');
    expect(parseEventId('0')).toBe('0');
    expect(parseEventId('-1')).toBeNull();
    expect(parseEventId('01')).toBeNull();
    expect(parseEventId('18446744073709551616')).toBeNull();
  });

  it('canonicalizes valid wallet addresses', () => {
    expect(parsePublicKey(PublicKey.default.toBase58())).toBe(PublicKey.default.toBase58());
    expect(parsePublicKey('not-a-public-key')).toBeNull();
  });

  it('accepts only a bounded v0 ENTRY transaction with matching payment summary', () => {
    const buyer = PublicKey.unique();
    expect(isPreparedTransaction(validBuilderResult(buyer), {
      eventId: '42',
      account: buyer.toBase58(),
    })).toBe(true);
  });

  it('rejects transaction summaries with mismatched fee payer or payment total', () => {
    const buyer = PublicKey.unique();
    const result = validBuilderResult(buyer);
    result.summary.feePayer = PublicKey.unique().toBase58();
    expect(isPreparedTransaction(result, { eventId: '42', account: buyer.toBase58() })).toBe(false);

    const invalidTotal = validBuilderResult(buyer);
    invalidTotal.summary.payments[1].amount = '0.140000';
    expect(isPreparedTransaction(invalidTotal, { eventId: '42', account: buyer.toBase58() })).toBe(false);
  });

  it('rejects transactions that do not invoke ENTRY', () => {
    const buyer = PublicKey.unique();
    const result = validBuilderResult(buyer);
    result.transaction = serializedTransaction(buyer, PublicKey.default);
    expect(isPreparedTransaction(result, { eventId: '42', account: buyer.toBase58() })).toBe(false);
  });

  it('rejects unsafe transaction-builder paths that escape the configured origin', async () => {
    const original = process.env.ENTRY_TRANSACTION_BUILDER_URL;
    process.env.ENTRY_TRANSACTION_BUILDER_URL = 'https://trusted.example/tx/';
    try {
      const result = await requestPreparedTransaction(
        '//evil.example/steal',
        { eventId: '42', account: PublicKey.unique().toBase58() },
        { eventId: '42', account: PublicKey.unique().toBase58() },
      );
      expect(result).toEqual({
        status: 503,
        error: 'The trusted Devnet transaction builder is not configured.',
      });
    } finally {
      if (original === undefined) delete process.env.ENTRY_TRANSACTION_BUILDER_URL;
      else process.env.ENTRY_TRANSACTION_BUILDER_URL = original;
    }
  });
});
