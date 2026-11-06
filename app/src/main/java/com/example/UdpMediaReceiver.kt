package com.example

import android.util.Log
import java.io.IOException
import java.net.DatagramPacket
import java.net.DatagramSocket
import java.net.SocketException
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.atomic.AtomicLong

/**
 * UdpMediaReceiver binds a local UDP socket on the media port (default 40121)
 * and receives real-time AudioBridge binary datagrams (30-byte header + Opus/PCM payload).
 *
 * Header Format (30 bytes, Big Endian):
 *  0..3:  Magic "ABRG" (0x41, 0x42, 0x52, 0x47)
 *  4:     Protocol Version (0x01)
 *  5:     Packet Type (0x01 = AUDIO_DATA)
 *  6:     Flags (0x01 = HAS_FEC)
 *  7:     Reserved (0x00)
 *  8..11: Stream ID (u32)
 * 12..15: Session ID (u32)
 * 16..23: Sequence Number (u64)
 * 24..31: Timestamp in microseconds (u64)
 * 28..29: Payload length (u16)
 */
class UdpMediaReceiver(
    private val port: Int = 40121,
    private val onPacketReceived: (seq: Long, timestampUs: Long, payload: ByteArray) -> Unit
) {
    companion object {
        private const val TAG = "AudioBridge_UDP"
        const val HEADER_SIZE = 30
        const val MAGIC = 0x41425247 // "ABRG" in hex
        const val MAX_DATAGRAM_SIZE = 2048
    }

    private var socket: DatagramSocket? = null
    private var receiveThread: Thread? = null
    private val isRunning = AtomicBoolean(false)

    val totalPacketsReceived = AtomicLong(0)
    val totalBytesReceived = AtomicLong(0)
    val malformedPacketsDiscarded = AtomicLong(0)

    fun start() {
        if (isRunning.compareAndSet(false, true)) {
            try {
                socket = DatagramSocket(port).apply {
                    receiveBufferSize = 256 * 1024 // 256 KB socket buffer for high burst tolerance
                    soTimeout = 2000 // 2s timeout for graceful interruption checking
                }
                Log.i(TAG, "UdpMediaReceiver listening on port $port")

                receiveThread = Thread {
                    val buffer = ByteArray(MAX_DATAGRAM_SIZE)
                    val packet = DatagramPacket(buffer, buffer.size)

                    while (isRunning.get() && !Thread.currentThread().isInterrupted) {
                        try {
                            socket?.receive(packet)
                            val length = packet.length
                            if (length < HEADER_SIZE) {
                                malformedPacketsDiscarded.incrementAndGet()
                                continue
                            }

                            val byteBuffer = ByteBuffer.wrap(buffer, 0, length).order(ByteOrder.BIG_ENDIAN)

                            // Validate Magic "ABRG"
                            val magic = byteBuffer.int
                            if (magic != MAGIC) {
                                malformedPacketsDiscarded.incrementAndGet()
                                continue
                            }

                            val version = byteBuffer.get().toInt() and 0xFF
                            val flags = byteBuffer.get().toInt() and 0xFF
                            val codec = byteBuffer.get().toInt() and 0xFF
                            val reserved = byteBuffer.get().toInt() and 0xFF
                            val streamId = byteBuffer.int
                            val seq = byteBuffer.long
                            val timestampUs = byteBuffer.long
                            val payloadLen = byteBuffer.short.toInt() and 0xFFFF

                            if (length < HEADER_SIZE + payloadLen) {
                                malformedPacketsDiscarded.incrementAndGet()
                                continue
                            }

                            val payload = ByteArray(payloadLen)
                            byteBuffer.get(payload, 0, payloadLen)

                            totalPacketsReceived.incrementAndGet()
                            totalBytesReceived.addAndGet(length.toLong())

                            onPacketReceived(seq, timestampUs, payload)

                        } catch (e: SocketException) {
                            if (!isRunning.get()) break
                            Log.w(TAG, "SocketException in receive loop", e)
                        } catch (e: IOException) {
                            // Timeout or standard IO exception, continue loop
                        } catch (e: Exception) {
                            Log.e(TAG, "Unexpected error in UDP receive loop", e)
                        }
                    }
                    Log.i(TAG, "UDP receive thread exiting cleanly")
                }.apply {
                    name = "AudioBridge_UdpReceiver"
                    priority = Thread.MAX_PRIORITY - 1
                    start()
                }

            } catch (e: Exception) {
                Log.e(TAG, "Failed to bind UDP socket on port $port", e)
                isRunning.set(false)
            }
        }
    }

    fun stop() {
        if (isRunning.compareAndSet(true, false)) {
            try {
                socket?.close()
                socket = null
            } catch (e: Exception) {
                Log.w(TAG, "Error closing UDP socket", e)
            }
            receiveThread?.interrupt()
            receiveThread = null
            Log.i(TAG, "UdpMediaReceiver stopped")
        }
    }
}
