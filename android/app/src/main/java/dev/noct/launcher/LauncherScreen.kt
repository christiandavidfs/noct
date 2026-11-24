package dev.noct.launcher

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.os.BatteryManager
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
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
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
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
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.PointerEventPass
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.input.pointer.positionChange
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.core.content.ContextCompat
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.time.LocalDateTime
import java.time.format.DateTimeFormatter
import java.util.Locale
import kotlin.math.abs
import kotlin.math.max
import kotlin.math.roundToInt

@Composable
fun HomeScreen(
    prefs: LauncherPrefs,
    recents: List<AppEntry>,
    hasUsageAccess: Boolean,
    themeName: String?,
    onOpenDrawer: () -> Unit,
    onOpenSettings: () -> Unit,
    onOpenSync: () -> Unit,
    onRequestUsageAccess: () -> Unit,
) {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    val thresholdPx = with(LocalDensity.current) { prefs.thresholdDp.dp.toPx() }
    var recentIndex by rememberSaveable { mutableIntStateOf(0) }
    var pendingName by remember { mutableStateOf<String?>(null) }
    var switching by remember { mutableStateOf(false) }
    val battery = rememberBattery()

    fun switchRecent(direction: Int) {
        if (switching) return
        if (recents.isEmpty()) {
            onOpenDrawer()
            return
        }
        val count = recents.size
        val next = Math.floorMod(recentIndex + direction, count)
        recentIndex = next
        val app = recents[next]
        pendingName = app.label
        switching = true
        scope.launch {
            delay(280)
            AppRepository.launch(context, app.packageName)
            pendingName = null
            switching = false
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
            .homeGestures(
                axis = prefs.axis,
                invert = prefs.invert,
                thresholdPx = thresholdPx,
                onSwitchRecent = { switchRecent(it) },
                onOpenDrawer = onOpenDrawer,
            )
    ) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .statusBarsPadding()
                .navigationBarsPadding()
                .padding(horizontal = 26.dp, vertical = 18.dp),
            verticalArrangement = Arrangement.SpaceBetween,
        ) {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .clickable(onClick = onOpenSettings),
            ) {
                if (prefs.showClock) {
                    ClockWidget(clock24h = prefs.clock24h)
                }
                if (prefs.showBattery) {
                    Spacer(Modifier.height(10.dp))
                    BatteryWidget(battery)
                }
            }

            Column(
                modifier = Modifier.fillMaxWidth(),
                verticalArrangement = Arrangement.spacedBy(18.dp),
            ) {
                if (prefs.showRecents) {
                    RecentsWidget(
                        recents = recents,
                        hasUsageAccess = hasUsageAccess,
                        onRequestUsageAccess = onRequestUsageAccess,
                        onOpenDrawer = onOpenDrawer,
                    )
                }
                if (prefs.showSync) {
                    SyncWidget(themeName = themeName, onOpenSync = onOpenSync)
                }
            }

            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween,
            ) {
                Text(
                    text = "dos dedos · apps",
                    fontSize = 10.sp,
                    color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.45f),
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
                    text = "ajustes",
                    fontSize = 10.sp,
                    color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.45f),
                    modifier = Modifier.clickable(onClick = onOpenSettings),
                )
            }
        }

        AnimatedVisibility(
            visible = pendingName != null,
            enter = fadeIn(),
            exit = fadeOut(),
            modifier = Modifier
                .align(Alignment.TopCenter)
                .statusBarsPadding()
                .padding(top = 8.dp),
        ) {
            Box(
                modifier = Modifier
                    .clip(RoundedCornerShape(50))
                    .background(MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.78f))
                    .padding(horizontal = 18.dp, vertical = 8.dp)
            ) {
                Text(
                    text = "→ ${pendingName ?: ""}",
                    fontSize = 13.sp,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }
}

@Composable
private fun ClockWidget(clock24h: Boolean) {
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
private fun RecentsWidget(
    recents: List<AppEntry>,
    hasUsageAccess: Boolean,
    onRequestUsageAccess: () -> Unit,
    onOpenDrawer: () -> Unit,
) {
    val context = LocalContext.current
    Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
        Text(
            text = "EN MARCHA",
            fontSize = 10.sp,
            letterSpacing = 1.5.sp,
            color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f),
        )
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
            recents.take(5).forEachIndexed { index, app ->
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    modifier = Modifier
                        .fillMaxWidth()
                        .clickable { AppRepository.launch(context, app.packageName) }
                        .padding(vertical = 2.dp),
                ) {
                    Text(
                        text = "${index + 1}",
                        fontSize = 11.sp,
                        color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.45f),
                        modifier = Modifier.width(20.dp),
                    )
                    Text(
                        text = app.label,
                        fontSize = 18.sp,
                        color = MaterialTheme.colorScheme.onSurface,
                    )
                }
            }
            Text(
                text = "Dos dedos para cambiar",
                fontSize = 10.sp,
                color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.45f),
            )
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

private fun Modifier.homeGestures(
    axis: SwipeAxis,
    invert: Boolean,
    thresholdPx: Float,
    onSwitchRecent: (Int) -> Unit,
    onOpenDrawer: () -> Unit,
): Modifier = pointerInput(axis, invert, thresholdPx) {
    awaitEachGesture {
        awaitFirstDown(requireUnconsumed = false)
        var maxPointers = 1
        var dx = 0f
        var dy = 0f
        var pressed = true

        while (pressed) {
            val event = awaitPointerEvent(PointerEventPass.Initial)
            maxPointers = max(maxPointers, event.changes.count { it.pressed })
            val change = event.changes.firstOrNull { it.pressed }
            if (change != null) {
                dx += change.positionChange().x
                dy += change.positionChange().y
            }
            if (maxPointers >= 2) {
                event.changes.forEach { it.consume() }
            }
            pressed = event.changes.any { it.pressed }
        }

        if (maxPointers >= 2) {
            val delta = if (axis == SwipeAxis.HORIZONTAL) dx else dy
            if (abs(delta) >= thresholdPx) {
                var direction = if (delta < 0) 1 else -1
                if (invert) direction = -direction
                onSwitchRecent(direction)
            }
        } else if (dy < -thresholdPx && abs(dy) > abs(dx)) {
            onOpenDrawer()
        }
    }
}
