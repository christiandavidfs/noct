package dev.noct.launcher

import android.content.Intent
import android.net.Uri
import android.provider.Settings
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.ArrowBack
import androidx.compose.material3.Button
import androidx.compose.material3.FilterChip
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Slider
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import kotlinx.coroutines.delay
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlin.math.roundToInt

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsScreen(
    prefs: LauncherPrefs,
    hasUsageAccess: Boolean,
    onChange: (LauncherPrefs) -> Unit,
    onBack: () -> Unit,
    onOpenSync: () -> Unit,
    onOpenWidgets: () -> Unit,
    onOpenEditHome: () -> Unit,
    onRequestUsageAccess: () -> Unit,
) {
    Scaffold(
        containerColor = MaterialTheme.colorScheme.background.copy(alpha = 0.94f),
        topBar = {
            TopAppBar(
                title = { Text(stringResource(R.string.settings_title)) },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(
                            Icons.AutoMirrored.Outlined.ArrowBack,
                            contentDescription = stringResource(R.string.back),
                        )
                    }
                },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = MaterialTheme.colorScheme.background.copy(alpha = 0.0f),
                    titleContentColor = MaterialTheme.colorScheme.onBackground,
                    navigationIconContentColor = MaterialTheme.colorScheme.onBackground,
                ),
            )
        },
    ) { padding ->
        LazyColumn(
            modifier = Modifier
                .fillMaxWidth()
                .padding(padding),
            contentPadding = PaddingValues(horizontal = 24.dp, vertical = 12.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp),
        ) {
            item { SectionTitle(stringResource(R.string.settings_widgets_android)) }
            item {
                Button(onClick = onOpenWidgets, modifier = Modifier.fillMaxWidth()) {
                    Text(stringResource(R.string.settings_manage_widgets))
                }
                Text(
                    text = stringResource(R.string.settings_android_widgets_desc),
                    fontSize = 12.sp,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.padding(top = 6.dp),
                )
            }

            item { HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant) }

            item { SectionTitle(stringResource(R.string.settings_home_widgets)) }
            item {
                SwitchSetting(
                    title = stringResource(R.string.settings_clock),
                    description = stringResource(R.string.settings_clock_desc),
                    checked = prefs.showClock,
                    onChange = { onChange(prefs.copy(showClock = it)) },
                )
            }
            item {
                SwitchSetting(
                    title = stringResource(R.string.settings_battery),
                    description = stringResource(R.string.settings_battery_desc),
                    checked = prefs.showBattery,
                    onChange = { onChange(prefs.copy(showBattery = it)) },
                )
            }
            item {
                SwitchSetting(
                    title = stringResource(R.string.settings_recents),
                    description = stringResource(R.string.settings_recents_desc),
                    checked = prefs.showRecents,
                    onChange = { onChange(prefs.copy(showRecents = it)) },
                )
            }
            item {
                SwitchSetting(
                    title = stringResource(R.string.settings_synced_theme),
                    description = stringResource(R.string.settings_synced_theme_desc),
                    checked = prefs.showSync,
                    onChange = { onChange(prefs.copy(showSync = it)) },
                )
            }
            item {
                SwitchSetting(
                    title = stringResource(R.string.settings_clock_24h),
                    description = stringResource(R.string.settings_clock_24h_desc),
                    checked = prefs.clock24h,
                    onChange = { onChange(prefs.copy(clock24h = it)) },
                )
            }
            item {
                OutlinedButton(
                    onClick = onOpenEditHome,
                    modifier = Modifier.fillMaxWidth(),
                ) {
                    Text(stringResource(R.string.settings_edit_home))
                }
            }

            item { HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant) }

            item { SectionTitle(stringResource(R.string.settings_switch_apps)) }
            item {
                Text(
                    text = stringResource(R.string.settings_switch_apps_desc),
                    fontSize = 12.sp,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            item {
                SwitchSetting(
                    title = stringResource(R.string.settings_handle),
                    description = stringResource(R.string.settings_handle_desc),
                    checked = prefs.gestureHandle,
                    onChange = { onChange(prefs.copy(gestureHandle = it)) },
                )
            }
            item {
                Text(
                    text = stringResource(R.string.settings_size),
                    fontSize = 13.sp,
                    fontWeight = FontWeight.Bold,
                    color = MaterialTheme.colorScheme.primary,
                )
                Row(
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                    modifier = Modifier.padding(top = 6.dp),
                ) {
                    listOf(R.string.size_small to 0, R.string.size_medium to 1, R.string.size_large to 2).forEach { (label, value) ->
                        FilterChip(
                            selected = prefs.gestureSize == value,
                            onClick = { onChange(prefs.copy(gestureSize = value)) },
                            label = { Text(stringResource(label)) },
                        )
                    }
                }
            }
            item {
                Text(
                    text = stringResource(R.string.settings_color),
                    fontSize = 13.sp,
                    fontWeight = FontWeight.Bold,
                    color = MaterialTheme.colorScheme.primary,
                )
                Row(
                    horizontalArrangement = Arrangement.spacedBy(10.dp),
                    modifier = Modifier.padding(top = 8.dp),
                ) {
                    listOf("#7CFFB2", "#FFFFFF", "#FF6B6B", "#6BB5FF", "#FFC46B", "#C49BFF")
                        .forEach { hex ->
                            val selected = prefs.gestureColor == hex
                            Box(
                                modifier = Modifier
                                    .size(30.dp)
                                    .clip(CircleShape)
                                    .background(Color(android.graphics.Color.parseColor(hex)))
                                    .border(
                                        width = if (selected) 2.dp else 1.dp,
                                        color = if (selected) {
                                            MaterialTheme.colorScheme.onSurface
                                        } else {
                                            MaterialTheme.colorScheme.outlineVariant
                                        },
                                        shape = CircleShape,
                                    )
                                    .clickable { onChange(prefs.copy(gestureColor = hex)) }
                            )
                        }
                }
            }
            item {
                Text(
                    text = stringResource(R.string.settings_opacity, prefs.gestureOpacity),
                    fontSize = 13.sp,
                    fontWeight = FontWeight.Bold,
                    color = MaterialTheme.colorScheme.primary,
                )
                Slider(
                    value = prefs.gestureOpacity.toFloat(),
                    onValueChange = { onChange(prefs.copy(gestureOpacity = it.roundToInt())) },
                    valueRange = 20f..100f,
                )
            }
            item {
                Text(
                    text = stringResource(R.string.settings_shape),
                    fontSize = 13.sp,
                    fontWeight = FontWeight.Bold,
                    color = MaterialTheme.colorScheme.primary,
                )
                Row(
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                    modifier = Modifier.padding(top = 6.dp),
                ) {
                    listOf(R.string.shape_pill to 0, R.string.shape_circle to 1, R.string.shape_square to 2).forEach { (label, value) ->
                        FilterChip(
                            selected = prefs.gestureShape == value,
                            onClick = { onChange(prefs.copy(gestureShape = value)) },
                            label = { Text(stringResource(label)) },
                        )
                    }
                }
            }
            item { HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant) }

            item { SectionTitle(stringResource(R.string.settings_clicks)) }
            item {
                TapActionRow(
                    title = stringResource(R.string.settings_single_tap),
                    selected = prefs.gestureTapAction,
                    onSelect = { onChange(prefs.copy(gestureTapAction = it)) },
                )
            }
            item {
                TapActionRow(
                    title = stringResource(R.string.settings_double_tap),
                    selected = prefs.gestureDoubleTapAction,
                    onSelect = { onChange(prefs.copy(gestureDoubleTapAction = it)) },
                )
            }

            item {
                val context = LocalContext.current
                val enabled = isGestureServiceEnabled(context)
                Text(
                    text = stringResource(
                        if (enabled) R.string.settings_accessibility_on
                        else R.string.settings_accessibility_off
                    ),
                    fontSize = 12.sp,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                OutlinedButton(
                    onClick = {
                        runCatching {
                            context.startActivity(
                                Intent(Settings.ACTION_ACCESSIBILITY_SETTINGS)
                                    .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                            )
                        }
                    },
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(top = 8.dp),
                ) {
                    Text(
                        stringResource(
                            if (enabled) R.string.settings_accessibility_review
                            else R.string.settings_accessibility_enable
                        )
                    )
                }
            }

            item { HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant) }

            item { SectionTitle(stringResource(R.string.settings_shizuku)) }
            item {
                val context = LocalContext.current
                var state by remember { mutableIntStateOf(-1) }
                var tick by remember { mutableIntStateOf(0) }
                LaunchedEffect(Unit) {
                    while (true) {
                        state = when {
                            !ShizukuBridge.isRunning() -> 0
                            !ShizukuBridge.hasPermission() -> 1
                            else -> 2
                        }
                        tick++
                        delay(1500)
                    }
                }
                val installed = remember(tick, state) {
                    runCatching {
                        context.packageManager.getPackageInfo(SHIZUKU_PACKAGE, 0)
                    }.isSuccess
                }
                Text(
                    text = when (state) {
                        0 -> stringResource(
                            if (installed) R.string.shizuku_installed_stopped
                            else R.string.shizuku_not_installed
                        )
                        1 -> stringResource(R.string.shizuku_no_permission)
                        2 -> stringResource(R.string.shizuku_ready)
                        else -> stringResource(R.string.shizuku_checking)
                    },
                    fontSize = 12.sp,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                when {
                    !installed -> OutlinedButton(
                        onClick = {
                            val intent = Intent(
                                Intent.ACTION_VIEW,
                                Uri.parse("https://shizuku.rikka.app/download/"),
                            ).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                            runCatching { context.startActivity(intent) }
                        },
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(top = 8.dp),
                    ) {
                        Text(stringResource(R.string.shizuku_install))
                    }

                    state == 0 -> OutlinedButton(
                        onClick = {
                            val intent = context.packageManager
                                .getLaunchIntentForPackage(SHIZUKU_PACKAGE)
                            if (intent != null) {
                                intent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                                runCatching { context.startActivity(intent) }
                            }
                        },
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(top = 8.dp),
                    ) {
                        Text(stringResource(R.string.shizuku_open))
                    }

                    state == 1 -> OutlinedButton(
                        onClick = { ShizukuBridge.requestPermission() },
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(top = 8.dp),
                    ) {
                        Text(stringResource(R.string.shizuku_grant))
                    }
                }
            }

            item { HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant) }

            item { SectionTitle(stringResource(R.string.settings_recents)) }
            item {
                Text(
                    text = stringResource(
                        if (hasUsageAccess) R.string.settings_usage_granted
                        else R.string.settings_usage_denied
                    ),
                    fontSize = 13.sp,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                OutlinedButton(
                    onClick = onRequestUsageAccess,
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(top = 8.dp),
                ) {
                    Text(stringResource(if (hasUsageAccess) R.string.settings_usage_review else R.string.settings_usage_grant))
                }
                Text(
                    text = stringResource(R.string.settings_usage_clones),
                    fontSize = 11.sp,
                    color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.8f),
                    modifier = Modifier.padding(top = 8.dp),
                )
            }

            item { HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant) }

            item { SectionTitle(stringResource(R.string.settings_theme_sync)) }
            item {
                Button(onClick = onOpenSync, modifier = Modifier.fillMaxWidth()) {
                    Text(stringResource(R.string.settings_sync_desktop))
                }
                Text(
                    text = stringResource(
                        if (prefs.themeJson == null) R.string.settings_no_theme
                        else R.string.settings_theme_available
                    ),
                    fontSize = 12.sp,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.padding(top = 8.dp),
                )
            }

            item {
                OutlinedButton(
                    onClick = { onChange(LauncherPrefs()) },
                    modifier = Modifier.fillMaxWidth(),
                ) {
                    Text(stringResource(R.string.settings_reset))
                }
            }
        }
    }
}

@Composable
private fun SectionTitle(text: String) {
    Text(
        text = text,
        fontSize = 13.sp,
        fontWeight = FontWeight.Bold,
        color = MaterialTheme.colorScheme.primary,
    )
}

@Composable
private fun SwitchSetting(
    title: String,
    description: String,
    checked: Boolean,
    onChange: (Boolean) -> Unit,
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.SpaceBetween,
    ) {
        Column(modifier = Modifier.weight(1f)) {
            Text(title, fontSize = 14.sp, color = MaterialTheme.colorScheme.onBackground)
            Text(description, fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
        Switch(checked = checked, onCheckedChange = onChange)
    }
}

@Composable
private fun TapActionRow(
    title: String,
    selected: Int,
    onSelect: (Int) -> Unit,
) {
    Text(
        text = title,
        fontSize = 13.sp,
        fontWeight = FontWeight.Bold,
        color = MaterialTheme.colorScheme.primary,
    )
    Row(
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        modifier = Modifier.padding(top = 6.dp),
    ) {
        listOf(
            R.string.action_home to GestureTapAction.HOME,
            R.string.action_apps to GestureTapAction.APPS,
            R.string.action_settings to GestureTapAction.SETTINGS,
        ).forEach { (label, value) ->
            FilterChip(
                selected = selected == value,
                onClick = { onSelect(value) },
                label = { Text(stringResource(label)) },
            )
        }
    }
}
