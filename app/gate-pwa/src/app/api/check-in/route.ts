import { PublicKey, VersionedTransaction } from '@solana/web3.js';
import { parseFreshQr } from '@entry/shared';

const entryProgramId = new PublicKey('EsW1tbFTY87F7k669NzJ1mdp7kVgTb2DXyANvpSoU2y9');

type CheckInResponse = {
  transaction: string;
  message: string;
  summary: {
    eventId: string;
    ticketAsset: string;
    gateAuthority: string;
    feePayer: string;
    cluster: 'devnet';
  };
};

function parsePublicKey(value: unknown): string | null {
  if (typeof value !== 'string') return null;
  try {
    return new PublicKey(value).toBase58();
  } catch {
    return null;
  }
}

function isCheckInResponse(value: unknown, expected: {
  account: string;
  payload: { event: string; assetId: string };
}): value is CheckInResponse {
  if (typeof value !== 'object' || value === null) return false;
  const result = value as Partial<CheckInResponse>;
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
    || summary.eventId !== expected.payload.event
    || summary.ticketAsset !== expected.payload.assetId
    || summary.gateAuthority !== expected.account
    || summary.feePayer !== expected.account
    || summary.cluster !== 'devnet'
    || parsePublicKey(summary.ticketAsset) !== summary.ticketAsset
    || parsePublicKey(summary.gateAuthority) !== summary.gateAuthority
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

  return true;
}

function getCheckInServiceUrl(): URL | null {
  const configured = process.env.ENTRY_CHECKIN_SERVICE_URL;
  if (!configured) return null;
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
    return new URL('check-in', base.toString().endsWith('/') ? base : `${base}/`);
  } catch {
    return null;
  }
}

export async function POST(request: Request) {
  const contentLength = Number(request.headers.get('content-length') ?? 0);
  if (contentLength > 16_384) {
    return Response.json({ message: 'Request body is too large.' }, { status: 413 });
  }

  let body: unknown;
  try {
    body = await request.json();
  } catch {
    return Response.json({ message: 'Request body must be valid JSON.' }, { status: 400 });
  }
  if (typeof body !== 'object' || body === null) {
    return Response.json({ message: 'Gate account and ticket QR payload are required.' }, { status: 400 });
  }

  const account = parsePublicKey((body as { account?: unknown }).account);
  const payload = parseFreshQr((body as { payload?: unknown }).payload);
  if (!account || !payload) {
    return Response.json({ message: 'A valid gate account and fresh ENTRY QR payload are required.' }, { status: 400 });
  }

  const endpoint = getCheckInServiceUrl();
  if (!endpoint) {
    return Response.json({ message: 'The trusted Devnet proof and check-in service is not configured.' }, { status: 503 });
  }

  try {
    const response = await fetch(endpoint, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ account, payload }),
      cache: 'no-store',
      redirect: 'error',
      signal: AbortSignal.timeout(10_000),
    });
    if (!response.ok) {
      return Response.json({ message: 'Online ticket verification could not prepare a check-in transaction.' }, { status: 502 });
    }
    const result: unknown = await response.json();
    if (!isCheckInResponse(result, { account, payload })) {
      return Response.json({ message: 'Check-in service returned an invalid or unsafe transaction summary.' }, { status: 502 });
    }
    return Response.json(result, { headers: { 'Cache-Control': 'no-store' } });
  } catch {
    return Response.json({ message: 'The trusted Devnet check-in service is unavailable.' }, { status: 502 });
  }
}
