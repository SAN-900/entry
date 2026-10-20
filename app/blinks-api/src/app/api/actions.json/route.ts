const headers = {
  'Access-Control-Allow-Origin': '*',
  'Access-Control-Allow-Methods': 'GET,OPTIONS',
  'Access-Control-Allow-Headers': 'Content-Type',
  'Cache-Control': 'public, max-age=300',
};

export function OPTIONS() {
  return new Response(null, { status: 204, headers });
}

export function GET() {
  const origin = process.env.PUBLIC_ACTIONS_ORIGIN;
  if (!origin) {
    return Response.json({ error: 'PUBLIC_ACTIONS_ORIGIN is not configured.' }, { status: 503, headers });
  }

  try {
    const parsed = new URL(origin);
    const localHttp = parsed.protocol === 'http:' && ['localhost', '127.0.0.1', '[::1]'].includes(parsed.hostname);
    if (
      (parsed.protocol !== 'https:' && !localHttp)
      || parsed.username
      || parsed.password
      || parsed.search
      || parsed.hash
    ) {
      return Response.json({ error: 'PUBLIC_ACTIONS_ORIGIN must be an HTTPS origin (HTTP is allowed only on localhost).' }, { status: 500, headers });
    }
  } catch {
    return Response.json({ error: 'PUBLIC_ACTIONS_ORIGIN must be a valid URL.' }, { status: 500, headers });
  }

  return Response.json({
    rules: [
      { pathPattern: '/api/actions/buy', apiPath: '/api/actions/buy' },
      { pathPattern: '/api/actions/resell', apiPath: '/api/actions/resell' },
    ],
  }, { headers });
}
