package dev.noct.launcher

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.BackHandler
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.compose.LifecycleEventEffect
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

object AppCache {
    @Volatile
    var apps: List<AppEntry>? = null
}

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        enableEdgeToEdge()
        super.onCreate(savedInstanceState)
        setContent {
            NoctLauncherApp()
        }
    }
}

@Composable
private fun NoctLauncherApp() {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    val store = remember { Prefs(context) }
    var prefs by remember { mutableStateOf(store.load()) }
    var apps by remember { mutableStateOf<List<AppEntry>>(emptyList()) }
    var recents by remember { mutableStateOf<List<AppEntry>>(emptyList()) }
    var hasUsageAccess by remember { mutableStateOf(UsageAccess.hasPermission(context)) }
    var loading by remember { mutableStateOf(true) }
    var screen by rememberSaveable { mutableStateOf("home") }

    fun refreshRecents() {
        hasUsageAccess = UsageAccess.hasPermission(context)
        if (apps.isEmpty()) return
        scope.launch {
            val list = withContext(Dispatchers.IO) { RecentApps.load(context, apps) }
            recents = list
        }
    }

    LaunchedEffect(Unit) {
        val cached = AppCache.apps
        if (cached != null) {
            apps = cached
        } else {
            apps = AppRepository.load(context)
            AppCache.apps = apps
        }
        loading = false
        refreshRecents()
    }

    LifecycleEventEffect(Lifecycle.Event.ON_RESUME) {
        hasUsageAccess = UsageAccess.hasPermission(context)
        refreshRecents()
    }

    val palette = remember(prefs.themeJson) { parseNoctTheme(prefs.themeJson) }
    val scheme = palette?.toColorScheme() ?: NoctDefaultDark

    BackHandler(enabled = screen != "home") {
        screen = when (screen) {
            "sync" -> "settings"
            else -> "home"
        }
    }

    MaterialTheme(colorScheme = scheme) {
        Surface(modifier = Modifier.fillMaxSize(), color = Color.Transparent) {
            when (screen) {
                "drawer" -> DrawerScreen(
                    apps = apps,
                    onBack = { screen = "home" },
                    onLaunch = { app ->
                        screen = "home"
                        AppRepository.launch(context, app.packageName)
                    },
                )

                "settings" -> SettingsScreen(
                    prefs = prefs,
                    hasUsageAccess = hasUsageAccess,
                    onChange = {
                        prefs = it
                        store.save(it)
                    },
                    onBack = { screen = "home" },
                    onOpenSync = { screen = "sync" },
                    onRequestUsageAccess = {
                        UsageAccess.openSettings(context)
                    },
                )

                "sync" -> SyncScreen(
                    prefs = prefs,
                    onBack = { screen = "settings" },
                    onSynced = {
                        prefs = it
                        store.save(it)
                    },
                )

                else -> Box(modifier = Modifier.fillMaxSize()) {
                    HomeScreen(
                        prefs = prefs,
                        recents = recents,
                        hasUsageAccess = hasUsageAccess,
                        themeName = palette?.name,
                        onOpenDrawer = { screen = "drawer" },
                        onOpenSettings = { screen = "settings" },
                        onOpenSync = { screen = "sync" },
                        onRequestUsageAccess = { UsageAccess.openSettings(context) },
                    )
                    if (loading) {
                        CircularProgressIndicator(
                            modifier = Modifier.align(Alignment.Center),
                            color = MaterialTheme.colorScheme.primary,
                        )
                    }
                }
            }
        }
    }
}
