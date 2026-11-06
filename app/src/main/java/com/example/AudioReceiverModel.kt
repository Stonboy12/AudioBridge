package com.example

enum class ConnectionStatus {
    DISCONNECTED,
    DISCOVERING,
    PAIRING,
    CONNECTING,
    CONNECTED,
    STREAMING,
    RECONNECTING,
    ERROR
}

enum class AudioCodec(val displayName: String) {
    OPUS("Opus (Interactive LowDelay)"),
    PCM("Linear PCM (Uncompressed)")
}

enum class BufferMode(val displayName: String, val targetDelayMs: Int) {
    LOW_LATENCY("Low Latency (~30 ms)", 30),
    ADAPTIVE("Adaptive Jitter (~60 ms)", 60),
    CONSERVATIVE("Conservative (~120 ms)", 120)
}

enum class AudioOutputRoute(val displayName: String, val iconName: String) {
    SPEAKER("Built-in Speaker", "speaker"),
    WIRED_HEADSET("Wired Headphones", "headset"),
    BLUETOOTH("Bluetooth A2DP", "bluetooth")
}

data class AudioProfileConfig(
    val codec: AudioCodec = AudioCodec.OPUS,
    val sampleRate: Int = 48000,
    val channels: Int = 2,
    val bitrateKbps: Int = 128,
    val frameDurationMs: Int = 20,
    val bufferMode: BufferMode = BufferMode.ADAPTIVE,
    val isMuted: Boolean = false
) {
    val samplesPerFrame: Int
        get() = (sampleRate * frameDurationMs) / 1000

    val totalSamplesPerFrame: Int
        get() = samplesPerFrame * channels
}

data class AudioSourceItem(
    val id: String,
    val name: String,
    val isSystem: Boolean,
    val isAvailable: Boolean = true,
    val processName: String? = null,
    val processId: Int? = null,
    val listenerCount: Int = 0
)

data class MultiDeviceInfo(
    val totalConnectedDevices: Int = 1,
    val sharedCaptureActive: Boolean = true,
    val currentDeviceIndex: Int = 1,
    val activeFanoutStreams: Int = 1
)

data class DesktopHost(
    val id: String,
    val name: String,
    val ip: String,
    val port: Int = 40120,
    val isPaired: Boolean = false,
    val authToken: String? = null,
    val isOnline: Boolean = true
)

data class PairingState(
    val isPairingActive: Boolean = false,
    val challengeId: String = "",
    val host: DesktopHost? = null,
    val code: String = "",
    val expiresInSecs: Int = 180,
    val attemptsRemaining: Int = 3,
    val errorMessage: String? = null
)

data class ReconnectionState(
    val isReconnecting: Boolean = false,
    val attempt: Int = 0,
    val nextRetryMs: Long = 0
)

data class DiagnosticsTelemetry(
    val estimatedLatencyMs: Float = 58.2f,
    val rttMs: Float = 1.4f,
    val jitterMs: Float = 2.4f,
    val packetLossPercent: Float = 0.0f,
    val bitrateKbps: Float = 128.5f,
    val bufferDepthMs: Float = 42.0f,
    val underrunCount: Long = 0,
    val packetsReceived: Long = 0,
    val packetsLost: Long = 0,
    val packetsConcealed: Long = 0,
    val packetsDroppedLate: Long = 0,
    val malformedPackets: Long = 0,
    val sessionDurationSecs: Long = 0
)

data class Stage0Experiment(
    val id: String,
    val title: String,
    val description: String,
    var status: ExperimentStatus = ExperimentStatus.PENDING,
    var detailMessage: String = "",
    var metric: String = ""
)

enum class ExperimentStatus {
    PENDING,
    RUNNING,
    PASSED,
    FAILED,
    SKIPPED
}
