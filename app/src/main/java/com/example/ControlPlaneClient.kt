package com.example

import android.util.Log
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.withContext
import org.json.JSONObject
import java.io.BufferedReader
import java.io.InputStreamReader
import java.io.OutputStreamWriter
import java.net.InetSocketAddress
import java.net.Socket
import java.util.concurrent.atomic.AtomicBoolean
import kotlin.random.Random

/**
 * ControlPlaneClient handles reliable TCP communication with the Desktop AudioBridge engine.
 * Manages Hello handshake, Pairing verification, Session authentication, and keepalives.
 */
class ControlPlaneClient(
    val hostIp: String,
    val controlPort: Int = 40120,
    private val onDisconnect: (reason: String) -> Unit
) {
    companion object {
        private const val TAG = "AudioBridge_Control"
        private const val CONNECT_TIMEOUT_MS = 3000
        private const val SO_TIMEOUT_MS = 6000
    }

    private var socket: Socket? = null
    private var reader: BufferedReader? = null
    private var writer: OutputStreamWriter? = null
    private val isConnected = AtomicBoolean(false)

    suspend fun connect(): Boolean = withContext(Dispatchers.IO) {
        try {
            val sock = Socket()
            sock.connect(InetSocketAddress(hostIp, controlPort), CONNECT_TIMEOUT_MS)
            sock.soTimeout = SO_TIMEOUT_MS

            val r = BufferedReader(InputStreamReader(sock.getInputStream()))
            val w = OutputStreamWriter(sock.getOutputStream())

            socket = sock
            reader = r
            writer = w
            isConnected.set(true)
            Log.i(TAG, "Connected to Desktop Control Plane at $hostIp:$controlPort")
            true
        } catch (e: Exception) {
            Log.w(TAG, "Failed to connect to Desktop at $hostIp:$controlPort: ${e.message}")
            disconnect("Connection failed")
            false
        }
    }

    suspend fun sendHello(clientId: String, clientName: String): JSONObject? = withContext(Dispatchers.IO) {
        val payload = JSONObject().apply {
            put("type", "Hello")
            put("payload", JSONObject().apply {
                put("client_id", clientId)
                put("client_name", clientName)
                put("client_version", "1.0.0")
                put("protocol_version", 1)
            })
        }
        sendAndReceive(payload)
    }

    suspend fun sendAuthenticate(clientId: String, authToken: String): JSONObject? = withContext(Dispatchers.IO) {
        val payload = JSONObject().apply {
            put("type", "Authenticate")
            put("payload", JSONObject().apply {
                put("client_id", clientId)
                put("auth_token", authToken)
            })
        }
        sendAndReceive(payload)
    }

    suspend fun sendPairRequest(clientId: String, clientName: String): JSONObject? = withContext(Dispatchers.IO) {
        val payload = JSONObject().apply {
            put("type", "PairRequest")
            put("payload", JSONObject().apply {
                put("client_id", clientId)
                put("client_name", clientName)
            })
        }
        sendAndReceive(payload)
    }

    suspend fun sendPairVerify(challengeId: String, code: String): JSONObject? = withContext(Dispatchers.IO) {
        val payload = JSONObject().apply {
            put("type", "PairVerify")
            put("payload", JSONObject().apply {
                put("challenge_id", challengeId)
                put("pairing_code", code)
            })
        }
        sendAndReceive(payload)
    }

    suspend fun sendStartStream(mediaPort: Int): JSONObject? = withContext(Dispatchers.IO) {
        val payload = JSONObject().apply {
            put("type", "StartStream")
            put("payload", JSONObject().apply {
                put("media_port", mediaPort)
            })
        }
        sendAndReceive(payload)
    }

    suspend fun sendStopStream(): JSONObject? = withContext(Dispatchers.IO) {
        val payload = JSONObject().apply {
            put("type", "StopStream")
            put("payload", JSONObject())
        }
        sendAndReceive(payload)
    }

    suspend fun sendPing(sendTimeUs: Long): JSONObject? = withContext(Dispatchers.IO) {
        val payload = JSONObject().apply {
            put("type", "Ping")
            put("payload", JSONObject().apply {
                put("client_send_time_us", sendTimeUs)
            })
        }
        sendAndReceive(payload)
    }

    suspend fun sendRevokeDevice(clientId: String): JSONObject? = withContext(Dispatchers.IO) {
        val payload = JSONObject().apply {
            put("type", "RevokeDevice")
            put("payload", JSONObject().apply {
                put("client_id", clientId)
            })
        }
        sendAndReceive(payload)
    }

    suspend fun sendGetSources(): JSONObject? = withContext(Dispatchers.IO) {
        val payload = JSONObject().apply {
            put("type", "GetSources")
            put("payload", JSONObject())
        }
        sendAndReceive(payload)
    }

    suspend fun sendSelectSource(sourceId: String): JSONObject? = withContext(Dispatchers.IO) {
        val payload = JSONObject().apply {
            put("type", "SelectSource")
            put("payload", JSONObject().apply {
                put("source_id", sourceId)
            })
        }
        sendAndReceive(payload)
    }

    suspend fun sendSetProfile(codec: String, bitrateKbps: Int, bufferMode: String): JSONObject? = withContext(Dispatchers.IO) {
        val payload = JSONObject().apply {
            put("type", "SetProfile")
            put("payload", JSONObject().apply {
                put("profile", JSONObject().apply {
                    put("profile_id", "profile_$bitrateKbps")
                    put("codec", codec)
                    put("sample_rate", 48000)
                    put("channels", 2)
                    put("bitrate_kbps", bitrateKbps)
                    put("frame_duration_ms", 20)
                    put("buffer_mode", bufferMode)
                })
            })
        }
        sendAndReceive(payload)
    }

    suspend fun sendSetDeviceMute(isMuted: Boolean, volume: Float): JSONObject? = withContext(Dispatchers.IO) {
        val payload = JSONObject().apply {
            put("type", "SetDeviceMute")
            put("payload", JSONObject().apply {
                put("is_muted", isMuted)
                put("volume", volume.toDouble())
            })
        }
        sendAndReceive(payload)
    }

    suspend fun sendGetSessions(): JSONObject? = withContext(Dispatchers.IO) {
        val payload = JSONObject().apply {
            put("type", "GetSessions")
            put("payload", JSONObject())
        }
        sendAndReceive(payload)
    }

    private fun sendAndReceive(json: JSONObject): JSONObject? {
        if (!isConnected.get() || writer == null || reader == null) return null
        return try {
            val outLine = json.toString() + "\n"
            writer?.write(outLine)
            writer?.flush()

            val inLine = reader?.readLine()
            if (inLine != null) {
                JSONObject(inLine)
            } else {
                disconnect("Server closed connection")
                null
            }
        } catch (e: Exception) {
            Log.w(TAG, "Error in sendAndReceive: ${e.message}")
            disconnect("Socket error: ${e.message}")
            null
        }
    }

    fun disconnect(reason: String = "User disconnected") {
        if (isConnected.compareAndSet(true, false)) {
            try {
                writer?.close()
                reader?.close()
                socket?.close()
            } catch (_: Exception) {}
            writer = null
            reader = null
            socket = null
            Log.i(TAG, "Control plane disconnected ($reason)")
            onDisconnect(reason)
        }
    }

    val active: Boolean
        get() = isConnected.get()
}

/**
 * Calculates exponential backoff delay with random jitter (PRD Section 8)
 */
fun calculateBackoffDelay(attempt: Int, baseDelayMs: Long = 500, maxDelayMs: Long = 8000): Long {
    val factor = 1L shl attempt.coerceIn(0, 5)
    val exponential = (baseDelayMs * factor).coerceAtMost(maxDelayMs)
    val jitter = Random.nextLong(0, 250) // add 0-250ms random jitter
    return exponential + jitter
}
