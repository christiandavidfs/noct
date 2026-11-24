package dev.noct.launcher

import android.app.usage.UsageEvents
import android.app.usage.UsageStatsManager
import android.content.Context

object RecentApps {
    private val ignored = setOf(
        "com.android.systemui",
        "com.android.settings",
        "com.motorola.launcher3",
        "com.google.android.apps.nexuslauncher",
    )

    fun load(context: Context, apps: List<AppEntry>, limit: Int = 5): List<AppEntry> {
        if (!UsageAccess.hasPermission(context) || apps.isEmpty()) return emptyList()
        val manager = context.getSystemService(Context.USAGE_STATS_SERVICE) as UsageStatsManager
        val end = System.currentTimeMillis()
        val start = end - 12 * 60 * 60 * 1000L
        val events = manager.queryEvents(start, end)
        val lastSeen = HashMap<String, Long>()
        val event = UsageEvents.Event()
        while (events.hasNextEvent()) {
            events.getNextEvent(event)
            if (event.eventType == UsageEvents.Event.MOVE_TO_FOREGROUND ||
                event.eventType == UsageEvents.Event.ACTIVITY_RESUMED
            ) {
                lastSeen[event.packageName] = event.timeStamp
            }
        }
        val byPackage = apps.associateBy { it.packageName }
        return lastSeen.entries
            .asSequence()
            .filter { (pkg, _) -> pkg != context.packageName && pkg !in ignored }
            .sortedByDescending { it.value }
            .mapNotNull { byPackage[it.key] }
            .take(limit)
            .toList()
    }
}
