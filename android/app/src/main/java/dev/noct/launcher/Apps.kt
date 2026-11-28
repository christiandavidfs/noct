package dev.noct.launcher

import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.pm.LauncherApps
import android.os.Process
import android.os.UserHandle
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asImageBitmap
import androidx.core.graphics.drawable.toBitmap
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

data class AppEntry(
    val packageName: String,
    val label: String,
    val icon: ImageBitmap?,
    val component: String,
    val user: UserHandle?,
    val profileLabel: String? = null,
) {
    val key: String
        get() = "$component@${user?.hashCode() ?: 0}"
}

object AppRepository {
    suspend fun load(context: Context): List<AppEntry> = withContext(Dispatchers.IO) {
        val launcherApps = context.getSystemService(LauncherApps::class.java)
        val currentUser = Process.myUserHandle()
        val profiles = runCatching { launcherApps.profiles }
            .getOrNull()
            ?: listOf(currentUser)
        val density = context.resources.displayMetrics.densityDpi

        val entries = mutableListOf<AppEntry>()
        for (profile in profiles) {
            val activities = runCatching {
                launcherApps.getActivityList(null, profile)
            }.getOrNull() ?: continue
            for (info in activities) {
                val packageName = info.applicationInfo.packageName
                if (packageName == context.packageName) continue
                val rawLabel = info.label?.toString() ?: packageName
                val userId = profile.hashCode()
                val profileLabel = if (profile == currentUser) {
                    null
                } else {
                    when (userId) {
                        10 -> context.getString(R.string.profile_10)
                        11 -> context.getString(R.string.profile_clone)
                        else -> context.getString(R.string.profile_number, userId)
                    }
                }
                val label = if (profileLabel != null) "$rawLabel · $profileLabel" else rawLabel
                val icon = runCatching {
                    info.getBadgedIcon(density)
                        .toBitmap(width = 120, height = 120)
                        .asImageBitmap()
                }.getOrNull()
                entries += AppEntry(
                    packageName = packageName,
                    label = label,
                    icon = icon,
                    component = info.componentName.flattenToString(),
                    user = profile,
                    profileLabel = profileLabel,
                )
            }
        }

        entries
            .distinctBy { it.key }
            .sortedBy { it.label.lowercase() }
    }

    fun launch(context: Context, app: AppEntry): Boolean {
        val component = ComponentName.unflattenFromString(app.component)
        val user = app.user
        if (component != null && user != null) {
            val launcherApps = context.getSystemService(LauncherApps::class.java)
            val started = runCatching {
                launcherApps.startMainActivity(component, user, null, null)
            }.isSuccess
            if (started) return true
        }

        val packageManager = context.packageManager
        val intent = packageManager.getLaunchIntentForPackage(app.packageName)
            ?: Intent(Intent.ACTION_MAIN)
                .addCategory(Intent.CATEGORY_LAUNCHER)
                .setPackage(app.packageName)
        intent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_RESET_TASK_IF_NEEDED)
        return runCatching { context.startActivity(intent) }.isSuccess
    }
}
