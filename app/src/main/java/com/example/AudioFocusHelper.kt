package com.example

import android.content.Context
import android.media.AudioAttributes
import android.media.AudioFocusRequest
import android.media.AudioManager
import android.os.Build
import android.util.Log

/**
 * AudioFocusHelper handles audio focus lifecycle according to Android media guidelines:
 * - AUDIOFOCUS_GAIN: Full volume playback resumed
 * - AUDIOFOCUS_LOSS: Another app took exclusive audio (e.g. YouTube/Spotify), abandon & pause
 * - AUDIOFOCUS_LOSS_TRANSIENT: Short interruption (e.g. phone call ringing), pause temporarily
 * - AUDIOFOCUS_LOSS_TRANSIENT_CAN_DUCK: Notification/navigation prompt, lower volume to 20%
 */
class AudioFocusHelper(
    private val context: Context,
    private val onFocusChange: (FocusState) -> Unit
) {
    companion object {
        private const val TAG = "AudioBridge_AudioFocus"
    }

    enum class FocusState {
        GRANTED,
        DUCK,
        PAUSE_TRANSIENT,
        LOST
    }

    private val audioManager = context.getSystemService(Context.AUDIO_SERVICE) as AudioManager
    private var focusRequest: AudioFocusRequest? = null
    private var hasFocus = false

    private val focusChangeListener = AudioManager.OnAudioFocusChangeListener { focusChange ->
        when (focusChange) {
            AudioManager.AUDIOFOCUS_GAIN -> {
                Log.d(TAG, "Audio focus GAIN: Restoring full volume")
                hasFocus = true
                onFocusChange(FocusState.GRANTED)
            }
            AudioManager.AUDIOFOCUS_LOSS_TRANSIENT_CAN_DUCK -> {
                Log.d(TAG, "Audio focus LOSS_TRANSIENT_CAN_DUCK: Lowering volume to duck level (20%)")
                onFocusChange(FocusState.DUCK)
            }
            AudioManager.AUDIOFOCUS_LOSS_TRANSIENT -> {
                Log.d(TAG, "Audio focus LOSS_TRANSIENT: Pausing audio temporarily")
                onFocusChange(FocusState.PAUSE_TRANSIENT)
            }
            AudioManager.AUDIOFOCUS_LOSS -> {
                Log.d(TAG, "Audio focus LOSS: Abandoning and stopping stream")
                hasFocus = false
                onFocusChange(FocusState.LOST)
            }
        }
    }

    fun requestFocus(): Boolean {
        if (hasFocus) return true

        val result = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val playbackAttributes = AudioAttributes.Builder()
                .setUsage(AudioAttributes.USAGE_MEDIA)
                .setContentType(AudioAttributes.CONTENT_TYPE_MUSIC)
                .build()

            val request = AudioFocusRequest.Builder(AudioManager.AUDIOFOCUS_GAIN)
                .setAudioAttributes(playbackAttributes)
                .setAcceptsDelayedFocusGain(true)
                .setOnAudioFocusChangeListener(focusChangeListener)
                .build()

            focusRequest = request
            audioManager.requestAudioFocus(request)
        } else {
            @Suppress("DEPRECATION")
            audioManager.requestAudioFocus(
                focusChangeListener,
                AudioManager.STREAM_MUSIC,
                AudioManager.AUDIOFOCUS_GAIN
            )
        }

        hasFocus = (result == AudioManager.AUDIOFOCUS_REQUEST_GRANTED)
        Log.i(TAG, "Audio focus request result: $result (granted=$hasFocus)")
        return hasFocus
    }

    fun abandonFocus() {
        if (!hasFocus) return

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            focusRequest?.let { audioManager.abandonAudioFocusRequest(it) }
            focusRequest = null
        } else {
            @Suppress("DEPRECATION")
            audioManager.abandonAudioFocus(focusChangeListener)
        }

        hasFocus = false
        Log.i(TAG, "Audio focus abandoned")
    }
}
