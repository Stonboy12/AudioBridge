package com.example

import android.content.Context
import android.net.nsd.NsdManager
import android.net.nsd.NsdServiceInfo
import android.util.Log

/**
 * NsdDiscoveryManager implements local network device discovery using Android NsdManager.
 * Discovers Desktop AudioBridge services broadcasting "_audiobridge._tcp".
 * Resolves IP, port, and computer name for one-tap pairing and connection.
 */
class NsdDiscoveryManager(
    context: Context,
    private val onHostFound: (DesktopHost) -> Unit,
    private val onHostLost: (String) -> Unit
) {
    companion object {
        private const val TAG = "AudioBridge_NSD"
        const val SERVICE_TYPE = "_audiobridge._tcp."
    }

    private val nsdManager = context.getSystemService(Context.NSD_SERVICE) as? NsdManager
    private var isDiscoveryActive = false

    private val discoveryListener = object : NsdManager.DiscoveryListener {
        override fun onDiscoveryStarted(regType: String) {
            Log.d(TAG, "mDNS Service discovery started for $regType")
            isDiscoveryActive = true
        }

        override fun onServiceFound(service: NsdServiceInfo) {
            Log.d(TAG, "mDNS Service found: ${service.serviceName}")
            if (service.serviceType.contains("audiobridge")) {
                try {
                    nsdManager?.resolveService(service, createResolveListener())
                } catch (e: Exception) {
                    Log.w(TAG, "Failed to initiate service resolve", e)
                }
            }
        }

        override fun onServiceLost(service: NsdServiceInfo) {
            Log.d(TAG, "mDNS Service lost: ${service.serviceName}")
            onHostLost(service.serviceName)
        }

        override fun onDiscoveryStopped(serviceType: String) {
            Log.d(TAG, "mDNS Service discovery stopped: $serviceType")
            isDiscoveryActive = false
        }

        override fun onStartDiscoveryFailed(serviceType: String, errorCode: Int) {
            Log.e(TAG, "Start discovery failed: Error code $errorCode")
            isDiscoveryActive = false
        }

        override fun onStopDiscoveryFailed(serviceType: String, errorCode: Int) {
            Log.e(TAG, "Stop discovery failed: Error code $errorCode")
            isDiscoveryActive = false
        }
    }

    private fun createResolveListener() = object : NsdManager.ResolveListener {
        override fun onResolveFailed(serviceInfo: NsdServiceInfo, errorCode: Int) {
            Log.w(TAG, "Resolve failed for ${serviceInfo.serviceName}: code $errorCode")
        }

        override fun onServiceResolved(serviceInfo: NsdServiceInfo) {
            val hostAddress = serviceInfo.host?.hostAddress
            val port = serviceInfo.port
            val serviceName = serviceInfo.serviceName

            Log.i(TAG, "Service resolved: $serviceName at $hostAddress:$port")

            if (!hostAddress.isNullOrEmpty() && port > 0) {
                val host = DesktopHost(
                    id = "host_${serviceName.hashCode()}",
                    name = serviceName,
                    ip = hostAddress,
                    port = port,
                    isPaired = false,
                    isOnline = true
                )
                onHostFound(host)
            }
        }
    }

    fun startDiscovery() {
        if (!isDiscoveryActive && nsdManager != null) {
            try {
                nsdManager.discoverServices(SERVICE_TYPE, NsdManager.PROTOCOL_DNS_SD, discoveryListener)
            } catch (e: Exception) {
                Log.e(TAG, "Error starting NSD discovery", e)
            }
        }
    }

    fun stopDiscovery() {
        if (isDiscoveryActive && nsdManager != null) {
            try {
                nsdManager.stopServiceDiscovery(discoveryListener)
            } catch (e: Exception) {
                Log.w(TAG, "Error stopping NSD discovery", e)
            }
            isDiscoveryActive = false
        }
    }
}
