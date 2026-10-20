import { PublicKey, VersionedTransaction } from '@solana/web3.js';

const entryProgramId = new PublicKey('EsW1tbFTY87F7k669NzJ1mdp7kVgTb2DXyANvpSoU2y9');

export const actionHeaders = {
  'Access-Control-Allow-Origin': '*',
  'Access-Control-Allow-Methods': 'GET,POST,OPTIONS',
  'Access-Control-Allow-Headers': 'Content-Type',
  'Content-Type': 'application/json',
};

export type PreparedTransaction = {
  transaction: string;
  message: string;
  summary: {
    eventId: string;
    amount: string;
    token: 'USDC';
    feePayer: string;
    cluster: 'devnet';
    payments: Array<{ recipient: string; amount: string }>;
  };
};

export function parseEventId(value: string | null): string | null {
  if (!value || !/^(0|[1-9]\d{0,19})$/.test(value)) return null;
  try {
    return BigInt(value) <= 18_446_744_073_709_551_615n ? value : null;
  } catch {
    return null;
  }
}

export function parsePublicKey(value: unknown): string | null {
  if (typeof value !== 'string') return null;
  try {
    return new PublicKey(value).toBase58();
  } catch {
    return null;
  }
}

function toMicrounits(value: string): bigint | null {
  if (!/^(0|[1-9]\d{0,12})(?:\.\d{1,6})?$/.test(value)) return null;
  const [whole, fraction = ''] = value.split('.');
  try {
    return BigInt(whole) * 1_000_000n + BigInt(fraction.padEnd(6, '0'));
  } catch {
    return null;
  }
}

export function isPreparedTransaction(value: unknown, expected: {
  eventId: string;
  account: string;
}): value is PreparedTransaction {
  if (typeof value !== 'object' || value === null) return false;
  const result = value as Partial<PreparedTransaction>;
  const summary = result.summary;
  if (
    typeof result.transaction !== 'string'
    || result.transaction.length === 0
    || result.transaction.length > 1800
    || !/^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/.test(result.transaction)
    || Buffer.from(result.transaction, 'base64').toString('base64') !== result.transaction
    || Buffer.from(result.transaction, 'base64').byteLength > 1232
    || typeof result.message !== 'string'
    || result.message.length === 0
    || result.message.length > 500
    || typeof summary !== 'object'
    || summary === null
    || summary.eventId !== expected.eventId
    || summary.token !== 'USDC'
    || summary.cluster !== 'devnet'
    || summary.feePayer !== expected.account
    || typeof summary.amount !== 'string'
    || !Array.isArray(summary.payments)
    || summary.payments.length === 0
    || summary.payments.length > 4
  ) return false;

  let transaction: VersionedTransaction;
  try {
    transaction = VersionedTransaction.deserialize(Buffer.from(result.transaction, 'base64'));
  } catch {
    return false;
  }
  if (transaction.message.version !== 0) return false;
  if (transaction.message.staticAccountKeys[0]?.toBase58() !== expected.account) return false;
  const callsEntry = transaction.message.compiledInstructions.some((instruction) =>
    transaction.message.staticAccountKeys[instruction.programIdIndex]?.equals(entryProgramId),
  );
  if (!callsEntry) return false;

  const total = toMicrounits(summary.amount);
  if (total === null || total <= 0n) return false;
  let paymentTotal = 0n;
  for (const payment of summary.payments) {
    if (
      typeof payment !== 'object'
      || payment === null
      || parsePublicKey(payment.recipient) !== payment.recipient
      || typeof payment.amount !== 'string'
    ) return false;
    const amount = toMicrounits(payment.amount);
    if (amount === null || amount <= 0n) return false;
    paymentTotal += amount;
  }
  return paymentTotal === total;
}

function getBuilderEndpoint(path: string): URL | null {
  const configured = process.env.ENTRY_TRANSACTION_BUILDER_URL;
  if (!configured) return null;
  const trimmedPath = path.trim();
  if (!trimmedPath || trimmedPath.includes('\\') || trimmedPath.includes('?') || trimmedPath.includes('#') || trimmedPath.includes('..')) {
    return null;
  }
  try {
    const base = new URL(configured);
    const localHttp = base.protocol === 'http:' && ['localhost', '127.0.0.1', '[::1]'].includes(base.hostname);
    if (
      (base.protocol !== 'https:' && !localHttp)
      || base.username
      || base.password
      || base.search
      || base.hash
    ) return null;
    const candidate = new URL(trimmedPath, base.toString().endsWith('/') ? base : `${base}/`);
    if (candidate.origin !== base.origin || candidate.username || candidate.password) return null;
    return candidate;
  } catch {
    return null;
  }
}

export async function requestPreparedTransaction(
  path: string,
  body: Record<string, unknown>,
  expected: { eventId: string; account: string },
): Promise<{ transaction: PreparedTransaction; error?: never } | { transaction?: never; error: string; status: number }> {
  const endpoint = getBuilderEndpoint(path);
  if (!endpoint) {
    return {
      status: 503,
      error: 'The trusted Devnet transaction builder is not configured.',
    };
  }

  try {
    const response = await fetch(endpoint, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
      cache: 'no-store',
      redirect: 'error',
      signal: AbortSignal.timeout(10_000),
    });
    if (!response.ok) {
      return {
        status: 502,
        error: 'The trusted transaction builder could not prepare this transaction.',
      };
    }
    const result: unknown = await response.json();
    if (!isPreparedTransaction(result, expected)) {
      return {
        status: 502,
        error: 'The transaction builder returned an invalid or unsafe transaction summary.',
      };
    }
    return { transaction: result };
  } catch {
    return {
      status: 502,
      error: 'The trusted transaction builder is unavailable or returned invalid data.',
    };
  }
}

export function jsonResponse(body: unknown, status = 200): Response {
  return Response.json(body, { status, headers: actionHeaders });
}
