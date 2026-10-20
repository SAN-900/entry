'use client';

import { useCallback, useEffect, useRef, useState } from 'react';
import { BrowserQRCodeReader, type IScannerControls } from '@zxing/browser';
import { PublicKey, VersionedTransaction } from '@solana/web3.js';
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';
import { parseFreshQr, parseQrPayload, type EntryQrPayload } from '@entry/shared';

type QueuedScan = {
  id: string;
  payload: EntryQrPayload;
  scannedAt: number;
};

type CheckInSummary = {
  eventId: string;
  ticketAsset: string;
  gateAuthority: string;
  feePayer: string;
  cluster: 'devnet';
};

type PreparedCheckIn = {
  scanId: string;
  transaction: VersionedTransaction;
  message: string;
  summary: CheckInSummary;
};

const queueStorageKey = 'entry.gate.pending-scans.v1';

function decodeBase64(value: string): Uint8Array {
  const binary = atob(value);
  return Uint8Array.from(binary, (character) => character.charCodeAt(0));
}

function isCheckInSummary(value: unknown): value is CheckInSummary {
  if (typeof value !== 'object' || value === null) return false;
  const summary = value as Partial<CheckInSummary>;
  if (
    typeof summary.eventId !== 'string'
    || typeof summary.ticketAsset !== 'string'
    || typeof summary.gateAuthority !== 'string'
    || typeof summary.feePayer !== 'string'
    || summary.cluster !== 'devnet'
  ) return false;
  try {
    new PublicKey(summary.ticketAsset);
    new PublicKey(summary.gateAuthority);
    new PublicKey(summary.feePayer);
  } catch {
    return false;
  }
  return true;
}

export default function Home() {
  const { connection } = useConnection();
  const { publicKey, connected, sendTransaction } = useWallet();
  const videoRef = useRef<HTMLVideoElement>(null);
  const scannerRef = useRef<IScannerControls | null>(null);
  const [queue, setQueue] = useState<QueuedScan[]>([]);
  const [queueLoaded, setQueueLoaded] = useState(false);
  const [cameraActive, setCameraActive] = useState(false);
  const [workingScanId, setWorkingScanId] = useState<string | null>(null);
  const [prepared, setPrepared] = useState<PreparedCheckIn | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [signature, setSignature] = useState<string | null>(null);
  const [isOnline, setIsOnline] = useState(true);
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    try {
      const stored = localStorage.getItem(queueStorageKey);
      if (stored) {
        const parsed: unknown = JSON.parse(stored);
        if (!Array.isArray(parsed)) throw new Error('Saved scan queue has an invalid format.');
        const restored: QueuedScan[] = [];
        for (const item of parsed as unknown[]) {
          if (typeof item !== 'object' || item === null) throw new Error('Saved scan queue contains invalid data.');
          const entry = item as Partial<QueuedScan>;
          const payload = parseQrPayload(entry.payload);
          if (
            typeof entry.id !== 'string'
            || !Number.isSafeInteger(entry.scannedAt)
            || !payload
          ) continue;
          restored.push({ id: entry.id, payload, scannedAt: entry.scannedAt as number });
        }
        setQueue(restored);
      }
    } catch (loadError) {
      setError(loadError instanceof Error ? loadError.message : 'Could not load saved scans.');
    } finally {
      setQueueLoaded(true);
    }
  }, []);

  useEffect(() => {
    if (!queueLoaded) return;
    try {
      localStorage.setItem(queueStorageKey, JSON.stringify(queue));
    } catch (storageError) {
      setError(storageError instanceof Error ? storageError.message : 'Could not save the scan queue.');
    }
  }, [queue, queueLoaded]);

  useEffect(() => {
    const updateOnline = () => setIsOnline(navigator.onLine);
    updateOnline();
    window.addEventListener('online', updateOnline);
    window.addEventListener('offline', updateOnline);
    if ('serviceWorker' in navigator) {
      void navigator.serviceWorker.register('/service-worker.js').catch((registrationError: unknown) => {
        setError(registrationError instanceof Error ? registrationError.message : 'Could not register offline app support.');
      });
    }
    return () => {
      window.removeEventListener('online', updateOnline);
      window.removeEventListener('offline', updateOnline);
    };
  }, []);

  useEffect(() => () => {
    scannerRef.current?.stop();
  }, []);

  const stopScanner = useCallback(() => {
    scannerRef.current?.stop();
    scannerRef.current = null;
    setCameraActive(false);
  }, []);

  const queuePayload = useCallback((payload: EntryQrPayload) => {
    const duplicate = queue.some((entry) =>
      entry.payload.event === payload.event
      && entry.payload.assetId === payload.assetId
      && entry.payload.nonce === payload.nonce,
    );
    if (duplicate) {
      setNotice('This ticket is already in the pending queue. It has not been admitted.');
      return;
    }
    if (queue.length >= 50) {
      setError('The local pending queue is full. Sync or clear pending scans before adding more.');
      return;
    }
    setQueue([...queue, {
      id: crypto.randomUUID(),
      payload,
      scannedAt: Date.now(),
    }]);
    setNotice('Scan saved locally as unverified and pending. No admission was granted.');
    setSignature(null);
  }, [queue]);

  const startScanner = async () => {
    setError(null);
    setNotice(null);
    if (!videoRef.current) {
      setError('Camera preview is not available in this browser.');
      return;
    }

    try {
      const reader = new BrowserQRCodeReader();
      const controls = await reader.decodeFromVideoDevice(
        undefined,
        videoRef.current,
        (result) => {
          if (!result) return;
          const payload = parseFreshQr(result.getText());
          stopScanner();
          if (!payload) {
            setError('QR code is invalid or expired. It was not added to the queue.');
            return;
          }
          queuePayload(payload);
        },
      );
      scannerRef.current = controls;
      setCameraActive(true);
    } catch (cameraError) {
      stopScanner();
      setError(cameraError instanceof Error ? cameraError.message : 'Camera access failed.');
    }
  };

  const prepareCheckIn = async (scan: QueuedScan) => {
    if (!publicKey) {
      setError('Connect the configured gate-authority wallet before online verification.');
      return;
    }

    setWorkingScanId(scan.id);
    setPrepared(null);
    setError(null);
    setNotice(null);
    try {
      const response = await fetch('/api/check-in', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ account: publicKey.toBase58(), payload: scan.payload }),
      });
      const result = await response.json() as {
        transaction?: unknown;
        message?: unknown;
        summary?: unknown;
      };
      if (!response.ok) {
        throw new Error(typeof result.message === 'string' ? result.message : 'Online verification is unavailable.');
      }
      if (
        typeof result.transaction !== 'string'
        || typeof result.message !== 'string'
        || !isCheckInSummary(result.summary)
        || result.summary.feePayer !== publicKey.toBase58()
        || result.summary.eventId !== scan.payload.event
        || result.summary.ticketAsset !== scan.payload.assetId
        || result.summary.gateAuthority !== publicKey.toBase58()
      ) {
        throw new Error('Check-in service returned an invalid transaction or authority summary.');
      }

      setPrepared({
        scanId: scan.id,
        transaction: VersionedTransaction.deserialize(decodeBase64(result.transaction)),
        message: result.message,
        summary: result.summary,
      });
    } catch (requestError) {
      setError(requestError instanceof Error ? requestError.message : 'Online verification failed.');
    } finally {
      setWorkingScanId(null);
    }
  };

  const simulateCheckIn = async () => {
    if (!prepared) return;
    setError(null);
    setNotice(null);
    try {
      const result = await connection.simulateTransaction(prepared.transaction, {
        commitment: 'confirmed',
        replaceRecentBlockhash: true,
        sigVerify: false,
      });
      if (result.value.err) {
        const detail = result.value.logs?.slice(-4).join(' ') || JSON.stringify(result.value.err);
        throw new Error(`Simulation failed: ${detail}`);
      }
      setNotice('Simulation passed. Review the event and gate authority, then confirm in your wallet.');
    } catch (simulationError) {
      setError(simulationError instanceof Error ? simulationError.message : 'Check-in simulation failed.');
    }
  };

  const confirmCheckIn = async () => {
    if (!prepared || !publicKey || !notice?.startsWith('Simulation passed')) return;
    setSubmitting(true);
    setError(null);
    setNotice(null);
    try {
      const sentSignature = await sendTransaction(prepared.transaction, connection);
      setSignature(sentSignature);
      await connection.confirmTransaction(sentSignature, 'confirmed');
      setQueue((current) => current.filter((scan) => scan.id !== prepared.scanId));
      setPrepared(null);
      setNotice('On-chain redemption confirmed. The pending scan was removed.');
    } catch (sendError) {
      setError(sendError instanceof Error ? sendError.message : 'Check-in transaction was not confirmed.');
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <main className="shell">
      <header className="topbar">
        <a className="brand" href="/" aria-label="ENTRY Gate home">
          <span className="brand-mark" aria-hidden="true">E</span>
          <span>ENTRY <span className="brand-subtitle">GATE</span></span>
        </a>
        <div className="topbar-actions">
          <span className={`mode-pill ${isOnline ? '' : 'offline-mode'}`}><span aria-hidden="true" />{isOnline ? 'Online' : 'Offline · queue only'}</span>
          <span className="network-label">Devnet</span>
          <WalletMultiButton className="gate-wallet-button" />
        </div>
      </header>

      <section className="hero">
        <div className="hero-copy">
          <p className="eyebrow">Event access control</p>
          <h1>Verify.<br />Admit.<br />Prevent reuse.</h1>
          <p className="hero-description">
            Scan tickets, queue safely while offline, then verify and redeem
            online with the configured gate-authority wallet.
          </p>
        </div>
        <aside className="readiness-card" aria-labelledby="readiness-title">
          <div className="readiness-heading">
            <div>
              <p className="eyebrow">Gate operator</p>
              <h2 id="readiness-title">{connected ? 'Wallet connected' : 'Connect authority wallet'}</h2>
            </div>
            <span className="readiness-icon" aria-hidden="true">{connected ? '✓' : '!'}</span>
          </div>
          <p className="operator-address">{publicKey?.toBase58() ?? 'The active wallet must match the event gate authority.'}</p>
          <p className="readiness-note">
            Scans stay pending until online proof verification and a confirmed
            on-chain check-in both succeed.
          </p>
        </aside>
      </section>

      <section className="scanner-layout" aria-label="Gate scanner workspace">
        <section className="panel scanner-panel" aria-labelledby="scanner-title">
          <div className="section-heading">
            <div>
              <p className="eyebrow">Ticket capture</p>
              <h2 id="scanner-title">Camera scanner</h2>
            </div>
            <span className="offline-pill"><span aria-hidden="true" />{cameraActive ? 'Camera active' : 'Camera idle'}</span>
          </div>

          <div className="scanner-window">
            <video className="camera-preview" ref={videoRef} muted playsInline aria-label="Ticket QR camera preview" />
            {!cameraActive && <span className="camera-placeholder">Camera preview appears here</span>}
            <div className="scanner-controls">
              <button className="scan-button" type="button" onClick={() => void startScanner()} disabled={cameraActive}>
                Start camera
              </button>
              <button className="secondary-button" type="button" onClick={stopScanner} disabled={!cameraActive}>
                Stop camera
              </button>
            </div>
            <p className="scanner-explainer">
              Captures a fresh ENTRY QR and queues it locally. Scanning alone
              never admits a guest; QR signature and ticket proofs are checked online.
            </p>
          </div>
          {error && <p className="inline-error" role="alert">{error}</p>}
          {notice && <p className="inline-success" role="status">{notice}</p>}
          {signature && <p className="signature-link">Confirmed signature: <a href={`https://explorer.solana.com/tx/${signature}?cluster=devnet`} target="_blank" rel="noreferrer">{signature}</a></p>}
        </section>

        <aside className="panel activity-panel" aria-labelledby="activity-title">
          <div className="section-heading">
            <div>
              <p className="eyebrow">Persistent local queue</p>
              <h2 id="activity-title">Pending scans</h2>
            </div>
            <span className="count-pill">{queue.length} pending</span>
          </div>
          <p className="queue-warning">
            Pending is not admitted. Do not admit guests from this list until
            online redemption is confirmed.
          </p>
          {queue.length === 0 ? (
            <div className="activity-empty">
              <span className="activity-icon" aria-hidden="true">↗</span>
              <h3>No pending scans</h3>
              <p>Scanned tickets will be saved on this device until verified or cleared by confirmed redemption.</p>
            </div>
          ) : (
            <ul className="pending-list">
              {queue.map((scan) => (
                <li className="pending-item" key={scan.id}>
                  <div className="pending-copy">
                    <strong>{parseFreshQr(scan.payload) ? 'Unverified ticket' : 'Expired · cannot redeem'}</strong>
                    <span>Event {scan.payload.event} · {new Date(scan.scannedAt).toLocaleTimeString()}</span>
                    <code>{scan.payload.assetId}</code>
                  </div>
                  <button
                    className="secondary-button"
                    type="button"
                    onClick={() => void prepareCheckIn(scan)}
                    disabled={!connected || !isOnline || !parseFreshQr(scan.payload) || workingScanId !== null}
                  >
                    {workingScanId === scan.id ? 'Checking…' : 'Verify online'}
                  </button>
                </li>
              ))}
            </ul>
          )}

          {prepared && (
            <section className="checkin-review" aria-labelledby="checkin-review-title">
              <p className="eyebrow">Review redemption</p>
              <h3 id="checkin-review-title">{prepared.message}</h3>
              <dl>
                <div><dt>Event</dt><dd>{prepared.summary.eventId}</dd></div>
                <div><dt>Ticket asset</dt><dd>{prepared.summary.ticketAsset}</dd></div>
                <div><dt>Gate authority</dt><dd>{prepared.summary.gateAuthority}</dd></div>
                <div><dt>Network</dt><dd>{prepared.summary.cluster}</dd></div>
              </dl>
              <div className="scanner-controls">
                <button className="secondary-button" type="button" onClick={() => void simulateCheckIn()} disabled={!isOnline || submitting}>Simulate</button>
                <button className="scan-button" type="button" onClick={() => void confirmCheckIn()} disabled={!isOnline || submitting || !notice?.startsWith('Simulation passed')}>
                  Confirm &amp; redeem
                </button>
              </div>
            </section>
          )}
        </aside>
      </section>

      <footer className="footer">
        <span>ENTRY Gate</span>
        <span>Devnet only · Offline scans remain pending · No admission without confirmed redemption</span>
      </footer>
    </main>
  );
}
