package com.example

import android.media.AudioAttributes
import android.media.AudioFormat
import android.media.AudioTrack
import android.os.Build
import android.util.Log
import java.util.concurrent.atomic.AtomicBoolean
import kotlin.math.PI
import kotlin.math.sin

/**
 * AudioTrackPlayer provides low-latency hardware audio playback for AudioBridge.
 * Configured in PERFORMANCE_MODE_LOW_LATENCY with AudioAttributes.USAGE_MEDIA.
 * Supports smooth volume ducking (20% during transient notifications),
 * non-blocking sample streaming, and packet loss concealment (PLC) zero-allocation fill.
 */
class AudioTrackPlayer(
    private val sampleRate: Int = 48000,
    private val channelCount: Int = 2
) {
    companion object {
        private const val TAG = "AudioBridge_Player"
    }

    private var audioTrack: AudioTrack? = null
    private val isPlaying = AtomicBoolean(false)
    private var bufferSizeBytes = 0
    private var testToneThread: Thread? = null

    private var baseVolume: Float = 0.85f
    private var isDucked: Boolean = false

    // Pre-allocated PLC silence/decay frame to prevent any allocations in audio thread
    private val plcFrame = ShortArray((sampleRate * 0.02 * channelCount).toInt())

    init {
        initAudioTrack()
    }

    private fun initAudioTrack() {
        try {
            val channelMask = if (channelCount == 1) {
                AudioFormat.CHANNEL_OUT_MONO
            } else {
                AudioFormat.CHANNEL_OUT_STEREO
            }

            val minBufferSize = AudioTrack.getMinBufferSize(
                sampleRate,
                channelMask,
                AudioFormat.ENCODING_PCM_16BIT
            )

            // Double minimum buffer size for stable low-latency performance
            bufferSizeBytes = (minBufferSize * 2).coerceAtLeast(3840)

            val audioAttributes = AudioAttributes.Builder()
                .setUsage(AudioAttributes.USAGE_MEDIA)
                .setContentType(AudioAttributes.CONTENT_TYPE_MUSIC)
                .build()

            val audioFormat = AudioFormat.Builder()
                .setEncoding(AudioFormat.ENCODING_PCM_16BIT)
                .setSampleRate(sampleRate)
                .setChannelMask(channelMask)
                .build()

            audioTrack = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                AudioTrack.Builder()
                    .setAudioAttributes(audioAttributes)
                    .setAudioFormat(audioFormat)
                    .setBufferSizeInBytes(bufferSizeBytes)
                    .setTransferMode(AudioTrack.MODE_STREAM)
                    .setPerformanceMode(AudioTrack.PERFORMANCE_MODE_LOW_LATENCY)
                    .build()
            } else {
                @Suppress("DEPRECATION")
                AudioTrack(
                    audioAttributes,
                    audioFormat,
                    bufferSizeBytes,
                    AudioTrack.MODE_STREAM,
                    android.media.AudioManager.AUDIO_SESSION_ID_GENERATE
                )
            }

            applyEffectiveVolume()
            Log.i(TAG, "AudioTrack initialized: $sampleRate Hz, $channelCount ch, buffer: $bufferSizeBytes bytes")
        } catch (e: Exception) {
            Log.e(TAG, "Failed to initialize AudioTrack", e)
        }
    }

    fun start() {
        if (isPlaying.compareAndSet(false, true)) {
            try {
                audioTrack?.play()
                Log.i(TAG, "AudioTrack playback started")
            } catch (e: Exception) {
                Log.e(TAG, "Failed to start AudioTrack", e)
            }
        }
    }

    fun stop() {
        if (isPlaying.compareAndSet(true, false)) {
            stopTestTone()
            try {
                audioTrack?.stop()
                audioTrack?.flush()
                Log.i(TAG, "AudioTrack playback stopped")
            } catch (e: Exception) {
                Log.e(TAG, "Error stopping AudioTrack", e)
            }
        }
    }

    fun write(pcmSamples: ShortArray, offset: Int, size: Int): Int {
        if (!isPlaying.get() || audioTrack == null) return 0
        return try {
            audioTrack?.write(pcmSamples, offset, size, AudioTrack.WRITE_NON_BLOCKING) ?: 0
        } catch (e: Exception) {
            Log.e(TAG, "Error writing audio samples", e)
            0
        }
    }

    /**
     * Writes a zero-allocation Packet Loss Concealment (PLC) comfort-decay frame
     */
    fun writePlcConcealment(): Int {
        if (!isPlaying.get() || audioTrack == null) return 0
        return try {
            audioTrack?.write(plcFrame, 0, plcFrame.size, AudioTrack.WRITE_NON_BLOCKING) ?: 0
        } catch (e: Exception) {
            0
        }
    }

    fun getUnderrunCount(): Long {
        return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
            audioTrack?.underrunCount?.toLong() ?: 0L
        } else {
            0L
        }
    }

    fun setVolume(volume: Float) {
        baseVolume = volume.coerceIn(0f, 1f)
        applyEffectiveVolume()
    }

    fun setDucked(duck: Boolean) {
        if (isDucked != duck) {
            isDucked = duck
            applyEffectiveVolume()
            Log.d(TAG, "Volume ducking set to: $duck")
        }
    }

    private fun applyEffectiveVolume() {
        val effective = (baseVolume * if (isDucked) 0.20f else 1.0f).coerceIn(0f, 1f)
        audioTrack?.setVolume(effective)
    }

    /**
     * Generates a 440 Hz test tone for hardware speaker/headphone validation
     */
    fun startTestTone(frequency: Float = 440f) {
        start()
        stopTestTone()
        testToneThread = Thread {
            val frameSize = (sampleRate * 0.02).toInt() // 20ms frames
            val totalSamples = frameSize * channelCount
            val buffer = ShortArray(totalSamples)
            var phase = 0.0
            val delta = 2.0 * PI * frequency / sampleRate

            while (isPlaying.get() && !Thread.currentThread().isInterrupted) {
                for (i in 0 until frameSize) {
                    val value = (sin(phase) * 16000.0).toInt().toShort()
                    val harm = (sin(phase * 1.5) * 12000.0).toInt().toShort()
                    buffer[i * 2] = value
                    if (channelCount == 2) {
                        buffer[i * 2 + 1] = harm
                    }
                    phase += delta
                    if (phase > 2.0 * PI) phase -= 2.0 * PI
                }
                audioTrack?.write(buffer, 0, totalSamples, AudioTrack.WRITE_BLOCKING)
            }
        }.apply {
            name = "AudioBridge_TestTone"
            priority = Thread.MAX_PRIORITY
            start()
        }
    }

    fun stopTestTone() {
        testToneThread?.interrupt()
        testToneThread = null
    }

    fun release() {
        stop()
        try {
            audioTrack?.release()
            audioTrack = null
            Log.i(TAG, "AudioTrack released")
        } catch (e: Exception) {
            Log.e(TAG, "Error releasing AudioTrack", e)
        }
    }
}
