package com.example

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.media.AudioManager
import android.util.Log

/**
 * AudioNoisyReceiver intercepts ACTION_AUDIO_BECOMING_NOISY.
 * Triggered when a wired headset is unplugged or a Bluetooth audio device is disconnected.
 * Stops/pauses streaming immediately so audio does not blast out through speakers.
 */
class AudioNoisyReceiver(
    private val onBecomingNoisy: () -> Unit
) : BroadcastReceiver() {

    companion object {
        private const val TAG = "AudioBridge_Noisy"
    }

    private var isRegistered = false

    override fun onReceive(context: Context?, intent: Intent?) {
        if (intent?.action == AudioManager.ACTION_AUDIO_BECOMING_NOISY) {
            Log.i(TAG, "Audio becoming noisy (headphones/BT disconnected) -> Pausing stream")
            onBecomingNoisy()
        }
    }

    fun register(context: Context) {
        if (!isRegistered) {
            val filter = IntentFilter(AudioManager.ACTION_AUDIO_BECOMING_NOISY)
            context.registerReceiver(this, filter)
            isRegistered = true
            Log.d(TAG, "AudioNoisyReceiver registered")
        }
    }

    fun unregister(context: Context) {
        if (isRegistered) {
            try {
                context.unregisterReceiver(this)
            } catch (e: Exception) {
                Log.w(TAG, "Error unregistering AudioNoisyReceiver", e)
            }
            isRegistered = false
            Log.d(TAG, "AudioNoisyReceiver unregistered")
        }
    }
}
