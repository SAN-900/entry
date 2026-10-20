const releaseGates = [
  {
    number: '01',
    title: 'Prove ticket controls on a validator',
    description:
      'Validate the Bubblegum V2 tree and Core collection plugins, then exercise mint, freeze, resale, direct transfer, and payment rollback against a local validator.',
    status: 'Required before event creation',
  },
  {
    number: '02',
    title: 'Connect a trusted transaction service',
    description:
      'Build bounded purchase and resale transactions from current event state and fresh DAS proofs. Gatekeeper co-signing must stay server-side.',
    status: 'Not connected',
  },
  {
    number: '03',
    title: 'Enable online ticket redemption',
    description:
      'Verify ticket ownership before submitting a single-use check-in. Offline scanning is intentionally not offered.',
    status: 'Not connected',
  },
  {
    number: '04',
    title: 'Provision and monitor devnet resources',
    description:
      'Create the Bubblegum tree and Core collection, configure event authorities, and rehearse recovery and pause procedures.',
    status: 'Operator setup required',
  },
];

const protocolFeatures = [
  'Organizer-controlled event pause and resume',
  'Rotatable gate authority',
  'Per-event check-in bitmap and bounded batch redemption',
  'Wallet purchase limits and capped, settled resale',
];

export default function Home() {
  return (
    <main className="shell">
      <header className="topbar">
        <a className="brand" href="/" aria-label="ENTRY Organizer home">
          <span className="brand-mark" aria-hidden="true">E</span>
          <span>ENTRY <span className="brand-subtitle">ORGANIZER</span></span>
        </a>
        <div className="topbar-actions">
          <span className="network-pill"><span aria-hidden="true" />Devnet readiness</span>
          <button className="wallet-button" type="button" disabled title="Wallet connection is not implemented yet">
            Connect wallet
          </button>
        </div>
      </header>

      <section className="hero">
        <div className="hero-copy">
          <p className="eyebrow">Organizer workspace</p>
          <h1>Build an event people can trust.</h1>
          <p className="hero-description">
            Create transparent ticket rules, manage gate access, and keep every
            release milestone visible before tickets go on sale.
          </p>
        </div>
        <div className="hero-status" role="status" aria-live="polite">
          <span className="status-indicator" aria-hidden="true" />
          <div>
            <strong>Setup in progress</strong>
            <span>Transactions are not enabled</span>
          </div>
        </div>
      </section>

      <section className="workspace-grid" aria-label="Organizer workspace">
        <section className="panel events-panel" aria-labelledby="events-title">
          <div className="section-heading">
            <div>
              <p className="eyebrow">Your workspace</p>
              <h2 id="events-title">Events</h2>
            </div>
            <button
              className="secondary-button"
              type="button"
              disabled
              title="Event creation is blocked until asset policy validation is implemented"
            >
              <span aria-hidden="true">+</span> Create event
            </button>
          </div>
          <div className="empty-state">
            <span className="empty-icon" aria-hidden="true">✳</span>
            <h3>No events connected</h3>
            <p>
              Connect an organizer wallet to view on-chain events. Event
              creation will become available after the asset-policy release
              gate passes.
            </p>
            <span className="text-link" aria-disabled="true">Wallet integration pending</span>
          </div>
        </section>

        <aside className="panel protocol-panel" aria-labelledby="protocol-title">
          <p className="eyebrow">Program capabilities</p>
          <h2 id="protocol-title">On-chain foundation</h2>
          <p className="panel-description">
            Implemented in the Anchor program; organizer controls are not yet
            wired to this app.
          </p>
          <ul className="feature-list">
            {protocolFeatures.map((feature) => (
              <li key={feature}>
                <span className="check-icon" aria-hidden="true">✓</span>
                {feature}
              </li>
            ))}
          </ul>
          <div className="notice">
            <strong>Prototype boundary</strong>
            <p>
              Bubblegum V2 transfers, collection plugins, and atomic payment
              rollback have not been validated on a running validator.
            </p>
          </div>
        </aside>
      </section>

      <section className="release-section" aria-labelledby="release-title">
        <div className="section-heading release-heading">
          <div>
            <p className="eyebrow">Release checklist</p>
            <h2 id="release-title">What must happen before launch</h2>
          </div>
          <span className="release-count">4 release gates</span>
        </div>
        <ol className="release-list">
          {releaseGates.map((gate) => (
            <li className="release-item" key={gate.number}>
              <span className="release-number">{gate.number}</span>
              <div className="release-copy">
                <h3>{gate.title}</h3>
                <p>{gate.description}</p>
              </div>
              <span className="release-status">{gate.status}</span>
            </li>
          ))}
        </ol>
      </section>

      <footer className="footer">
        <span>ENTRY Organizer</span>
        <span>Devnet only · No customer payments · No production claims</span>
      </footer>
    </main>
  );
}
