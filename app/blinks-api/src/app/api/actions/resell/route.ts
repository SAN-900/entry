import {
  actionHeaders,
  jsonResponse,
  parseEventId,
  parsePublicKey,
  requestPreparedTransaction,
} from '../helpers';

export function OPTIONS() {
  return new Response(null, { status: 204, headers: actionHeaders });
}

function readResaleRequest(url: URL): { eventId: string; assetId: string } | null {
  const eventId = parseEventId(url.searchParams.get('event'));
  const assetId = parsePublicKey(url.searchParams.get('asset'));
  return eventId && assetId ? { eventId, assetId } : null;
}

export async function GET(request: Request) {
  const url = new URL(request.url);
  const resale = readResaleRequest(url);
  if (!resale) return jsonResponse({ message: 'Valid event and ticket asset identifiers are required.' }, 400);

  return jsonResponse({
    type: 'action',
    icon: `${url.origin}/api/actions/icon`,
    title: 'Buy a resale ENTRY ticket',
    description: `Prepare a Devnet USDC purchase of a listed ticket for event ${resale.eventId}. Review and simulate the transaction before signing.`,
    label: 'Buy resale ticket',
    links: {
      actions: [{
        label: 'Buy resale ticket',
        href: `${url.origin}/api/actions/resell?event=${encodeURIComponent(resale.eventId)}&asset=${encodeURIComponent(resale.assetId)}`,
        type: 'transaction',
      }],
    },
  });
}

export async function POST(request: Request) {
  const url = new URL(request.url);
  const resale = readResaleRequest(url);
  if (!resale) return jsonResponse({ message: 'Valid event and ticket asset identifiers are required.' }, 400);

  let body: unknown;
  try {
    body = await request.json();
  } catch {
    return jsonResponse({ message: 'Request body must be valid JSON.' }, 400);
  }
  if (typeof body !== 'object' || body === null) {
    return jsonResponse({ message: 'Request must include a buyer account.' }, 400);
  }
  const account = parsePublicKey((body as { account?: unknown }).account);
  if (!account) return jsonResponse({ message: 'A valid buyer account is required.' }, 400);

  const result = await requestPreparedTransaction(
    'resale',
    { ...resale, account },
    { eventId: resale.eventId, account },
  );
  if (result.error) return jsonResponse({ message: result.error }, result.status);
  return jsonResponse(result.transaction);
}
