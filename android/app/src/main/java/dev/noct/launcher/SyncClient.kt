package dev.noct.launcher

import android.app.WallpaperManager
import android.content.Context
import android.graphics.BitmapFactory
import org.json.JSONObject
import java.io.IOException
import java.net.HttpURLConnection
import java.net.URL
import java.net.URLEncoder

data class SyncPayload(
    val themeJson: String,
    val name: String,
    val wallpaper: ByteArray?,
)

object SyncClient {

    fun normalizeBase(raw: String): String {
        var base = raw.trim()
        if (base.isEmpty()) throw IllegalArgumentException("URL vacía")
        if (!base.startsWith("http://") && !base.startsWith("https://")) {
            base = "http://$base"
        }
        return base.trimEnd('/')
    }

    fun fetch(rawBase: String, token: String): SyncPayload {
        val base = normalizeBase(rawBase)
        val encoded = URLEncoder.encode(token, "UTF-8")
        val themeJson = getText("$base/theme?token=$encoded")
        val name = runCatching {
            JSONObject(themeJson).optString("name", "Noct")
        }.getOrDefault("Noct")
        val wallpaper = runCatching {
            getBytes("$base/wallpaper?token=$encoded")
        }.getOrNull()
        return SyncPayload(themeJson, name, wallpaper)
    }

    fun applyWallpaper(context: Context, bytes: ByteArray): Boolean {
        val bitmap = BitmapFactory.decodeByteArray(bytes, 0, bytes.size) ?: return false
        return runCatching {
            WallpaperManager.getInstance(context).setBitmap(bitmap)
            true
        }.getOrDefault(false)
    }

    private fun open(url: String): HttpURLConnection {
        val connection = URL(url).openConnection() as HttpURLConnection
        connection.requestMethod = "GET"
        connection.connectTimeout = 4000
        connection.readTimeout = 12000
        connection.instanceFollowRedirects = true
        return connection
    }

    private fun getText(url: String): String = getBytes(url).toString(Charsets.UTF_8)

    private fun getBytes(url: String): ByteArray {
        val connection = open(url)
        try {
            val code = connection.responseCode
            if (code !in 200..299) {
                val detail = connection.errorStream?.bufferedReader()?.use { it.readText() }
                throw IOException("HTTP $code${if (detail.isNullOrBlank()) "" else ": $detail"}")
            }
            return connection.inputStream.use { it.readBytes() }
        } finally {
            connection.disconnect()
        }
    }
}
