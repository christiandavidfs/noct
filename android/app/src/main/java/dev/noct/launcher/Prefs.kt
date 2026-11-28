package dev.noct.launcher

import android.content.Context

val DefaultHomeOrder = listOf("clock", "battery", "recents", "widgets", "sync")

data class LauncherPrefs(
    val homeOrder: List<String> = DefaultHomeOrder,
    val showClock: Boolean = true,
    val showBattery: Boolean = true,
    val showRecents: Boolean = true,
    val showSync: Boolean = true,
    val clock24h: Boolean = true,
    val serverUrl: String = "",
    val token: String = "",
    val themeJson: String? = null,
    val dismissed: Map<String, Long> = emptyMap(),
    val gestureHandle: Boolean = true,
)

class Prefs(context: Context) {
    private val sp = context.getSharedPreferences("noct_launcher", Context.MODE_PRIVATE)

    fun load(): LauncherPrefs = LauncherPrefs(
        homeOrder = sp.getString("homeOrder", null)
            ?.split("|")
            ?.filter { it.isNotBlank() }
            ?.takeIf { it.isNotEmpty() }
            ?: DefaultHomeOrder,
        showClock = sp.getBoolean("showClock", true),
        showBattery = sp.getBoolean("showBattery", true),
        showRecents = sp.getBoolean("showRecents", true),
        showSync = sp.getBoolean("showSync", true),
        clock24h = sp.getBoolean("clock24h", true),
        serverUrl = sp.getString("serverUrl", "") ?: "",
        token = sp.getString("token", "") ?: "",
        themeJson = sp.getString("themeJson", null),
        gestureHandle = sp.getBoolean("gestureHandle", true),
        dismissed = sp.getStringSet("dismissed", emptySet())
            ?.mapNotNull { entry ->
                val separator = entry.lastIndexOf(':')
                if (separator <= 0) null
                else entry.substring(0, separator) to
                    (entry.substring(separator + 1).toLongOrNull() ?: 0L)
            }
            ?.toMap()
            ?: emptyMap(),
    )

    fun save(prefs: LauncherPrefs) {
        sp.edit()
            .putString("homeOrder", prefs.homeOrder.joinToString("|"))
            .putBoolean("showClock", prefs.showClock)
            .putBoolean("showBattery", prefs.showBattery)
            .putBoolean("showRecents", prefs.showRecents)
            .putBoolean("showSync", prefs.showSync)
            .putBoolean("clock24h", prefs.clock24h)
            .putBoolean("gestureHandle", prefs.gestureHandle)
            .putString("serverUrl", prefs.serverUrl)
            .putString("token", prefs.token)
            .putString("themeJson", prefs.themeJson)
            .putStringSet(
                "dismissed",
                prefs.dismissed.map { (pkg, at) -> "$pkg:$at" }.toSet(),
            )
            .apply()
    }
}
