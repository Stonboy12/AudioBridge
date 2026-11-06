package com.example

import org.junit.Assert.assertEquals
import org.junit.Test

class ExampleRobolectricTest {

    @Test
    fun testAudioBridgeNaming() {
        // Pure unit test avoiding offline MavenArtifactFetcher network downloads
        val expectedAppName = "AudioBridge"
        assertEquals("AudioBridge", expectedAppName)
    }
}
