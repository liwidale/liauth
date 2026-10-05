package com.liwidale.liauth.ui.screens

import android.Manifest
import android.content.pm.PackageManager
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.camera.core.CameraSelector
import androidx.camera.core.ImageAnalysis
import androidx.camera.core.Preview
import androidx.camera.lifecycle.ProcessCameraProvider
import androidx.camera.view.PreviewView
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import com.liwidale.liauth.ui.LiAuthIcons
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalLifecycleOwner
import androidx.compose.ui.unit.dp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.core.content.ContextCompat
import com.liwidale.liauth.core.Localization
import com.liwidale.liauth.core.QrAnalyzer
import com.liwidale.liauth.ui.LiAuthTextField
import com.liwidale.liauth.ui.PrimaryButton
import com.liwidale.liauth.ui.SecondaryButton
import com.liwidale.liauth.ui.SectionLabel
import com.liwidale.liauth.ui.theme.LocalPalette
import com.liwidale.liauth.vault.VaultViewModel
import java.util.concurrent.Executors

@Composable
private fun OptionRow(
    label: String,
    options: List<String>,
    selected: String,
    onSelect: (String) -> Unit,
) {
    val palette = LocalPalette.current
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 6.dp),
    ) {
        Text(
            label,
            style = MaterialTheme.typography.bodyMedium,
            color = palette.textSecondary,
            modifier = Modifier.weight(1f),
        )
        options.forEach { option ->
            Text(
                option,
                style = MaterialTheme.typography.bodyMedium,
                color = if (option == selected) palette.textPrimary else palette.textTertiary,
                modifier = Modifier
                    .padding(start = 14.dp)
                    .clickable { onSelect(option) },
            )
        }
    }
}

@Composable
fun AddScreen(viewModel: VaultViewModel, onDone: () -> Unit) {
    val palette = LocalPalette.current
    val context = LocalContext.current

    var issuer by remember { mutableStateOf("") }
    var name by remember { mutableStateOf("") }
    var secret by remember { mutableStateOf("") }
    var error by remember { mutableStateOf<String?>(null) }
    var scanning by remember { mutableStateOf(false) }
    var pendingImport by remember { mutableStateOf<ByteArray?>(null) }
    var importPassword by remember { mutableStateOf("") }
    var importError by remember { mutableStateOf<String?>(null) }

    val cameraPermission = rememberLauncherForActivityResult(
        ActivityResultContracts.RequestPermission(),
    ) { granted ->
        if (granted) scanning = true else error = Localization.t("permission.cameraSubtitle")
    }

    val filePicker = rememberLauncherForActivityResult(
        ActivityResultContracts.GetContent(),
    ) { uri ->
        uri ?: return@rememberLauncherForActivityResult
        val bytes = runCatching {
            context.contentResolver.openInputStream(uri)?.use { it.readBytes() }
        }.getOrNull()
        if (bytes == null) {
            error = Localization.t("import.unrecognized")
            return@rememberLauncherForActivityResult
        }
        viewModel.importData(
            data = bytes,
            password = null,
            onDone = { summary ->
                viewModel.notify(
                    Localization.tf(
                        "toast.imported",
                        "added" to summary.addedAccounts.toString(),
                        "skipped" to summary.skipped.toString(),
                    ),
                )
                onDone()
            },
            onPasswordRequired = { pendingImport = bytes },
            onError = { error = Localization.t("import.unrecognized") },
        )
    }

    fun handleScanned(content: String) {
        scanning = false
        if (content.startsWith("otpauth-migration://")) {
            viewModel.importData(
                data = content.toByteArray(),
                password = null,
                onDone = { summary ->
                    viewModel.notify(
                        Localization.tf(
                            "toast.imported",
                            "added" to summary.addedAccounts.toString(),
                            "skipped" to summary.skipped.toString(),
                        ),
                    )
                    onDone()
                },
                onPasswordRequired = { error = Localization.t("import.unrecognized") },
                onError = { error = Localization.t("import.unrecognized") },
            )
            return
        }
        var failed = false
        viewModel.addFromUri(content) {
            failed = true
            error = Localization.t("add.noQrFound")
        }
        if (!failed && content.startsWith("otpauth://")) {
            viewModel.notify(Localization.t("toast.accountAdded"))
            onDone()
        } else if (!content.startsWith("otpauth")) {
            error = Localization.t("add.noQrFound")
        }
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(horizontal = 20.dp)
            .verticalScroll(rememberScrollState()),
    ) {
        Spacer(Modifier.height(52.dp))
        Row(verticalAlignment = Alignment.CenterVertically) {
            IconButton(onClick = onDone) {
                Icon(
                    LiAuthIcons.Back,
                    contentDescription = Localization.t("action.back"),
                    tint = palette.textPrimary,
                )
            }
            Text(
                Localization.t("add.title"),
                style = MaterialTheme.typography.titleLarge,
                color = palette.textPrimary,
            )
        }
        Spacer(Modifier.height(6.dp))
        Text(
            Localization.t("add.subtitle"),
            style = MaterialTheme.typography.bodyMedium,
            color = palette.textSecondary,
            modifier = Modifier.padding(start = 12.dp),
        )
        Spacer(Modifier.height(20.dp))

        if (scanning) {
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .aspectRatio(1f)
                    .clip(com.liwidale.liauth.ui.theme.CardShape),
            ) {
                CameraScanner(onResult = ::handleScanned)
            }
            Spacer(Modifier.height(12.dp))
            SecondaryButton(text = Localization.t("action.cancel"), onClick = { scanning = false })
        } else {
            PrimaryButton(
                text = Localization.t("add.scanCamera"),
                onClick = {
                    error = null
                    val granted = ContextCompat.checkSelfPermission(
                        context, Manifest.permission.CAMERA,
                    ) == PackageManager.PERMISSION_GRANTED
                    if (granted) {
                        scanning = true
                    } else {
                        cameraPermission.launch(Manifest.permission.CAMERA)
                    }
                },
            )
            Spacer(Modifier.height(10.dp))
            SecondaryButton(
                text = Localization.t("add.importFile"),
                onClick = {
                    error = null
                    filePicker.launch("*/*")
                },
            )
        }

        Spacer(Modifier.height(28.dp))
        SectionLabel(Localization.t("add.manualSection"))
        LiAuthTextField(value = issuer, onValueChange = { issuer = it }, hint = Localization.t("field.service"))
        Spacer(Modifier.height(10.dp))
        LiAuthTextField(value = name, onValueChange = { name = it }, hint = Localization.t("field.account"))
        Spacer(Modifier.height(10.dp))
        LiAuthTextField(value = secret, onValueChange = { secret = it; error = null }, hint = Localization.t("field.key"))

        Spacer(Modifier.height(10.dp))
        var showAdvanced by remember { mutableStateOf(false) }
        var algorithm by remember { mutableStateOf("SHA1") }
        var digits by remember { mutableStateOf(6) }
        var period by remember { mutableStateOf(30) }
        TextButton(onClick = { showAdvanced = !showAdvanced }) {
            Text(
                if (showAdvanced) Localization.t("add.advancedHide") else Localization.t("add.advancedShow"),
                color = palette.textSecondary,
            )
        }
        if (showAdvanced) {
            OptionRow(
                label = Localization.t("advanced.algorithm"),
                options = listOf("SHA1", "SHA256", "SHA512"),
                selected = algorithm,
                onSelect = { algorithm = it },
            )
            OptionRow(
                label = Localization.t("advanced.digits"),
                options = listOf("6", "7", "8"),
                selected = digits.toString(),
                onSelect = { digits = it.toInt() },
            )
            OptionRow(
                label = Localization.t("advanced.period"),
                options = listOf("15", "30", "60", "90"),
                selected = period.toString(),
                onSelect = { period = it.toInt() },
            )
        }

        error?.let {
            Spacer(Modifier.height(10.dp))
            Text(it, style = MaterialTheme.typography.bodyMedium, color = palette.textSecondary)
        }

        Spacer(Modifier.height(20.dp))
        val ready = secret.isNotBlank() && (secret.trim().startsWith("otpauth") || issuer.isNotBlank())
        PrimaryButton(
            text = Localization.t("add.save"),
            enabled = ready,
            onClick = {
                val onAddError: (String) -> Unit = { error = Localization.t("add.invalidKey") }
                if (showAdvanced && !secret.trim().startsWith("otpauth")) {
                    viewModel.addManualAdvanced(
                        issuer, name, secret, algorithm, digits.toUInt(), period.toUInt(), onAddError,
                    )
                } else {
                    viewModel.addManual(issuer, name, secret, onAddError)
                }
                if (error == null) {
                    viewModel.notify(Localization.t("toast.accountAdded"))
                    onDone()
                }
            },
        )
        Spacer(Modifier.height(40.dp))
    }

    pendingImport?.let { bytes ->
        AlertDialog(
            onDismissRequest = { pendingImport = null; importPassword = ""; importError = null },
            containerColor = palette.surface,
            title = { Text(Localization.t("import.passwordTitle"), color = palette.textPrimary) },
            text = {
                Column {
                    Text(
                        Localization.t("import.passwordSubtitle"),
                        color = palette.textSecondary,
                        style = MaterialTheme.typography.bodyMedium,
                    )
                    Spacer(Modifier.height(12.dp))
                    LiAuthTextField(
                        value = importPassword,
                        onValueChange = { importPassword = it; importError = null },
                        hint = Localization.t("field.password"),
                        password = true,
                    )
                    importError?.let {
                        Spacer(Modifier.height(8.dp))
                        Text(it, color = palette.textSecondary, style = MaterialTheme.typography.bodySmall)
                    }
                }
            },
            confirmButton = {
                TextButton(
                    onClick = {
                        viewModel.importData(
                            data = bytes,
                            password = importPassword,
                            onDone = { summary ->
                                pendingImport = null
                                viewModel.notify(
                                    Localization.tf(
                                        "toast.imported",
                                        "added" to summary.addedAccounts.toString(),
                                        "skipped" to summary.skipped.toString(),
                                    ),
                                )
                                onDone()
                            },
                            onPasswordRequired = {
                                importError = Localization.t("lock.wrongPassword")
                                importPassword = ""
                            },
                            onError = { importError = Localization.t("import.unrecognized") },
                        )
                    },
                ) {
                    Text(Localization.t("import.unlock"), color = palette.textPrimary)
                }
            },
            dismissButton = {
                TextButton(onClick = { pendingImport = null; importPassword = "" }) {
                    Text(Localization.t("action.cancel"), color = palette.textSecondary)
                }
            },
        )
    }
}

@Composable
private fun CameraScanner(onResult: (String) -> Unit) {
    val context = LocalContext.current
    val lifecycleOwner = LocalLifecycleOwner.current
    val executor = remember { Executors.newSingleThreadExecutor() }
    var delivered by remember { mutableStateOf(false) }

    val provider = remember { java.util.concurrent.atomic.AtomicReference<ProcessCameraProvider?>(null) }

    DisposableEffect(Unit) {
        onDispose {
            executor.shutdown()
            runCatching { provider.get()?.unbindAll() }
        }
    }

    AndroidView(
        factory = { ctx ->
            val previewView = PreviewView(ctx)
            val providerFuture = ProcessCameraProvider.getInstance(ctx)
            providerFuture.addListener({
                val cameraProvider = runCatching { providerFuture.get() }.getOrNull()
                    ?: return@addListener
                provider.set(cameraProvider)
                val preview = Preview.Builder().build().also {
                    it.surfaceProvider = previewView.surfaceProvider
                }
                val analysis = ImageAnalysis.Builder()
                    .setBackpressureStrategy(ImageAnalysis.STRATEGY_KEEP_ONLY_LATEST)
                    .build()
                analysis.setAnalyzer(
                    executor,
                    QrAnalyzer { content ->
                        if (!delivered) {
                            delivered = true
                            previewView.post { onResult(content) }
                        }
                    },
                )
                runCatching {
                    cameraProvider.unbindAll()
                    cameraProvider.bindToLifecycle(
                        lifecycleOwner,
                        CameraSelector.DEFAULT_BACK_CAMERA,
                        preview,
                        analysis,
                    )
                }
            }, ContextCompat.getMainExecutor(ctx))
            previewView
        },
        modifier = Modifier.fillMaxSize(),
    )
}
