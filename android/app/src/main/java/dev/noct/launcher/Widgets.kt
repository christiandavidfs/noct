package dev.noct.launcher

import android.appwidget.AppWidgetHost
import android.appwidget.AppWidgetManager
import android.appwidget.AppWidgetProviderInfo
import android.content.ComponentName
import android.content.Context
import android.os.Bundle
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.viewinterop.AndroidView
import org.json.JSONArray
import org.json.JSONObject

data class HostedWidget(
    val id: Int,
    val provider: String,
    val heightDp: Int = 0,
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
                            heightDp = item.optInt("height", 0),
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
                    .put("height", widget.heightDp)
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

/**
 * Widget real con altura redimensionable.
 *
 * En modo edición aparece un asa en la parte inferior: arrastrar cambia el
 * alto y, al soltar, se persiste y se avisa al proveedor con
 * `updateAppWidgetSize`/`updateAppWidgetOptions` (mismo mecanismo que usan los
 * launchers de AOSP).
 */
@Composable
fun HostedWidgetView(
    host: AppWidgetHost,
    widget: HostedWidget,
    modifier: Modifier = Modifier,
    editMode: Boolean = false,
    onResize: (Int) -> Unit = {},
) {
    val context = LocalContext.current
    val manager = remember { AppWidgetManager.getInstance(context) }
    val info = remember(widget.id) { manager.getAppWidgetInfo(widget.id) }
    if (info == null) {
        Text(
            text = "Widget no disponible",
            modifier = modifier.fillMaxWidth(),
        )
        return
    }

    val density = LocalDensity.current
    var widgetHeight by remember(widget.id) {
        mutableIntStateOf(
            widget.heightDp.takeIf { it > 0 }
                ?: info.minHeight.coerceAtLeast(90)
        )
    }
    val heightPx = with(density) { widgetHeight.dp.roundToPx() }

    Column(modifier) {
        AndroidView(
            factory = { host.createView(context, widget.id, info) },
            modifier = Modifier
                .fillMaxWidth()
                .height(widgetHeight.dp),
            update = { view ->
                runCatching {
                    view.updateAppWidgetSize(null, heightPx, heightPx, heightPx, heightPx)
                }
                runCatching {
                    manager.updateAppWidgetOptions(
                        widget.id,
                        Bundle().apply {
                            putInt(AppWidgetManager.OPTION_APPWIDGET_MIN_WIDTH, heightPx)
                            putInt(AppWidgetManager.OPTION_APPWIDGET_MAX_WIDTH, heightPx)
                            putInt(AppWidgetManager.OPTION_APPWIDGET_MIN_HEIGHT, heightPx)
                            putInt(AppWidgetManager.OPTION_APPWIDGET_MAX_HEIGHT, heightPx)
                        },
                    )
                }
            },
        )

        if (editMode) {
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(26.dp)
                    .pointerInput(widget.id) {
                        detectDragGestures(
                            onDragEnd = { onResize(widgetHeight) },
                        ) { change, drag ->
                            change.consume()
                            val delta = (drag.y / density.density).toInt()
                            widgetHeight = (widgetHeight + delta).coerceIn(70, 1200)
                        }
                    },
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    text = "↕ redimensionar",
                    fontSize = 11.sp,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }
}
