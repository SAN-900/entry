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

export async function GET(request: Request) {
  const url = new URL(request.url);
  const eventId = parseEventId(url.searchParams.get('event'));
  if (!eventId) return jsonResponse({ message: 'A valid event ID is required.' }, 400);

  return jsonResponse({
    type: 'action',
    icon: `${url.origin}/api/actions/icon`,
    title: 'Buy ENTRY ticket',
    description: `Prepare a Devnet USDC purchase for ENTRY event ${eventId}. Review and simulate the transaction before signing.`,
    label: 'Buy ticket',
    links: {
      actions: [{
        label: 'Buy ticket',
        href: `${url.origin}/api/actions/buy?event=${encodeURIComponent(eventId)}`,
        type: 'transaction',
      }],
    },
  });
}

export async function POST(request: Request) {
  const url = new URL(request.url);
  const eventId = parseEventId(url.searchParams.get('event'));
  if (!eventId) return jsonResponse({ message: 'A valid event ID is required.' }, 400);

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
    'purchase',
    { eventId, account },
    { eventId, account },
  );
  if (result.error) return jsonResponse({ message: result.error }, result.status);
  return jsonResponse(result.transaction);
}
