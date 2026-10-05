package com.liwidale.liauth.ui.screens

import android.content.Intent
import android.net.Uri
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.biometric.BiometricManager
import androidx.biometric.BiometricPrompt
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import com.liwidale.liauth.ui.LiAuthIcons
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import androidx.core.content.ContextCompat
import androidx.fragment.app.FragmentActivity
import com.liwidale.liauth.core.DeviceKeystore
import com.liwidale.liauth.core.Localization
import com.liwidale.liauth.ui.LiAuthSwitch
import com.liwidale.liauth.ui.LiAuthTextField
import com.liwidale.liauth.ui.SectionLabel
import com.liwidale.liauth.ui.theme.ControlShape
import com.liwidale.liauth.ui.theme.LocalPalette
import com.liwidale.liauth.vault.VaultViewModel

private const val PROJECT_URL = "https://github.com/liwidale/liauth"
private const val DEVELOPER_URL = "https://github.com/liwidale"
private const val VERSION = "2.0.0"

@Composable
fun SettingsScreen(
    viewModel: VaultViewModel,
    onDone: () -> Unit,
    onThemeChanged: (String) -> Unit,
    onCaptureProtectionChanged: (Boolean) -> Unit,
) {
    val palette = LocalPalette.current
    val context = LocalContext.current

    var theme by remember { mutableStateOf(viewModel.getSetting("theme") ?: "system") }
    var blockCapture by remember {
        mutableStateOf(
            context.getSharedPreferences("liauth.prefs", 0).getBoolean("blockCapture", true),
        )
    }
    var biometricEnabled by remember { mutableStateOf(DeviceKeystore.isEnabled(context)) }
    var showExport by remember { mutableStateOf(false) }
    var showChangePassword by remember { mutableStateOf(false) }
    var versionClicks by remember { mutableIntStateOf(0) }
    var advancedVisible by remember { mutableStateOf(viewModel.getSetting("advancedVisible") == "true") }

    val biometricAvailable = remember {
        BiometricManager.from(context)
            .canAuthenticate(BiometricManager.Authenticators.BIOMETRIC_STRONG) ==
            BiometricManager.BIOMETRIC_SUCCESS
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
                Localization.t("settings.title"),
                style = MaterialTheme.typography.titleLarge,
                color = palette.textPrimary,
            )
        }
        Spacer(Modifier.height(20.dp))

        SectionLabel(Localization.t("settings.appearance"))
        SettingRow(Localization.t("settings.language")) {
            val languages = Localization.availableLanguages()
            val current = Localization.language.value
            var expanded by remember { mutableStateOf(false) }
            Box {
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    modifier = Modifier
                        .border(1.dp, palette.border, ControlShape)
                        .clickable { expanded = true }
                        .padding(start = 14.dp, end = 10.dp, top = 8.dp, bottom = 8.dp),
                ) {
                    Text(
                        languages.firstOrNull { it.first == current }?.second ?: current,
                        style = MaterialTheme.typography.bodyMedium,
                        color = palette.textPrimary,
                    )
                    Spacer(Modifier.width(8.dp))
                    Icon(
                        LiAuthIcons.ChevronDown,
                        contentDescription = null,
                        tint = palette.textTertiary,
                        modifier = Modifier.size(14.dp),
                    )
                }
                DropdownMenu(expanded = expanded, onDismissRequest = { expanded = false }) {
                    languages.forEach { (code, name) ->
                        DropdownMenuItem(
                            text = {
                                Text(
                                    name,
                                    color = if (code == current) palette.textPrimary else palette.textSecondary,
                                )
                            },
                            onClick = {
                                expanded = false
                                Localization.setLanguage(context, code)
                            },
                        )
                    }
                }
            }
        }
        var animations by remember { mutableStateOf(viewModel.getSetting("animations") != "false") }
        SettingRow(Localization.t("settings.animations")) {
            LiAuthSwitch(checked = animations) {
                animations = it
                viewModel.setSetting("animations", it.toString())
            }
        }
        Hint(Localization.t("settings.animationsHint"))

        var brandIcons by remember { mutableStateOf(viewModel.getSetting("brandIcons") == "true") }
        SettingRow(Localization.t("settings.brandIcons")) {
            LiAuthSwitch(checked = brandIcons) {
                brandIcons = it
                viewModel.setSetting("brandIcons", it.toString())
            }
        }
        Hint(Localization.t("settings.brandIconsHint"))

        SettingRow(Localization.t("settings.theme")) {
            val options = listOf(
                "system" to Localization.t("settings.themeSystem"),
                "light" to Localization.t("settings.themeLight"),
                "dark" to Localization.t("settings.themeDark"),
            )
            Row {
                options.forEach { (value, label) ->
                    Text(
                        label,
                        style = MaterialTheme.typography.bodyMedium,
                        color = if (value == theme) palette.textPrimary else palette.textTertiary,
                        modifier = Modifier
                            .clickable {
                                theme = value
                                viewModel.setSetting("theme", value)
                                onThemeChanged(value)
                            }
                            .padding(start = 14.dp),
                    )
                }
            }
        }

        Spacer(Modifier.height(20.dp))
        SectionLabel(Localization.t("settings.security"))
        var hideCodes by remember { mutableStateOf(viewModel.getSetting("hideCodes") == "true") }
        SettingRow(Localization.t("settings.hideCodes")) {
            LiAuthSwitch(checked = hideCodes) {
                hideCodes = it
                viewModel.setSetting("hideCodes", it.toString())
            }
        }
        Hint(Localization.t("settings.hideCodesHint"))
        SettingRow(Localization.t("settings.blockCapture")) {
            LiAuthSwitch(checked = blockCapture) {
                blockCapture = it
                onCaptureProtectionChanged(it)
            }
        }
        Hint(Localization.t("settings.blockCaptureHint"))

        if (biometricAvailable) {
            SettingRow(Localization.t("settings.biometricUnlock")) {
                LiAuthSwitch(checked = biometricEnabled) { enable ->
                    if (enable) {
                        val activity = context as? FragmentActivity ?: return@LiAuthSwitch
                        val cipher = runCatching { DeviceKeystore.encryptCipher() }.getOrNull()
                            ?: return@LiAuthSwitch
                        val prompt = BiometricPrompt(
                            activity,
                            ContextCompat.getMainExecutor(context),
                            object : BiometricPrompt.AuthenticationCallback() {
                                override fun onAuthenticationSucceeded(
                                    result: BiometricPrompt.AuthenticationResult,
                                ) {
                                    result.cryptoObject?.cipher?.let { c ->
                                        viewModel.enableBiometricSlot(c) {}
                                        biometricEnabled = true
                                    }
                                }
                            },
                        )
                        val info = BiometricPrompt.PromptInfo.Builder()
                            .setTitle(Localization.t("lock.biometricPrompt"))
                            .setNegativeButtonText(Localization.t("action.cancel"))
                            .build()
                        prompt.authenticate(info, BiometricPrompt.CryptoObject(cipher))
                    } else {
                        viewModel.disableBiometricSlot()
                        biometricEnabled = false
                    }
                }
            }
            Hint(Localization.t("settings.biometricHint"))
        }

        ActionRow(Localization.t("settings.changePassword")) { showChangePassword = true }
        ActionRow(Localization.t("settings.lockNow")) { viewModel.lock() }

        Spacer(Modifier.height(20.dp))
        SectionLabel(Localization.t("settings.backup"))
        ActionRow(Localization.t("settings.exportBackup")) { showExport = true }
        Hint(Localization.t("settings.exportHint"))

        var autoBackup by remember { mutableStateOf(viewModel.autoBackupEnabled()) }
        SettingRow(Localization.t("settings.autoBackup")) {
            LiAuthSwitch(checked = autoBackup) { enabled ->
                autoBackup = enabled
                viewModel.setAutoBackup(enabled) {}
            }
        }
        Hint(Localization.t("settings.autoBackupHint"))

        var webdavConfigured by remember { mutableStateOf(viewModel.webdavIsConfigured()) }
        var showWebdav by remember { mutableStateOf(false) }
        SettingRow(Localization.t("webdav.title")) {
            Row {
                if (webdavConfigured) {
                    Text(
                        Localization.t("webdav.syncNow"),
                        style = MaterialTheme.typography.bodyMedium,
                        color = palette.textPrimary,
                        modifier = Modifier
                            .clickable {
                                viewModel.webdavSyncNow(
                                    onDone = { viewModel.notify(Localization.t("webdav.done")) },
                                    onError = { viewModel.notify(it) },
                                )
                            }
                            .padding(start = 14.dp),
                    )
                    Text(
                        Localization.t("webdav.disable"),
                        style = MaterialTheme.typography.bodyMedium,
                        color = palette.textTertiary,
                        modifier = Modifier
                            .clickable {
                                viewModel.webdavConfigure("", "", "", "", onDone = {
                                    webdavConfigured = false
                                }, onError = {})
                            }
                            .padding(start = 14.dp),
                    )
                } else {
                    Text(
                        Localization.t("webdav.configure"),
                        style = MaterialTheme.typography.bodyMedium,
                        color = palette.textPrimary,
                        modifier = Modifier
                            .clickable { showWebdav = true }
                            .padding(start = 14.dp),
                    )
                }
            }
        }
        Hint(Localization.t("webdav.hint"))
        if (showWebdav) {
            WebDavDialog(
                viewModel = viewModel,
                onConfigured = { webdavConfigured = true; showWebdav = false },
                onDismiss = { showWebdav = false },
            )
        }

        Spacer(Modifier.height(20.dp))
        SectionLabel(Localization.t("settings.data"))
        SettingRow(Localization.t("timeSync.title")) {
            Text(
                Localization.t("timeSync.run"),
                style = MaterialTheme.typography.bodyMedium,
                color = palette.textPrimary,
                modifier = Modifier.clickable {
                    viewModel.syncTimeDrift(
                        onDone = { offset ->
                            viewModel.notify(
                                Localization.tf("timeSync.result", "seconds" to offset.toString()),
                            )
                        },
                        onError = { viewModel.notify(Localization.t("sync.failed")) },
                    )
                },
            )
        }
        Hint(Localization.tf("timeSync.current", "seconds" to viewModel.timeDriftSeconds().toString()))

        Spacer(Modifier.height(20.dp))
        SectionLabel(Localization.t("settings.about"))
        SettingRow(Localization.t("settings.project")) {
            Text(
                "liwidale/liauth",
                style = MaterialTheme.typography.bodyMedium,
                color = palette.textPrimary,
                modifier = Modifier.clickable {
                    context.startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(PROJECT_URL)))
                },
            )
        }
        SettingRow(Localization.t("settings.developer")) {
            Text(
                "liwidale",
                style = MaterialTheme.typography.bodyMedium,
                color = palette.textPrimary,
                modifier = Modifier.clickable {
                    context.startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(DEVELOPER_URL)))
                },
            )
        }
        SettingRow(Localization.t("settings.version")) {
            Text(
                VERSION,
                style = MaterialTheme.typography.bodyMedium,
                color = palette.textSecondary,
                modifier = Modifier.clickable {
                    versionClicks += 1
                    if (versionClicks >= 5 && !advancedVisible) {
                        advancedVisible = true
                        viewModel.setSetting("advancedVisible", "true")
                        viewModel.notify(Localization.t("settings.advancedUnlocked"))
                    }
                },
            )
        }

        if (advancedVisible) {
            Spacer(Modifier.height(20.dp))
            SectionLabel(Localization.t("settings.advanced"))
            Hint(Localization.t("settings.advancedHint"))
            SettingRow(Localization.t("settings.advancedVisible")) {
                LiAuthSwitch(checked = advancedVisible) {
                    advancedVisible = it
                    viewModel.setSetting("advancedVisible", it.toString())
                }
            }
        }
        Spacer(Modifier.height(48.dp))
    }

    if (showExport) {
        ExportDialog(viewModel = viewModel, onDismiss = { showExport = false })
    }
    if (showChangePassword) {
        ChangePasswordDialog(viewModel = viewModel, onDismiss = { showChangePassword = false })
    }
}

@Composable
private fun SettingRow(label: String, content: @Composable () -> Unit) {
    val palette = LocalPalette.current
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 10.dp),
    ) {
        Text(
            label,
            style = MaterialTheme.typography.bodyLarge,
            color = palette.textPrimary,
            modifier = Modifier.weight(1f),
        )
        content()
    }
}

@Composable
private fun ActionRow(label: String, onClick: () -> Unit) {
    val palette = LocalPalette.current
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(onClick = onClick)
            .padding(vertical = 12.dp),
    ) {
        Text(label, style = MaterialTheme.typography.bodyLarge, color = palette.textPrimary)
    }
}

@Composable
private fun Hint(text: String) {
    val palette = LocalPalette.current
    Text(
        text,
        style = MaterialTheme.typography.bodySmall,
        color = palette.textTertiary,
        modifier = Modifier.padding(bottom = 6.dp),
    )
}

@Composable
private fun WebDavDialog(
    viewModel: VaultViewModel,
    onConfigured: () -> Unit,
    onDismiss: () -> Unit,
) {
    val palette = LocalPalette.current
    var url by remember { mutableStateOf("") }
    var username by remember { mutableStateOf("") }
    var password by remember { mutableStateOf("") }
    var backupPassword by remember { mutableStateOf("") }
    var busy by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }

    AlertDialog(
        onDismissRequest = onDismiss,
        containerColor = palette.surface,
        title = { Text(Localization.t("webdav.title"), color = palette.textPrimary) },
        text = {
            Column {
                LiAuthTextField(value = url, onValueChange = { url = it }, hint = Localization.t("webdav.url"))
                Spacer(Modifier.height(8.dp))
                LiAuthTextField(
                    value = username,
                    onValueChange = { username = it },
                    hint = Localization.t("webdav.username"),
                )
                Spacer(Modifier.height(8.dp))
                LiAuthTextField(
                    value = password,
                    onValueChange = { password = it },
                    hint = Localization.t("webdav.password"),
                    password = true,
                )
                Spacer(Modifier.height(8.dp))
                LiAuthTextField(
                    value = backupPassword,
                    onValueChange = { backupPassword = it },
                    hint = Localization.t("webdav.backupPassword"),
                    password = true,
                )
                Spacer(Modifier.height(4.dp))
                Text(
                    Localization.t("webdav.backupPasswordHint"),
                    style = MaterialTheme.typography.bodySmall,
                    color = palette.textTertiary,
                )
                error?.let {
                    Spacer(Modifier.height(8.dp))
                    Text(it, style = MaterialTheme.typography.bodySmall, color = palette.textSecondary)
                }
            }
        },
        confirmButton = {
            TextButton(
                enabled = !busy && url.trim().startsWith("http") && backupPassword.length >= 4,
                onClick = {
                    busy = true
                    viewModel.webdavConfigure(
                        url.trim(), username, password, backupPassword,
                        onDone = {
                            viewModel.notify(Localization.t("webdav.done"))
                            onConfigured()
                        },
                        onError = {
                            busy = false
                            error = it
                        },
                    )
                },
            ) {
                Text(
                    if (busy) Localization.t("webdav.uploading") else Localization.t("action.save"),
                    color = palette.textPrimary,
                )
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) {
                Text(Localization.t("action.cancel"), color = palette.textSecondary)
            }
        },
    )
}

@Composable
private fun ExportDialog(viewModel: VaultViewModel, onDismiss: () -> Unit) {
    val palette = LocalPalette.current
    var password by remember { mutableStateOf("") }
    var confirm by remember { mutableStateOf("") }
    var pendingBytes by remember { mutableStateOf<ByteArray?>(null) }

    val saver = rememberLauncherForActivityResult(
        ActivityResultContracts.CreateDocument("application/octet-stream"),
    ) { uri ->
        val bytes = pendingBytes
        if (uri != null && bytes != null) {
            val written = runCatching {
                viewModel.getApplication<android.app.Application>().contentResolver
                    .openOutputStream(uri)?.use { it.write(bytes) }
            }.isSuccess
            viewModel.notify(
                if (written) Localization.t("toast.backupSaved") else Localization.t("error.saveFailed"),
            )
        }
        pendingBytes = null
        onDismiss()
    }

    AlertDialog(
        onDismissRequest = onDismiss,
        containerColor = palette.surface,
        title = { Text(Localization.t("export.title"), color = palette.textPrimary) },
        text = {
            Column {
                Text(
                    Localization.t("export.subtitle"),
                    color = palette.textSecondary,
                    style = MaterialTheme.typography.bodyMedium,
                )
                Spacer(Modifier.height(12.dp))
                LiAuthTextField(
                    value = password,
                    onValueChange = { password = it },
                    hint = Localization.t("field.password"),
                    password = true,
                )
                Spacer(Modifier.height(8.dp))
                LiAuthTextField(
                    value = confirm,
                    onValueChange = { confirm = it },
                    hint = Localization.t("field.confirmPassword"),
                    password = true,
                )
            }
        },
        confirmButton = {
            TextButton(
                enabled = password.length >= 4 && password == confirm,
                onClick = {
                    viewModel.exportBackup(
                        password,
                        onReady = { bytes ->
                            pendingBytes = bytes
                            saver.launch("liauth-backup.liauth")
                        },
                        onError = {},
                    )
                },
            ) {
                Text(Localization.t("export.save"), color = palette.textPrimary)
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) {
                Text(Localization.t("action.cancel"), color = palette.textSecondary)
            }
        },
    )
}

@Composable
private fun ChangePasswordDialog(viewModel: VaultViewModel, onDismiss: () -> Unit) {
    val palette = LocalPalette.current
    var current by remember { mutableStateOf("") }
    var new by remember { mutableStateOf("") }
    var confirm by remember { mutableStateOf("") }
    var error by remember { mutableStateOf<String?>(null) }

    AlertDialog(
        onDismissRequest = onDismiss,
        containerColor = palette.surface,
        title = { Text(Localization.t("settings.changePassword"), color = palette.textPrimary) },
        text = {
            Column {
                LiAuthTextField(
                    value = current,
                    onValueChange = { current = it; error = null },
                    hint = Localization.t("field.currentPassword"),
                    password = true,
                )
                Spacer(Modifier.height(8.dp))
                LiAuthTextField(
                    value = new,
                    onValueChange = { new = it },
                    hint = Localization.t("field.newPassword"),
                    password = true,
                )
                Spacer(Modifier.height(8.dp))
                LiAuthTextField(
                    value = confirm,
                    onValueChange = { confirm = it },
                    hint = Localization.t("field.confirmPassword"),
                    password = true,
                )
                error?.let {
                    Spacer(Modifier.height(8.dp))
                    Text(it, color = palette.textSecondary, style = MaterialTheme.typography.bodySmall)
                }
            }
        },
        confirmButton = {
            TextButton(
                enabled = current.isNotEmpty() && new.length >= 4 && new == confirm,
                onClick = {
                    viewModel.changePassword(
                        current,
                        new,
                        onDone = {
                            viewModel.notify(Localization.t("toast.passwordChanged"))
                            onDismiss()
                        },
                        onError = {
                            error = Localization.t("lock.wrongPassword")
                            current = ""
                        },
                    )
                },
            ) {
                Text(Localization.t("action.save"), color = palette.textPrimary)
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) {
                Text(Localization.t("action.cancel"), color = palette.textSecondary)
            }
        },
    )
}
