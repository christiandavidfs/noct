package dev.noct.launcher

import android.appwidget.AppWidgetHost
import android.appwidget.AppWidgetManager
import android.appwidget.AppWidgetProviderInfo
import android.content.Context
import android.content.Intent
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.Image
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.ArrowBack
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun WidgetPickerScreen(
    host: AppWidgetHost,
    onAdded: (HostedWidget) -> Unit,
    onBack: () -> Unit,
) {
    val context = LocalContext.current
    val manager = remember { AppWidgetManager.getInstance(context) }
    val providers = remember { installedWidgetProviders(context) }
    var pendingProvider by remember { mutableStateOf<AppWidgetProviderInfo?>(null) }
    var pendingId by remember { mutableIntStateOf(-1) }
    var status by remember { mutableStateOf("") }
    var selected by remember { mutableStateOf<WidgetProvider?>(null) }

    val configureLauncher = rememberLauncherForActivityResult(
        ActivityResultContracts.StartActivityForResult()
    ) { result ->
        val provider = pendingProvider
        if (result.resultCode == android.app.Activity.RESULT_OK &&
            provider != null && pendingId != -1
        ) {
            onAdded(HostedWidget(pendingId, provider.provider.flattenToString()))
            status = context.getString(R.string.picker_added)
        } else if (pendingId != -1) {
            host.deleteAppWidgetId(pendingId)
        }
        pendingProvider = null
        pendingId = -1
    }

    val finishAdd: (AppWidgetProviderInfo, Int) -> Unit = { provider, id ->
        if (provider.configure != null) {
            pendingProvider = provider
            pendingId = id
            val intent = Intent(AppWidgetManager.ACTION_APPWIDGET_CONFIGURE)
                .setComponent(provider.configure)
                .putExtra(AppWidgetManager.EXTRA_APPWIDGET_ID, id)
            configureLauncher.launch(intent)
        } else {
            onAdded(HostedWidget(id, provider.provider.flattenToString()))
            status = context.getString(R.string.picker_added)
        }
    }

    val bindLauncher = rememberLauncherForActivityResult(
        ActivityResultContracts.StartActivityForResult()
    ) { result ->
        val provider = pendingProvider
        val id = result.data?.getIntExtra(AppWidgetManager.EXTRA_APPWIDGET_ID, pendingId)
            ?: pendingId
        if (result.resultCode == android.app.Activity.RESULT_OK &&
            provider != null && id != -1
        ) {
            pendingId = id
            finishAdd(provider, id)
        } else {
            if (pendingId != -1) host.deleteAppWidgetId(pendingId)
            pendingProvider = null
            pendingId = -1
            status = context.getString(R.string.picker_cancelled)
        }
    }

    fun beginAdd(provider: AppWidgetProviderInfo) {
        val id = host.allocateAppWidgetId()
        pendingProvider = provider
        pendingId = id
        status = ""
        val allowed = manager.bindAppWidgetIdIfAllowed(id, provider.provider)
        if (allowed) {
            finishAdd(provider, id)
        } else {
            val intent = Intent(AppWidgetManager.ACTION_APPWIDGET_BIND)
                .putExtra(AppWidgetManager.EXTRA_APPWIDGET_ID, id)
                .putExtra(AppWidgetManager.EXTRA_APPWIDGET_PROVIDER, provider.provider)
            bindLauncher.launch(intent)
        }
    }

    Scaffold(
        containerColor = MaterialTheme.colorScheme.background.copy(alpha = 0.95f),
        topBar = {
            TopAppBar(
                title = { Text(stringResource(R.string.picker_title)) },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(Icons.AutoMirrored.Outlined.ArrowBack, contentDescription = stringResource(R.string.back))
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
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(padding),
        ) {
            if (status.isNotEmpty()) {
                Text(
                    text = status,
                    fontSize = 12.sp,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.padding(horizontal = 24.dp, vertical = 8.dp),
                )
            }
            LazyColumn(
                contentPadding = PaddingValues(horizontal = 20.dp, vertical = 4.dp),
                verticalArrangement = Arrangement.spacedBy(0.dp),
            ) {
                items(providers, key = { it.info.provider.flattenToString() }) { provider ->
                    val thumb = remember(provider.info.provider) {
                        previewBitmap(context, provider.info)
                    }
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        modifier = Modifier
                            .fillMaxWidth()
                            .clickable { selected = provider }
                            .padding(vertical = 10.dp),
                    ) {
                        if (thumb != null) {
                            Image(
                                bitmap = thumb.asImageBitmap(),
                                contentDescription = provider.label,
                                contentScale = ContentScale.Fit,
                                modifier = Modifier.size(40.dp),
                            )
                        } else {
                            Box(modifier = Modifier.size(40.dp))
                        }
                        Text(
                            text = provider.label,
                            fontSize = 16.sp,
                            color = MaterialTheme.colorScheme.onSurface,
                            modifier = Modifier
                                .padding(start = 12.dp)
                                .weight(1f),
                        )
                    }
                    HorizontalDivider(
                        color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.25f),
                    )
                }
            }
        }
    }

    val current = selected
    if (current != null) {
        val preview = remember(current.info.provider) {
            previewBitmap(context, current.info)
        }
        AlertDialog(
            onDismissRequest = { selected = null },
            title = { Text(current.label) },
            text = {
                if (preview != null) {
                    Image(
                        bitmap = preview.asImageBitmap(),
                        contentDescription = current.label,
                        contentScale = ContentScale.Fit,
                        modifier = Modifier
                            .fillMaxWidth()
                            .width(240.dp),
                    )
                } else {
                    Text(
                        text = stringResource(R.string.picker_no_preview),
                        fontSize = 13.sp,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            },
            confirmButton = {
                TextButton(onClick = {
                    selected = null
                    beginAdd(current.info)
                }) { Text(stringResource(R.string.picker_add)) }
            },
            dismissButton = {
                TextButton(onClick = { selected = null }) { Text(stringResource(R.string.picker_cancel)) }
            },
        )
    }
}

private fun previewBitmap(context: Context, info: AppWidgetProviderInfo): Bitmap? {
    if (info.previewImage == 0) return null
    return runCatching {
        val resources = context.packageManager
            .getResourcesForApplication(info.provider.packageName)
        BitmapFactory.decodeResource(resources, info.previewImage)
    }.getOrNull()
}
