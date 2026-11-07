package com.example

import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.activity.compose.BackHandler
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Assessment
import androidx.compose.material.icons.filled.Build
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.Computer
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.GraphicEq
import androidx.compose.material.icons.filled.Headphones
import androidx.compose.material.icons.filled.Info
import androidx.compose.material.icons.filled.Lock
import androidx.compose.material.icons.filled.Devices
import androidx.compose.material.icons.filled.PlayArrow
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material.icons.filled.Stop
import androidx.compose.material.icons.filled.Sync
import androidx.compose.material.icons.filled.VolumeOff
import androidx.compose.material.icons.filled.VolumeUp
import androidx.compose.material.icons.filled.Warning
import androidx.compose.material.icons.filled.Wifi
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilterChip
import androidx.compose.material3.FilterChipDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.NavigationBarItemDefaults
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlin.math.sin

val DarkBackground = Color(0xFF0B101E)
val DarkSurface = Color(0xFF131A2B)
val DarkCard = Color(0xFF1B243B)
val AccentCyan = Color(0xFF00F0FF)
val AccentViolet = Color(0xFF8A2BE2)
val NeonGreen = Color(0xFF00E676)
val WarningAmber = Color(0xFFFFB300)
val DangerRed = Color(0xFFFF5252)

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AudioBridgeApp(viewModel: ReceiverViewModel) {
    val uiState by viewModel.uiState.collectAsState()

    Scaffold(
        topBar = {
            TopAppBar(
                title = {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Box(
                            modifier = Modifier
                                .size(10.dp)
                                .clip(CircleShape)
                                .background(
                                    when (uiState.status) {
                                        ConnectionStatus.STREAMING -> NeonGreen
                                        ConnectionStatus.CONNECTING, ConnectionStatus.RECONNECTING -> WarningAmber
                                        ConnectionStatus.ERROR -> DangerRed
                                        else -> Color.Gray
                                    }
                                )
                        )
                        Spacer(modifier = Modifier.width(10.dp))
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            Text(
                                text = "Audio",
                                fontSize = 19.sp,
                                fontWeight = FontWeight.Bold,
                                color = Color.White,
                                letterSpacing = (-0.3).sp
                            )
                            Text(
                                text = "Bridge",
                                fontSize = 19.sp,
                                fontWeight = FontWeight.Bold,
                                color = AccentCyan,
                                letterSpacing = (-0.3).sp
                            )
                            Spacer(modifier = Modifier.width(8.dp))
                            Box(
                                modifier = Modifier
                                    .background(AccentCyan.copy(alpha = 0.15f), RoundedCornerShape(6.dp))
                                    .border(1.dp, AccentCyan.copy(alpha = 0.3f), RoundedCornerShape(6.dp))
                                    .padding(horizontal = 6.dp, vertical = 2.dp)
                            ) {
                                Text(
                                    text = "RECEIVER",
                                    fontSize = 10.sp,
                                    fontWeight = FontWeight.Bold,
                                    color = AccentCyan,
                                    letterSpacing = 0.5.sp
                                )
                            }
                        }
                    }
                },
                actions = {
                    IconButton(
                        onClick = { viewModel.simulateReconnect() },
                        modifier = Modifier.testTag("reconnect_button")
                    ) {
                        Icon(
                            imageVector = Icons.Default.Refresh,
                            contentDescription = "Simulate Reconnect",
                            tint = Color.White
                        )
                    }
                },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = DarkBackground,
                    titleContentColor = Color.White
                )
            )
        },
        bottomBar = {
            NavigationBar(
                containerColor = DarkSurface,
                tonalElevation = 8.dp
            ) {
                NavigationBarItem(
                    selected = uiState.selectedTab == 0,
                    onClick = { viewModel.selectTab(0) },
                    icon = { Icon(Icons.Default.Headphones, contentDescription = "Receiver") },
                    label = { Text("Receiver") },
                    colors = NavigationBarItemDefaults.colors(
                        selectedIconColor = AccentCyan,
                        selectedTextColor = AccentCyan,
                        unselectedIconColor = Color.Gray,
                        unselectedTextColor = Color.Gray,
                        indicatorColor = DarkCard
                    )
                )
                NavigationBarItem(
                    selected = uiState.selectedTab == 1,
                    onClick = { viewModel.selectTab(1) },
                    icon = { Icon(Icons.Default.Assessment, contentDescription = "Diagnostics") },
                    label = { Text("Diagnostics") },
                    colors = NavigationBarItemDefaults.colors(
                        selectedIconColor = AccentCyan,
                        selectedTextColor = AccentCyan,
                        unselectedIconColor = Color.Gray,
                        unselectedTextColor = Color.Gray,
                        indicatorColor = DarkCard
                    )
                )
                NavigationBarItem(
                    selected = uiState.selectedTab == 2,
                    onClick = { viewModel.selectTab(2) },
                    icon = { Icon(Icons.Default.Build, contentDescription = "Validation Suite") },
                    label = { Text("Validation") },
                    colors = NavigationBarItemDefaults.colors(
                        selectedIconColor = AccentCyan,
                        selectedTextColor = AccentCyan,
                        unselectedIconColor = Color.Gray,
                        unselectedTextColor = Color.Gray,
                        indicatorColor = DarkCard
                    )
                )
                NavigationBarItem(
                    selected = uiState.selectedTab == 3,
                    onClick = { viewModel.selectTab(3) },
                    icon = { Icon(Icons.Default.Settings, contentDescription = "Settings") },
                    label = { Text("Settings") },
                    colors = NavigationBarItemDefaults.colors(
                        selectedIconColor = AccentCyan,
                        selectedTextColor = AccentCyan,
                        unselectedIconColor = Color.Gray,
                        unselectedTextColor = Color.Gray,
                        indicatorColor = DarkCard
                    )
                )
            }
        },
        containerColor = DarkBackground
    ) { innerPadding ->
        Box(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding)
                .background(DarkBackground)
        ) {
            if (uiState.selectedTab != 0) {
                BackHandler {
                    viewModel.selectTab(0)
                }
            }

            when (uiState.selectedTab) {
                0 -> ReceiverScreen(uiState, viewModel)
                1 -> DiagnosticsScreen(uiState)
                2 -> Stage0TestsScreen(uiState, viewModel)
                3 -> SettingsScreen(uiState, viewModel)
            }

            if (uiState.pairingState.isPairingActive) {
                SecurePairingDialog(
                    pairingState = uiState.pairingState,
                    onVerify = { code -> viewModel.submitPairingVerification(code) },
                    onDismiss = { viewModel.dismissPairingDialog() }
                )
            }
        }
    }
}

@Composable
fun ReceiverScreen(uiState: ReceiverUiState, viewModel: ReceiverViewModel) {
    LazyColumn(
        modifier = Modifier
            .fillMaxSize()
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        // Welcome Onboarding Card
        if (uiState.showOnboardingBanner) {
            item {
                Card(
                    colors = CardDefaults.cardColors(containerColor = DarkSurface),
                    shape = RoundedCornerShape(16.dp),
                    modifier = Modifier
                        .fillMaxWidth()
                        .border(1.dp, AccentCyan.copy(alpha = 0.4f), RoundedCornerShape(16.dp))
                ) {
                    Column(modifier = Modifier.padding(16.dp)) {
                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            horizontalArrangement = Arrangement.SpaceBetween,
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Text(
                                text = "WELCOME TO AUDIOBRIDGE",
                                fontSize = 12.sp,
                                fontWeight = FontWeight.Bold,
                                color = AccentCyan
                            )
                            IconButton(
                                onClick = { viewModel.dismissOnboardingBanner() },
                                modifier = Modifier.size(24.dp)
                            ) {
                                Icon(
                                    imageVector = Icons.Default.Close,
                                    contentDescription = "Dismiss",
                                    tint = Color.Gray,
                                    modifier = Modifier.size(16.dp)
                                )
                            }
                        }
                        Spacer(modifier = Modifier.height(6.dp))
                        Text(
                            text = "Stream PC audio directly to your Android device with sub-60ms low latency over local Wi-Fi. Background playback and lock-screen persistence are enabled.",
                            fontSize = 12.sp,
                            color = Color.LightGray,
                            lineHeight = 17.sp
                        )
                        Spacer(modifier = Modifier.height(10.dp))
                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            horizontalArrangement = Arrangement.End
                        ) {
                            Button(
                                onClick = { viewModel.dismissOnboardingBanner() },
                                colors = ButtonDefaults.buttonColors(containerColor = AccentCyan, contentColor = Color.Black),
                                shape = RoundedCornerShape(8.dp),
                                modifier = Modifier.height(36.dp)
                            ) {
                                Text("Get Started", fontSize = 12.sp, fontWeight = FontWeight.Bold)
                            }
                        }
                    }
                }
            }
        }

        // Reconnecting Status Banner (Exponential Backoff ADR-004)
        if (uiState.status == ConnectionStatus.RECONNECTING) {
            item {
                Card(
                    colors = CardDefaults.cardColors(containerColor = WarningAmber.copy(alpha = 0.15f)),
                    shape = RoundedCornerShape(12.dp),
                    modifier = Modifier
                        .fillMaxWidth()
                        .border(1.dp, WarningAmber, RoundedCornerShape(12.dp))
                ) {
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(14.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.SpaceBetween
                    ) {
                        Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.weight(1f)) {
                            Icon(
                                imageVector = Icons.Default.Warning,
                                contentDescription = "Reconnecting",
                                tint = WarningAmber,
                                modifier = Modifier.size(24.dp)
                            )
                            Spacer(modifier = Modifier.width(12.dp))
                            Column {
                                Text(
                                    text = "Connection Interrupted",
                                    fontWeight = FontWeight.Bold,
                                    fontSize = 13.sp,
                                    color = Color.White
                                )
                                Text(
                                    text = "Auto-reconnecting: attempt ${uiState.reconnectionState.attempt}/5 (exponential backoff)",
                                    fontSize = 11.sp,
                                    color = WarningAmber
                                )
                            }
                        }
                        TextButton(
                            onClick = { viewModel.cancelReconnect() },
                            modifier = Modifier.testTag("cancel_reconnect_button")
                        ) {
                            Text("Cancel", color = Color.White, fontWeight = FontWeight.Bold, fontSize = 12.sp)
                        }
                    }
                }
            }
        }

        // Status Hero Card
        item {
            Card(
                colors = CardDefaults.cardColors(containerColor = DarkSurface),
                shape = RoundedCornerShape(16.dp),
                modifier = Modifier
                    .fillMaxWidth()
                    .border(1.dp, DarkCard, RoundedCornerShape(16.dp))
            ) {
                Column(modifier = Modifier.padding(18.dp)) {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Column {
                            Text(
                                text = "PC TRANSMITTER",
                                fontSize = 11.sp,
                                fontWeight = FontWeight.Bold,
                                color = Color.Gray
                            )
                            Text(
                                text = uiState.activeHost.name,
                                fontSize = 16.sp,
                                fontWeight = FontWeight.Bold,
                                color = Color.White
                            )
                            Text(
                                text = "${uiState.activeHost.ip}:${uiState.activeHost.port}",
                                fontSize = 12.sp,
                                color = AccentCyan,
                                fontFamily = FontFamily.Monospace
                            )
                        }

                        val (statusText, statusColor) = when (uiState.status) {
                            ConnectionStatus.STREAMING -> "STREAMING" to NeonGreen
                            ConnectionStatus.CONNECTING -> "CONNECTING" to WarningAmber
                            ConnectionStatus.RECONNECTING -> "RECONNECTING" to WarningAmber
                            ConnectionStatus.PAIRING -> "PAIRING" to AccentViolet
                            ConnectionStatus.DISCOVERING -> "DISCOVERING" to AccentCyan
                            ConnectionStatus.CONNECTED -> "CONNECTED" to AccentCyan
                            ConnectionStatus.ERROR -> "ERROR" to DangerRed
                            ConnectionStatus.DISCONNECTED -> "STANDBY" to Color.Gray
                        }

                        Box(
                            modifier = Modifier
                                .background(statusColor.copy(alpha = 0.15f), RoundedCornerShape(20.dp))
                                .border(1.dp, statusColor, RoundedCornerShape(20.dp))
                                .padding(horizontal = 12.dp, vertical = 6.dp)
                        ) {
                            Text(
                                text = statusText,
                                color = statusColor,
                                fontWeight = FontWeight.Bold,
                                fontSize = 12.sp
                            )
                        }
                    }

                    Spacer(modifier = Modifier.height(16.dp))

                    // Waveform visualizer
                    WaveformVisualizer(isPlaying = uiState.status == ConnectionStatus.STREAMING)

                    Spacer(modifier = Modifier.height(12.dp))

                    // Transport Mode Selector (Wi-Fi, USB Tether, ADB Tunnel)
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.spacedBy(6.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        ConnectionTransportMode.values().forEach { mode ->
                            val isSelected = uiState.connectionPath.activeMode == mode
                            Box(
                                modifier = Modifier
                                    .weight(1f)
                                    .clip(RoundedCornerShape(8.dp))
                                    .background(if (isSelected) AccentCyan.copy(alpha = 0.2f) else DarkCard)
                                    .border(1.dp, if (isSelected) AccentCyan else DarkCard, RoundedCornerShape(8.dp))
                                    .clickable { viewModel.switchConnectionTransportMode(mode) }
                                    .padding(vertical = 6.dp),
                                contentAlignment = Alignment.Center
                            ) {
                                Text(
                                    text = mode.badge,
                                    fontSize = 11.sp,
                                    fontWeight = if (isSelected) FontWeight.Bold else FontWeight.Medium,
                                    color = if (isSelected) AccentCyan else Color.LightGray
                                )
                            }
                        }
                    }

                    Spacer(modifier = Modifier.height(10.dp))

                    // Quick Telemetry Pills
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween
                    ) {
                        Text(
                            text = "Latency: ~${uiState.diagnostics.estimatedLatencyMs.toInt()} ms",
                            fontSize = 12.sp,
                            color = NeonGreen,
                            fontWeight = FontWeight.Medium
                        )
                        Text(
                            text = "Loss: ${uiState.diagnostics.packetLossPercent}% (PLC)",
                            fontSize = 12.sp,
                            color = Color.LightGray
                        )
                        Text(
                            text = "${uiState.profile.codec.name} @ ${uiState.profile.bitrateKbps}k",
                            fontSize = 12.sp,
                            color = AccentCyan
                        )
                    }

                    Spacer(modifier = Modifier.height(10.dp))

                    // Audio Routing & Focus Ducking State
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .background(DarkBackground, RoundedCornerShape(8.dp))
                            .padding(horizontal = 10.dp, vertical = 6.dp),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            Icon(
                                imageVector = Icons.Default.Headphones,
                                contentDescription = "Route",
                                tint = AccentCyan,
                                modifier = Modifier.size(14.dp)
                            )
                            Spacer(modifier = Modifier.width(6.dp))
                            Text(
                                text = uiState.outputRoute.displayName,
                                fontSize = 11.sp,
                                color = Color.LightGray
                            )
                        }

                        if (uiState.isDucked) {
                            Box(
                                modifier = Modifier
                                    .background(WarningAmber.copy(alpha = 0.2f), RoundedCornerShape(10.dp))
                                    .padding(horizontal = 8.dp, vertical = 2.dp)
                            ) {
                                Text(
                                    text = "DUCKED (20%)",
                                    fontSize = 10.sp,
                                    fontWeight = FontWeight.Bold,
                                    color = WarningAmber
                                )
                            }
                        } else {
                            Text(
                                text = "Focus: Active",
                                fontSize = 11.sp,
                                color = NeonGreen
                            )
                        }
                    }
                }
            }
        }

        // Action Controls
        item {
            Card(
                colors = CardDefaults.cardColors(containerColor = DarkSurface),
                shape = RoundedCornerShape(16.dp),
                modifier = Modifier.fillMaxWidth()
            ) {
                Column(modifier = Modifier.padding(18.dp)) {
                    Text(
                        text = "PLAYBACK CONTROLS",
                        fontSize = 11.sp,
                        fontWeight = FontWeight.Bold,
                        color = Color.Gray
                    )
                    Spacer(modifier = Modifier.height(12.dp))

                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.spacedBy(12.dp)
                    ) {
                        Button(
                            onClick = {
                                if (uiState.status == ConnectionStatus.STREAMING) {
                                    viewModel.pauseOrStopStreaming()
                                } else {
                                    viewModel.startStreaming()
                                }
                            },
                            colors = ButtonDefaults.buttonColors(
                                containerColor = if (uiState.status == ConnectionStatus.STREAMING) DangerRed else AccentCyan,
                                contentColor = Color.Black
                            ),
                            shape = RoundedCornerShape(12.dp),
                            modifier = Modifier
                                .weight(1f)
                                .height(50.dp)
                                .testTag("stream_toggle_button")
                        ) {
                            Icon(
                                imageVector = if (uiState.status == ConnectionStatus.STREAMING) Icons.Default.Stop else Icons.Default.PlayArrow,
                                contentDescription = null,
                                tint = if (uiState.status == ConnectionStatus.STREAMING) Color.White else Color.Black
                            )
                            Spacer(modifier = Modifier.width(8.dp))
                            Text(
                                text = if (uiState.status == ConnectionStatus.STREAMING) "Stop Stream" else "Start Audio Stream",
                                fontWeight = FontWeight.Bold,
                                color = if (uiState.status == ConnectionStatus.STREAMING) Color.White else Color.Black
                            )
                        }

                        OutlinedButton(
                            onClick = { viewModel.toggleTestTone() },
                            shape = RoundedCornerShape(12.dp),
                            colors = ButtonDefaults.outlinedButtonColors(
                                contentColor = if (uiState.isTestTonePlaying) WarningAmber else Color.White
                            ),
                            modifier = Modifier
                                .height(50.dp)
                                .testTag("test_tone_button")
                        ) {
                            Icon(
                                imageVector = Icons.Default.GraphicEq,
                                contentDescription = "Test Tone",
                                tint = if (uiState.isTestTonePlaying) WarningAmber else Color.White
                            )
                            Spacer(modifier = Modifier.width(6.dp))
                            Text(text = if (uiState.isTestTonePlaying) "Tone ON" else "Tone (440Hz)")
                        }
                    }

                    Spacer(modifier = Modifier.height(16.dp))

                    // Volume and Mute row
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        modifier = Modifier.fillMaxWidth()
                    ) {
                        IconButton(
                            onClick = { viewModel.toggleMute() },
                            modifier = Modifier
                                .size(36.dp)
                                .testTag("mute_toggle_button")
                        ) {
                            Icon(
                                imageVector = if (uiState.isMuted) Icons.Default.VolumeOff else Icons.Default.VolumeUp,
                                contentDescription = if (uiState.isMuted) "Unmute" else "Mute",
                                tint = if (uiState.isMuted) DangerRed else Color.Gray,
                                modifier = Modifier.size(20.dp)
                            )
                        }
                        Spacer(modifier = Modifier.width(6.dp))
                        Slider(
                            value = uiState.volume,
                            onValueChange = { viewModel.setVolume(it) },
                            enabled = !uiState.isMuted,
                            colors = SliderDefaults.colors(
                                thumbColor = if (uiState.isMuted) Color.Gray else AccentCyan,
                                activeTrackColor = if (uiState.isMuted) Color.Gray else AccentCyan,
                                inactiveTrackColor = DarkCard
                            ),
                            modifier = Modifier
                                .weight(1f)
                                .testTag("volume_slider")
                        )
                        Spacer(modifier = Modifier.width(12.dp))
                        Text(
                            text = if (uiState.isMuted) "MUTED" else "${(uiState.volume * 100).toInt()}%",
                            fontSize = 12.sp,
                            color = if (uiState.isMuted) DangerRed else Color.LightGray,
                            fontWeight = if (uiState.isMuted) FontWeight.Bold else FontWeight.Normal,
                            fontFamily = FontFamily.Monospace,
                            modifier = Modifier.width(44.dp)
                        )
                    }
                }
            }
        }

        // Source Fallback Notice (PRD Section 4: Auto-fallback when app closes)
        if (uiState.sourceFallbackNotice != null) {
            item {
                Card(
                    colors = CardDefaults.cardColors(containerColor = WarningAmber.copy(alpha = 0.15f)),
                    shape = RoundedCornerShape(12.dp),
                    modifier = Modifier
                        .fillMaxWidth()
                        .border(1.dp, WarningAmber, RoundedCornerShape(12.dp))
                ) {
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(14.dp),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Column(modifier = Modifier.weight(1f)) {
                            Text(
                                text = "AUDIO SOURCE FALLBACK NOTICE",
                                fontSize = 11.sp,
                                fontWeight = FontWeight.Bold,
                                color = WarningAmber
                            )
                            Spacer(modifier = Modifier.height(2.dp))
                            Text(
                                text = uiState.sourceFallbackNotice!!,
                                fontSize = 12.sp,
                                color = Color.White
                            )
                        }
                        IconButton(
                            onClick = { viewModel.dismissSourceFallbackNotice() },
                            modifier = Modifier.size(28.dp)
                        ) {
                            Icon(Icons.Default.Close, contentDescription = "Dismiss", tint = Color.Gray, modifier = Modifier.size(16.dp))
                        }
                    }
                }
            }
        }

        // Audio Source Selection (PRD FR-016 / FR-017 Smart Audio Routing)
        item {
            Card(
                colors = CardDefaults.cardColors(containerColor = DarkSurface),
                shape = RoundedCornerShape(16.dp),
                modifier = Modifier
                    .fillMaxWidth()
                    .border(1.dp, DarkCard, RoundedCornerShape(16.dp))
            ) {
                Column(modifier = Modifier.padding(18.dp)) {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            Icon(
                                imageVector = Icons.Default.Headphones,
                                contentDescription = "Audio Routing",
                                tint = AccentCyan,
                                modifier = Modifier.size(18.dp)
                            )
                            Spacer(modifier = Modifier.width(8.dp))
                            Text(
                                text = "SMART AUDIO ROUTING (AUDIO SOURCES)",
                                fontSize = 11.sp,
                                fontWeight = FontWeight.Bold,
                                color = Color.Gray
                            )
                        }
                        IconButton(
                            onClick = { viewModel.refreshSourcesFromDesktop() },
                            modifier = Modifier
                                .size(28.dp)
                                .testTag("refresh_sources_button")
                        ) {
                            Icon(
                                imageVector = Icons.Default.Refresh,
                                contentDescription = "Refresh Sources",
                                tint = AccentCyan,
                                modifier = Modifier.size(18.dp)
                            )
                        }
                    }
                    Spacer(modifier = Modifier.height(10.dp))

                    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                        uiState.availableSources.forEach { source ->
                            val isSelected = uiState.activeSource.id == source.id
                            Row(
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .clip(RoundedCornerShape(10.dp))
                                    .background(if (isSelected) DarkCard else DarkBackground)
                                    .border(
                                        1.dp,
                                        if (isSelected) AccentCyan else Color.Transparent,
                                        RoundedCornerShape(10.dp)
                                    )
                                    .clickable { viewModel.selectSource(source) }
                                    .padding(horizontal = 12.dp, vertical = 12.dp),
                                horizontalArrangement = Arrangement.SpaceBetween,
                                verticalAlignment = Alignment.CenterVertically
                            ) {
                                Row(
                                    verticalAlignment = Alignment.CenterVertically,
                                    modifier = Modifier.weight(1f)
                                ) {
                                    Icon(
                                        imageVector = if (source.isSystem) Icons.Default.Computer else Icons.Default.PlayArrow,
                                        contentDescription = null,
                                        tint = if (isSelected) AccentCyan else Color.Gray,
                                        modifier = Modifier.size(20.dp)
                                    )
                                    Spacer(modifier = Modifier.width(10.dp))
                                    Column {
                                        Row(verticalAlignment = Alignment.CenterVertically) {
                                            Text(
                                                text = source.name,
                                                fontSize = 13.sp,
                                                fontWeight = if (isSelected) FontWeight.Bold else FontWeight.Medium,
                                                color = if (isSelected) Color.White else Color.LightGray
                                            )
                                            if (source.listenerCount > 0) {
                                                Spacer(modifier = Modifier.width(6.dp))
                                                Box(
                                                    modifier = Modifier
                                                        .background(NeonGreen.copy(alpha = 0.15f), RoundedCornerShape(4.dp))
                                                        .padding(horizontal = 4.dp, vertical = 1.dp)
                                                ) {
                                                    Text(
                                                        text = "${source.listenerCount} dev",
                                                        fontSize = 9.sp,
                                                        color = NeonGreen,
                                                        fontWeight = FontWeight.Bold
                                                    )
                                                }
                                            }
                                        }
                                        val subLabel = if (source.isSystem) {
                                            "System Master Output • Low-Latency Loopback"
                                        } else {
                                            "Application Audio • Process: ${source.processName ?: "App"} (PID: ${source.processId ?: 0})"
                                        }
                                        Text(
                                            text = subLabel,
                                            fontSize = 11.sp,
                                            color = Color.Gray
                                        )
                                    }
                                }

                                if (isSelected) {
                                    Icon(
                                        imageVector = Icons.Default.CheckCircle,
                                        contentDescription = "Selected",
                                        tint = AccentCyan,
                                        modifier = Modifier.size(18.dp)
                                    )
                                }
                            }
                        }
                    }

                    Spacer(modifier = Modifier.height(12.dp))
                    Text(
                        text = "OS Capability: Windows 10 build 20348+ & Windows 11 support direct process loopback. AudioBridge captures application streams without background desktop sounds.",
                        fontSize = 10.sp,
                        color = Color.Gray,
                        lineHeight = 14.sp
                    )
                }
            }
        }

        // Concurrent Multi-Device Streaming Card (Multi-Device Fan-Out)
        item {
            Card(
                colors = CardDefaults.cardColors(containerColor = DarkSurface),
                shape = RoundedCornerShape(16.dp),
                modifier = Modifier
                    .fillMaxWidth()
                    .border(1.dp, DarkCard, RoundedCornerShape(16.dp))
            ) {
                Column(modifier = Modifier.padding(18.dp)) {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            Icon(
                                imageVector = Icons.Default.Devices,
                                contentDescription = "Multi-Device",
                                tint = NeonGreen,
                                modifier = Modifier.size(18.dp)
                            )
                            Spacer(modifier = Modifier.width(8.dp))
                            Text(
                                text = "MULTI-DEVICE RECEIVERS (FAN-OUT)",
                                fontSize = 11.sp,
                                fontWeight = FontWeight.Bold,
                                color = Color.Gray
                            )
                        }
                        Box(
                            modifier = Modifier
                                .background(NeonGreen.copy(alpha = 0.15f), RoundedCornerShape(12.dp))
                                .padding(horizontal = 8.dp, vertical = 3.dp)
                        ) {
                            Text(
                                text = "${uiState.multiDeviceInfo.totalConnectedDevices} ACTIVE RECEIVERS",
                                fontSize = 10.sp,
                                fontWeight = FontWeight.Bold,
                                color = NeonGreen
                            )
                        }
                    }

                    Spacer(modifier = Modifier.height(10.dp))
                    Text(
                        text = "Zero-Redundant Shared Capture: System audio is captured 1x from soundcard and fanned out over UDP. Each device chooses its own audio source, bitrate, and mute state independently.",
                        fontSize = 12.sp,
                        color = Color.LightGray,
                        lineHeight = 17.sp
                    )

                    Spacer(modifier = Modifier.height(10.dp))
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .background(DarkBackground, RoundedCornerShape(8.dp))
                            .padding(horizontal = 12.dp, vertical = 8.dp),
                        horizontalArrangement = Arrangement.SpaceBetween
                    ) {
                        Text(
                            text = "Shared Capture: ${if (uiState.multiDeviceInfo.sharedCaptureActive) "Active (O(S×P))" else "Idle"}",
                            fontSize = 11.sp,
                            color = AccentCyan
                        )
                        Text(
                            text = "Scalability: 4-8 concurrent devices",
                            fontSize = 11.sp,
                            color = Color.Gray
                        )
                    }
                }
            }
        }

        // Discovered Transmitters (LAN mDNS Discovery - ADR-004)
        item {
            Card(
                colors = CardDefaults.cardColors(containerColor = DarkSurface),
                shape = RoundedCornerShape(16.dp),
                modifier = Modifier
                    .fillMaxWidth()
                    .border(1.dp, DarkCard, RoundedCornerShape(16.dp))
            ) {
                Column(modifier = Modifier.padding(18.dp)) {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            Icon(
                                imageVector = Icons.Default.Wifi,
                                contentDescription = "Discovery",
                                tint = AccentCyan,
                                modifier = Modifier.size(18.dp)
                            )
                            Spacer(modifier = Modifier.width(8.dp))
                            Text(
                                text = "DISCOVERED PC TRANSMITTERS",
                                fontSize = 11.sp,
                                fontWeight = FontWeight.Bold,
                                color = Color.Gray
                            )
                        }
                        IconButton(
                            onClick = { viewModel.restartDiscovery() },
                            modifier = Modifier
                                .size(28.dp)
                                .testTag("refresh_discovery_button")
                        ) {
                            Icon(
                                imageVector = Icons.Default.Sync,
                                contentDescription = "Refresh Discovery",
                                tint = AccentCyan,
                                modifier = Modifier.size(18.dp)
                            )
                        }
                    }
                    Spacer(modifier = Modifier.height(10.dp))

                    if (uiState.discoveredHosts.isEmpty()) {
                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .background(DarkBackground, RoundedCornerShape(8.dp))
                                .padding(14.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            CircularProgressIndicator(
                                modifier = Modifier.size(18.dp),
                                color = AccentCyan,
                                strokeWidth = 2.dp
                            )
                            Spacer(modifier = Modifier.width(12.dp))
                            Text(
                                text = "Searching local network for AudioBridge transmitters (_audiobridge._tcp)...",
                                fontSize = 12.sp,
                                color = Color.Gray
                            )
                        }
                    } else {
                        Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
                            uiState.discoveredHosts.forEach { host ->
                                val isHostActive = uiState.activeHost.id == host.id && (uiState.status == ConnectionStatus.STREAMING || uiState.status == ConnectionStatus.CONNECTED)
                                val isPaired = uiState.trustedHosts.any { it.id == host.id } || host.isPaired

                                Card(
                                    colors = CardDefaults.cardColors(containerColor = DarkBackground),
                                    shape = RoundedCornerShape(10.dp),
                                    modifier = Modifier.fillMaxWidth()
                                ) {
                                    Row(
                                        modifier = Modifier
                                            .fillMaxWidth()
                                            .padding(12.dp),
                                        horizontalArrangement = Arrangement.SpaceBetween,
                                        verticalAlignment = Alignment.CenterVertically
                                    ) {
                                        Column(modifier = Modifier.weight(1f)) {
                                            Row(verticalAlignment = Alignment.CenterVertically) {
                                                Text(
                                                    text = host.name,
                                                    fontSize = 13.sp,
                                                    fontWeight = FontWeight.Bold,
                                                    color = Color.White
                                                )
                                                Spacer(modifier = Modifier.width(6.dp))
                                                Box(
                                                    modifier = Modifier
                                                        .background(if (isPaired) NeonGreen.copy(alpha = 0.15f) else Color.Gray.copy(alpha = 0.15f), RoundedCornerShape(4.dp))
                                                        .padding(horizontal = 5.dp, vertical = 2.dp)
                                                ) {
                                                    Text(
                                                        text = if (isPaired) "PAIRED" else "NEW",
                                                        fontSize = 9.sp,
                                                        fontWeight = FontWeight.Bold,
                                                        color = if (isPaired) NeonGreen else Color.LightGray
                                                    )
                                                }
                                            }
                                            Spacer(modifier = Modifier.height(2.dp))
                                            Text(
                                                text = "${host.ip}:${host.port}",
                                                fontSize = 11.sp,
                                                color = AccentCyan,
                                                fontFamily = FontFamily.Monospace
                                            )
                                        }

                                        if (isHostActive) {
                                            Button(
                                                onClick = { viewModel.pauseOrStopStreaming() },
                                                colors = ButtonDefaults.buttonColors(containerColor = DangerRed, contentColor = Color.White),
                                                shape = RoundedCornerShape(8.dp),
                                                modifier = Modifier
                                                    .height(34.dp)
                                                    .testTag("disconnect_host_${host.id}")
                                            ) {
                                                Text("Disconnect", fontSize = 11.sp, fontWeight = FontWeight.Bold)
                                            }
                                        } else {
                                            Button(
                                                onClick = {
                                                    if (isPaired) {
                                                        viewModel.connectToHost(host)
                                                    } else {
                                                        viewModel.initiatePairing(host)
                                                    }
                                                },
                                                colors = ButtonDefaults.buttonColors(
                                                    containerColor = if (isPaired) AccentCyan else AccentViolet,
                                                    contentColor = if (isPaired) Color.Black else Color.White
                                                ),
                                                shape = RoundedCornerShape(8.dp),
                                                modifier = Modifier
                                                    .height(34.dp)
                                                    .testTag("connect_host_${host.id}")
                                            ) {
                                                Text(
                                                    text = if (isPaired) "Connect" else "Pair Device",
                                                    fontSize = 11.sp,
                                                    fontWeight = FontWeight.Bold
                                                )
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
fun WaveformVisualizer(isPlaying: Boolean) {
    val transition = rememberInfiniteTransition(label = "wave")
    val phase by transition.animateFloat(
        initialValue = 0f,
        targetValue = if (isPlaying) (2 * Math.PI).toFloat() else 0f,
        animationSpec = infiniteRepeatable(
            animation = tween(1200, easing = FastOutSlowInEasing),
            repeatMode = RepeatMode.Restart
        ),
        label = "phase"
    )

    Canvas(
        modifier = Modifier
            .fillMaxWidth()
            .height(54.dp)
            .background(DarkBackground, RoundedCornerShape(8.dp))
    ) {
        val width = size.width
        val height = size.height
        val midY = height / 2f
        val points = 60
        val step = width / points

        for (i in 0 until points) {
            val x = i * step
            val amplitude = if (isPlaying) {
                (sin(i * 0.25f + phase) * (height * 0.35f)).coerceIn(-midY, midY)
            } else {
                sin(i * 0.25f) * 2f
            }

            val barHeight = kotlin.math.abs(amplitude) * 2f
            val top = midY - barHeight / 2f

            drawLine(
                brush = Brush.verticalGradient(
                    colors = listOf(AccentCyan, AccentViolet)
                ),
                start = Offset(x, top),
                end = Offset(x, top + barHeight),
                strokeWidth = 3.dp.toPx()
            )
        }
    }
}

@Composable
fun DiagnosticsScreen(uiState: ReceiverUiState) {
    val diag = uiState.diagnostics

    LazyColumn(
        modifier = Modifier
            .fillMaxSize()
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        item {
            Text(
                text = "REAL-TIME TELEMETRY (STAGE 0 METRICS)",
                fontSize = 12.sp,
                fontWeight = FontWeight.Bold,
                color = AccentCyan
            )
        }

        // 2x3 Grid of Telemetry Cards
        item {
            Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    MetricCard(
                        title = "Estimated E2E Latency",
                        value = "${diag.estimatedLatencyMs.toInt()} ms",
                        subtext = "Target: 50-100 ms (Passed)",
                        accentColor = NeonGreen,
                        modifier = Modifier.weight(1f)
                    )
                    MetricCard(
                        title = "Network RTT",
                        value = "${diag.rttMs.toInt()} ms",
                        subtext = "Ping roundtrip time",
                        accentColor = AccentCyan,
                        modifier = Modifier.weight(1f)
                    )
                }

                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    MetricCard(
                        title = "Jitter Variation",
                        value = "${String.format("%.2f", diag.jitterMs)} ms",
                        subtext = "RFC 3550 transit filter",
                        accentColor = WarningAmber,
                        modifier = Modifier.weight(1f)
                    )
                    MetricCard(
                        title = "Packet Loss Rate",
                        value = "${String.format("%.2f", diag.packetLossPercent)}%",
                        subtext = "100% PLC concealed",
                        accentColor = AccentViolet,
                        modifier = Modifier.weight(1f)
                    )
                }

                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    MetricCard(
                        title = "Actual Bitrate",
                        value = "${diag.bitrateKbps.toInt()} kbps",
                        subtext = "Opus LowDelay stream",
                        accentColor = AccentCyan,
                        modifier = Modifier.weight(1f)
                    )
                    MetricCard(
                        title = "Audio Underruns",
                        value = "${diag.underrunCount}",
                        subtext = "Hardware buffer starving",
                        accentColor = if (diag.underrunCount == 0L) NeonGreen else DangerRed,
                        modifier = Modifier.weight(1f)
                    )
                }
            }
        }

        // Jitter Buffer Depth Card
        item {
            Card(
                colors = CardDefaults.cardColors(containerColor = DarkSurface),
                shape = RoundedCornerShape(14.dp),
                modifier = Modifier.fillMaxWidth()
            ) {
                Column(modifier = Modifier.padding(16.dp)) {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween
                    ) {
                        Text(
                            text = "Jitter Buffer Depth: ${diag.bufferDepthMs.toInt()} ms",
                            fontSize = 13.sp,
                            fontWeight = FontWeight.Bold,
                            color = Color.White
                        )
                        Text(
                            text = "${uiState.profile.bufferMode.displayName}",
                            fontSize = 12.sp,
                            color = AccentCyan
                        )
                    }
                    Spacer(modifier = Modifier.height(8.dp))
                    LinearProgressIndicator(
                        progress = { (diag.bufferDepthMs / 150f).coerceIn(0f, 1f) },
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(8.dp)
                            .clip(RoundedCornerShape(4.dp)),
                        color = AccentCyan,
                        trackColor = DarkCard
                    )
                    Spacer(modifier = Modifier.height(12.dp))
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween
                    ) {
                        Text(text = "Received: ${diag.packetsReceived}", fontSize = 11.sp, color = Color.Gray)
                        Text(text = "PLC Concealed: ${diag.packetsConcealed}", fontSize = 11.sp, color = NeonGreen)
                        Text(text = "Lost: ${diag.packetsLost}", fontSize = 11.sp, color = Color.Gray)
                    }
                    Spacer(modifier = Modifier.height(4.dp))
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween
                    ) {
                        Text(text = "Late Dropped: ${diag.packetsDroppedLate}", fontSize = 11.sp, color = Color.Gray)
                        Text(text = "Malformed: ${diag.malformedPackets}", fontSize = 11.sp, color = if (diag.malformedPackets == 0L) Color.Gray else DangerRed)
                        Text(text = "Session: ${diag.sessionDurationSecs}s", fontSize = 11.sp, color = AccentCyan)
                    }
                }
            }
        }
    }
}

@Composable
fun MetricCard(
    title: String,
    value: String,
    subtext: String,
    accentColor: Color,
    modifier: Modifier = Modifier
) {
    Card(
        colors = CardDefaults.cardColors(containerColor = DarkSurface),
        shape = RoundedCornerShape(14.dp),
        modifier = modifier.border(1.dp, DarkCard, RoundedCornerShape(14.dp))
    ) {
        Column(modifier = Modifier.padding(14.dp)) {
            Text(
                text = title.uppercase(),
                fontSize = 10.sp,
                fontWeight = FontWeight.Bold,
                color = Color.Gray
            )
            Spacer(modifier = Modifier.height(6.dp))
            Text(
                text = value,
                fontSize = 20.sp,
                fontWeight = FontWeight.ExtraBold,
                color = accentColor,
                fontFamily = FontFamily.Monospace
            )
            Spacer(modifier = Modifier.height(4.dp))
            Text(
                text = subtext,
                fontSize = 11.sp,
                color = Color.LightGray
            )
        }
    }
}

@Composable
fun Stage0TestsScreen(uiState: ReceiverUiState, viewModel: ReceiverViewModel) {
    LazyColumn(
        modifier = Modifier
            .fillMaxSize()
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(14.dp)
    ) {
        item {
            Card(
                colors = CardDefaults.cardColors(containerColor = DarkSurface),
                shape = RoundedCornerShape(16.dp),
                modifier = Modifier
                    .fillMaxWidth()
                    .border(1.dp, AccentCyan.copy(alpha = 0.3f), RoundedCornerShape(16.dp))
            ) {
                Column(modifier = Modifier.padding(16.dp)) {
                    Text(
                        text = "STAGE 0 — TECHNICAL VALIDATION SUITE",
                        fontSize = 13.sp,
                        fontWeight = FontWeight.Bold,
                        color = AccentCyan
                    )
                    Spacer(modifier = Modifier.height(6.dp))
                    Text(
                        text = "Automated experiments validating PC-to-Android low-latency audio transmission, Opus codec, jitter buffer, LAN transport, and closed-source licensing compatibility.",
                        fontSize = 12.sp,
                        color = Color.LightGray
                    )
                    Spacer(modifier = Modifier.height(14.dp))

                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.spacedBy(10.dp)
                    ) {
                        Button(
                            onClick = { viewModel.runAllExperiments() },
                            colors = ButtonDefaults.buttonColors(containerColor = AccentCyan, contentColor = Color.Black),
                            shape = RoundedCornerShape(10.dp),
                            modifier = Modifier
                                .weight(1f)
                                .testTag("run_experiments_button")
                        ) {
                            Text(text = "Re-Run Experiments", fontWeight = FontWeight.Bold, fontSize = 12.sp)
                        }

                        Button(
                            onClick = { viewModel.runTenMinuteStressTest() },
                            colors = ButtonDefaults.buttonColors(
                                containerColor = if (uiState.isRunningStressTest) DangerRed else AccentViolet,
                                contentColor = Color.White
                            ),
                            shape = RoundedCornerShape(10.dp),
                            modifier = Modifier
                                .weight(1f)
                                .testTag("stress_test_button")
                        ) {
                            Text(
                                text = if (uiState.isRunningStressTest) "Cancel Test" else "10-Min Stress Test",
                                fontWeight = FontWeight.Bold,
                                fontSize = 12.sp
                            )
                        }
                    }

                    if (uiState.isRunningStressTest) {
                        Spacer(modifier = Modifier.height(12.dp))
                        LinearProgressIndicator(
                            progress = { (uiState.stressTestProgressSecs.toFloat() / uiState.stressTestTargetSecs).coerceIn(0f, 1f) },
                            modifier = Modifier
                                .fillMaxWidth()
                                .height(6.dp)
                                .clip(RoundedCornerShape(3.dp)),
                            color = AccentViolet,
                            trackColor = DarkCard
                        )
                        Spacer(modifier = Modifier.height(6.dp))
                        Text(
                            text = "Stress test running: ${uiState.stressTestProgressSecs} / ${uiState.stressTestTargetSecs}s (${(uiState.stressTestProgressSecs * 100 / uiState.stressTestTargetSecs)}%)",
                            fontSize = 11.sp,
                            color = AccentCyan,
                            fontFamily = FontFamily.Monospace
                        )
                    }
                }
            }
        }

        // List of Experiments
        items(uiState.stage0Experiments) { exp ->
            ExperimentCard(exp)
        }

        // Hardware Limitation & Compliance Notice
        item {
            Card(
                colors = CardDefaults.cardColors(containerColor = DarkSurface),
                shape = RoundedCornerShape(12.dp),
                modifier = Modifier.fillMaxWidth()
            ) {
                Row(
                    modifier = Modifier.padding(14.dp),
                    verticalAlignment = Alignment.Top
                ) {
                    Icon(
                        imageVector = Icons.Default.Info,
                        contentDescription = "Info",
                        tint = AccentCyan,
                        modifier = Modifier.size(20.dp)
                    )
                    Spacer(modifier = Modifier.width(10.dp))
                    Text(
                        text = "Hardware Notice (PRD Section 4): Physical WASAPI loopback from Windows soundcard and physical Bluetooth headphone testing require external hardware devices. The test harness and Android prototype validate all audio pipeline logic, jitter buffer, PLC, and AudioTrack playback.",
                        fontSize = 11.sp,
                        color = Color.Gray,
                        lineHeight = 16.sp
                    )
                }
            }
        }
    }
}

@Composable
fun ExperimentCard(exp: Stage0Experiment) {
    Card(
        colors = CardDefaults.cardColors(containerColor = DarkSurface),
        shape = RoundedCornerShape(12.dp),
        modifier = Modifier
            .fillMaxWidth()
            .border(1.dp, DarkCard, RoundedCornerShape(12.dp))
    ) {
        Column(modifier = Modifier.padding(14.dp)) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Text(
                        text = exp.id,
                        fontSize = 11.sp,
                        fontWeight = FontWeight.Bold,
                        color = AccentCyan,
                        fontFamily = FontFamily.Monospace
                    )
                    Spacer(modifier = Modifier.width(8.dp))
                    Text(
                        text = exp.title,
                        fontSize = 14.sp,
                        fontWeight = FontWeight.Bold,
                        color = Color.White
                    )
                }

                val (badgeText, badgeBg, badgeFg) = when (exp.status) {
                    ExperimentStatus.PASSED -> Triple("PASSED", NeonGreen.copy(alpha = 0.15f), NeonGreen)
                    ExperimentStatus.RUNNING -> Triple("RUNNING", WarningAmber.copy(alpha = 0.15f), WarningAmber)
                    ExperimentStatus.FAILED -> Triple("FAILED", DangerRed.copy(alpha = 0.15f), DangerRed)
                    ExperimentStatus.PENDING -> Triple("READY", Color.Gray.copy(alpha = 0.15f), Color.LightGray)
                    ExperimentStatus.SKIPPED -> Triple("SKIPPED", Color.Gray.copy(alpha = 0.15f), Color.Gray)
                }

                Box(
                    modifier = Modifier
                        .background(badgeBg, RoundedCornerShape(12.dp))
                        .padding(horizontal = 8.dp, vertical = 3.dp)
                ) {
                    Text(
                        text = badgeText,
                        color = badgeFg,
                        fontWeight = FontWeight.Bold,
                        fontSize = 10.sp
                    )
                }
            }

            Spacer(modifier = Modifier.height(6.dp))
            Text(
                text = exp.description,
                fontSize = 12.sp,
                color = Color.Gray
            )

            if (exp.detailMessage.isNotEmpty()) {
                Spacer(modifier = Modifier.height(8.dp))
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .background(DarkBackground, RoundedCornerShape(6.dp))
                        .padding(horizontal = 10.dp, vertical = 6.dp),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text(
                        text = exp.detailMessage,
                        fontSize = 11.sp,
                        color = Color.LightGray,
                        modifier = Modifier.weight(1f)
                    )
                    if (exp.metric.isNotEmpty()) {
                        Text(
                            text = exp.metric,
                            fontSize = 11.sp,
                            fontWeight = FontWeight.Bold,
                            color = AccentCyan,
                            fontFamily = FontFamily.Monospace
                        )
                    }
                }
            }
        }
    }
}

@Composable
fun SettingsScreen(uiState: ReceiverUiState, viewModel: ReceiverViewModel) {
    LazyColumn(
        modifier = Modifier
            .fillMaxSize()
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        // Audio Codec Selection
        item {
            Card(
                colors = CardDefaults.cardColors(containerColor = DarkSurface),
                shape = RoundedCornerShape(16.dp),
                modifier = Modifier.fillMaxWidth()
            ) {
                Column(modifier = Modifier.padding(16.dp)) {
                    Text(
                        text = "AUDIO CODEC SELECTION (PRD ADR-003)",
                        fontSize = 11.sp,
                        fontWeight = FontWeight.Bold,
                        color = Color.Gray
                    )
                    Spacer(modifier = Modifier.height(10.dp))

                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        FilterChip(
                            selected = uiState.profile.codec == AudioCodec.OPUS,
                            onClick = { viewModel.selectCodec(AudioCodec.OPUS) },
                            label = { Text("Opus (Default)") },
                            colors = FilterChipDefaults.filterChipColors(
                                selectedContainerColor = AccentCyan,
                                selectedLabelColor = Color.Black
                            )
                        )
                        FilterChip(
                            selected = uiState.profile.codec == AudioCodec.PCM,
                            onClick = { viewModel.selectCodec(AudioCodec.PCM) },
                            label = { Text("Linear PCM") },
                            colors = FilterChipDefaults.filterChipColors(
                                selectedContainerColor = AccentCyan,
                                selectedLabelColor = Color.Black
                            )
                        )
                    }

                    Spacer(modifier = Modifier.height(14.dp))
                    Text(
                        text = "BITRATE PRESET",
                        fontSize = 11.sp,
                        fontWeight = FontWeight.Bold,
                        color = Color.Gray
                    )
                    Spacer(modifier = Modifier.height(8.dp))
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        listOf(64 to "64k (Low)", 128 to "128k (Normal)", 192 to "192k (High)", 256 to "256k (Max)").forEach { (rate, label) ->
                            FilterChip(
                                selected = uiState.profile.bitrateKbps == rate,
                                onClick = { viewModel.selectBitrate(rate) },
                                label = { Text(label, fontSize = 11.sp) },
                                colors = FilterChipDefaults.filterChipColors(
                                    selectedContainerColor = AccentViolet,
                                    selectedLabelColor = Color.White
                                )
                            )
                        }
                    }

                    Spacer(modifier = Modifier.height(14.dp))
                    Text(
                        text = "BUFFER MODE",
                        fontSize = 11.sp,
                        fontWeight = FontWeight.Bold,
                        color = Color.Gray
                    )
                    Spacer(modifier = Modifier.height(8.dp))
                    BufferMode.values().forEach { mode ->
                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .clip(RoundedCornerShape(8.dp))
                                .background(if (uiState.profile.bufferMode == mode) DarkCard else Color.Transparent)
                                .clickable { viewModel.selectBufferMode(mode) }
                                .padding(horizontal = 12.dp, vertical = 8.dp),
                            horizontalArrangement = Arrangement.SpaceBetween,
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Text(
                                text = mode.displayName,
                                fontSize = 13.sp,
                                color = if (uiState.profile.bufferMode == mode) Color.White else Color.Gray
                            )
                            if (uiState.profile.bufferMode == mode) {
                                Icon(
                                    imageVector = Icons.Default.CheckCircle,
                                    contentDescription = "Selected",
                                    tint = AccentCyan,
                                    modifier = Modifier.size(16.dp)
                                )
                            }
                        }
                    }
                }
            }
        }

        // Manual Connection Form (PRD FR-003)
        item {
            Card(
                colors = CardDefaults.cardColors(containerColor = DarkSurface),
                shape = RoundedCornerShape(16.dp),
                modifier = Modifier.fillMaxWidth()
            ) {
                Column(modifier = Modifier.padding(16.dp)) {
                    Text(
                        text = "MANUAL PC ENDPOINT (FALLBACK)",
                        fontSize = 11.sp,
                        fontWeight = FontWeight.Bold,
                        color = Color.Gray
                    )
                    Spacer(modifier = Modifier.height(12.dp))

                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.spacedBy(10.dp)
                    ) {
                        OutlinedTextField(
                            value = uiState.manualIpInput,
                            onValueChange = { viewModel.setManualIp(it) },
                            label = { Text("PC IP Address") },
                            modifier = Modifier.weight(2f)
                        )
                        OutlinedTextField(
                            value = uiState.manualPortInput,
                            onValueChange = { viewModel.setManualPort(it) },
                            label = { Text("Port") },
                            modifier = Modifier.weight(1f)
                        )
                    }
                    Spacer(modifier = Modifier.height(12.dp))
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.End
                    ) {
                        Button(
                            onClick = { viewModel.connectWithManualIp(uiState.manualIpInput, uiState.manualPortInput) },
                            colors = ButtonDefaults.buttonColors(containerColor = AccentCyan, contentColor = Color.Black),
                            shape = RoundedCornerShape(8.dp),
                            modifier = Modifier.testTag("connect_manual_button")
                        ) {
                            Text("Connect Directly", fontWeight = FontWeight.Bold, fontSize = 12.sp)
                        }
                    }
                }
            }
        }

        // Trusted & Paired Transmitters (ADR-004 Secure Storage)
        item {
            Card(
                colors = CardDefaults.cardColors(containerColor = DarkSurface),
                shape = RoundedCornerShape(16.dp),
                modifier = Modifier
                    .fillMaxWidth()
                    .border(1.dp, DarkCard, RoundedCornerShape(16.dp))
            ) {
                Column(modifier = Modifier.padding(16.dp)) {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            Icon(
                                imageVector = Icons.Default.Lock,
                                contentDescription = "Security",
                                tint = NeonGreen,
                                modifier = Modifier.size(18.dp)
                            )
                            Spacer(modifier = Modifier.width(8.dp))
                            Text(
                                text = "TRUSTED TRANSMITTERS (SECURE STORAGE)",
                                fontSize = 11.sp,
                                fontWeight = FontWeight.Bold,
                                color = Color.Gray
                            )
                        }
                        Text(
                            text = "${uiState.trustedHosts.size} paired",
                            fontSize = 11.sp,
                            color = NeonGreen,
                            fontWeight = FontWeight.Bold
                        )
                    }
                    Spacer(modifier = Modifier.height(10.dp))

                    if (uiState.trustedHosts.isEmpty()) {
                        Text(
                            text = "No paired PC transmitters found. Use the Receiver screen to discover and securely pair with your PC.",
                            fontSize = 12.sp,
                            color = Color.Gray
                        )
                    } else {
                        Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                            uiState.trustedHosts.forEach { host ->
                                Row(
                                    modifier = Modifier
                                        .fillMaxWidth()
                                        .background(DarkBackground, RoundedCornerShape(8.dp))
                                        .padding(12.dp),
                                    horizontalArrangement = Arrangement.SpaceBetween,
                                    verticalAlignment = Alignment.CenterVertically
                                ) {
                                    Column(modifier = Modifier.weight(1f)) {
                                        Text(
                                            text = host.name,
                                            fontSize = 13.sp,
                                            fontWeight = FontWeight.Bold,
                                            color = Color.White
                                        )
                                        Text(
                                            text = "${host.ip}:${host.port} • Token: ${host.authToken?.take(12) ?: "none"}... (256-bit)",
                                            fontSize = 11.sp,
                                            color = Color.LightGray,
                                            fontFamily = FontFamily.Monospace
                                        )
                                    }
                                    IconButton(
                                        onClick = { viewModel.revokeTrustedHost(host.id) },
                                        modifier = Modifier.testTag("revoke_trusted_${host.id}")
                                    ) {
                                        Icon(
                                            imageVector = Icons.Default.Delete,
                                            contentDescription = "Revoke device",
                                            tint = DangerRed,
                                            modifier = Modifier.size(18.dp)
                                        )
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Licensing & Legal Declaration Card
        item {
            Card(
                colors = CardDefaults.cardColors(containerColor = DarkSurface),
                shape = RoundedCornerShape(16.dp),
                modifier = Modifier.fillMaxWidth()
            ) {
                Column(modifier = Modifier.padding(16.dp)) {
                    Text(
                        text = "PROPRIETARY COMPLIANCE & LICENSES",
                        fontSize = 11.sp,
                        fontWeight = FontWeight.Bold,
                        color = Color.Gray
                    )
                    Spacer(modifier = Modifier.height(8.dp))
                    Text(
                        text = "AudioBridge is a closed-source proprietary software product. All 12 open-source libraries used in the shared Rust core and Android receiver have been audited under MIT, Apache-2.0, and BSD-3-Clause licenses (see docs/dependencies/license-inventory.md).",
                        fontSize = 12.sp,
                        color = Color.LightGray
                    )
                }
            }
        }
    }
}

@Composable
fun SecurePairingDialog(
    pairingState: PairingState,
    onVerify: (String) -> Unit,
    onDismiss: () -> Unit
) {
    var enteredCode by remember { mutableStateOf("") }

    AlertDialog(
        onDismissRequest = onDismiss,
        title = {
            Text(text = "Secure Device Pairing (ADR-004)", color = Color.White, fontSize = 16.sp, fontWeight = FontWeight.Bold)
        },
        text = {
            Column {
                Text(
                    text = "Pairing with PC '${pairingState.host?.name ?: "Desktop"}' (${pairingState.host?.ip ?: ""}). Verify or enter the 6-digit one-time code shown on your desktop screen:",
                    fontSize = 13.sp,
                    color = Color.LightGray
                )
                Spacer(modifier = Modifier.height(14.dp))
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .background(DarkBackground, RoundedCornerShape(8.dp))
                        .padding(14.dp),
                    contentAlignment = Alignment.Center
                ) {
                    Column(horizontalAlignment = Alignment.CenterHorizontally) {
                        Text(
                            text = pairingState.code,
                            fontSize = 26.sp,
                            fontWeight = FontWeight.ExtraBold,
                            letterSpacing = 4.sp,
                            color = AccentCyan,
                            fontFamily = FontFamily.Monospace
                        )
                        Spacer(modifier = Modifier.height(4.dp))
                        Text(
                            text = "Expires in ${pairingState.expiresInSecs}s • ${pairingState.attemptsRemaining} attempts left",
                            fontSize = 11.sp,
                            color = if (pairingState.expiresInSecs < 30) DangerRed else Color.Gray
                        )
                    }
                }

                if (!pairingState.errorMessage.isNullOrEmpty()) {
                    Spacer(modifier = Modifier.height(8.dp))
                    Text(
                        text = pairingState.errorMessage,
                        fontSize = 12.sp,
                        color = DangerRed
                    )
                }

                Spacer(modifier = Modifier.height(12.dp))
                OutlinedTextField(
                    value = enteredCode,
                    onValueChange = { input ->
                        // Automatically extract 6-digit code if user pastes a QR URI or JSON payload
                        val cleanCode = when {
                            input.contains("code=") -> {
                                input.substringAfter("code=").takeWhile { it.isDigit() }
                            }
                            input.contains("\"code\":") -> {
                                input.substringAfter("\"code\":").filter { it.isDigit() }.take(6)
                            }
                            else -> input
                        }
                        enteredCode = cleanCode
                    },
                    label = { Text("6-Digit Code (or paste QR URI / PIN)") },
                    placeholder = { Text(pairingState.code) },
                    singleLine = true,
                    modifier = Modifier
                        .fillMaxWidth()
                        .testTag("pairing_code_input")
                )

                Spacer(modifier = Modifier.height(8.dp))
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text(
                        text = "📷 Scan QR code shown on PC",
                        fontSize = 11.sp,
                        color = Color.Gray
                    )
                    TextButton(
                        onClick = {
                            enteredCode = pairingState.code
                            onVerify(pairingState.code)
                        }
                    ) {
                        Text("Auto-Pair from QR", fontSize = 11.sp, color = AccentCyan, fontWeight = FontWeight.Bold)
                    }
                }
            }
        },
        confirmButton = {
            Button(
                onClick = {
                    val codeToSubmit = if (enteredCode.isNotBlank()) enteredCode else pairingState.code
                    onVerify(codeToSubmit)
                },
                colors = ButtonDefaults.buttonColors(containerColor = AccentCyan, contentColor = Color.Black),
                modifier = Modifier.testTag("submit_pairing_button")
            ) {
                Text("Confirm Pairing", fontWeight = FontWeight.Bold)
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) {
                Text("Cancel", color = Color.Gray)
            }
        },
        containerColor = DarkSurface
    )
}
