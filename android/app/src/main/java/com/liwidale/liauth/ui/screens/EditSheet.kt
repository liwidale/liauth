package com.liwidale.liauth.ui.screens

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilterChip
import androidx.compose.material3.FilterChipDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.liwidale.liauth.core.Localization
import com.liwidale.liauth.ui.LiAuthTextField
import com.liwidale.liauth.ui.PrimaryButton
import com.liwidale.liauth.ui.SectionLabel
import com.liwidale.liauth.ui.theme.LocalPalette
import com.liwidale.liauth.vault.VaultViewModel
import uniffi.liauth.AccountView
import uniffi.liauth.CategoryView

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun EditSheet(
    viewModel: VaultViewModel,
    account: AccountView,
    categories: List<CategoryView>,
    onDismiss: () -> Unit,
) {
    val palette = LocalPalette.current
    var issuer by remember { mutableStateOf(account.issuer) }
    var name by remember { mutableStateOf(account.name) }
    var categoryId by remember { mutableStateOf(account.categoryId) }
    var newCategory by remember { mutableStateOf("") }
    var notes by remember { mutableStateOf(account.notes) }
    var recoveryCodes by remember { mutableStateOf(account.recoveryCodes.joinToString("\n")) }

    ModalBottomSheet(
        onDismissRequest = onDismiss,
        containerColor = palette.surface,
        shape = com.liwidale.liauth.ui.theme.DialogShape,
    ) {
        Column(
            modifier = Modifier
                .padding(horizontal = 24.dp)
                .verticalScroll(rememberScrollState()),
        ) {
            Text(
                Localization.t("edit.title"),
                style = MaterialTheme.typography.titleLarge,
                color = palette.textPrimary,
            )
            Spacer(Modifier.height(16.dp))
            LiAuthTextField(value = issuer, onValueChange = { issuer = it }, hint = Localization.t("field.service"))
            Spacer(Modifier.height(10.dp))
            LiAuthTextField(value = name, onValueChange = { name = it }, hint = Localization.t("field.account"))
            Spacer(Modifier.height(18.dp))

            SectionLabel(Localization.t("edit.category"))
            LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                item {
                    CategorySelectChip(
                        text = Localization.t("edit.noCategory"),
                        selected = categoryId == null,
                        onClick = { categoryId = null },
                    )
                }
                items(categories) { category ->
                    CategorySelectChip(
                        text = category.name,
                        selected = categoryId == category.id,
                        onClick = { categoryId = category.id },
                    )
                }
            }
            Spacer(Modifier.height(10.dp))
            Row(verticalAlignment = androidx.compose.ui.Alignment.CenterVertically) {
                Column(modifier = Modifier.weight(1f)) {
                    LiAuthTextField(
                        value = newCategory,
                        onValueChange = { newCategory = it },
                        hint = Localization.t("categories.newPlaceholder"),
                    )
                }
                Spacer(Modifier.width(10.dp))
                androidx.compose.material3.TextButton(
                    onClick = {
                        if (newCategory.isNotBlank()) {
                            viewModel.addCategory(newCategory.trim()) {}
                            newCategory = ""
                        }
                    },
                ) {
                    Text(Localization.t("categories.add"), color = palette.textPrimary)
                }
            }

            Spacer(Modifier.height(18.dp))
            SectionLabel(Localization.t("edit.notes"))
            MultilineField(
                value = notes,
                onValueChange = { notes = it },
                hint = Localization.t("edit.notesHint"),
                minLines = 2,
            )
            Spacer(Modifier.height(12.dp))
            SectionLabel(Localization.t("edit.recoveryCodes"))
            MultilineField(
                value = recoveryCodes,
                onValueChange = { recoveryCodes = it },
                hint = Localization.t("edit.recoveryHint"),
                minLines = 3,
            )

            Spacer(Modifier.height(22.dp))
            PrimaryButton(
                text = Localization.t("action.save"),
                onClick = {
                    viewModel.updateAccount(account.id, issuer.trim(), name.trim(), categoryId, account.pinned) {}
                    viewModel.updateNotes(
                        account.id,
                        notes.trim(),
                        recoveryCodes.lines().map { it.trim() }.filter { it.isNotEmpty() },
                    ) {}
                    viewModel.notify(Localization.t("toast.saved"))
                    onDismiss()
                },
                modifier = Modifier.fillMaxWidth(),
            )
            Spacer(Modifier.height(32.dp))
        }
    }
}

@Composable
private fun MultilineField(
    value: String,
    onValueChange: (String) -> Unit,
    hint: String,
    minLines: Int,
) {
    val palette = LocalPalette.current
    androidx.compose.material3.OutlinedTextField(
        value = value,
        onValueChange = onValueChange,
        placeholder = { Text(hint, color = palette.textTertiary) },
        minLines = minLines,
        shape = com.liwidale.liauth.ui.theme.ControlShape,
        colors = androidx.compose.material3.OutlinedTextFieldDefaults.colors(
            focusedContainerColor = palette.surfaceRaised,
            unfocusedContainerColor = palette.surfaceRaised,
            focusedBorderColor = palette.borderStrong,
            unfocusedBorderColor = palette.border,
            focusedTextColor = palette.textPrimary,
            unfocusedTextColor = palette.textPrimary,
            cursorColor = palette.textPrimary,
        ),
        modifier = Modifier.fillMaxWidth(),
    )
}

@Composable
private fun CategorySelectChip(text: String, selected: Boolean, onClick: () -> Unit) {
    val palette = LocalPalette.current
    FilterChip(
        selected = selected,
        onClick = onClick,
        label = { Text(text) },
        shape = com.liwidale.liauth.ui.theme.ControlShape,
        colors = FilterChipDefaults.filterChipColors(
            containerColor = palette.surfaceRaised,
            labelColor = palette.textSecondary,
            selectedContainerColor = palette.accent,
            selectedLabelColor = palette.accentText,
        ),
    )
}
