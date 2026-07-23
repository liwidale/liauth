package com.liwidale.liauth.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.liwidale.liauth.core.Localization
import com.liwidale.liauth.ui.Avatar
import com.liwidale.liauth.ui.LiAuthIcons
import com.liwidale.liauth.ui.theme.CardShape
import com.liwidale.liauth.ui.theme.LocalPalette
import com.liwidale.liauth.vault.VaultViewModel

@Composable
fun TrashScreen(viewModel: VaultViewModel, onDone: () -> Unit) {
    val palette = LocalPalette.current
    val trash by viewModel.trash.collectAsState()
    val brandIcons = viewModel.getSetting("brandIcons") == "true"

    LaunchedEffect(Unit) { viewModel.refreshTrash() }

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
                Localization.t("trash.title"),
                style = MaterialTheme.typography.titleLarge,
                color = palette.textPrimary,
            )
        }
        Spacer(Modifier.height(6.dp))
        Text(
            Localization.t("trash.subtitle"),
            style = MaterialTheme.typography.bodyMedium,
            color = palette.textSecondary,
        )
        Spacer(Modifier.height(16.dp))

        if (trash.isEmpty()) {
            Column(
                horizontalAlignment = Alignment.CenterHorizontally,
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(top = 48.dp),
            ) {
                Text(
                    Localization.t("trash.empty"),
                    style = MaterialTheme.typography.bodyMedium,
                    color = palette.textTertiary,
                )
            }
        } else {
            LazyColumn(
                verticalArrangement = Arrangement.spacedBy(10.dp),
                modifier = Modifier.fillMaxSize(),
            ) {
                items(trash, key = { it.id }) { entry ->
                    val title = entry.issuer.ifEmpty { entry.name }
                    val nowSeconds = System.currentTimeMillis() / 1000
                    val daysLeft = ((entry.purgeAt - nowSeconds).coerceAtLeast(0) + 86_399) / 86_400
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        modifier = Modifier
                            .fillMaxWidth()
                            .background(palette.surface, CardShape)
                            .border(1.dp, palette.border, CardShape)
                            .padding(horizontal = 16.dp, vertical = 12.dp),
                    ) {
                        Avatar(title = title, size = 38.dp, branded = brandIcons)
                        Spacer(Modifier.width(12.dp))
                        Column(modifier = Modifier.weight(1f)) {
                            Text(title, style = MaterialTheme.typography.titleMedium, color = palette.textPrimary)
                            if (entry.issuer.isNotEmpty() && entry.name.isNotEmpty()) {
                                Text(
                                    entry.name,
                                    style = MaterialTheme.typography.bodySmall,
                                    color = palette.textSecondary,
                                )
                            }
                            Text(
                                Localization.tf("trash.daysLeft", "days" to daysLeft.toString()),
                                style = MaterialTheme.typography.bodySmall,
                                color = palette.textTertiary,
                            )
                        }
                        Column(horizontalAlignment = Alignment.End) {
                            TextButton(onClick = {
                                viewModel.restoreAccount(entry.id) {}
                                viewModel.notify(Localization.t("trash.restored"))
                            }) {
                                Text(Localization.t("trash.restore"), color = palette.textPrimary)
                            }
                            TextButton(onClick = {
                                viewModel.purgeAccount(entry.id) {}
                                viewModel.notify(Localization.t("toast.accountDeleted"))
                            }) {
                                Text(Localization.t("trash.deleteForever"), color = palette.danger)
                            }
                        }
                    }
                }
                item { Spacer(Modifier.height(32.dp)) }
            }
        }
    }
}
