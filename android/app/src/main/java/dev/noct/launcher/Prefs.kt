package dev.noct.launcher

import android.content.Context

enum class SwipeAxis { HORIZONTAL, VERTICAL }

data class LauncherPrefs(
    val axis: SwipeAxis = SwipeAxis.HORIZONTAL,
    val invert: Boolean = false,
    val thresholdDp: Int = 64,
    val showClock: Boolean = true,
    val showBattery: Boolean = true,
    val showRecents: Boolean = true,
    val showSync: Boolean = true,
    val clock24h: Boolean = true,
    val serverUrl: String = "",
    val token: String = "",
    val themeJson: String? = null,
)

class Prefs(context: Context) {
    private val sp = context.getSharedPreferences("noct_launcher", Context.MODE_PRIVATE)

    fun load(): LauncherPrefs = LauncherPrefs(
        axis = runCatching { SwipeAxis.valueOf(sp.getString("axis", "HORIZONTAL")!!) }
            .getOrDefault(SwipeAxis.HORIZONTAL),
        invert = sp.getBoolean("invert", false),
        thresholdDp = sp.getInt("thresholdDp", 64),
        showClock = sp.getBoolean("showClock", true),
        showBattery = sp.getBoolean("showBattery", true),
        showRecents = sp.getBoolean("showRecents", true),
        showSync = sp.getBoolean("showSync", true),
        clock24h = sp.getBoolean("clock24h", true),
        serverUrl = sp.getString("serverUrl", "") ?: "",
        token = sp.getString("token", "") ?: "",
        themeJson = sp.getString("themeJson", null),
    )

    fun save(prefs: LauncherPrefs) {
        sp.edit()
            .putString("axis", prefs.axis.name)
            .putBoolean("invert", prefs.invert)
            .putInt("thresholdDp", prefs.thresholdDp)
            .putBoolean("showClock", prefs.showClock)
            .putBoolean("showBattery", prefs.showBattery)
            .putBoolean("showRecents", prefs.showRecents)
            .putBoolean("showSync", prefs.showSync)
            .putBoolean("clock24h", prefs.clock24h)
            .putString("serverUrl", prefs.serverUrl)
            .putString("token", prefs.token)
            .putString("themeJson", prefs.themeJson)
            .apply()
    }
}
