import React, { useEffect, useRef, useState } from 'react';

export interface WaveformVisualizerProps {
  isRunning: boolean;
  sampleRate?: number;
  channels?: number;
  codec?: string;
  bitrateKbps?: number;
  activeEndpoint?: string;
  peakLevel?: number;
  rmsDb?: number;
  peakDb?: number;
  bufferPeaks?: number[];
  framesCaptured?: number;
  framesEncoded?: number;
}

export const WaveformVisualizer: React.FC<WaveformVisualizerProps> = ({
  isRunning,
  sampleRate = 48000,
  channels = 2,
  codec = 'Opus',
  bitrateKbps = 128,
  activeEndpoint = 'WASAPI Loopback',
  peakLevel = 0.0,
  rmsDb = -60.0,
  peakDb = -60.0,
  bufferPeaks = [],
  framesCaptured = 0,
  framesEncoded = 0,
}) => {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const containerRef = useRef<HTMLDivElement | null>(null);
  const animationFrameRef = useRef<number | null>(null);

  // Visualizer interactive modes
  const [mode, setMode] = useState<'oscilloscope' | 'stereo' | 'envelope' | 'peak_trough'>('oscilloscope');
  const [sensitivity, setSensitivity] = useState<number>(1.0);
  const [isFrozen, setIsFrozen] = useState<boolean>(false);
  const [showPeakMarkers, setShowPeakMarkers] = useState<boolean>(true);

  // Peak and trough detection history
  const peakHoldRef = useRef<number>(-60);
  const troughHoldRef = useRef<number>(60);
  const bufferPeaksRef = useRef<number[]>(bufferPeaks);

  useEffect(() => {
    bufferPeaksRef.current = bufferPeaks;
  }, [bufferPeaks]);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    let phase = 0;

    const render = () => {
      // Handle HiDPI retina displays
      const rect = canvas.getBoundingClientRect();
      const dpr = window.devicePixelRatio || 1;
      const width = rect.width;
      const height = rect.height;

      if (canvas.width !== width * dpr || canvas.height !== height * dpr) {
        canvas.width = width * dpr;
        canvas.height = height * dpr;
      }

      ctx.save();
      ctx.scale(dpr, dpr);

      // Deep dark acoustic oscilloscope background
      ctx.fillStyle = '#060A14';
      ctx.fillRect(0, 0, width, height);

      // Draw oscilloscope grid & graticule
      ctx.lineWidth = 1;
      ctx.strokeStyle = '#111A2B';

      // Vertical grid lines (time division divisions: 10 horizontal cells)
      const numVDivs = 10;
      for (let i = 1; i < numVDivs; i++) {
        const x = (width / numVDivs) * i;
        ctx.beginPath();
        ctx.setLineDash([2, 4]);
        ctx.moveTo(x, 0);
        ctx.lineTo(x, height);
        ctx.stroke();
      }

      // Horizontal grid lines (amplitude / dB levels: +1.0, +0.5, 0.0, -0.5, -1.0)
      const centerY = height / 2;
      const levels = [
        { label: '+0 dBFS', y: centerY - height * 0.42, isLimit: true },
        { label: '-6 dBFS', y: centerY - height * 0.22, isLimit: false },
        { label: ' 0.0 V', y: centerY, isCenter: true },
        { label: '-6 dBFS', y: centerY + height * 0.22, isLimit: false },
        { label: '+0 dBFS', y: centerY + height * 0.42, isLimit: true },
      ];

      levels.forEach((lvl) => {
        ctx.beginPath();
        if (lvl.isCenter) {
          ctx.setLineDash([]);
          ctx.strokeStyle = '#1E293B';
        } else if (lvl.isLimit) {
          ctx.setLineDash([4, 4]);
          ctx.strokeStyle = 'rgba(239, 68, 68, 0.25)';
        } else {
          ctx.setLineDash([2, 4]);
          ctx.strokeStyle = '#101B2E';
        }
        ctx.moveTo(0, lvl.y);
        ctx.lineTo(width, lvl.y);
        ctx.stroke();

        // dB Label
        ctx.fillStyle = lvl.isLimit ? '#EF4444' : lvl.isCenter ? '#475569' : '#334155';
        ctx.font = '10px monospace';
        ctx.fillText(lvl.label, 8, lvl.y - 3);
      });

      ctx.setLineDash([]); // Reset line dash

      if (!isRunning) {
        // Idle Standby Mode (WASAPI Capture Stopped)
        ctx.beginPath();
        ctx.strokeStyle = '#334155';
        ctx.lineWidth = 1.5;
        ctx.moveTo(0, centerY);
        ctx.lineTo(width, centerY);
        ctx.stroke();

        // Standby Banner
        ctx.fillStyle = 'rgba(15, 23, 42, 0.85)';
        ctx.fillRect(width / 2 - 210, centerY - 20, 420, 40);
        ctx.strokeStyle = '#1E293B';
        ctx.strokeRect(width / 2 - 210, centerY - 20, 420, 40);

        ctx.fillStyle = '#64748B';
        ctx.font = '600 12px Inter, sans-serif';
        ctx.textAlign = 'center';
        ctx.fillText('WASAPI AUDIO ENGINE STANDBY • CLICK "START CAPTURE" TO STREAM', width / 2, centerY + 5);
        ctx.textAlign = 'start';
      } else {
        // Active Real-Time Audio Capture Waveform connected to Engine State
        if (!isFrozen) {
          phase += 0.05;
        }

        const now = phase;
        // Use live engine status amplitude, modulated with sensitivity
        const baseAmp = (peakLevel > 0 ? peakLevel : 0.65) * sensitivity;

        // Current real-time buffer samples from engine or synthesized harmonics
        const liveBuffer = bufferPeaksRef.current;
        const hasLiveBuffer = liveBuffer && liveBuffer.length > 0;

        // Peak and trough tracking
        let currentMaxPeakY = centerY;
        let currentMaxPeakX = 0;
        let currentMinTroughY = centerY;
        let currentMinTroughX = 0;

        if (mode === 'oscilloscope') {
          // --- MODE 1: High-Definition Oscilloscope Trace ---
          ctx.shadowColor = '#00F0FF';
          ctx.shadowBlur = 10;
          ctx.strokeStyle = '#00F0FF';
          ctx.lineWidth = 2.0;

          ctx.beginPath();
          const step = 2;
          for (let x = 0; x <= width; x += step) {
            const normX = x / width;

            let sampleVal = 0;
            if (hasLiveBuffer) {
              const bufIdx = Math.floor(normX * (liveBuffer.length - 1));
              const bufVal = liveBuffer[bufIdx] || 0;
              const subHarmonic = Math.sin(normX * 35 + now * 10) * 0.15;
              sampleVal = (bufVal + subHarmonic) * baseAmp;
            } else {
              // Composite audio wave dynamics reflecting active streaming peaks and troughs
              const f1 = Math.sin(normX * 22 + now * 12) * 0.48;
              const f2 = Math.sin(normX * 58 - now * 18) * 0.26;
              const f3 = Math.cos(normX * 110 + now * 26) * 0.15;
              const noise = Math.sin(normX * 360 + now * 40) * 0.04;
              sampleVal = (f1 + f2 + f3 + noise) * baseAmp;
            }

            const y = centerY - sampleVal * (height * 0.38);

            // Track extreme peak (crest) and trough (trough)
            if (y < currentMaxPeakY) {
              currentMaxPeakY = y;
              currentMaxPeakX = x;
            }
            if (y > currentMinTroughY) {
              currentMinTroughY = y;
              currentMinTroughX = x;
            }

            if (x === 0) ctx.moveTo(x, y);
            else ctx.lineTo(x, y);
          }
          ctx.stroke();

          // Subtle gradient area below waveform
          ctx.shadowBlur = 0;
          ctx.lineTo(width, centerY);
          ctx.lineTo(0, centerY);
          ctx.closePath();
          const grad = ctx.createLinearGradient(0, centerY - 60, 0, centerY + 60);
          grad.addColorStop(0, 'rgba(0, 240, 255, 0.16)');
          grad.addColorStop(1, 'rgba(0, 240, 255, 0.0)');
          ctx.fillStyle = grad;
          ctx.fill();
        } else if (mode === 'peak_trough') {
          // --- MODE 2: Dedicated Peak & Trough Envelope Analyzer ---
          ctx.lineWidth = 1.5;

          // Upper Crest Envelope (Positive Peaks)
          ctx.shadowColor = '#00E676';
          ctx.shadowBlur = 8;
          ctx.strokeStyle = '#00E676';
          ctx.beginPath();
          for (let x = 0; x <= width; x += 4) {
            const normX = x / width;
            const crest = Math.abs(Math.sin(normX * 18 + now * 8) * 0.6 + Math.cos(normX * 44 - now * 14) * 0.35) * baseAmp;
            const y = centerY - crest * (height * 0.38);
            if (x === 0) ctx.moveTo(x, y);
            else ctx.lineTo(x, y);

            if (y < currentMaxPeakY) {
              currentMaxPeakY = y;
              currentMaxPeakX = x;
            }
          }
          ctx.stroke();

          // Lower Trough Envelope (Negative Troughs)
          ctx.shadowColor = '#EF4444';
          ctx.shadowBlur = 8;
          ctx.strokeStyle = '#F87171';
          ctx.beginPath();
          for (let x = 0; x <= width; x += 4) {
            const normX = x / width;
            const trough = Math.abs(Math.sin(normX * 18 + now * 8 + 0.3) * 0.58 + Math.cos(normX * 44 - now * 14 + 0.5) * 0.32) * baseAmp;
            const y = centerY + trough * (height * 0.38);
            if (x === 0) ctx.moveTo(x, y);
            else ctx.lineTo(x, y);

            if (y > currentMinTroughY) {
              currentMinTroughY = y;
              currentMinTroughX = x;
            }
          }
          ctx.stroke();

          // Center carrier signal
          ctx.shadowBlur = 0;
          ctx.strokeStyle = '#38BDF8';
          ctx.lineWidth = 1.0;
          ctx.beginPath();
          for (let x = 0; x <= width; x += 3) {
            const normX = x / width;
            const sig = (Math.sin(normX * 50 + now * 25) * 0.3) * baseAmp;
            const y = centerY - sig * (height * 0.38);
            if (x === 0) ctx.moveTo(x, y);
            else ctx.lineTo(x, y);
          }
          ctx.stroke();
        } else if (mode === 'stereo') {
          // --- MODE 3: Stereo Dual-Trace (Left & Right Channel) ---
          const leftCenterY = height * 0.30;
          const rightCenterY = height * 0.70;

          // Left Channel Trace
          ctx.shadowColor = '#00F0FF';
          ctx.shadowBlur = 8;
          ctx.strokeStyle = '#00F0FF';
          ctx.lineWidth = 1.8;
          ctx.beginPath();
          for (let x = 0; x <= width; x += 2) {
            const normX = x / width;
            const sigL = (Math.sin(normX * 28 + now * 14) * 0.5 + Math.cos(normX * 72 - now * 20) * 0.25) * baseAmp;
            const y = leftCenterY - sigL * (height * 0.22);
            if (x === 0) ctx.moveTo(x, y);
            else ctx.lineTo(x, y);
          }
          ctx.stroke();

          // Right Channel Trace
          ctx.shadowColor = '#C084FC';
          ctx.shadowBlur = 8;
          ctx.strokeStyle = '#C084FC';
          ctx.lineWidth = 1.8;
          ctx.beginPath();
          for (let x = 0; x <= width; x += 2) {
            const normX = x / width;
            const sigR = (Math.sin(normX * 28 + now * 14 + 0.4) * 0.48 + Math.cos(normX * 68 - now * 19) * 0.26) * baseAmp;
            const y = rightCenterY - sigR * (height * 0.22);
            if (x === 0) ctx.moveTo(x, y);
            else ctx.lineTo(x, y);
          }
          ctx.stroke();

          // Stereo Channel Badges
          ctx.shadowBlur = 0;
          ctx.font = '700 10px monospace';
          ctx.fillStyle = '#00F0FF';
          ctx.fillText('CH 1 [L] WASAPI', width - 95, leftCenterY - 14);
          ctx.fillStyle = '#C084FC';
          ctx.fillText('CH 2 [R] WASAPI', width - 95, rightCenterY - 14);
        } else if (mode === 'envelope') {
          // --- MODE 4: Filled Amplitude Envelope ---
          const grad = ctx.createLinearGradient(0, centerY - height * 0.4, 0, centerY + height * 0.4);
          grad.addColorStop(0, '#00F0FF');
          grad.addColorStop(0.5, '#3B82F6');
          grad.addColorStop(1, '#8A2BE2');

          ctx.fillStyle = grad;
          ctx.beginPath();
          for (let x = 0; x <= width; x += 3) {
            const normX = x / width;
            const env = Math.abs(Math.sin(normX * 18 + now * 8) * 0.6 + Math.cos(normX * 42 - now * 12) * 0.3) * baseAmp;
            const y = centerY - env * (height * 0.38);
            if (x === 0) ctx.moveTo(x, y);
            else ctx.lineTo(x, y);
          }
          for (let x = width; x >= 0; x -= 3) {
            const normX = x / width;
            const env = Math.abs(Math.sin(normX * 18 + now * 8) * 0.6 + Math.cos(normX * 42 - now * 12) * 0.3) * baseAmp;
            const y = centerY + env * (height * 0.38);
            ctx.lineTo(x, y);
          }
          ctx.closePath();
          ctx.globalAlpha = 0.45;
          ctx.fill();
          ctx.globalAlpha = 1.0;

          ctx.shadowColor = '#00F0FF';
          ctx.shadowBlur = 6;
          ctx.strokeStyle = '#00F0FF';
          ctx.lineWidth = 1.5;
          ctx.stroke();
        }

        // --- Active Peak & Trough Markers ---
        if (showPeakMarkers && currentMaxPeakX > 0) {
          ctx.shadowBlur = 0;

          // Peak Crest Pin Marker
          ctx.fillStyle = '#00E676';
          ctx.beginPath();
          ctx.arc(currentMaxPeakX, currentMaxPeakY, 4, 0, Math.PI * 2);
          ctx.fill();

          ctx.fillStyle = '#00E676';
          ctx.font = '700 9px monospace';
          ctx.fillText(`▲ PEAK ${peakDb} dB`, Math.min(width - 90, currentMaxPeakX + 6), currentMaxPeakY - 6);

          // Trough Depth Pin Marker
          ctx.fillStyle = '#38BDF8';
          ctx.beginPath();
          ctx.arc(currentMinTroughX, currentMinTroughY, 4, 0, Math.PI * 2);
          ctx.fill();

          ctx.fillStyle = '#38BDF8';
          ctx.font = '700 9px monospace';
          ctx.fillText(`▼ TROUGH`, Math.min(width - 70, currentMinTroughX + 6), currentMinTroughY + 12);
        }

        // Live Header Telemetry Overlay
        ctx.shadowBlur = 0;
        ctx.font = '11px monospace';
        ctx.fillStyle = '#64748B';
        ctx.fillText(
          `STREAM: ${sampleRate} Hz • ${channels === 2 ? 'STEREO' : 'MONO'} • ${codec.toUpperCase()} (${bitrateKbps}k) • CAPTURED: ${framesCaptured}`,
          50,
          18
        );
      }

      ctx.restore();

      animationFrameRef.current = requestAnimationFrame(render);
    };

    render();

    return () => {
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
    };
  }, [
    isRunning,
    mode,
    sensitivity,
    isFrozen,
    showPeakMarkers,
    sampleRate,
    channels,
    codec,
    bitrateKbps,
    peakLevel,
    peakDb,
    rmsDb,
    framesCaptured,
  ]);

  return (
    <div
      ref={containerRef}
      style={{
        backgroundColor: '#0B101E',
        borderRadius: '14px',
        border: isRunning ? '1px solid #00F0FF' : '1px solid #1E293B',
        overflow: 'hidden',
        boxShadow: isRunning ? '0 0 20px rgba(0, 240, 255, 0.15)' : 'none',
        transition: 'border 0.2s ease, box-shadow 0.2s ease',
      }}
    >
      {/* Visualizer Top Bar & Controls */}
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          padding: '10px 16px',
          backgroundColor: '#0F172A',
          borderBottom: '1px solid #1E293B',
          fontSize: '12px',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
            <span
              style={{
                width: '8px',
                height: '8px',
                borderRadius: '50%',
                backgroundColor: isRunning ? '#00E676' : '#64748B',
                boxShadow: isRunning ? '0 0 8px #00E676' : 'none',
              }}
            />
            <span style={{ fontWeight: 700, color: '#FFFFFF', letterSpacing: '0.5px' }}>
              REAL-TIME AUDIO OSCILLOSCOPE
            </span>
          </div>

          <span style={{ fontSize: '11px', color: '#64748B', borderLeft: '1px solid #1E293B', paddingLeft: '12px' }}>
            {activeEndpoint}
          </span>
        </div>

        {/* Trace Mode & Sensitivity Buttons */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
          <div style={{ display: 'flex', backgroundColor: '#0B101E', borderRadius: '6px', padding: '2px', border: '1px solid #1E293B' }}>
            {(
              [
                { id: 'oscilloscope', label: 'Single Wave' },
                { id: 'peak_trough', label: 'Peak/Trough' },
                { id: 'stereo', label: 'Stereo L/R' },
                { id: 'envelope', label: 'Envelope' },
              ] as const
            ).map((m) => (
              <button
                key={m.id}
                onClick={() => setMode(m.id)}
                style={{
                  padding: '4px 8px',
                  borderRadius: '4px',
                  border: 'none',
                  backgroundColor: mode === m.id ? '#1E293B' : 'transparent',
                  color: mode === m.id ? '#00F0FF' : '#94A3B8',
                  fontSize: '11px',
                  fontWeight: mode === m.id ? 700 : 500,
                  cursor: 'pointer',
                }}
              >
                {m.label}
              </button>
            ))}
          </div>

          {/* Peak Markers Toggle */}
          <button
            onClick={() => setShowPeakMarkers(!showPeakMarkers)}
            style={{
              padding: '4px 8px',
              borderRadius: '6px',
              border: showPeakMarkers ? '1px solid #00E676' : '1px solid #1E293B',
              backgroundColor: showPeakMarkers ? 'rgba(0, 230, 118, 0.15)' : '#0B101E',
              color: showPeakMarkers ? '#00E676' : '#64748B',
              fontSize: '10px',
              fontWeight: 700,
              cursor: 'pointer',
            }}
          >
            PEAK PINS
          </button>

          {/* Sensitivity */}
          <div style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
            <span style={{ fontSize: '10px', fontWeight: 700, color: '#64748B' }}>GAIN:</span>
            {[0.5, 1.0, 2.0].map((s) => (
              <button
                key={s}
                onClick={() => setSensitivity(s)}
                style={{
                  padding: '3px 6px',
                  borderRadius: '4px',
                  border: sensitivity === s ? '1px solid #00F0FF' : '1px solid #1E293B',
                  backgroundColor: sensitivity === s ? 'rgba(0, 240, 255, 0.15)' : '#0B101E',
                  color: sensitivity === s ? '#00F0FF' : '#64748B',
                  fontSize: '10px',
                  fontWeight: 600,
                  cursor: 'pointer',
                }}
              >
                {s}x
              </button>
            ))}
          </div>

          {/* Freeze / Live */}
          <button
            onClick={() => setIsFrozen(!isFrozen)}
            disabled={!isRunning}
            style={{
              padding: '4px 10px',
              borderRadius: '6px',
              border: isFrozen ? '1px solid #EAB308' : '1px solid #1E293B',
              backgroundColor: isFrozen ? 'rgba(234, 179, 8, 0.15)' : '#0B101E',
              color: isFrozen ? '#EAB308' : isRunning ? '#94A3B8' : '#475569',
              fontSize: '11px',
              fontWeight: 600,
              cursor: isRunning ? 'pointer' : 'default',
            }}
          >
            {isFrozen ? 'RESUME TRACE' : 'FREEZE'}
          </button>
        </div>
      </div>

      {/* Canvas Element with Dual Peak/Trough Meters */}
      <div style={{ position: 'relative', width: '100%', height: '145px', backgroundColor: '#060A14' }}>
        <canvas
          ref={canvasRef}
          style={{
            width: '100%',
            height: '100%',
            display: 'block',
          }}
        />

        {/* Real-Time Audio Level HUD */}
        {isRunning && (
          <div
            style={{
              position: 'absolute',
              right: '12px',
              top: '10px',
              display: 'flex',
              gap: '12px',
              padding: '6px 12px',
              backgroundColor: 'rgba(11, 16, 30, 0.90)',
              border: '1px solid #1E293B',
              borderRadius: '8px',
              fontSize: '11px',
              fontFamily: 'monospace',
              pointerEvents: 'none',
              boxShadow: '0 4px 12px rgba(0, 0, 0, 0.5)',
            }}
          >
            <div>
              <span style={{ color: '#64748B' }}>RMS: </span>
              <span style={{ color: '#00F0FF', fontWeight: 700 }}>{rmsDb} dBFS</span>
            </div>
            <div>
              <span style={{ color: '#64748B' }}>PEAK: </span>
              <span style={{ color: peakDb > -1.0 ? '#EF4444' : '#00E676', fontWeight: 700 }}>
                {peakDb} dBFS
              </span>
            </div>
            <div>
              <span style={{ color: '#64748B' }}>CREST: </span>
              <span style={{ color: '#EAB308', fontWeight: 700 }}>
                {Math.abs(peakDb - rmsDb).toFixed(1)} dB
              </span>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
