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
import android.provider.Settings
import android.view.Gravity
import android.view.MotionEvent
import android.view.View
import android.view.WindowManager
import android.view.accessibility.AccessibilityEvent
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import kotlin.math.abs

/**
 * Handle flotante para cambiar de app con un gesto, sin root.
 *
 * - Deslizar hacia arriba: siguiente app viva.
 * - Deslizar hacia abajo: anterior.
 * - Toque: home.
 *
 * La lista de apps vivas es una aproximación con UsageStats (Android no expone
 * las tareas de otras apps). Al cambiar se lanza la app con su intent de
 * launcher: Android decide si la reanuda o la reinicia.
 */
class GestureNavService : AccessibilityService() {

    private lateinit var windowManager: WindowManager
    private var handle: View? = null
    private var currentPackage: String? = null
    private var apps: List<AppEntry> = emptyList()
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main.immediate)

    private var downY = 0f
    private var downAt = 0L
    private var dragging = false

    private val prefsListener =
        SharedPreferences.OnSharedPreferenceChangeListener { _, key ->
            if (key == "gestureHandle") updateHandle()
        }

    override fun onServiceConnected() {
        super.onServiceConnected()
        windowManager = getSystemService(Context.WINDOW_SERVICE) as WindowManager
        getSharedPreferences("noct_launcher", Context.MODE_PRIVATE)
            .registerOnSharedPreferenceChangeListener(prefsListener)
        updateHandle()
        scope.launch {
            apps = loadApps()
        }
    }

    override fun onAccessibilityEvent(event: AccessibilityEvent?) {
        if (event?.eventType != AccessibilityEvent.TYPE_WINDOW_STATE_CHANGED) return
        val pkg = event.packageName?.toString() ?: return
        if (pkg != packageName && pkg != "com.android.systemui") {
            currentPackage = pkg
        }
    }

    override fun onInterrupt() = Unit

    override fun onDestroy() {
        getSharedPreferences("noct_launcher", Context.MODE_PRIVATE)
            .unregisterOnSharedPreferenceChangeListener(prefsListener)
        removeHandle()
        scope.cancel()
        super.onDestroy()
    }

    private fun updateHandle() {
        if (Prefs(this).load().gestureHandle) showHandle() else removeHandle()
    }

    private fun showHandle() {
        if (handle != null) return
        val density = resources.displayMetrics.density
        val width = (12 * density).toInt()
        val height = (110 * density).toInt()

        val view = View(this).apply {
            background = GradientDrawable().apply {
                shape = GradientDrawable.RECTANGLE
                cornerRadius = width.toFloat()
                setColor(Color.argb(120, 124, 255, 178))
                setStroke((1 * density).toInt(), Color.argb(190, 190, 240, 212))
            }
            alpha = 0.62f
            contentDescription = "Cambiar de app"
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
            x = (6 * density).toInt()
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
                downY = event.rawY
                downAt = System.currentTimeMillis()
                dragging = false
            }

            MotionEvent.ACTION_MOVE -> {
                val dy = event.rawY - downY
                if (abs(dy) > 6 * density) dragging = true
                view.translationY = dy * 0.3f
            }

            MotionEvent.ACTION_UP -> {
                val dy = event.rawY - downY
                val elapsed = System.currentTimeMillis() - downAt
                view.translationY = 0f
                when {
                    !dragging && elapsed < 300 -> performGlobalAction(GLOBAL_ACTION_HOME)
                    dy <= -40 * density -> switchApp(1)
                    dy >= 40 * density -> switchApp(-1)
                }
            }

            MotionEvent.ACTION_CANCEL -> view.translationY = 0f
        }
        return true
    }

    private suspend fun loadApps(): List<AppEntry> = withContext(Dispatchers.IO) {
        runCatching { AppRepository.load(this@GestureNavService) }
            .getOrDefault(emptyList())
    }

    private fun switchApp(direction: Int) {
        scope.launch {
            if (apps.isEmpty()) apps = loadApps()
            val alive = withContext(Dispatchers.IO) {
                RecentApps.load(this@GestureNavService, apps, limit = 8)
            }
            if (alive.size < 2) return@launch

            val list = alive.map { it.app }
            val current = list.indexOfFirst { it.packageName == currentPackage }
            val index = if (current < 0) {
                0
            } else {
                ((current + direction) % list.size + list.size) % list.size
            }
            val target = list[index]
            if (target.packageName == currentPackage) return@launch

            val intent = packageManager.getLaunchIntentForPackage(target.packageName)
                ?: return@launch
            intent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            runCatching { startActivity(intent) }
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
