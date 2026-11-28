package dev.noct.launcher

import android.content.Intent
import android.os.Bundle
import android.util.Log

/**
 * Servicio que Shizuku ejecuta con identidad shell (sin root).
 *
 * Desde ahí sí se puede usar la API oculta `ActivityTaskManager`:
 * - `getRecentTasks` para la lista real de tareas (abiertas o no).
 * - `startActivityFromRecents` para traer una tarea existente al frente sin
 *   relanzarla, que es lo que hace el Recents del sistema.
 */
class NoctShellService : INoctShell.Stub() {

    @Volatile
    var lastError: String? = null

    private val activityTaskManager: Any? by lazy {
        runCatching {
            Class.forName("android.app.ActivityTaskManager")
                .getMethod("getService")
                .invoke(null)
        }.getOrNull()
    }

    override fun recentTasks(): String {
        val service = activityTaskManager ?: return ""
        return runCatching {
            val method = service.javaClass.getMethod(
                "getRecentTasks",
                Int::class.javaPrimitiveType,
                Int::class.javaPrimitiveType,
                Int::class.javaPrimitiveType,
            )
            val slice = method.invoke(service, 24, 0, 0)
            val list = slice.javaClass.getMethod("getList").invoke(slice) as List<*>
            val builder = StringBuilder()
            for (item in list) {
                val info = item ?: continue
                val taskId = info.javaClass.getField("taskId").getInt(info)
                val base = info.javaClass.getField("baseIntent").get(info) as? Intent
                val pkg = base?.component?.packageName
                    ?: base?.`package`
                    ?: continue
                if (pkg == "com.android.systemui" ||
                    pkg == "dev.noct.launcher" ||
                    pkg == "moe.shizuku.privileged.api" ||
                    base?.hasCategory(Intent.CATEGORY_HOME) == true
                ) continue
                builder.append(taskId).append('\t').append(pkg).append('\n')
            }
            builder.toString()
        }.onFailure { lastError = "recentTasks ${it.javaClass.simpleName}: ${it.message}" }.getOrDefault("")
    }

    override fun switchTo(taskId: Int): Boolean {
        val service = activityTaskManager ?: return false
        return runCatching {
            val method = service.javaClass.getMethod(
                "startActivityFromRecents",
                Int::class.javaPrimitiveType,
                Bundle::class.java,
            )
            method.invoke(service, taskId, null)
            true
        }.onFailure { lastError = "${it.javaClass.simpleName}: ${it.message}" }.getOrDefault(false)
    }

    override fun lastError(): String = lastError ?: ""

    override fun destroy() {
        System.exit(0)
    }
}
