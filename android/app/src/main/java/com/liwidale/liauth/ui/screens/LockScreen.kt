package com.liwidale.liauth.ui.screens

import androidx.biometric.BiometricManager
import androidx.biometric.BiometricPrompt
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.core.content.ContextCompat
import androidx.fragment.app.FragmentActivity
import com.liwidale.liauth.core.DeviceKeystore
import com.liwidale.liauth.core.Localization
import com.liwidale.liauth.ui.LiAuthTextField
import com.liwidale.liauth.ui.PrimaryButton
import com.liwidale.liauth.ui.theme.LocalPalette
import com.liwidale.liauth.vault.VaultState
import com.liwidale.liauth.vault.VaultViewModel

@Composable
fun LockScreen(viewModel: VaultViewModel) {
    val palette = LocalPalette.current
    val context = LocalContext.current
    val state by viewModel.state.collectAsState()
    val onboarding = state == VaultState.Onboarding

    var password by remember { mutableStateOf("") }
    var confirm by remember { mutableStateOf("") }
    var error by remember { mutableStateOf<String?>(null) }
    var resetStage by remember { mutableStateOf(0) }
    var resetInput by remember { mutableStateOf("") }
    var lockoutSeconds by remember { mutableStateOf(0uL) }

    // Progressive anti-brute-force delay: tick the countdown once a second
    // while it is active and keep the unlock button disabled.
    LaunchedEffect(onboarding) {
        if (!onboarding) {
            while (true) {
                lockoutSeconds = viewModel.lockoutRemainingSeconds()
                kotlinx.coroutines.delay(1000)
            }
        }
    }

    val biometricAvailable = remember {
        BiometricManager.from(context)
            .canAuthenticate(BiometricManager.Authenticators.BIOMETRIC_STRONG) ==
            BiometricManager.BIOMETRIC_SUCCESS
    }

    fun promptBiometric() {
        val activity = context as? FragmentActivity ?: return
        val cipher = runCatching { DeviceKeystore.decryptCipher(context) }.getOrNull() ?: return
        val prompt = BiometricPrompt(
            activity,
            ContextCompat.getMainExecutor(context),
            object : BiometricPrompt.AuthenticationCallback() {
                override fun onAuthenticationSucceeded(result: BiometricPrompt.AuthenticationResult) {
                    val unlocked = result.cryptoObject?.cipher
                        ?.let { DeviceKeystore.readSlotKey(context, it) }
                    if (unlocked != null) {
                        viewModel.unlockWithDeviceSlot(unlocked) {
                            error = Localization.t("lock.biometricUnavailable")
                        }
                    } else {
                        error = Localization.t("lock.biometricUnavailable")
                    }
                }
            },
        )
        val info = BiometricPrompt.PromptInfo.Builder()
            .setTitle(Localization.t("lock.biometricPrompt"))
            .setNegativeButtonText(Localization.t("action.cancel"))
            .build()
        prompt.authenticate(info, BiometricPrompt.CryptoObject(cipher))
    }

    LaunchedEffect(Unit) {
        if (!onboarding && biometricAvailable && DeviceKeystore.isEnabled(context)) {
            promptBiometric()
        }
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(horizontal = 32.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
    ) {
        androidx.compose.foundation.Image(
            painter = androidx.compose.ui.res.painterResource(com.liwidale.liauth.R.drawable.liauth_wordmark),
            contentDescription = Localization.t("app.name"),
            colorFilter = androidx.compose.ui.graphics.ColorFilter.tint(palette.textPrimary),
            modifier = Modifier.height(34.dp),
        )
        Spacer(Modifier.height(6.dp))
        Text(
            text = if (onboarding) Localization.t("onboarding.subtitle") else Localization.t("lock.subtitle"),
            style = MaterialTheme.typography.bodyLarge,
            color = palette.textSecondary,
            textAlign = TextAlign.Center,
        )
        Spacer(Modifier.height(40.dp))

        Column(modifier = Modifier.widthIn(max = 340.dp)) {
            LiAuthTextField(
                value = password,
                onValueChange = { password = it; error = null },
                hint = Localization.t("field.password"),
                password = true,
            )
            if (onboarding) {
                Spacer(Modifier.height(10.dp))
                LiAuthTextField(
                    value = confirm,
                    onValueChange = { confirm = it; error = null },
                    hint = Localization.t("field.confirmPassword"),
                    password = true,
                )
            }
            if (lockoutSeconds > 0uL) {
                Spacer(Modifier.height(10.dp))
                Text(
                    Localization.tf("lock.tooManyAttempts", "seconds" to lockoutSeconds.toString()),
                    style = MaterialTheme.typography.bodyMedium,
                    color = palette.textSecondary,
                )
            } else {
                error?.let {
                    Spacer(Modifier.height(10.dp))
                    Text(it, style = MaterialTheme.typography.bodyMedium, color = palette.textSecondary)
                }
            }
            Spacer(Modifier.height(20.dp))

            val ready = if (onboarding) {
                password.length >= 4 && password == confirm
            } else {
                password.isNotEmpty() && lockoutSeconds == 0uL
            }
            PrimaryButton(
                text = if (onboarding) Localization.t("onboarding.create") else Localization.t("lock.unlock"),
                enabled = ready,
                onClick = {
                    if (onboarding) {
                        viewModel.createVault(password) { error = Localization.t("error.createFailed") }
                    } else {
                        viewModel.unlock(password) {
                            lockoutSeconds = viewModel.lockoutRemainingSeconds()
                            error = Localization.t("lock.wrongPassword")
                            password = ""
                        }
                    }
                },
            )

            if (!onboarding && biometricAvailable && DeviceKeystore.isEnabled(context)) {
                Spacer(Modifier.height(8.dp))
                TextButton(
                    onClick = { promptBiometric() },
                    modifier = Modifier.align(Alignment.CenterHorizontally),
                ) {
                    Text(
                        Localization.t("settings.biometricUnlock"),
                        color = palette.textSecondary,
                        style = MaterialTheme.typography.bodyMedium,
                    )
                }
            }

            if (!onboarding) {
                Spacer(Modifier.height(4.dp))
                TextButton(
                    onClick = { resetStage = 1; resetInput = "" },
                    modifier = Modifier.align(Alignment.CenterHorizontally),
                ) {
                    Text(
                        Localization.t("lock.forgot"),
                        color = palette.textSecondary,
                        style = MaterialTheme.typography.bodyMedium,
                    )
                }
            }
        }
    }

    if (resetStage == 1) {
        androidx.compose.material3.AlertDialog(
            onDismissRequest = { resetStage = 0 },
            containerColor = palette.surface,
            shape = com.liwidale.liauth.ui.theme.DialogShape,
            title = { Text(Localization.t("reset.title"), color = palette.textPrimary) },
            text = { Text(Localization.t("reset.warning"), color = palette.textSecondary) },
            confirmButton = {
                TextButton(onClick = { resetStage = 2 }) {
                    Text(Localization.t("reset.yes"), color = palette.danger)
                }
            },
            dismissButton = {
                TextButton(onClick = { resetStage = 0 }) {
                    Text(Localization.t("action.cancel"), color = palette.textSecondary)
                }
            },
        )
    }
    if (resetStage == 2) {
        val phrase = "delete all data"
        androidx.compose.material3.AlertDialog(
            onDismissRequest = { resetStage = 0 },
            containerColor = palette.surface,
            shape = com.liwidale.liauth.ui.theme.DialogShape,
            title = { Text(Localization.t("reset.title"), color = palette.textPrimary) },
            text = {
                Column {
                    Text(
                        Localization.tf("reset.typePhrase", "phrase" to phrase),
                        color = palette.textSecondary,
                    )
                    Spacer(Modifier.height(12.dp))
                    LiAuthTextField(value = resetInput, onValueChange = { resetInput = it }, hint = phrase)
                }
            },
            confirmButton = {
                TextButton(
                    enabled = resetInput.trim().equals(phrase, ignoreCase = true),
                    onClick = {
                        resetStage = 0
                        viewModel.resetVault()
                        viewModel.notify(Localization.t("reset.done"))
                    },
                ) {
                    Text(Localization.t("reset.confirm"), color = palette.danger)
                }
            },
            dismissButton = {
                TextButton(onClick = { resetStage = 0 }) {
                    Text(Localization.t("action.cancel"), color = palette.textSecondary)
                }
            },
        )
    }
}
