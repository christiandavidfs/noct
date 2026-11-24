package dev.noct.launcher

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.ArrowBack
import androidx.compose.material3.Button
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilterChip
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
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsScreen(
    prefs: LauncherPrefs,
    hasUsageAccess: Boolean,
    onChange: (LauncherPrefs) -> Unit,
    onBack: () -> Unit,
    onOpenSync: () -> Unit,
    onOpenWidgets: () -> Unit,
    onRequestUsageAccess: () -> Unit,
) {
    Scaffold(
        containerColor = MaterialTheme.colorScheme.background.copy(alpha = 0.94f),
        topBar = {
            TopAppBar(
                title = { Text("Ajustes de Noct") },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(
                            Icons.AutoMirrored.Outlined.ArrowBack,
                            contentDescription = "Volver",
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
            item { SectionTitle("Widgets de Android") }
            item {
                Button(onClick = onOpenWidgets, modifier = Modifier.fillMaxWidth()) {
                    Text("Añadir o quitar widgets")
                }
                Text(
                    text = "Los widgets reales de Android se colocan en el home.",
                    fontSize = 12.sp,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.padding(top = 6.dp),
                )
            }

            item { HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant) }

            item { SectionTitle("Widgets del home") }
            item {
                SwitchSetting(
                    title = "Reloj",
                    description = "Hora y fecha grandes",
                    checked = prefs.showClock,
                    onChange = { onChange(prefs.copy(showClock = it)) },
                )
            }
            item {
                SwitchSetting(
                    title = "Batería",
                    description = "Nivel y estado de carga",
                    checked = prefs.showBattery,
                    onChange = { onChange(prefs.copy(showBattery = it)) },
                )
            }
            item {
                SwitchSetting(
                    title = "Apps en marcha",
                    description = "Lista de apps recientes",
                    checked = prefs.showRecents,
                    onChange = { onChange(prefs.copy(showRecents = it)) },
                )
            }
            item {
                SwitchSetting(
                    title = "Tema sincronizado",
                    description = "Nombre del tema y acceso a sync",
                    checked = prefs.showSync,
                    onChange = { onChange(prefs.copy(showSync = it)) },
                )
            }
            item {
                SwitchSetting(
                    title = "Reloj 24 h",
                    description = "Desactiva para formato 12 h",
                    checked = prefs.clock24h,
                    onChange = { onChange(prefs.copy(clock24h = it)) },
                )
            }

            item { HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant) }

            item { SectionTitle("Gesto de dos dedos") }
            item {
                Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                    FilterChip(
                        selected = prefs.axis == SwipeAxis.HORIZONTAL,
                        onClick = { onChange(prefs.copy(axis = SwipeAxis.HORIZONTAL)) },
                        label = { Text("Horizontal") },
                    )
                    FilterChip(
                        selected = prefs.axis == SwipeAxis.VERTICAL,
                        onClick = { onChange(prefs.copy(axis = SwipeAxis.VERTICAL)) },
                        label = { Text("Vertical") },
                    )
                }
                Text(
                    text = "Dos dedos cambian de app entre las que están en marcha.",
                    fontSize = 12.sp,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.padding(top = 6.dp),
                )
            }
            item {
                SliderSetting(
                    title = "Sensibilidad",
                    value = prefs.thresholdDp.toFloat(),
                    valueText = "${prefs.thresholdDp} dp",
                    range = 24f..160f,
                    steps = 0,
                    onChange = { onChange(prefs.copy(thresholdDp = it.toInt())) },
                )
            }
            item {
                SwitchSetting(
                    title = "Invertir dirección",
                    description = "Cambia el sentido del gesto",
                    checked = prefs.invert,
                    onChange = { onChange(prefs.copy(invert = it)) },
                )
            }
            item {
                SwitchSetting(
                    title = "Solo desde el borde",
                    description = "Evita falsos positivos al hacer scroll",
                    checked = prefs.edgeOnly,
                    onChange = { onChange(prefs.copy(edgeOnly = it)) },
                )
            }
            if (prefs.edgeOnly) {
                item {
                    SliderSetting(
                        title = "Ancho del borde",
                        value = prefs.edgeWidthDp.toFloat(),
                        valueText = "${prefs.edgeWidthDp} dp",
                        range = 16f..60f,
                        steps = 0,
                        onChange = { onChange(prefs.copy(edgeWidthDp = it.toInt())) },
                    )
                }
            }

            item { HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant) }

            item { SectionTitle("Apps en marcha") }
            item {
                Text(
                    text = if (hasUsageAccess) {
                        "Acceso de uso concedido. Noct puede ver las apps recientes."
                    } else {
                        "Sin acceso de uso. Android no deja ver qué apps están en marcha."
                    },
                    fontSize = 13.sp,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                OutlinedButton(
                    onClick = onRequestUsageAccess,
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(top = 8.dp),
                ) {
                    Text(if (hasUsageAccess) "Revisar acceso de uso" else "Dar acceso de uso")
                }
            }

            item { HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant) }

            item { SectionTitle("Tema y sync") }
            item {
                Button(onClick = onOpenSync, modifier = Modifier.fillMaxWidth()) {
                    Text("Sincronizar con el escritorio")
                }
                Text(
                    text = if (prefs.themeJson == null) {
                        "Sin tema sincronizado. Usa noct sync serve en el escritorio."
                    } else {
                        "Tema sincronizado disponible."
                    },
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
                    Text("Restablecer ajustes")
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
private fun SliderSetting(
    title: String,
    value: Float,
    valueText: String,
    range: ClosedFloatingPointRange<Float>,
    steps: Int,
    onChange: (Float) -> Unit,
) {
    Column {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
        ) {
            Text(title, fontSize = 14.sp, color = MaterialTheme.colorScheme.onBackground)
            Text(valueText, fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
        Slider(
            value = value,
            onValueChange = onChange,
            valueRange = range,
            steps = steps,
        )
    }
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
