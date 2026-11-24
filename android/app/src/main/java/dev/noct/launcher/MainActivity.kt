package dev.noct.launcher

import android.appwidget.AppWidgetHost
import android.appwidget.AppWidgetManager
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
import androidx.compose.runtime.DisposableEffect
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
import androidx.lifecycle.repeatOnLifecycle
import androidx.compose.ui.platform.LocalLifecycleOwner
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
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

private fun normalizeHomeOrder(order: List<String>, widgets: List<HostedWidget>): List<String> {
    val baseSections = listOf("clock", "battery", "recents", "sync")
    val widgetKeys = widgets.map { "widget:${it.id}" }
    val oldGroupIndex = order.indexOf("widgets")
    val kept = order
        .filter { it != "widgets" && (it in baseSections || it in widgetKeys) }
        .toMutableList()

    baseSections.forEach { section ->
        if (section !in kept) kept.add(section)
    }

    val missing = widgetKeys.filter { it !in kept }
    missing.forEachIndexed { index, key ->
        val position = if (oldGroupIndex >= 0) {
            (oldGroupIndex + index).coerceIn(0, kept.size)
        } else {
            kept.size
        }
        kept.add(position, key)
    }
    return kept.distinct()
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
    var editHome by rememberSaveable { mutableStateOf(false) }

    fun moveSection(section: String, delta: Int) {
        val order = prefs.homeOrder.toMutableList()
        val index = order.indexOf(section)
        if (index < 0) return
        val target = (index + delta).coerceIn(0, order.lastIndex)
        if (target == index) return
        order.removeAt(index)
        order.add(target, section)
        prefs = prefs.copy(homeOrder = order)
        store.save(prefs)
    }

    val widgetHost = remember { AppWidgetHost(context, 1024) }
    DisposableEffect(widgetHost) {
        widgetHost.startListening()
        onDispose { widgetHost.stopListening() }
    }
    val widgetStore = remember { WidgetStore(context) }
    var hostedWidgets by remember { mutableStateOf(widgetStore.load()) }

    fun saveWidgets(list: List<HostedWidget>) {
        hostedWidgets = list
        widgetStore.save(list)
    }

    LaunchedEffect(Unit) {
        val manager = AppWidgetManager.getInstance(context)
        val valid = hostedWidgets.filter { manager.getAppWidgetInfo(it.id) != null }
        if (valid.size != hostedWidgets.size) {
            hostedWidgets.filterNot { it in valid }.forEach {
                runCatching { widgetHost.deleteAppWidgetId(it.id) }
            }
            saveWidgets(valid)
        }
        val normalized = normalizeHomeOrder(prefs.homeOrder, valid)
        if (normalized != prefs.homeOrder) {
            prefs = prefs.copy(homeOrder = normalized)
            store.save(prefs)
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
    }

    val lifecycleOwner = LocalLifecycleOwner.current
    LaunchedEffect(lifecycleOwner, apps, prefs.dismissed) {
        if (apps.isEmpty()) return@LaunchedEffect
        lifecycleOwner.repeatOnLifecycle(Lifecycle.State.RESUMED) {
            while (true) {
                hasUsageAccess = UsageAccess.hasPermission(context)
                // UsageStats tarda un poco en registrar el cambio de app.
                delay(250)
                val loaded = withContext(Dispatchers.IO) { RecentApps.load(context, apps) }
                val dismissed = prefs.dismissed
                val visible = loaded.filter { recent ->
                    val dismissedAt = dismissed[recent.app.packageName]
                    dismissedAt == null || recent.lastUsed > dismissedAt
                }
                val active = dismissed.filter { (pkg, at) ->
                    loaded.any { it.app.packageName == pkg && it.lastUsed > at }
                }
                if (active != dismissed) {
                    prefs = prefs.copy(dismissed = active)
                    store.save(prefs)
                }
                recents = visible.map { it.app }
                delay(2500)
            }
        }
    }

    fun dismissRecent(app: AppEntry) {
        prefs = prefs.copy(
            dismissed = prefs.dismissed + (app.packageName to System.currentTimeMillis()),
        )
        store.save(prefs)
    }

    val palette = remember(prefs.themeJson) { parseNoctTheme(prefs.themeJson) }
    val scheme = palette?.toColorScheme() ?: NoctDefaultDark

    BackHandler(enabled = screen != "home" || editHome) {
        if (editHome) {
            editHome = false
        } else {
            screen = when (screen) {
                "widgetPicker" -> "widgets"
                "widgets" -> "settings"
                "sync" -> "settings"
                else -> "home"
            }
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
                        AppRepository.launch(context, app)
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
                    onOpenWidgets = { screen = "widgets" },
                    onOpenEditHome = {
                        editHome = true
                        screen = "home"
                    },
                    onRequestUsageAccess = {
                        UsageAccess.openSettings(context)
                    },
                )

                "widgets" -> WidgetsScreen(
                    hosted = hostedWidgets,
                    onAdd = { screen = "widgetPicker" },
                    onRemove = { widget ->
                        runCatching { widgetHost.deleteAppWidgetId(widget.id) }
                        val widgets = hostedWidgets.filterNot { it.id == widget.id }
                        saveWidgets(widgets)
                        val order = normalizeHomeOrder(prefs.homeOrder, widgets)
                        prefs = prefs.copy(homeOrder = order)
                        store.save(prefs)
                    },
                    onBack = { screen = "settings" },
                )

                "widgetPicker" -> WidgetPickerScreen(
                    host = widgetHost,
                    onAdded = { widget ->
                        val widgets = hostedWidgets + widget
                        saveWidgets(widgets)
                        val order = normalizeHomeOrder(prefs.homeOrder, widgets)
                        prefs = prefs.copy(homeOrder = order)
                        store.save(prefs)
                        screen = "widgets"
                    },
                    onBack = { screen = "widgets" },
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
                        widgetHost = widgetHost,
                        hostedWidgets = hostedWidgets,
                        editMode = editHome,
                        onEnterEdit = { editHome = true },
                        onExitEdit = { editHome = false },
                        onMoveSection = { section, delta -> moveSection(section, delta) },
                        onDismissRecent = { dismissRecent(it) },
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
