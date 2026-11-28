package dev.noct.launcher

import android.app.ActivityManager
import android.appwidget.AppWidgetHost
import androidx.activity.compose.BackHandler
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.os.BatteryManager
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Close
import androidx.compose.material.icons.outlined.KeyboardArrowDown
import androidx.compose.material.icons.outlined.KeyboardArrowUp
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.core.content.ContextCompat
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import java.time.LocalDateTime
import java.time.format.DateTimeFormatter
import java.util.Locale
import kotlin.math.roundToInt

@Composable
fun HomeScreen(
    prefs: LauncherPrefs,
    recents: List<AppEntry>,
    hasUsageAccess: Boolean,
    themeName: String?,
    widgetHost: AppWidgetHost,
    hostedWidgets: List<HostedWidget>,
    editMode: Boolean,
    onEnterEdit: () -> Unit,
    onExitEdit: () -> Unit,
    onMoveSection: (String, Int) -> Unit,
    onDismissRecent: (AppEntry) -> Unit,
    onOpenDrawer: () -> Unit,
    onOpenSettings: () -> Unit,
    onOpenSync: () -> Unit,
    onRequestUsageAccess: () -> Unit,
    onResizeWidget: (Int, Int) -> Unit,
) {
    val battery = rememberBattery()
    val context = LocalContext.current
    var dismissed by remember { mutableStateOf<Set<String>>(emptySet()) }
    val visibleRecents = recents.filterNot { it.packageName in dismissed }

    fun closeRecent(app: AppEntry) {
        dismissed = dismissed + app.packageName
        onDismissRecent(app)
        runCatching {
            val manager = context.getSystemService(android.content.Context.ACTIVITY_SERVICE) as ActivityManager
            manager.killBackgroundProcesses(app.packageName)
        }
    }

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(
                Brush.verticalGradient(
                    listOf(
                        MaterialTheme.colorScheme.surface.copy(alpha = 0.30f),
                        MaterialTheme.colorScheme.background.copy(alpha = 0.62f),
                    )
                )
            )
    ) {
        LazyColumn(
            modifier = Modifier
                .fillMaxSize()
                .statusBarsPadding(),
            contentPadding = PaddingValues(
                start = 26.dp,
                end = 26.dp,
                top = 18.dp,
                bottom = 92.dp,
            ),
            verticalArrangement = Arrangement.spacedBy(18.dp),
        ) {
            items(prefs.homeOrder, key = { it }) { section ->
                val index = prefs.homeOrder.indexOf(section)
                val canUp = index > 0
                val canDown = index in 0 until prefs.homeOrder.lastIndex
                when (section) {
                    "clock" -> if (prefs.showClock) {
                        EditableSection(
                            title = "Reloj",
                            section = section,
                            editing = editMode,
                            canUp = canUp,
                            canDown = canDown,
                            onMove = onMoveSection,
                        ) {
                            ClockWidget(
                                clock24h = prefs.clock24h,
                                onTap = onOpenSettings,
                                onLongPress = onEnterEdit,
                            )
                        }
                    }

                    "battery" -> if (prefs.showBattery) {
                        EditableSection(
                            title = "Batería",
                            section = section,
                            editing = editMode,
                            canUp = canUp,
                            canDown = canDown,
                            onMove = onMoveSection,
                        ) {
                            BatteryWidget(battery)
                        }
                    }

                    "recents" -> if (prefs.showRecents) {
                        EditableSection(
                            title = "Apps activas",
                            section = section,
                            editing = editMode,
                            canUp = canUp,
                            canDown = canDown,
                            onMove = onMoveSection,
                        ) {
                            RecentsCarousel(
                                recents = visibleRecents,
                                hasUsageAccess = hasUsageAccess,
                                onRequestUsageAccess = onRequestUsageAccess,
                                onOpenDrawer = onOpenDrawer,
                                onClose = { closeRecent(it) },
                            )
                        }
                    }

                    "sync" -> if (prefs.showSync) {
                        EditableSection(
                            title = "Tema",
                            section = section,
                            editing = editMode,
                            canUp = canUp,
                            canDown = canDown,
                            onMove = onMoveSection,
                        ) {
                            SyncWidget(themeName = themeName, onOpenSync = onOpenSync)
                        }
                    }
                    else -> if (section.startsWith("widget:")) {
                        val id = section.removePrefix("widget:").toIntOrNull()
                        val widget = hostedWidgets.firstOrNull { it.id == id }
                        if (widget != null) {
                            EditableSection(
                                title = widgetLabel(context, widget.provider),
                                section = section,
                                editing = editMode,
                                canUp = canUp,
                                canDown = canDown,
                                onMove = onMoveSection,
                            ) {
                                HostedWidgetView(
                                    host = widgetHost,
                                    widget = widget,
                                    editMode = editMode,
                                    onResize = { height -> onResizeWidget(widget.id, height) },
                                    modifier = Modifier
                                        .fillMaxWidth()
                                        .padding(vertical = 2.dp),
                                )
                            }
                        }
                    }

                }
            }
        }

        Row(
            modifier = Modifier
                .align(Alignment.BottomCenter)
                .fillMaxWidth()
                .navigationBarsPadding()
                .padding(horizontal = 24.dp, vertical = 14.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween,
        ) {
            Text(
                text = "apps",
                fontSize = 10.sp,
                color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.45f),
                modifier = Modifier.clickable(onClick = onOpenDrawer),
            )
            Box(
                modifier = Modifier
                    .width(44.dp)
                    .height(4.dp)
                    .clip(CircleShape)
                    .background(MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.35f))
                    .clickable(onClick = onOpenDrawer)
            )
            Text(
                text = if (editMode) "listo" else "ajustes",
                fontSize = 10.sp,
                color = if (editMode) {
                    MaterialTheme.colorScheme.primary
                } else {
                    MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.45f)
                },
                modifier = Modifier.clickable(onClick = if (editMode) onExitEdit else onOpenSettings),
            )
        }
    }
}

@Composable
private fun EditableSection(
    title: String,
    section: String,
    editing: Boolean,
    canUp: Boolean,
    canDown: Boolean,
    onMove: (String, Int) -> Unit,
    content: @Composable () -> Unit,
) {
    Column(
        modifier = if (editing) {
            Modifier
                .fillMaxWidth()
                .clip(RoundedCornerShape(18.dp))
                .background(MaterialTheme.colorScheme.surface.copy(alpha = 0.16f))
                .padding(10.dp)
        } else {
            Modifier
        },
    ) {
        if (editing) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    text = title.uppercase(),
                    fontSize = 10.sp,
                    letterSpacing = 1.5.sp,
                    color = MaterialTheme.colorScheme.primary,
                )
                Row {
                    IconButton(
                        onClick = { onMove(section, -1) },
                        enabled = canUp,
                        modifier = Modifier.size(30.dp),
                    ) {
                        Icon(Icons.Outlined.KeyboardArrowUp, contentDescription = "Subir")
                    }
                    IconButton(
                        onClick = { onMove(section, 1) },
                        enabled = canDown,
                        modifier = Modifier.size(30.dp),
                    ) {
                        Icon(Icons.Outlined.KeyboardArrowDown, contentDescription = "Bajar")
                    }
                }
            }
        }
        content()
    }
}

@Composable
private fun ClockWidget(
    clock24h: Boolean,
    onTap: () -> Unit,
    onLongPress: () -> Unit,
) {
    var now by remember { mutableStateOf(LocalDateTime.now()) }
    LaunchedEffect(Unit) {
        while (true) {
            now = LocalDateTime.now()
            delay(1000)
        }
    }
    val pattern = if (clock24h) "HH:mm" else "h:mm a"
    val time = now.format(DateTimeFormatter.ofPattern(pattern, Locale.getDefault()))
    val date = now.format(DateTimeFormatter.ofPattern("EEEE d MMMM", Locale.getDefault()))

    Column(
        modifier = Modifier
            .fillMaxWidth()
            .pointerInput(Unit) {
                detectTapGestures(onTap = { onTap() }, onLongPress = { onLongPress() })
            },
    ) {
        Text(
            text = time,
            fontSize = 76.sp,
            fontWeight = FontWeight.Thin,
            color = MaterialTheme.colorScheme.onSurface,
            lineHeight = 80.sp,
        )
        Text(
            text = date.replaceFirstChar { it.uppercase() },
            fontSize = 15.sp,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}

private data class BatteryInfo(val level: Int, val charging: Boolean)

@Composable
private fun rememberBattery(): BatteryInfo {
    val context = LocalContext.current
    var info by remember { mutableStateOf(BatteryInfo(0, false)) }
    DisposableEffect(context) {
        val receiver = object : BroadcastReceiver() {
            override fun onReceive(context: Context?, intent: Intent?) {
                if (intent == null) return
                val level = intent.getIntExtra(BatteryManager.EXTRA_LEVEL, 0)
                val scale = intent.getIntExtra(BatteryManager.EXTRA_SCALE, 100).coerceAtLeast(1)
                val status = intent.getIntExtra(BatteryManager.EXTRA_STATUS, -1)
                info = BatteryInfo(
                    level = (level * 100f / scale).roundToInt(),
                    charging = status == BatteryManager.BATTERY_STATUS_CHARGING ||
                        status == BatteryManager.BATTERY_STATUS_FULL,
                )
            }
        }
        val filter = IntentFilter(Intent.ACTION_BATTERY_CHANGED)
        val sticky = ContextCompat.registerReceiver(
            context,
            receiver,
            filter,
            ContextCompat.RECEIVER_NOT_EXPORTED,
        )
        sticky?.let { intent ->
            val level = intent.getIntExtra(BatteryManager.EXTRA_LEVEL, 0)
            val scale = intent.getIntExtra(BatteryManager.EXTRA_SCALE, 100).coerceAtLeast(1)
            val status = intent.getIntExtra(BatteryManager.EXTRA_STATUS, -1)
            info = BatteryInfo(
                level = (level * 100f / scale).roundToInt(),
                charging = status == BatteryManager.BATTERY_STATUS_CHARGING ||
                    status == BatteryManager.BATTERY_STATUS_FULL,
            )
        }
        onDispose { context.unregisterReceiver(receiver) }
    }
    return info
}

@Composable
private fun BatteryWidget(battery: BatteryInfo) {
    Row(verticalAlignment = Alignment.CenterVertically) {
        Text(
            text = "${battery.level}%",
            fontSize = 13.sp,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        if (battery.charging) {
            Spacer(Modifier.width(8.dp))
            Text(
                text = "cargando",
                fontSize = 11.sp,
                color = MaterialTheme.colorScheme.primary,
            )
        }
        Spacer(Modifier.width(12.dp))
        Box(
            modifier = Modifier
                .width(110.dp)
                .height(2.dp)
                .clip(CircleShape)
                .background(MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.18f))
        ) {
            Box(
                modifier = Modifier
                    .fillMaxWidth(battery.level / 100f)
                    .height(2.dp)
                    .background(MaterialTheme.colorScheme.primary.copy(alpha = 0.8f))
            )
        }
    }
}

@Composable
private fun RecentsCarousel(
    recents: List<AppEntry>,
    hasUsageAccess: Boolean,
    onRequestUsageAccess: () -> Unit,
    onOpenDrawer: () -> Unit,
    onClose: (AppEntry) -> Unit,
) {
    val context = LocalContext.current
    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(
                text = "ACTIVAS",
                fontSize = 10.sp,
                letterSpacing = 1.5.sp,
                color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f),
            )
            Text(
                text = if (recents.isEmpty()) "" else "aprox · desliza · toca",
                fontSize = 10.sp,
                color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.45f),
            )
        }
        if (recents.isEmpty()) {
            Text(
                text = if (hasUsageAccess) "Sin apps recientes" else "Sin acceso de uso",
                fontSize = 17.sp,
                color = MaterialTheme.colorScheme.onSurface,
            )
            Text(
                text = if (hasUsageAccess) "Abre una app y vuelve" else "Toca para dar acceso",
                fontSize = 11.sp,
                color = MaterialTheme.colorScheme.primary.copy(alpha = 0.85f),
                modifier = Modifier.clickable {
                    if (hasUsageAccess) onOpenDrawer() else onRequestUsageAccess()
                },
            )
        } else {
            val virtualPages = 10000
            val pagerState = rememberPagerState(initialPage = virtualPages / 2) {
                virtualPages
            }
            HorizontalPager(
                state = pagerState,
                pageSpacing = 12.dp,
                modifier = Modifier
                    .fillMaxWidth()
                    .height(164.dp),
            ) { page ->
                val app = recents[page % recents.size]
                Box(
                    modifier = Modifier
                        .fillMaxSize()
                        .clip(RoundedCornerShape(24.dp))
                        .background(MaterialTheme.colorScheme.surface.copy(alpha = 0.30f)),
                ) {
                    Column(
                        modifier = Modifier
                            .fillMaxSize()
                            .clickable { AppRepository.launch(context, app) }
                            .padding(horizontal = 20.dp, vertical = 18.dp),
                        verticalArrangement = Arrangement.SpaceBetween,
                    ) {
                        Text(
                            text = app.label,
                            fontSize = 30.sp,
                            lineHeight = 34.sp,
                            color = MaterialTheme.colorScheme.onSurface,
                            modifier = Modifier.padding(end = 36.dp),
                        )
                        Text(
                            text = "${(pagerState.currentPage % recents.size) + 1} / ${recents.size}",
                            fontSize = 11.sp,
                            color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.55f),
                        )
                    }
                    IconButton(
                        onClick = { onClose(app) },
                        modifier = Modifier
                            .align(Alignment.TopEnd)
                            .padding(4.dp),
                    ) {
                        Icon(
                            Icons.Outlined.Close,
                            contentDescription = "Cerrar app",
                            tint = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.65f),
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun SyncWidget(themeName: String?, onOpenSync: () -> Unit) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(18.dp))
            .background(MaterialTheme.colorScheme.surface.copy(alpha = 0.28f))
            .clickable(onClick = onOpenSync)
            .padding(horizontal = 14.dp, vertical = 10.dp),
    ) {
        Text(
            text = "TEMA",
            fontSize = 10.sp,
            letterSpacing = 1.5.sp,
            color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f),
        )
        Text(
            text = themeName ?: "Sin sincronizar",
            fontSize = 15.sp,
            color = MaterialTheme.colorScheme.onSurface,
        )
        Text(
            text = if (themeName == null) "Toca para sincronizar" else "Sincronizado con el escritorio",
            fontSize = 10.sp,
            color = MaterialTheme.colorScheme.primary.copy(alpha = 0.8f),
        )
    }
}
