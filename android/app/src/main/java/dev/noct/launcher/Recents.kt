package dev.noct.launcher

import android.app.usage.UsageEvents
import android.app.usage.UsageStatsManager
import android.content.Context
import kotlin.math.max

data class RecentApp(
    val app: AppEntry,
    val lastUsed: Long,
)

/**
 * Estado aproximado de apps "vivas" a partir de UsageStats.
 *
 * No hay API pública para listar tareas de otras apps en Android moderno, así
 * que reconstruimos el estado por paquete: mientras el último evento de una
 * actividad sea RESUMED/PAUSED/STOPPED (o tenga un foreground service), la app
 * se considera viva. ACTIVITY_DESTROYED (evento 24, no público) la retira.
 * Es una aproximación: si el sistema mata una app sin emitir DESTROYED, puede
 * quedar un falso positivo hasta que expire la ventana temporal.
 */
object RecentApps {
    private const val WINDOW_MS = 6 * 60 * 60 * 1000L
    private const val ACTIVITY_DESTROYED = 24 // hidden, pero algunas ROMs lo emiten

    private val ignored = setOf(
        "com.android.systemui",
        "com.android.settings",
        "com.motorola.launcher3",
        "com.google.android.apps.nexuslauncher",
    )

    // package -> (activity class -> último evento)
    private val activityState = mutableMapOf<String, MutableMap<String, Int>>()
    private val lastSeen = mutableMapOf<String, Long>()
    private var lastQuery = 0L
    private var initialized = false

    @Synchronized
    fun load(context: Context, apps: List<AppEntry>, limit: Int = 5): List<RecentApp> {
        if (!UsageAccess.hasPermission(context) || apps.isEmpty()) return emptyList()
        val manager = context.getSystemService(Context.USAGE_STATS_SERVICE) as UsageStatsManager
        val now = System.currentTimeMillis()
        val start = if (initialized) {
            max(lastQuery - 1000L, now - WINDOW_MS)
        } else {
            now - WINDOW_MS
        }

        val events = manager.queryEvents(start, now)
        val event = UsageEvents.Event()
        while (events.hasNextEvent()) {
            events.getNextEvent(event)
            val pkg = event.packageName ?: continue
            val activity = event.className ?: "__default"
            when (event.eventType) {
                UsageEvents.Event.ACTIVITY_RESUMED,
                UsageEvents.Event.ACTIVITY_PAUSED,
                UsageEvents.Event.ACTIVITY_STOPPED,
                -> {
                    activityState.getOrPut(pkg) { mutableMapOf() }[activity] = event.eventType
                    lastSeen[pkg] = max(lastSeen[pkg] ?: 0L, event.timeStamp)
                }

                ACTIVITY_DESTROYED -> {
                    val activities = activityState[pkg]
                    if (activities != null) {
                        activities.remove(activity)
                        if (activities.isEmpty()) {
                            activityState.remove(pkg)
                            lastSeen.remove(pkg)
                        }
                    }
                }

                UsageEvents.Event.FOREGROUND_SERVICE_START -> {
                    activityState.getOrPut(pkg) { mutableMapOf() }["__fgs"] = event.eventType
                    lastSeen[pkg] = max(lastSeen[pkg] ?: 0L, event.timeStamp)
                }

                UsageEvents.Event.FOREGROUND_SERVICE_STOP -> {
                    val activities = activityState[pkg]
                    if (activities != null) {
                        activities.remove("__fgs")
                        if (activities.isEmpty()) {
                            activityState.remove(pkg)
                            lastSeen.remove(pkg)
                        }
                    }
                }
            }
        }
        lastQuery = now
        initialized = true

        // Purga paquetes fuera de la ventana para no crecer sin límite.
        val expired = lastSeen.filterValues { now - it > WINDOW_MS }.keys
        expired.forEach {
            activityState.remove(it)
            lastSeen.remove(it)
        }

        val currentUser = apps.filter { it.profileLabel == null }.associateBy { it.packageName }
        val anyUser = apps.associateBy { it.packageName }

        return activityState.keys
            .asSequence()
            .filter { it !in ignored && it != context.packageName }
            .mapNotNull { pkg ->
                val app = currentUser[pkg] ?: anyUser[pkg] ?: return@mapNotNull null
                val timestamp = lastSeen[pkg] ?: return@mapNotNull null
                if (now - timestamp > WINDOW_MS) null else RecentApp(app, timestamp)
            }
            .sortedByDescending { it.lastUsed }
            .take(limit)
            .toList()
    }
}
