package com.example

import android.util.Log

/**
 * NativeAudioBridge provides the JNI binding interface to the native C++ Oboe / AAudio
 * audio sink and Rust audio receiver core.
 *
 * Implements graceful fallback: If the native shared library (.so) is not present
 * or fails to link on a target architecture, it flags isNativeAvailable = false
 * and delegates execution seamlessly to the high-performance AudioTrack pipeline.
 */
object NativeAudioBridge {
    private const val TAG = "AudioBridge_Native"
    private var isNativeLoaded = false

    init {
        try {
            System.loadLibrary("audiobridge_receiver")
            isNativeLoaded = true
            Log.i(TAG, "Native audiobridge_receiver library loaded successfully")
        } catch (e: UnsatisfiedLinkError) {
            Log.w(TAG, "Native library 'audiobridge_receiver' not found. Using high-compatibility AudioTrack engine.")
            isNativeLoaded = false
        } catch (e: Exception) {
            Log.w(TAG, "Failed to load native library", e)
            isNativeLoaded = false
        }
    }

    val isNativeAvailable: Boolean
        get() = isNativeLoaded

    // Native JNI method declarations
    external fun nativeInitEngine(sampleRate: Int, channels: Int, targetDelayMs: Int): Long
    external fun nativeProcessPacket(handle: Long, packetData: ByteArray, offset: Int, length: Int): Int
    external fun nativeGetBufferDepthMs(handle: Long): Float
    external fun nativeGetUnderrunCount(handle: Long): Long
    external fun nativeDestroyEngine(handle: Long)
}
