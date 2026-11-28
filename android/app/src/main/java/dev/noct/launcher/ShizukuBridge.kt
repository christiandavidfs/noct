package dev.noct.launcher

import android.content.ComponentName
import android.content.Context
import android.content.ServiceConnection
import android.content.pm.PackageManager
import android.os.IBinder
import rikka.shizuku.Shizuku

const val SHIZUKU_PACKAGE = "moe.shizuku.privileged.api"

/** Puente con Shizuku: lista de tareas reales y cambio sin relanzar. */
object ShizukuBridge {

    @Volatile
    var shell: INoctShell? = null
        private set

    private var binding = false

    fun invalidate() {
        shell = null
        binding = false
    }

    fun isRunning(): Boolean =
        runCatching { Shizuku.pingBinder() }.getOrDefault(false)

    fun hasPermission(): Boolean =
        runCatching {
            Shizuku.checkSelfPermission() == PackageManager.PERMISSION_GRANTED
        }.getOrDefault(false)

    fun requestPermission(code: Int = 4242) {
        runCatching { Shizuku.requestPermission(code) }
    }

    fun bind(context: Context) {
        if (binding || shell != null) return
        if (!isRunning() || !hasPermission()) return
        binding = true
        val args = Shizuku.UserServiceArgs(
            ComponentName(context, NoctShellService::class.java),
        )
            .daemon(false)
            .processNameSuffix("noctshell")
            .debuggable(false)
            .version(2)
        runCatching {
            Shizuku.bindUserService(
                args,
                object : ServiceConnection {
                    override fun onServiceConnected(name: ComponentName?, service: IBinder?) {
                        shell = INoctShell.Stub.asInterface(service)
                        binding = false
                    }

                    override fun onServiceDisconnected(name: ComponentName?) {
                        shell = null
                        binding = false
                    }
                },
            )
        }.onFailure { binding = false }
    }

    fun lastError(): String = runCatching { shell?.lastError() }.getOrNull().orEmpty()

    data class TaskRef(val id: Int, val packageName: String)

    fun tasks(): List<TaskRef> {
        val raw = runCatching { shell?.recentTasks() }.getOrNull() ?: return emptyList()
        return raw.lineSequence().mapNotNull { line ->
            val parts = line.split('\t')
            if (parts.size < 2) return@mapNotNull null
            val id = parts[0].toIntOrNull() ?: return@mapNotNull null
            TaskRef(id, parts[1])
        }.toList()
    }
}
