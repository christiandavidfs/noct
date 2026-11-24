package dev.noct.launcher

import android.appwidget.AppWidgetHost
import android.appwidget.AppWidgetManager
import android.appwidget.AppWidgetProviderInfo
import android.content.ComponentName
import android.content.Context
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import androidx.compose.ui.viewinterop.AndroidView
import org.json.JSONArray
import org.json.JSONObject

data class HostedWidget(
    val id: Int,
    val provider: String,
)

data class WidgetProvider(
    val info: AppWidgetProviderInfo,
    val label: String,
)

class WidgetStore(context: Context) {
    private val sp = context.getSharedPreferences("noct_widgets", Context.MODE_PRIVATE)

    fun load(): List<HostedWidget> {
        val raw = sp.getString("hosted", null) ?: return emptyList()
        return runCatching {
            val array = JSONArray(raw)
            buildList {
                for (index in 0 until array.length()) {
                    val item = array.getJSONObject(index)
                    add(
                        HostedWidget(
                            id = item.getInt("id"),
                            provider = item.getString("provider"),
                        )
                    )
                }
            }
        }.getOrDefault(emptyList())
    }

    fun save(widgets: List<HostedWidget>) {
        val array = JSONArray()
        widgets.forEach { widget ->
            array.put(
                JSONObject()
                    .put("id", widget.id)
                    .put("provider", widget.provider)
            )
        }
        sp.edit().putString("hosted", array.toString()).apply()
    }
}

fun installedWidgetProviders(context: Context): List<WidgetProvider> {
    val manager = AppWidgetManager.getInstance(context)
    val packageManager = context.packageManager
    return manager.installedProviders
        .map { info ->
            val label = runCatching { info.loadLabel(packageManager) }
                .getOrDefault(info.provider.className)
            WidgetProvider(info, label)
        }
        .sortedBy { it.label.lowercase() }
}

fun widgetLabel(context: Context, provider: String): String {
    val component = ComponentName.unflattenFromString(provider) ?: return provider
    val manager = AppWidgetManager.getInstance(context)
    val info = manager.installedProviders.firstOrNull { it.provider == component }
    return info?.let { runCatching { it.loadLabel(context.packageManager) }.getOrNull() }
        ?: component.className
}

@Composable
fun HostedWidgetView(
    host: AppWidgetHost,
    widget: HostedWidget,
    modifier: Modifier = Modifier,
) {
    val context = LocalContext.current
    val manager = remember { AppWidgetManager.getInstance(context) }
    val info = remember(widget.id) { manager.getAppWidgetInfo(widget.id) }
    if (info == null) {
        androidx.compose.material3.Text(
            text = "Widget no disponible",
            modifier = modifier.fillMaxWidth(),
        )
        return
    }
    AndroidView(
        factory = { host.createView(context, widget.id, info) },
        modifier = modifier
            .fillMaxWidth()
            .heightIn(min = info.minHeight.dp.coerceAtLeast(70.dp)),
    )
}
