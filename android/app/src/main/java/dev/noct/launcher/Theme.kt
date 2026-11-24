package dev.noct.launcher

import android.graphics.Color as AndroidColor
import androidx.compose.material3.ColorScheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.ui.graphics.Color
import org.json.JSONObject

data class NoctPalette(
    val dark: Boolean,
    val name: String,
    val roles: Map<String, Color>,
)

fun parseNoctTheme(json: String?): NoctPalette? {
    if (json.isNullOrBlank()) return null
    return runCatching {
        val root = JSONObject(json)
        val mode = root.optString("resolved_mode", root.optString("mode", "dark"))
        val dark = mode != "light"
        val paletteRoot = root.optJSONObject("palette") ?: return null
        val palette = paletteRoot.optJSONObject(if (dark) "dark" else "light") ?: return null
        val roles = buildMap {
            val keys = palette.keys()
            while (keys.hasNext()) {
                val key = keys.next()
                val raw = palette.optString(key)
                parseNoctColor(raw)?.let { put(key, it) }
            }
        }
        NoctPalette(dark, root.optString("name", "Noct"), roles)
    }.getOrNull()
}

private fun parseNoctColor(raw: String): Color? {
    val value = raw.trim()
    if (!value.startsWith("#")) return null
    return runCatching { Color(AndroidColor.parseColor(value)) }.getOrNull()
}

private fun NoctPalette.role(name: String, fallback: Color): Color = roles[name] ?: fallback

fun NoctPalette.toColorScheme(): ColorScheme {
    val base = if (dark) darkColorScheme() else lightColorScheme()
    return base.copy(
        primary = role("primary", base.primary),
        onPrimary = role("on_primary", base.onPrimary),
        primaryContainer = role("primary_container", base.primaryContainer),
        onPrimaryContainer = role("on_primary_container", base.onPrimaryContainer),
        inversePrimary = role("inverse_primary", base.inversePrimary),
        secondary = role("secondary", base.secondary),
        onSecondary = role("on_secondary", base.onSecondary),
        secondaryContainer = role("secondary_container", base.secondaryContainer),
        onSecondaryContainer = role("on_secondary_container", base.onSecondaryContainer),
        tertiary = role("tertiary", base.tertiary),
        onTertiary = role("on_tertiary", base.onTertiary),
        tertiaryContainer = role("tertiary_container", base.tertiaryContainer),
        onTertiaryContainer = role("on_tertiary_container", base.onTertiaryContainer),
        background = role("background", base.background),
        onBackground = role("on_background", base.onBackground),
        surface = role("surface", base.surface),
        onSurface = role("on_surface", base.onSurface),
        surfaceVariant = role("surface_variant", base.surfaceVariant),
        onSurfaceVariant = role("on_surface_variant", base.onSurfaceVariant),
        surfaceTint = role("surface_tint", base.surfaceTint),
        inverseSurface = role("inverse_surface", base.inverseSurface),
        inverseOnSurface = role("inverse_on_surface", base.inverseOnSurface),
        error = role("error", base.error),
        onError = role("on_error", base.onError),
        errorContainer = role("error_container", base.errorContainer),
        onErrorContainer = role("on_error_container", base.onErrorContainer),
        outline = role("outline", base.outline),
        outlineVariant = role("outline_variant", base.outlineVariant),
        scrim = role("scrim", base.scrim),
        surfaceBright = role("surface_bright", base.surfaceBright),
        surfaceDim = role("surface_dim", base.surfaceDim),
        surfaceContainer = role("surface_container", base.surfaceContainer),
        surfaceContainerHigh = role("surface_container_high", base.surfaceContainerHigh),
        surfaceContainerHighest = role("surface_container_highest", base.surfaceContainerHighest),
        surfaceContainerLow = role("surface_container_low", base.surfaceContainerLow),
        surfaceContainerLowest = role("surface_container_lowest", base.surfaceContainerLowest),
    )
}

val NoctDefaultDark: ColorScheme = darkColorScheme(
    primary = Color(0xFFE2E2E2),
    onPrimary = Color(0xFF1B1B1B),
    primaryContainer = Color(0xFF2A2A2A),
    onPrimaryContainer = Color(0xFFE2E2E2),
    secondary = Color(0xFFC6C6C6),
    onSecondary = Color(0xFF1B1B1B),
    background = Color(0xFF0F0F0F),
    onBackground = Color(0xFFE8E8E8),
    surface = Color(0xFF151515),
    onSurface = Color(0xFFE8E8E8),
    surfaceVariant = Color(0xFF2A2A2A),
    onSurfaceVariant = Color(0xFFC6C6C6),
    outline = Color(0xFF8A8A8A),
    outlineVariant = Color(0xFF3A3A3A),
)

val NoctDefaultLight: ColorScheme = lightColorScheme(
    primary = Color(0xFF1B1B1B),
    onPrimary = Color(0xFFF2F2F2),
    background = Color(0xFFF9F9F9),
    onBackground = Color(0xFF1B1B1B),
    surface = Color(0xFFF2F2F2),
    onSurface = Color(0xFF1B1B1B),
    surfaceVariant = Color(0xFFE2E2E2),
    onSurfaceVariant = Color(0xFF474747),
)
