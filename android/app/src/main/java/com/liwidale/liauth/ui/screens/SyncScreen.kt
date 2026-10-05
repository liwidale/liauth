package com.liwidale.liauth.ui.screens

import android.os.Build
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.border
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.size
import com.liwidale.liauth.ui.LiAuthIcons
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.liwidale.liauth.core.Localization
import com.liwidale.liauth.ui.Avatar
import com.liwidale.liauth.ui.LiAuthTextField
import com.liwidale.liauth.ui.PrimaryButton
import com.liwidale.liauth.ui.SecondaryButton
import com.liwidale.liauth.ui.theme.CardShape
import com.liwidale.liauth.ui.theme.LocalPalette
import com.liwidale.liauth.vault.SyncSendResult
import com.liwidale.liauth.vault.VaultViewModel
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import uniffi.liauth.SyncPeerView
import uniffi.liauth.SyncReceiveStatus
import uniffi.liauth.SyncSession

private sealed interface SyncMode {
    data object Menu : SyncMode
    data class Receiving(val session: SyncSession) : SyncMode
    data object Discovering : SyncMode
    data class PeerList(val peers: List<SyncPeerView>) : SyncMode
    data class EnterCode(val peer: SyncPeerView) : SyncMode
    data object Sending : SyncMode
    data class Done(val message: String) : SyncMode
    data class Failed(val message: String) : SyncMode
}

@Composable
fun SyncScreen(viewModel: VaultViewModel, onDone: () -> Unit) {
    val palette = LocalPalette.current
    val scope = rememberCoroutineScope()
    var mode by remember { mutableStateOf<SyncMode>(SyncMode.Menu) }
    var code by remember { mutableStateOf("") }

    DisposableEffect(Unit) {
        onDispose { viewModel.syncStopReceiver() }
    }

    LaunchedEffect(mode) {
        if (mode is SyncMode.Receiving) {
            while (mode is SyncMode.Receiving) {
                when (val status = viewModel.syncPollReceiver()) {
                    is SyncReceiveStatus.Completed -> {
                        viewModel.refreshAll()
                        mode = SyncMode.Done(
                            Localization.tf(
                                "sync.received",
                                "added" to status.addedAccounts.toString(),
                                "skipped" to status.skipped.toString(),
                            ),
                        )
                    }
                    is SyncReceiveStatus.Failed -> {
                        mode = SyncMode.Failed(Localization.t("sync.failed"))
                    }
                    SyncReceiveStatus.Waiting -> delay(400)
                }
            }
        }
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(horizontal = 20.dp),
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
                Localization.t("sync.title"),
                style = MaterialTheme.typography.titleLarge,
                color = palette.textPrimary,
            )
        }
        Spacer(Modifier.height(20.dp))

        when (val current = mode) {
            SyncMode.Menu -> {
                Text(
                    Localization.t("sync.subtitle"),
                    style = MaterialTheme.typography.bodyMedium,
                    color = palette.textSecondary,
                )
                Spacer(Modifier.height(20.dp))
                PrimaryButton(
                    text = Localization.t("sync.receive"),
                    onClick = {
                        val session = viewModel.syncStartReceiver(deviceName())
                        mode = if (session != null) {
                            SyncMode.Receiving(session)
                        } else {
                            SyncMode.Failed(Localization.t("sync.failed"))
                        }
                    },
                )
                Spacer(Modifier.height(10.dp))
                SecondaryButton(
                    text = Localization.t("sync.send"),
                    onClick = {
                        mode = SyncMode.Discovering
                        scope.launch {
                            val peers = viewModel.syncDiscover(10000u)
                            mode = SyncMode.PeerList(peers)
                        }
                    },
                )
                Spacer(Modifier.height(10.dp))
                Text(
                    Localization.t("sync.hint"),
                    style = MaterialTheme.typography.bodySmall,
                    color = palette.textTertiary,
                )
            }

            is SyncMode.Receiving -> {
                Text(
                    Localization.t("sync.receiveTitle"),
                    style = MaterialTheme.typography.bodyMedium,
                    color = palette.textSecondary,
                )
                Spacer(Modifier.height(28.dp))
                Text(
                    "${current.session.code.substring(0, 3)} ${current.session.code.substring(3)}",
                    style = com.liwidale.liauth.ui.theme.CodeTextStyle.copy(fontSize = 42.sp),
                    color = palette.textPrimary,
                    textAlign = TextAlign.Center,
                    modifier = Modifier.fillMaxWidth(),
                )
                Spacer(Modifier.height(28.dp))
                Text(
                    Localization.t("sync.receiveHint"),
                    style = MaterialTheme.typography.bodySmall,
                    color = palette.textTertiary,
                )
            }

            SyncMode.Discovering -> Centered {
                Text(
                    Localization.t("sync.searching"),
                    color = palette.textSecondary,
                    style = MaterialTheme.typography.bodyMedium,
                )
            }

            is SyncMode.PeerList -> {
                if (current.peers.isEmpty()) {
                    Text(
                        Localization.t("sync.noDevices"),
                        color = palette.textPrimary,
                        style = MaterialTheme.typography.titleMedium,
                    )
                    Spacer(Modifier.height(6.dp))
                    Text(
                        Localization.t("sync.noDevicesHint"),
                        color = palette.textTertiary,
                        style = MaterialTheme.typography.bodySmall,
                    )
                    Spacer(Modifier.height(16.dp))
                    SecondaryButton(text = Localization.t("action.back"), onClick = { mode = SyncMode.Menu })
                } else {
                    Text(
                        Localization.t("sync.selectDevice"),
                        color = palette.textSecondary,
                        style = MaterialTheme.typography.bodyMedium,
                    )
                    Spacer(Modifier.height(12.dp))
                    current.peers.forEach { peer ->
                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(vertical = 4.dp)
                                .background(palette.surface, CardShape)
                                .border(1.dp, palette.border, CardShape)
                                .clickable {
                                    code = ""
                                    mode = SyncMode.EnterCode(peer)
                                }
                                .padding(14.dp),
                        ) {
                            Avatar(title = peer.name, size = 36.dp)
                            Spacer(Modifier.height(0.dp))
                            Text(
                                peer.name,
                                color = palette.textPrimary,
                                style = MaterialTheme.typography.titleMedium,
                                modifier = Modifier.padding(start = 12.dp),
                            )
                        }
                    }
                }
            }

            is SyncMode.EnterCode -> {
                Text(
                    Localization.tf("sync.enterCode", "name" to current.peer.name),
                    color = palette.textSecondary,
                    style = MaterialTheme.typography.bodyMedium,
                )
                Spacer(Modifier.height(14.dp))
                LiAuthTextField(
                    value = code,
                    onValueChange = { value -> code = value.filter { it.isDigit() }.take(6) },
                    hint = "000 000",
                )
                Spacer(Modifier.height(16.dp))
                PrimaryButton(
                    text = Localization.t("sync.sendNow"),
                    enabled = code.length == 6,
                    onClick = {
                        mode = SyncMode.Sending
                        scope.launch {
                            val result = viewModel.syncSend(current.peer.addresses, current.peer.port, code)
                            mode = when (result) {
                                SyncSendResult.Sent -> SyncMode.Done(Localization.t("sync.sent"))
                                SyncSendResult.WrongCode -> SyncMode.Failed(Localization.t("sync.codeRejected"))
                                SyncSendResult.Failed -> SyncMode.Failed(Localization.t("sync.failed"))
                            }
                        }
                    },
                )
            }

            SyncMode.Sending -> Centered {
                Text(
                    Localization.t("sync.sending"),
                    color = palette.textSecondary,
                    style = MaterialTheme.typography.bodyMedium,
                )
            }

            is SyncMode.Done -> Centered {
                Icon(
                    LiAuthIcons.Check,
                    contentDescription = null,
                    tint = palette.textPrimary,
                    modifier = Modifier.size(36.dp),
                )
                Spacer(Modifier.height(10.dp))
                Text(
                    current.message,
                    color = palette.textSecondary,
                    style = MaterialTheme.typography.bodyMedium,
                    textAlign = TextAlign.Center,
                )
                Spacer(Modifier.height(18.dp))
                PrimaryButton(text = Localization.t("action.done"), onClick = onDone)
            }

            is SyncMode.Failed -> Centered {
                Text(
                    Localization.t("sync.failedTitle"),
                    color = palette.textPrimary,
                    style = MaterialTheme.typography.titleMedium,
                )
                Spacer(Modifier.height(6.dp))
                Text(
                    current.message,
                    color = palette.textTertiary,
                    style = MaterialTheme.typography.bodySmall,
                    textAlign = TextAlign.Center,
                )
                Spacer(Modifier.height(18.dp))
                SecondaryButton(text = Localization.t("action.back"), onClick = { mode = SyncMode.Menu })
            }
        }
    }
}

@Composable
private fun Centered(content: @Composable () -> Unit) {
    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
        modifier = Modifier
            .fillMaxWidth()
            .padding(top = 48.dp),
    ) {
        content()
    }
}

private fun deviceName(): String =
    "${Build.MANUFACTURER} ${Build.MODEL}".trim().ifEmpty { "Android" }
