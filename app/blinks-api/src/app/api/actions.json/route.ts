import { NextResponse } from 'next/server';
export function GET() {
  const origin = process.env.PUBLIC_ACTIONS_ORIGIN;
  if (!origin) return NextResponse.json({ error: 'PUBLIC_ACTIONS_ORIGIN is not configured' }, { status: 503 });
  return NextResponse.json({ rules: [
    { pathPattern: '/api/actions/buy/**', apiPath: '/api/actions/buy' },
    { pathPattern: '/api/actions/resell/**', apiPath: '/api/actions/resell' },
  ] }, { headers: { 'Access-Control-Allow-Origin': '*', 'Cache-Control': 'public, max-age=300' } });
}
