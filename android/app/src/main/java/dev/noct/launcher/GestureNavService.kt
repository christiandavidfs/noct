package dev.noct.launcher

import android.accessibilityservice.AccessibilityService
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.SharedPreferences
import android.graphics.Color
import android.graphics.PixelFormat
import android.graphics.drawable.GradientDrawable
import android.os.Build
import android.os.SystemClock
import android.provider.Settings
import android.view.Gravity
import android.view.MotionEvent
import android.view.View
import android.view.WindowManager
import android.view.accessibility.AccessibilityEvent
import android.widget.Toast
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.launch
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.withContext
import kotlin.math.abs
import rikka.shizuku.Shizuku

/**
 * Handle flotante para cambiar de app con un gesto, sin root.
 *
 * - Deslizar a la izquierda: app anterior; a la derecha: siguiente.
 * - Arriba/abajo siguen funcionando como alias (el eje dominante decide).
 * - Un toque y doble toque ejecutan la acción configurada en Ajustes.
 *
 * La lista de apps vivas es una aproximación con UsageStats (Android no expone
 * las tareas de otras apps). Si Shizuku está activo se usan las tareas reales
 * del sistema y `startActivityFromRecents`, que reanuda la tarea sin relanzarla.
 * Sin Shizuku se lanza la app con su intent de launcher.
 */
class GestureNavService : AccessibilityService() {

    private companion object {
        const val DOUBLE_TAP_MS = 300
    }

    private lateinit var windowManager: WindowManager
    private var handle: View? = null
    private var currentPackage: String? = null
    private var apps: List<AppEntry> = emptyList()
    // Ciclo estable de tareas para que subir/bajar recorra el historial sin
    // que el reordenamiento de Recents lo descoloque en cada cambio.
    private var cycle: List<ShizukuBridge.TaskRef> = emptyList()
    private var cursor = -1
    private var pendingTarget: String? = null
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main.immediate)

    private var downX = 0f
    private var downY = 0f
    private var downAt = 0L
    private var dragging = false
    private var tapJob: Job? = null
    private var lastTapAt = 0L

    private val binderReceivedListener =
        Shizuku.OnBinderReceivedListener { ShizukuBridge.bind(this) }
    private val binderDeadListener =
        Shizuku.OnBinderDeadListener { ShizukuBridge.invalidate() }

    private val styleKeys = setOf(
        "gestureHandle",
        "gestureSize",
        "gestureColor",
        "gestureOpacity",
        "gestureShape",
    )

    private val prefsListener =
        SharedPreferences.OnSharedPreferenceChangeListener { _, key ->
            if (key != null && key in styleKeys) updateHandle()
        }

    override fun onServiceConnected() {
        super.onServiceConnected()
        windowManager = getSystemService(Context.WINDOW_SERVICE) as WindowManager
        getSharedPreferences("noct_launcher", Context.MODE_PRIVATE)
            .registerOnSharedPreferenceChangeListener(prefsListener)
        updateHandle()
        runCatching {
            Shizuku.addBinderReceivedListenerSticky(binderReceivedListener)
            Shizuku.addBinderDeadListener(binderDeadListener)
        }
        ShizukuBridge.bind(this)
        scope.launch {
            apps = loadApps()
        }
    }

    override fun onAccessibilityEvent(event: AccessibilityEvent?) {
        if (event?.eventType != AccessibilityEvent.TYPE_WINDOW_STATE_CHANGED) return
        val pkg = event.packageName?.toString() ?: return
        when {
            // En el launcher no hay app "actual": el gesto debe ir a la más reciente.
            pkg == packageName -> {
                currentPackage = null
                cursor = -1
            }
            pkg == "com.android.systemui" -> Unit
            // Nuestro propio cambio: conservar el ciclo.
            pkg == pendingTarget -> {
                currentPackage = pkg
                pendingTarget = null
            }
            // Navegación externa (el usuario abrió otra app): ciclo nuevo.
            else -> {
                currentPackage = pkg
                cycle = emptyList()
                cursor = -1
            }
        }
    }

    override fun onInterrupt() = Unit

    override fun onDestroy() {
        getSharedPreferences("noct_launcher", Context.MODE_PRIVATE)
            .unregisterOnSharedPreferenceChangeListener(prefsListener)
        runCatching {
            Shizuku.removeBinderReceivedListener(binderReceivedListener)
            Shizuku.removeBinderDeadListener(binderDeadListener)
        }
        removeHandle()
        scope.cancel()
        super.onDestroy()
    }

    private fun updateHandle() {
        removeHandle()
        if (Prefs(this).load().gestureHandle) showHandle()
    }

    private fun showHandle() {
        if (handle != null) return
        val prefs = Prefs(this).load()
        val density = resources.displayMetrics.density

        val widthDp: Int
        val heightDp: Int
        when (prefs.gestureShape) {
            1 -> { // círculo
                widthDp = when (prefs.gestureSize) { 0 -> 32; 2 -> 52; else -> 42 }
                heightDp = widthDp
            }
            2 -> { // cuadrado
                widthDp = when (prefs.gestureSize) { 0 -> 24; 2 -> 40; else -> 32 }
                heightDp = widthDp
            }
            else -> { // pastilla
                widthDp = when (prefs.gestureSize) { 0 -> 8; 2 -> 18; else -> 12 }
                heightDp = when (prefs.gestureSize) { 0 -> 80; 2 -> 150; else -> 110 }
            }
        }
        val width = (widthDp * density).toInt()
        val height = (heightDp * density).toInt()
        val color = runCatching { Color.parseColor(prefs.gestureColor) }
            .getOrDefault(Color.rgb(124, 255, 178))
        val radius = when (prefs.gestureShape) {
            1 -> width / 2f
            2 -> 4 * density
            else -> width / 2f
        }

        val view = View(this).apply {
            background = GradientDrawable().apply {
                shape = GradientDrawable.RECTANGLE
                cornerRadius = radius
                setColor(color)
                setStroke((1 * density).toInt(), Color.argb(200, 255, 255, 255))
            }
            alpha = (prefs.gestureOpacity.coerceIn(10, 100)) / 100f
            contentDescription = getString(R.string.handle_description)
            setOnTouchListener { v, event -> handleTouch(v, event) }
        }

        val params = WindowManager.LayoutParams(
            width,
            height,
            WindowManager.LayoutParams.TYPE_ACCESSIBILITY_OVERLAY,
            WindowManager.LayoutParams.FLAG_NOT_FOCUSABLE or
                WindowManager.LayoutParams.FLAG_NOT_TOUCH_MODAL or
                WindowManager.LayoutParams.FLAG_LAYOUT_IN_SCREEN,
            PixelFormat.TRANSLUCENT,
        ).apply {
            gravity = Gravity.END or Gravity.CENTER_VERTICAL
            x = (28 * density).toInt()
        }

        runCatching { windowManager.addView(view, params) }
            .onSuccess { handle = view }
    }

    private fun removeHandle() {
        handle?.let { runCatching { windowManager.removeView(it) } }
        handle = null
    }

    private fun handleTouch(view: View, event: MotionEvent): Boolean {
        val density = resources.displayMetrics.density
        when (event.actionMasked) {
            MotionEvent.ACTION_DOWN -> {
                downX = event.rawX
                downY = event.rawY
                downAt = System.currentTimeMillis()
                dragging = false
            }

            MotionEvent.ACTION_MOVE -> {
                val dx = event.rawX - downX
                val dy = event.rawY - downY
                if (abs(dx) > 6 * density || abs(dy) > 6 * density) {
                    if (!dragging) {
                        // Un deslizamiento cancela el toque simple pendiente.
                        tapJob?.cancel()
                        lastTapAt = 0L
                    }
                    dragging = true
                }
                view.translationX = dx * 0.3f
                view.translationY = dy * 0.3f
            }

            MotionEvent.ACTION_UP -> {
                val dx = event.rawX - downX
                val dy = event.rawY - downY
                val elapsed = System.currentTimeMillis() - downAt
                val threshold = 40 * density
                view.translationX = 0f
                view.translationY = 0f
                when {
                    !dragging && elapsed < 300 -> handleTap()
                    abs(dx) >= abs(dy) && dx <= -threshold -> switchApp(-1)
                    abs(dx) >= abs(dy) && dx >= threshold -> switchApp(1)
                    dy <= -threshold -> switchApp(1)
                    dy >= threshold -> switchApp(-1)
                }
            }

            MotionEvent.ACTION_CANCEL -> {
                view.translationX = 0f
                view.translationY = 0f
            }
        }
        return true
    }

    private fun handleTap() {
        val now = SystemClock.uptimeMillis()
        if (now - lastTapAt in 1..DOUBLE_TAP_MS) {
            lastTapAt = 0L
            tapJob?.cancel()
            performTap(doubleTap = true)
            return
        }
        lastTapAt = now
        tapJob = scope.launch {
            delay(DOUBLE_TAP_MS.toLong())
            performTap(doubleTap = false)
        }
    }

    private fun performTap(doubleTap: Boolean) {
        val prefs = Prefs(this).load()
        val action = if (doubleTap) prefs.gestureDoubleTapAction else prefs.gestureTapAction
        val target = when (action) {
            GestureTapAction.APPS -> "drawer"
            GestureTapAction.SETTINGS -> "settings"
            else -> "home"
        }
        openLauncher(target)
    }

    private fun openLauncher(screen: String) {
        val intent = Intent(this, MainActivity::class.java).apply {
            addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_SINGLE_TOP)
            putExtra(MainActivity.EXTRA_SCREEN, screen)
        }
        runCatching { startActivity(intent) }
    }

    private suspend fun loadApps(): List<AppEntry> = withContext(Dispatchers.IO) {
        runCatching { AppRepository.load(this@GestureNavService) }
            .getOrDefault(emptyList())
    }

    private fun switchApp(direction: Int) {
        scope.launch {
            // Shizuku: tareas reales del sistema y cambio sin relanzar.
            val shellTasks = withContext(Dispatchers.IO) { ShizukuBridge.tasks() }
            if (shellTasks.isNotEmpty()) {
                val ids = shellTasks.mapTo(HashSet()) { it.id }
                if (cycle.isEmpty() || cycle.none { it.id in ids }) {
                    cycle = shellTasks
                    cursor = cycle.indexOfFirst { it.packageName == currentPackage }
                }
                if (currentPackage == null) cursor = -1

                val size = cycle.size
                val next = if (cursor < 0) {
                    if (direction > 0) 0 else size - 1
                } else {
                    ((cursor + direction) % size + size) % size
                }
                val target = cycle[next]
                if (target.packageName != currentPackage) {
                    val switched = withContext(Dispatchers.IO) {
                        ShizukuBridge.shell?.switchTo(target.id) == true
                    }
                    if (switched) {
                        cursor = next
                        currentPackage = target.packageName
                        pendingTarget = target.packageName
                        return@launch
                    }
                    Toast.makeText(
                        this@GestureNavService,
                        getString(R.string.shizuku_error, ShizukuBridge.lastError()),
                        Toast.LENGTH_LONG,
                    ).show()
                    cycle = emptyList()
                    cursor = -1
                }
            }

            if (apps.isEmpty()) apps = loadApps()
            val alive = withContext(Dispatchers.IO) {
                RecentApps.load(this@GestureNavService, apps, limit = 8)
            }
            if (alive.size < 2) return@launch

            val list = alive.map { it.app }
            // Si aún no conocemos la app en primer plano, la más reciente lo es.
            val currentPkg = currentPackage ?: list.first().packageName
            val current = list.indexOfFirst { it.packageName == currentPkg }
            val target = if (current >= 0) {
                list[((current + direction) % list.size + list.size) % list.size]
            } else {
                list.firstOrNull { it.packageName != currentPkg } ?: list.first()
            }
            if (target.packageName == currentPkg) return@launch

            val intent = packageManager.getLaunchIntentForPackage(target.packageName)
                ?: return@launch
            intent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            runCatching { startActivity(intent) }
            currentPackage = target.packageName
        }
    }
}

/** ¿Está el usuario a activado el servicio en Ajustes → Accesibilidad? */
fun isGestureServiceEnabled(context: Context): Boolean {
    val component = ComponentName(context, GestureNavService::class.java)
    val enabled = Settings.Secure.getString(
        context.contentResolver,
        Settings.Secure.ENABLED_ACCESSIBILITY_SERVICES,
    ) ?: return false
    return enabled.split(':').any {
        it.equals(component.flattenToString(), ignoreCase = true) ||
            it.equals(component.flattenToShortString(), ignoreCase = true)
    }
}
