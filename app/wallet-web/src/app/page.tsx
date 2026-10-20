'use client';

import { useCallback, useEffect, useState } from 'react';
import { PublicKey, VersionedTransaction } from '@solana/web3.js';
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';

type ActionSummary = {
  amount: string;
  token: 'USDC';
  feePayer: string;
  cluster: 'devnet';
  payments: Array<{ recipient: string; amount: string }>;
};

type PreparedAction = {
  transaction: VersionedTransaction;
  message: string;
  summary: ActionSummary;
};

const actionsUrl = process.env.NEXT_PUBLIC_ACTIONS_URL
  || 'http://localhost:3004/api/actions/buy';
const usdcMintAddress = process.env.NEXT_PUBLIC_USDC_MINT;

function decodeBase64(value: string): Uint8Array {
  const binary = atob(value);
  return Uint8Array.from(binary, (character) => character.charCodeAt(0));
}

function isActionSummary(value: unknown): value is ActionSummary {
  if (typeof value !== 'object' || value === null) return false;
  const summary = value as Partial<ActionSummary>;
  if (
    typeof summary.amount !== 'string'
    || summary.token !== 'USDC'
    || typeof summary.feePayer !== 'string'
    || summary.cluster !== 'devnet'
    || !Array.isArray(summary.payments)
    || summary.payments.length === 0
    || summary.payments.length > 4
  ) return false;

  try {
    new PublicKey(summary.feePayer);
    for (const payment of summary.payments) {
      new PublicKey(payment.recipient);
      if (!/^\d+(?:\.\d{1,6})?$/.test(payment.amount)) return false;
    }
  } catch {
    return false;
  }
  return /^\d+(?:\.\d{1,6})?$/.test(summary.amount);
}

export default function Home() {
  const { connection } = useConnection();
  const { publicKey, connected, sendTransaction } = useWallet();
  const [usdcBalance, setUsdcBalance] = useState<string | null>(null);
  const [balanceError, setBalanceError] = useState<string | null>(null);
  const [eventId, setEventId] = useState('');
  const [preparedAction, setPreparedAction] = useState<PreparedAction | null>(null);
  const [simulationMessage, setSimulationMessage] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);
  const [working, setWorking] = useState(false);
  const [signature, setSignature] = useState<string | null>(null);

  const refreshBalance = useCallback(async () => {
    if (!publicKey || !usdcMintAddress) {
      setUsdcBalance(null);
      setBalanceError(usdcMintAddress ? null : 'USDC mint is not configured.');
      return;
    }

    setBalanceError(null);
    try {
      const mint = new PublicKey(usdcMintAddress);
      const accounts = await connection.getParsedTokenAccountsByOwner(publicKey, { mint });
      const balance = accounts.value.reduce((total, account) => {
        const tokenAmount = account.account.data.parsed.info.tokenAmount;
        return total + Number(tokenAmount.uiAmountString ?? 0);
      }, 0);
      setUsdcBalance(balance.toFixed(6).replace(/\.?0+$/, ''));
    } catch (error) {
      setBalanceError(error instanceof Error ? error.message : 'Could not load the USDC balance.');
    }
  }, [connection, publicKey]);

  useEffect(() => {
    void refreshBalance();
  }, [refreshBalance]);

  const preparePurchase = async () => {
    if (!publicKey) {
      setActionError('Connect a wallet before preparing checkout.');
      return;
    }
    if (!/^(0|[1-9]\d{0,19})$/.test(eventId)) {
      setActionError('Enter a valid event ID.');
      return;
    }

    setWorking(true);
    setActionError(null);
    setPreparedAction(null);
    setSimulationMessage(null);
    setSignature(null);
    try {
      const metadataUrl = new URL(actionsUrl);
      metadataUrl.searchParams.set('event', eventId);
      const metadataResponse = await fetch(metadataUrl);
      const metadata = await metadataResponse.json() as {
        links?: { actions?: Array<{ href?: string }> };
        message?: string;
      };
      if (!metadataResponse.ok) {
        throw new Error(metadata.message || 'The Actions API could not load this event.');
      }

      const href = metadata.links?.actions?.[0]?.href;
      if (!href) throw new Error('The Actions API did not provide a purchase action.');
      const actionUrl = new URL(href, metadataUrl);
      actionUrl.searchParams.set('event', eventId);
      const response = await fetch(actionUrl, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ account: publicKey.toBase58() }),
      });
      const result = await response.json() as {
        transaction?: unknown;
        message?: unknown;
        summary?: unknown;
      };
      if (!response.ok) {
        throw new Error(typeof result.message === 'string' ? result.message : 'Checkout could not be prepared.');
      }
      if (
        typeof result.transaction !== 'string'
        || typeof result.message !== 'string'
        || !isActionSummary(result.summary)
        || result.summary.feePayer !== publicKey.toBase58()
      ) {
        throw new Error('The Actions API returned an invalid or incomplete transaction summary.');
      }

      setPreparedAction({
        transaction: VersionedTransaction.deserialize(decodeBase64(result.transaction)),
        message: result.message,
        summary: result.summary,
      });
    } catch (error) {
      setActionError(error instanceof Error ? error.message : 'Checkout could not be prepared.');
    } finally {
      setWorking(false);
    }
  };

  const simulatePurchase = async () => {
    if (!preparedAction) return;
    setWorking(true);
    setSimulationMessage(null);
    setActionError(null);
    try {
      const result = await connection.simulateTransaction(preparedAction.transaction, {
        commitment: 'confirmed',
        replaceRecentBlockhash: true,
        sigVerify: false,
      });
      if (result.value.err) {
        const detail = result.value.logs?.slice(-4).join(' ') || JSON.stringify(result.value.err);
        throw new Error(`Simulation failed: ${detail}`);
      }
      setSimulationMessage('Simulation passed. Review the payment details and confirm to open your wallet for signing.');
    } catch (error) {
      setActionError(error instanceof Error ? error.message : 'Transaction simulation failed.');
    } finally {
      setWorking(false);
    }
  };

  const confirmPurchase = async () => {
    if (!preparedAction || !publicKey || !simulationMessage) return;
    setWorking(true);
    setActionError(null);
    try {
      const sentSignature = await sendTransaction(preparedAction.transaction, connection);
      setSignature(sentSignature);
      setPreparedAction(null);
      setSimulationMessage(null);
      await connection.confirmTransaction(sentSignature, 'confirmed');
      await refreshBalance();
    } catch (error) {
      setActionError(error instanceof Error ? error.message : 'Transaction was not confirmed.');
    } finally {
      setWorking(false);
    }
  };

  return (
    <main className="shell">
      <header className="topbar">
        <a className="brand" href="/" aria-label="ENTRY Tickets home">
          <span className="brand-mark" aria-hidden="true">E</span>
          <span>ENTRY <span className="brand-subtitle">TICKETS</span></span>
        </a>
        <div className="topbar-actions">
          <span className="network-pill"><span aria-hidden="true" />Solana Devnet</span>
          <WalletMultiButton className="wallet-button" />
        </div>
      </header>

      <section className="hero">
        <div className="hero-copy">
          <p className="eyebrow">Your wallet workspace</p>
          <h1>Your tickets.<br />Your rules.</h1>
          <p className="hero-description">
            Connect a wallet to check your Devnet USDC balance and prepare
            event checkout through the ENTRY transaction service.
          </p>
        </div>
        <div className="wallet-status" role="status">
          <span className="status-indicator" aria-hidden="true" />
          <div>
            <strong>{connected ? 'Wallet connected' : 'Wallet not connected'}</strong>
            <span>{publicKey?.toBase58() ?? 'Connect Phantom, Solflare, or a Wallet Standard wallet'}</span>
          </div>
        </div>
      </section>

      <section className="content-grid" aria-label="Ticket workspace">
        <section className="panel tickets-panel" aria-labelledby="tickets-title">
          <div className="section-heading">
            <div>
              <p className="eyebrow">Wallet inventory</p>
              <h2 id="tickets-title">My tickets</h2>
            </div>
            <span className="count-pill">{connected ? 'Wallet connected' : 'Not connected'}</span>
          </div>

          <div className="empty-state">
            <span className="empty-icon" aria-hidden="true">◇</span>
            <h3>{connected ? 'Ticket lookup is not configured.' : 'Connect to get started.'}</h3>
            <p>
              {connected
                ? 'Ticket indexing and ownership-proof lookup are not configured. No ticket is assumed from wallet connection alone.'
                : 'Connect your wallet to check the configured Devnet USDC balance and prepare checkout.'}
            </p>
            {usdcBalance !== null && <p className="balance-value">USDC balance: {usdcBalance}</p>}
            {balanceError && <p className="inline-error" role="alert">{balanceError}</p>}
          </div>

          <form
            className="checkout-form"
            onSubmit={(event) => {
              event.preventDefault();
              void preparePurchase();
            }}
          >
            <div className="checkout-copy">
              <p className="eyebrow">Devnet checkout</p>
              <h3>Buy a ticket</h3>
              <p>Payment is available only when the trusted transaction builder returns a simulated, reviewable USDC purchase.</p>
            </div>
            <label htmlFor="event-id">Event ID</label>
            <div className="checkout-controls">
              <input
                id="event-id"
                inputMode="numeric"
                pattern="[0-9]*"
                value={eventId}
                onChange={(event) => setEventId(event.target.value)}
                placeholder="Enter event ID"
                aria-describedby="checkout-message"
              />
              <button className="primary-button" type="submit" disabled={!connected || working}>
                {working ? 'Preparing…' : 'Prepare checkout'}
              </button>
            </div>
            {(actionError || simulationMessage) && (
              <p className={actionError ? 'inline-error' : 'inline-success'} id="checkout-message" role={actionError ? 'alert' : 'status'}>
                {actionError || simulationMessage}
              </p>
            )}
          </form>

          {preparedAction && (
            <section className="transaction-review" aria-labelledby="review-title">
              <p className="eyebrow">Review before signing</p>
              <h3 id="review-title">{preparedAction.message}</h3>
              <dl>
                <div><dt>Event</dt><dd>{eventId}</dd></div>
                <div><dt>Total payment</dt><dd>{preparedAction.summary.amount} {preparedAction.summary.token}</dd></div>
                {preparedAction.summary.payments.map((payment) => (
                  <div key={payment.recipient}>
                    <dt>Recipient</dt>
                    <dd>{payment.amount} {preparedAction.summary.token} · {payment.recipient}</dd>
                  </div>
                ))}
                <div><dt>Fee payer</dt><dd>{preparedAction.summary.feePayer}</dd></div>
                <div><dt>Cluster</dt><dd>{preparedAction.summary.cluster}</dd></div>
              </dl>
              <div className="ticket-actions">
                <button className="secondary-button" type="button" onClick={() => void simulatePurchase()} disabled={working}>
                  Simulate transaction
                </button>
                <button className="primary-button" type="button" onClick={() => void confirmPurchase()} disabled={working || !simulationMessage}>
                  Confirm and sign
                </button>
              </div>
            </section>
          )}

          {signature && (
            <p className="inline-success" role="status">
              Transaction confirmed: <a href={`https://explorer.solana.com/tx/${signature}?cluster=devnet`} target="_blank" rel="noreferrer">{signature}</a>
            </p>
          )}
        </section>

        <aside className="panel rules-panel" aria-labelledby="rules-title">
          <p className="eyebrow">Devnet purchase flow</p>
          <h2 id="rules-title">Wallet-signed checkout.</h2>
          <p className="panel-description">
            The wallet signs only after an event-specific transaction is
            provided by the trusted builder, simulated, and reviewed.
          </p>
          <ul className="benefit-list">
            <li><span className="check-icon" aria-hidden="true">✓</span>USDC amount and recipient are shown before signing.</li>
            <li><span className="check-icon" aria-hidden="true">✓</span>Transaction simulation runs before wallet approval.</li>
            <li><span className="check-icon" aria-hidden="true">✓</span>Wallet connection never triggers a transfer.</li>
          </ul>
          <div className="notice">
            <strong>Backend setup required</strong>
            <p>
              Purchase transactions need a configured builder with current
              Bubblegum proofs and gatekeeper co-signing. The API rejects
              checkout when that service is absent.
            </p>
          </div>
        </aside>
      </section>

      <footer className="footer">
        <span>ENTRY Tickets</span>
        <span>Devnet only · Wallet signature required · No mainnet payments</span>
      </footer>
    </main>
  );
}
