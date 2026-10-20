import { NextResponse } from 'next/server';

const headers = { 'Access-Control-Allow-Origin': '*', 'Access-Control-Allow-Methods': 'GET,POST,OPTIONS', 'Access-Control-Allow-Headers': 'Content-Type' };
export function OPTIONS() { return new Response(null, { status: 204, headers }); }

export async function GET(request: Request) {
  const url = new URL(request.url);
  const eventId = url.searchParams.get('event');
  if (!eventId) return NextResponse.json({ message: 'Missing event identifier.' }, { status: 400, headers });
  // Demo metadata only. The actual on-chain price must be loaded from the event account before production.
  return NextResponse.json({
    title: 'Buy ENTRY ticket',
    icon: `${url.origin}/entry-icon.png`,
    description: `Purchase a ticket for event ${eventId}. Review and sign the transaction in your wallet.`,
    label: 'Buy ticket',
    links: { actions: [{ label: 'Buy ticket · demo', href: `/api/actions/buy?event=${encodeURIComponent(eventId)}`, type: 'transaction' }] },
  }, { headers });
}

export async function POST() {
  // Fail closed rather than returning a fabricated transaction. Wire this to the Anchor client,
  // validated event account, current wallet account, and a real purchase instruction.
  return NextResponse.json({ message: 'Purchase transaction builder is not configured yet.' }, { status: 501, headers });
}
