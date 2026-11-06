import React, { useEffect, useState } from 'react';
import QRCode from 'qrcode';

interface PendingPairingRequest {
  challengeId: string;
  clientName: string;
  clientIp: string;
  code: string;
  expiresInSecs: number;
}

interface PairingQRCodeProps {
  pendingPairing: PendingPairingRequest;
  controlPort?: number;
  hostName?: string;
  hostIp?: string;
  onApprove?: () => void;
  onReject?: () => void;
}

export const PairingQRCode: React.FC<PairingQRCodeProps> = ({
  pendingPairing,
  controlPort = 40120,
  hostName = 'Desktop-WASAPI',
  hostIp = '192.168.1.105',
  onApprove,
  onReject,
}) => {
  const [qrDataUrl, setQrDataUrl] = useState<string>('');
  const [copiedPayload, setCopiedPayload] = useState<boolean>(false);
  const [copiedUri, setCopiedUri] = useState<boolean>(false);

  const pairingPayload = JSON.stringify({
    protocol: 'audiobridge',
    v: 1,
    type: 'pairing_challenge',
    challenge_id: pendingPairing.challengeId,
    host_name: hostName,
    ip: hostIp,
    port: controlPort,
    code: pendingPairing.code,
    client_ip: pendingPairing.clientIp,
    client_name: pendingPairing.clientName,
    expires_in: pendingPairing.expiresInSecs,
  });

  const pairingUri = `audiobridge://pair?challenge=${encodeURIComponent(
    pendingPairing.challengeId
  )}&code=${encodeURIComponent(pendingPairing.code)}&host=${encodeURIComponent(
    hostName
  )}&ip=${encodeURIComponent(hostIp)}&port=${controlPort}`;

  // Generate QR Code data URL whenever pendingPairing changes
  useEffect(() => {
    let isMounted = true;
    QRCode.toDataURL(pairingPayload, {
      width: 220,
      margin: 1,
      color: {
        dark: '#0B101E',
        light: '#FFFFFF',
      },
      errorCorrectionLevel: 'M',
    })
      .then((url) => {
        if (isMounted) setQrDataUrl(url);
      })
      .catch((err) => {
        console.error('Failed to generate QR Code:', err);
      });

    return () => {
      isMounted = false;
    };
  }, [pendingPairing, controlPort, hostName, hostIp]);

  const handleCopyUri = () => {
    navigator.clipboard.writeText(pairingUri);
    setCopiedUri(true);
    setTimeout(() => setCopiedUri(false), 2000);
  };

  const handleCopyPayload = () => {
    navigator.clipboard.writeText(pairingPayload);
    setCopiedPayload(true);
    setTimeout(() => setCopiedPayload(false), 2000);
  };

  return (
    <div
      style={{
        backgroundColor: '#131A2B',
        border: '2px solid #00F0FF',
        borderRadius: '16px',
        padding: '24px',
        boxShadow: '0 0 30px rgba(0, 240, 255, 0.2)',
      }}
    >
      {/* Header */}
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
        <div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <span
              style={{
                fontSize: '11px',
                fontWeight: 700,
                color: '#00F0FF',
                backgroundColor: 'rgba(0, 240, 255, 0.12)',
                padding: '3px 8px',
                borderRadius: '6px',
                letterSpacing: '1px',
              }}
            >
              SECURE PAIRING CHALLENGE (ADR-004)
            </span>
            <span
              style={{
                fontSize: '11px',
                fontWeight: 700,
                color: '#A855F7',
                backgroundColor: 'rgba(168, 85, 247, 0.15)',
                padding: '3px 8px',
                borderRadius: '6px',
              }}
            >
              QR CODE ACTIVE
            </span>
          </div>
          <h3 style={{ fontSize: '18px', fontWeight: 700, color: '#FFFFFF', marginTop: '6px' }}>
            {pendingPairing.clientName} ({pendingPairing.clientIp})
          </h3>
          <p style={{ fontSize: '13px', color: '#94A3B8', marginTop: '2px' }}>
            Scan the QR code below from your Android phone or confirm the 6-digit one-time PIN:
          </p>
        </div>

        <div style={{ textAlign: 'right' }}>
          <span
            style={{
              fontSize: '12px',
              fontWeight: 700,
              color: pendingPairing.expiresInSecs < 30 ? '#EF4444' : '#EAB308',
              backgroundColor: 'rgba(15, 23, 42, 0.8)',
              padding: '6px 12px',
              borderRadius: '8px',
              border: '1px solid #1E293B',
            }}
          >
            ⏱ Expires in {pendingPairing.expiresInSecs}s
          </span>
        </div>
      </div>

      {/* Two-Column Responsive Layout: Left PIN / Right QR */}
      <div
        style={{
          display: 'grid',
          gridTemplateColumns: '1.2fr 1fr',
          gap: '24px',
          alignItems: 'center',
          margin: '20px 0',
          padding: '20px',
          backgroundColor: '#0B101E',
          borderRadius: '12px',
          border: '1px solid #1E293B',
        }}
      >
        {/* Left Column: 6-Digit PIN Display */}
        <div>
          <span style={{ fontSize: '11px', fontWeight: 700, color: '#64748B', letterSpacing: '0.5px' }}>
            ONE-TIME NUMERIC VERIFICATION CODE
          </span>
          <div
            style={{
              margin: '10px 0',
              padding: '16px',
              backgroundColor: '#131A2B',
              borderRadius: '10px',
              textAlign: 'center',
              letterSpacing: '8px',
              fontSize: '32px',
              fontWeight: 800,
              color: '#00F0FF',
              fontFamily: 'monospace',
              border: '1px solid #00F0FF',
              boxShadow: '0 0 15px rgba(0, 240, 255, 0.15)',
            }}
          >
            {pendingPairing.code}
          </div>

          <div style={{ fontSize: '12px', color: '#94A3B8', lineHeight: '1.5' }}>
            <div>
              Challenge ID: <code style={{ color: '#00E676' }}>{pendingPairing.challengeId}</code>
            </div>
            <div>
              Host Endpoint: <code style={{ color: '#FFFFFF' }}>{hostIp}:{controlPort}</code>
            </div>
          </div>

          <div style={{ display: 'flex', gap: '8px', marginTop: '16px' }}>
            <button
              onClick={handleCopyUri}
              style={{
                padding: '6px 12px',
                borderRadius: '6px',
                border: '1px solid #1E293B',
                backgroundColor: '#1E293B',
                color: copiedUri ? '#00E676' : '#94A3B8',
                fontSize: '11px',
                fontWeight: 600,
                cursor: 'pointer',
              }}
            >
              {copiedUri ? '✓ Copied Pairing URI' : 'Copy Pairing URI'}
            </button>
            <button
              onClick={handleCopyPayload}
              style={{
                padding: '6px 12px',
                borderRadius: '6px',
                border: '1px solid #1E293B',
                backgroundColor: '#1E293B',
                color: copiedPayload ? '#00E676' : '#94A3B8',
                fontSize: '11px',
                fontWeight: 600,
                cursor: 'pointer',
              }}
            >
              {copiedPayload ? '✓ Copied JSON' : 'Copy Payload'}
            </button>
          </div>
        </div>

        {/* Right Column: High-Contrast QR Code Card */}
        <div
          style={{
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            padding: '16px',
            backgroundColor: '#131A2B',
            borderRadius: '12px',
            border: '1px solid #1E293B',
          }}
        >
          <div
            style={{
              backgroundColor: '#FFFFFF',
              padding: '10px',
              borderRadius: '10px',
              boxShadow: '0 4px 15px rgba(0, 0, 0, 0.4)',
              display: 'flex',
              justifyContent: 'center',
              alignItems: 'center',
            }}
          >
            {qrDataUrl ? (
              <img
                src={qrDataUrl}
                alt="AudioBridge Pairing QR Code"
                style={{
                  width: '180px',
                  height: '180px',
                  display: 'block',
                  imageRendering: 'pixelated',
                }}
              />
            ) : (
              <div
                style={{
                  width: '180px',
                  height: '180px',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  color: '#64748B',
                  fontSize: '12px',
                }}
              >
                Generating QR...
              </div>
            )}
          </div>

          <div style={{ textAlign: 'center', marginTop: '12px' }}>
            <div style={{ fontSize: '11px', fontWeight: 700, color: '#00F0FF', letterSpacing: '0.5px' }}>
              📷 INSTANT SCAN CONNECTION
            </div>
            <div style={{ fontSize: '11px', color: '#64748B', marginTop: '2px', maxWidth: '200px' }}>
              Scan from Android camera or AudioBridge app to auto-confirm
            </div>
          </div>
        </div>
      </div>

      {/* Action Buttons */}
      <div style={{ display: 'flex', justifyContent: 'flex-end', gap: '12px' }}>
        {onReject && (
          <button
            onClick={onReject}
            style={{
              padding: '10px 20px',
              borderRadius: '8px',
              border: '1px solid #EF4444',
              backgroundColor: 'transparent',
              color: '#EF4444',
              fontWeight: 600,
              fontSize: '13px',
              cursor: 'pointer',
            }}
          >
            Reject Request
          </button>
        )}
        {onApprove && (
          <button
            onClick={onApprove}
            style={{
              padding: '10px 24px',
              borderRadius: '8px',
              border: 'none',
              backgroundColor: '#00E676',
              color: '#0B101E',
              fontWeight: 700,
              fontSize: '13px',
              cursor: 'pointer',
              boxShadow: '0 0 15px rgba(0, 230, 118, 0.3)',
            }}
          >
            Approve & Trust Device
          </button>
        )}
      </div>
    </div>
  );
};
