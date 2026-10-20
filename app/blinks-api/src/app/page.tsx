export default function Home() {
  return (
    <main style={{
      maxWidth: '72rem',
      margin: '0 auto',
      padding: '4rem 1.5rem',
      fontFamily: 'system-ui, sans-serif',
      color: '#e5eefb',
      background: '#07111f',
      minHeight: '100vh',
      boxSizing: 'border-box',
    }}>
      <div style={{ display: 'grid', gap: '1.5rem' }}>
        <div>
          <p style={{ margin: 0, letterSpacing: '0.12em', textTransform: 'uppercase', color: '#7dd3fc', fontSize: '0.76rem' }}>ENTRY</p>
          <h1 style={{ margin: '0.5rem 0 0', fontSize: 'clamp(2rem, 5vw, 3rem)' }}>Solana Actions API</h1>
        </div>

        <p style={{ margin: 0, fontSize: '1.05rem', lineHeight: 1.7, color: '#dfeaf9' }}>
          This service exposes the public Blink metadata and transaction builders for ENTRY ticket purchase and resale flows.
          The discovery manifest lives at <code>/api/actions.json</code>, while the action endpoints are mounted under <code>/api/actions</code>.
        </p>

        <section style={{ background: 'rgba(15, 23, 42, 0.75)', border: '1px solid rgba(125, 211, 252, 0.2)', borderRadius: '18px', padding: '1.25rem 1.5rem' }}>
          <h2 style={{ margin: '0 0 1rem', fontSize: '1.1rem' }}>Available actions</h2>
          <ul style={{ margin: 0, paddingLeft: '1.25rem', display: 'grid', gap: '0.5rem', lineHeight: 1.7 }}>
            <li><a href="/api/actions/buy?event=42" style={{ color: '#7dd3fc' }}>Buy ticket action</a> — prepares a purchase transaction for a valid event ID.</li>
            <li><a href="/api/actions/resell?event=42&asset=11111111111111111111111111111111" style={{ color: '#7dd3fc' }}>Resale action</a> — prepares resale checkout for a listed ticket asset.</li>
            <li><a href="/api/actions.json" style={{ color: '#7dd3fc' }}>Actions manifest</a> — exposes the route rules used by compatible clients.</li>
          </ul>
        </section>

        <section style={{ background: 'rgba(15, 23, 42, 0.75)', border: '1px solid rgba(125, 211, 252, 0.2)', borderRadius: '18px', padding: '1.25rem 1.5rem' }}>
          <h2 style={{ margin: '0 0 1rem', fontSize: '1.1rem' }}>Configuration</h2>
          <p style={{ margin: 0, lineHeight: 1.7, color: '#dfeaf9' }}>
            Set <code>PUBLIC_ACTIONS_ORIGIN</code> to the canonical public HTTPS origin for the Blink landing page and
            <code> ENTRY_TRANSACTION_BUILDER_URL</code> to the trusted backend that signs or prepares the transaction payload.
          </p>
        </section>
      </div>
    </main>
  );
}
