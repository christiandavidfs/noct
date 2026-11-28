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
                    title = "Apps activas (aprox.)",
                    description = "Carrusel de apps que siguen vivas",
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
            item {
                OutlinedButton(
                    onClick = onOpenEditHome,
                    modifier = Modifier.fillMaxWidth(),
                ) {
                    Text("Editar home (mover elementos)")
                }
            }

            item { HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant) }

            item { SectionTitle("Cambiar de app") }
            item {
                Text(
                    text = "El home muestra un carrusel horizontal con las apps que UsageStats considera activas. Es una aproximación: desliza con un dedo, toca para abrir y usa la X para quitarla.",
                    fontSize = 12.sp,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            item {
                SwitchSetting(
                    title = "Handle de gestos",
                    description = "Desliza arriba/abajo para cambiar de app; toca para ir al home",
                    checked = prefs.gestureHandle,
                    onChange = { onChange(prefs.copy(gestureHandle = it)) },
                )
            }
            item {
                Text(
                    text = "Tamaño",
                    fontSize = 13.sp,
                    fontWeight = FontWeight.Bold,
                    color = MaterialTheme.colorScheme.primary,
                )
                Row(
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                    modifier = Modifier.padding(top = 6.dp),
                ) {
                    listOf("Pequeño" to 0, "Mediano" to 1, "Grande" to 2).forEach { (label, value) ->
                        FilterChip(
                            selected = prefs.gestureSize == value,
                            onClick = { onChange(prefs.copy(gestureSize = value)) },
                            label = { Text(label) },
                        )
                    }
                }
            }
            item {
                Text(
                    text = "Color",
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
                    text = "Opacidad: ${prefs.gestureOpacity}%",
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
                    text = "Forma",
                    fontSize = 13.sp,
                    fontWeight = FontWeight.Bold,
                    color = MaterialTheme.colorScheme.primary,
                )
                Row(
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                    modifier = Modifier.padding(top = 6.dp),
                ) {
                    listOf("Pastilla" to 0, "Círculo" to 1, "Cuadrado" to 2).forEach { (label, value) ->
                        FilterChip(
                            selected = prefs.gestureShape == value,
                            onClick = { onChange(prefs.copy(gestureShape = value)) },
                            label = { Text(label) },
                        )
                    }
                }
            }
            item {
                val context = LocalContext.current
                val enabled = isGestureServiceEnabled(context)
                Text(
                    text = if (enabled) {
                        "Servicio de accesibilidad activo: el handle flotante está disponible."
                    } else {
                        "Servicio de accesibilidad desactivado. Actívalo en Ajustes → Accesibilidad para que el handle funcione."
                    },
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
                        if (enabled) "Revisar servicio de accesibilidad"
                        else "Activar servicio de accesibilidad"
                    )
                }
            }

            item { HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant) }

            item { SectionTitle("Shizuku (cambio real)") }
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
                        0 -> if (installed) {
                            "Shizuku está instalado pero no en ejecución. Ábrelo y arráncalo " +
                                "(depuración inalámbrica o ADB).\n\n" +
                                "Consejo: una vez arrancado puedes desactivar la depuración USB; " +
                                "Shizuku sigue funcionando hasta reiniciar el móvil."
                        } else {
                            "Shizuku no está instalado. Es necesario para cambiar de tarea sin " +
                                "relanzar las apps; sin él, los gestos usan el modo aproximado."
                        }
                        1 ->
                            "Shizuku está activo, pero Noct no tiene permiso para usarlo."
                        2 ->
                            "Shizuku listo: los gestos cambian de tarea sin relanzar la app."
                        else ->
                            "Comprobando Shizuku…"
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
                        Text("Instalar Shizuku")
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
                        Text("Abrir Shizuku")
                    }

                    state == 1 -> OutlinedButton(
                        onClick = { ShizukuBridge.requestPermission() },
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(top = 8.dp),
                    ) {
                        Text("Conceder permiso a Noct")
                    }
                }
            }

            item { HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant) }

            item { SectionTitle("Apps activas (aprox.)") }
            item {
                Text(
                    text = if (hasUsageAccess) {
                        "Acceso de uso concedido. Noct estima las apps activas con UsageStats."
                    } else {
                        "Sin acceso de uso. Android no deja estimar qué apps siguen activas."
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
                Text(
                    text = "Los clones del sistema (p. ej. el WhatsApp duplicado de Motorola) pueden no aparecer: Android no los expone a launchers de terceros.",
                    fontSize = 11.sp,
                    color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.8f),
                    modifier = Modifier.padding(top = 8.dp),
                )
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
