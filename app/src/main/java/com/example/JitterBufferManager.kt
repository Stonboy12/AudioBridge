package com.example

import java.util.TreeMap
import java.util.concurrent.atomic.AtomicLong
import kotlin.math.abs

class JitterBufferManager(
    private val profile: AudioProfileConfig
) {
    data class AudioPacket(
        val sequenceNumber: Long,
        val timestampUs: Long,
        val payload: ByteArray,
        val arrivalTimeMs: Long
    )

    private val packetQueue = TreeMap<Long, AudioPacket>()
    private var nextPlaySeq: Long? = null
    private var isBuffering = true
    private var lastTransitTimeMs: Float? = null

    val packetsReceived = AtomicLong(0)
    val packetsPlayed = AtomicLong(0)
    val packetsLostConcealed = AtomicLong(0)
    val packetsDroppedLate = AtomicLong(0)
    val underrunCount = AtomicLong(0)

    @Volatile
    var estimatedJitterMs: Float = 2.4f
        private set

    @Volatile
    var currentBufferDepthMs: Float = 40f
        private set

    @Synchronized
    fun pushPacket(seq: Long, timestampUs: Long, payload: ByteArray) {
        packetsReceived.incrementAndGet()
        val nowMs = System.currentTimeMillis()

        // Jitter calculation
        val sendMs = timestampUs / 1000f
        val transit = nowMs - sendMs
        lastTransitTimeMs?.let { prevTransit ->
            val diff = abs(transit - prevTransit)
            estimatedJitterMs += (diff - estimatedJitterMs) / 16f
        }
        lastTransitTimeMs = transit

        // Discard packet if it has arrived too late
        nextPlaySeq?.let { expected ->
            if (seq < expected) {
                packetsDroppedLate.incrementAndGet()
                return
            }
        }

        packetQueue[seq] = AudioPacket(seq, timestampUs, payload, nowMs)

        if (isBuffering) {
            val queuedDurationMs = packetQueue.size * profile.frameDurationMs
            if (queuedDurationMs >= profile.bufferMode.targetDelayMs) {
                isBuffering = false
                if (nextPlaySeq == null) {
                    nextPlaySeq = packetQueue.firstKey()
                }
            }
        }
    }

    sealed class PopResult {
        data class Packet(val packet: AudioPacket) : PopResult()
        data class ConcealLost(val seq: Long) : PopResult()
        object Underrun : PopResult()
    }

    @Synchronized
    fun popPacket(): PopResult {
        if (isBuffering) {
            return PopResult.Underrun
        }

        val expectedSeq = nextPlaySeq ?: run {
            if (packetQueue.isNotEmpty()) {
                val first = packetQueue.firstKey()
                nextPlaySeq = first
                first
            } else {
                underrunCount.incrementAndGet()
                return PopResult.Underrun
            }
        }

        val packet = packetQueue.remove(expectedSeq)
        updateDepth()

        return if (packet != null) {
            nextPlaySeq = expectedSeq + 1
            packetsPlayed.incrementAndGet()
            PopResult.Packet(packet)
        } else {
            // Check if there are future packets
            if (packetQueue.isNotEmpty() && packetQueue.lastKey() > expectedSeq) {
                // Next packet in sequence was lost, advance sequence and trigger PLC
                nextPlaySeq = expectedSeq + 1
                packetsLostConcealed.incrementAndGet()
                PopResult.ConcealLost(expectedSeq)
            } else {
                underrunCount.incrementAndGet()
                PopResult.Underrun
            }
        }
    }

    private fun updateDepth() {
        currentBufferDepthMs = packetQueue.size.toFloat() * profile.frameDurationMs
    }

    @Synchronized
    fun reset() {
        packetQueue.clear()
        nextPlaySeq = null
        isBuffering = true
        lastTransitTimeMs = null
    }
}
