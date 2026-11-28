package dev.noct.launcher

import android.content.Context

val DefaultHomeOrder = listOf("clock", "battery", "recents", "widgets", "sync")

object GestureTapAction {
    const val HOME = 0
    const val APPS = 1
    const val SETTINGS = 2
}

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
    val gestureSize: Int = 1,
    val gestureColor: String = "#7CFFB2",
    val gestureOpacity: Int = 60,
    val gestureShape: Int = 0,
    val gestureTapAction: Int = GestureTapAction.HOME,
    val gestureDoubleTapAction: Int = GestureTapAction.APPS,
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
        gestureSize = sp.getInt("gestureSize", 1),
        gestureColor = sp.getString("gestureColor", "#7CFFB2") ?: "#7CFFB2",
        gestureOpacity = sp.getInt("gestureOpacity", 60),
        gestureShape = sp.getInt("gestureShape", 0),
        gestureTapAction = sp.getInt("gestureTapAction", GestureTapAction.HOME),
        gestureDoubleTapAction = sp.getInt("gestureDoubleTapAction", GestureTapAction.APPS),
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
            .putInt("gestureSize", prefs.gestureSize)
            .putString("gestureColor", prefs.gestureColor)
            .putInt("gestureOpacity", prefs.gestureOpacity)
            .putInt("gestureShape", prefs.gestureShape)
            .putInt("gestureTapAction", prefs.gestureTapAction)
            .putInt("gestureDoubleTapAction", prefs.gestureDoubleTapAction)
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
