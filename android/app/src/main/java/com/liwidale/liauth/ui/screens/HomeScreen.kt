package com.liwidale.liauth.ui.screens

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Arrangement
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
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Checkbox
import androidx.compose.material3.CheckboxDefaults
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SwipeToDismissBox
import androidx.compose.material3.SwipeToDismissBoxValue
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.rememberSwipeToDismissBoxState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import com.liwidale.liauth.core.Localization
import com.liwidale.liauth.ui.Avatar
import com.liwidale.liauth.ui.CountdownBar
import com.liwidale.liauth.ui.HighlightedText
import com.liwidale.liauth.ui.LiAuthIcons
import com.liwidale.liauth.ui.LiAuthTextField
import com.liwidale.liauth.ui.PinMarker
import com.liwidale.liauth.ui.formatCode
import com.liwidale.liauth.ui.theme.CodeTextStyle
import com.liwidale.liauth.ui.theme.CardShape
import com.liwidale.liauth.ui.theme.ControlShape
import com.liwidale.liauth.ui.theme.DialogShape
import com.liwidale.liauth.ui.theme.LocalPalette
import com.liwidale.liauth.vault.VaultViewModel
import uniffi.liauth.AccountView
import uniffi.liauth.CategoryView
import uniffi.liauth.SearchResultView

@Composable
fun HomeScreen(
    viewModel: VaultViewModel,
    onAdd: () -> Unit,
    onSync: () -> Unit,
    onSettings: () -> Unit,
    onTrash: () -> Unit,
) {
    val palette = LocalPalette.current
    val context = LocalContext.current
    val accounts by viewModel.accounts.collectAsState()
    val codes by viewModel.codes.collectAsState()
    val categories by viewModel.categories.collectAsState()

    var search by remember { mutableStateOf("") }
    var selectedCategory by remember { mutableStateOf<String?>(null) }
    var deleting by remember { mutableStateOf<AccountView?>(null) }
    var editing by remember { mutableStateOf<AccountView?>(null) }
    var editingCategory by remember { mutableStateOf<CategoryView?>(null) }
    var addingCategory by remember { mutableStateOf(false) }
    var revealed by remember { mutableStateOf(setOf<String>()) }
    // Non-null while batch selection mode is active.
    var selection by remember { mutableStateOf<Set<String>?>(null) }
    var moveMenuOpen by remember { mutableStateOf(false) }
    val hideCodes = viewModel.getSetting("hideCodes") == "true"
    val brandIcons = viewModel.getSetting("brandIcons") == "true"

    // Typo-tolerant search runs in the Rust core and returns highlight indices.
    var results by remember { mutableStateOf<List<SearchResultView>>(emptyList()) }
    LaunchedEffect(search, accounts) {
        results = viewModel.searchAccounts(search)
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(horizontal = 20.dp),
    ) {
        Spacer(Modifier.height(52.dp))
        Row(verticalAlignment = Alignment.CenterVertically) {
            androidx.compose.foundation.Image(
                painter = androidx.compose.ui.res.painterResource(com.liwidale.liauth.R.drawable.liauth_wordmark),
                contentDescription = Localization.t("app.name"),
                colorFilter = androidx.compose.ui.graphics.ColorFilter.tint(palette.textPrimary),
                modifier = Modifier.height(22.dp),
            )
            Spacer(Modifier.weight(1f))
            // Same set and order as the Windows header.
            HeaderIcon(LiAuthIcons.Plus, Localization.t("nav.add"), onAdd)
            Spacer(Modifier.width(6.dp))
            HeaderIcon(LiAuthIcons.Sync, Localization.t("nav.sync"), onSync)
            Spacer(Modifier.width(6.dp))
            HeaderIcon(LiAuthIcons.Trash, Localization.t("trash.title"), onTrash)
            Spacer(Modifier.width(6.dp))
            HeaderIcon(LiAuthIcons.Settings, Localization.t("nav.settings"), onSettings)
        }
        Spacer(Modifier.height(14.dp))

        LiAuthTextField(value = search, onValueChange = { search = it }, hint = Localization.t("home.search"))
        Spacer(Modifier.height(12.dp))

        LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            item {
                CategoryChip(
                    text = Localization.t("home.all"),
                    selected = selectedCategory == null,
                    onClick = { selectedCategory = null },
                )
            }
            items(categories) { category ->
                // Tap filters, long-press edits — so the chip keeps its
                // primary job while staying editable in place.
                CategoryChip(
                    text = category.name,
                    selected = selectedCategory == category.id,
                    onClick = {
                        selectedCategory = if (selectedCategory == category.id) null else category.id
                    },
                    onLongClick = { editingCategory = category },
                )
            }
            item {
                CategoryChip(
                    text = Localization.t("home.manageCategories"),
                    selected = false,
                    onClick = { addingCategory = true },
                )
            }
        }
        Spacer(Modifier.height(12.dp))

        val filtered = results.filter { result ->
            selectedCategory == null || result.account.categoryId == selectedCategory
        }

        if (filtered.isEmpty()) {
            EmptyState(hasQuery = search.isNotBlank() || selectedCategory != null)
        } else {
            LazyColumn(
                verticalArrangement = Arrangement.spacedBy(10.dp),
                modifier = Modifier
                    .weight(1f)
                    .fillMaxWidth(),
            ) {
                items(filtered, key = { it.account.id }) { result ->
                    val account = result.account
                    SwipeableTokenCard(
                        result = result,
                        code = codes[account.id]?.code.orEmpty(),
                        secondsRemaining = codes[account.id]?.secondsRemaining ?: 0u,
                        hidden = hideCodes && account.id !in revealed,
                        brandIcons = brandIcons,
                        selecting = selection != null,
                        selected = selection?.contains(account.id) == true,
                        onToggleSelect = {
                            selection = selection?.let { current ->
                                if (account.id in current) current - account.id else current + account.id
                            }
                        },
                        onStartSelection = { selection = setOf(account.id) },
                        onReveal = { revealed = revealed + account.id },
                        onCopy = { code ->
                            copyCode(context, code)
                            viewModel.notify(Localization.t("toast.codeCopied"))
                        },
                        onNextCode = { viewModel.advanceCounter(account.id) },
                        onEdit = { editing = account },
                        onDelete = { deleting = account },
                        onTogglePin = {
                            viewModel.updateAccount(
                                account.id, account.issuer, account.name,
                                account.categoryId, !account.pinned,
                            ) {}
                        },
                    )
                }
                item { Spacer(Modifier.height(32.dp)) }
            }
        }

        selection?.let { selected ->
            Row(
                verticalAlignment = Alignment.CenterVertically,
                modifier = Modifier
                    .fillMaxWidth()
                    .background(palette.surface, CardShape)
                    .border(1.dp, palette.borderStrong, CardShape)
                    .padding(horizontal = 14.dp, vertical = 8.dp),
            ) {
                Text(
                    Localization.tf("batch.selected", "count" to selected.size.toString()),
                    style = MaterialTheme.typography.bodyMedium,
                    color = palette.textPrimary,
                    modifier = Modifier.weight(1f),
                )
                Box {
                    TextButton(onClick = { moveMenuOpen = true }, enabled = selected.isNotEmpty()) {
                        Text(Localization.t("batch.moveToGroup"), color = palette.textPrimary)
                    }
                    DropdownMenu(expanded = moveMenuOpen, onDismissRequest = { moveMenuOpen = false }) {
                        DropdownMenuItem(
                            text = { Text(Localization.t("edit.noCategory")) },
                            onClick = {
                                moveMenuOpen = false
                                viewModel.moveAccounts(selected.toList(), null) { moved ->
                                    viewModel.notify(
                                        Localization.tf("toast.movedToGroup", "count" to moved.toString()),
                                    )
                                }
                                selection = null
                            },
                        )
                        categories.forEach { category ->
                            DropdownMenuItem(
                                text = { Text(category.name) },
                                onClick = {
                                    moveMenuOpen = false
                                    viewModel.moveAccounts(selected.toList(), category.id) { moved ->
                                        viewModel.notify(
                                            Localization.tf("toast.movedToGroup", "count" to moved.toString()),
                                        )
                                    }
                                    selection = null
                                },
                            )
                        }
                    }
                }
                TextButton(
                    onClick = {
                        viewModel.deleteAccounts(selected.toList()) { trashed ->
                            viewModel.notify(
                                Localization.tf("toast.movedToTrash", "count" to trashed.toString()),
                            )
                        }
                        selection = null
                    },
                    enabled = selected.isNotEmpty(),
                ) {
                    Text(Localization.t("batch.delete"), color = palette.textPrimary)
                }
                TextButton(onClick = { selection = null }) {
                    Text(Localization.t("action.cancel"), color = palette.textSecondary)
                }
            }
            Spacer(Modifier.height(12.dp))
        }
    }

    deleting?.let { account ->
        DeleteDialog(
            account = account,
            onConfirm = {
                viewModel.deleteAccount(account.id) {}
                viewModel.notify(Localization.t("toast.accountTrashed"))
                deleting = null
            },
            onDismiss = { deleting = null },
        )
    }

    editing?.let { account ->
        EditSheet(
            viewModel = viewModel,
            account = account,
            categories = categories,
            onDismiss = { editing = null },
        )
    }

    if (addingCategory) {
        NewCategoryDialog(
            onAdd = { name ->
                viewModel.addCategory(name) {}
                addingCategory = false
            },
            onDismiss = { addingCategory = false },
        )
    }

    editingCategory?.let { category ->
        EditCategoryDialog(
            category = category,
            onRename = { name ->
                viewModel.renameCategory(category.id, name) {}
                viewModel.notify(Localization.t("toast.saved"))
                editingCategory = null
            },
            onDelete = {
                viewModel.deleteCategory(category.id) {}
                if (selectedCategory == category.id) selectedCategory = null
                editingCategory = null
            },
            onDismiss = { editingCategory = null },
        )
    }
}

/** Creates a group and nothing else; existing groups are edited by long-press. */
@Composable
private fun NewCategoryDialog(onAdd: (String) -> Unit, onDismiss: () -> Unit) {
    val palette = LocalPalette.current
    var name by remember { mutableStateOf("") }
    AlertDialog(
        onDismissRequest = onDismiss,
        containerColor = palette.surface,
        shape = DialogShape,
        title = { Text(Localization.t("categories.newTitle"), color = palette.textPrimary) },
        text = {
            Column {
                Text(
                    Localization.t("categories.subtitle"),
                    style = MaterialTheme.typography.bodySmall,
                    color = palette.textSecondary,
                )
                Spacer(Modifier.height(12.dp))
                LiAuthTextField(
                    value = name,
                    onValueChange = { name = it },
                    hint = Localization.t("categories.newPlaceholder"),
                )
                Spacer(Modifier.height(8.dp))
                Text(
                    Localization.t("categories.hintMobile"),
                    style = MaterialTheme.typography.bodySmall,
                    color = palette.textTertiary,
                )
            }
        },
        confirmButton = {
            TextButton(enabled = name.isNotBlank(), onClick = { onAdd(name.trim()) }) {
                Text(Localization.t("categories.add"), color = palette.textPrimary)
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) {
                Text(Localization.t("action.cancel"), color = palette.textSecondary)
            }
        },
    )
}

/** Rename or remove one group; opened by long-pressing its chip. */
@Composable
private fun EditCategoryDialog(
    category: CategoryView,
    onRename: (String) -> Unit,
    onDelete: () -> Unit,
    onDismiss: () -> Unit,
) {
    val palette = LocalPalette.current
    var name by remember(category.id) { mutableStateOf(category.name) }
    AlertDialog(
        onDismissRequest = onDismiss,
        containerColor = palette.surface,
        shape = DialogShape,
        title = { Text(Localization.t("categories.edit"), color = palette.textPrimary) },
        text = {
            Column {
                Text(
                    Localization.t("categories.editSubtitle"),
                    style = MaterialTheme.typography.bodySmall,
                    color = palette.textSecondary,
                )
                Spacer(Modifier.height(12.dp))
                LiAuthTextField(
                    value = name,
                    onValueChange = { name = it },
                    hint = Localization.t("categories.newPlaceholder"),
                )
                Spacer(Modifier.height(4.dp))
                TextButton(onClick = onDelete) {
                    Text(Localization.t("categories.delete"), color = palette.danger)
                }
            }
        },
        confirmButton = {
            TextButton(enabled = name.isNotBlank(), onClick = { onRename(name.trim()) }) {
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

@Composable
private fun HeaderIcon(icon: ImageVector, description: String, onClick: () -> Unit) {
    val palette = LocalPalette.current
    IconButton(
        onClick = onClick,
        modifier = Modifier
            .size(36.dp)
            .border(1.dp, palette.border, ControlShape),
    ) {
        Icon(
            icon,
            contentDescription = description,
            tint = palette.textPrimary,
            modifier = Modifier.size(18.dp),
        )
    }
}

@OptIn(ExperimentalFoundationApi::class)
@Composable
private fun CategoryChip(
    text: String,
    selected: Boolean,
    onClick: () -> Unit,
    onLongClick: (() -> Unit)? = null,
) {
    val palette = LocalPalette.current
    val background = if (selected) palette.accent else palette.background
    val content = if (selected) palette.accentText else palette.textSecondary
    Box(
        modifier = Modifier
            .background(background, ControlShape)
            .border(1.dp, if (selected) background else palette.border, ControlShape)
            .combinedClickable(onClick = onClick, onLongClick = onLongClick)
            .padding(horizontal = 14.dp, vertical = 7.dp),
    ) {
        Text(
            text,
            style = MaterialTheme.typography.labelSmall,
            color = content,
        )
    }
}

/** Swipe right to edit, swipe left to delete; the card itself handles taps. */
@Composable
private fun SwipeableTokenCard(
    result: SearchResultView,
    code: String,
    secondsRemaining: UInt,
    hidden: Boolean,
    brandIcons: Boolean,
    selecting: Boolean,
    selected: Boolean,
    onToggleSelect: () -> Unit,
    onStartSelection: () -> Unit,
    onReveal: () -> Unit,
    onCopy: (String) -> Unit,
    onNextCode: () -> Unit,
    onEdit: () -> Unit,
    onDelete: () -> Unit,
    onTogglePin: () -> Unit,
) {
    val palette = LocalPalette.current
    val dismissState = rememberSwipeToDismissBoxState(
        confirmValueChange = { value ->
            when (value) {
                SwipeToDismissBoxValue.StartToEnd -> onEdit()
                SwipeToDismissBoxValue.EndToStart -> onDelete()
                SwipeToDismissBoxValue.Settled -> {}
            }
            // The card always snaps back; deletion is confirmed via dialog.
            false
        },
    )

    if (selecting) {
        TokenCard(
            result, code, secondsRemaining, hidden, brandIcons,
            selecting, selected, onToggleSelect, onStartSelection,
            onReveal, onCopy, onNextCode, onEdit, onDelete, onTogglePin,
        )
        return
    }

    SwipeToDismissBox(
        state = dismissState,
        backgroundContent = {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                modifier = Modifier
                    .fillMaxSize()
                    .background(palette.surfaceRaised, CardShape)
                    .padding(horizontal = 18.dp),
            ) {
                Text(
                    Localization.t("menu.edit"),
                    style = MaterialTheme.typography.bodyMedium,
                    color = palette.textSecondary,
                )
                Spacer(Modifier.weight(1f))
                Text(
                    Localization.t("menu.delete"),
                    style = MaterialTheme.typography.bodyMedium,
                    color = palette.textSecondary,
                )
            }
        },
    ) {
        TokenCard(
            result, code, secondsRemaining, hidden, brandIcons,
            selecting, selected, onToggleSelect, onStartSelection,
            onReveal, onCopy, onNextCode, onEdit, onDelete, onTogglePin,
        )
    }
}

@OptIn(ExperimentalFoundationApi::class)
@Composable
private fun TokenCard(
    result: SearchResultView,
    code: String,
    secondsRemaining: UInt,
    hidden: Boolean,
    brandIcons: Boolean,
    selecting: Boolean,
    selected: Boolean,
    onToggleSelect: () -> Unit,
    onStartSelection: () -> Unit,
    onReveal: () -> Unit,
    onCopy: (String) -> Unit,
    onNextCode: () -> Unit,
    onEdit: () -> Unit,
    onDelete: () -> Unit,
    onTogglePin: () -> Unit,
) {
    val palette = LocalPalette.current
    var menuOpen by remember { mutableStateOf(false) }
    val account = result.account
    val title = account.issuer.ifEmpty { account.name }
    val titleIndices = if (account.issuer.isEmpty()) result.nameIndices else result.issuerIndices

    Box {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            modifier = Modifier
                .fillMaxWidth()
                .background(palette.surface, CardShape)
                .border(1.dp, palette.border, CardShape)
                .combinedClickable(
                    onClick = {
                        when {
                            selecting -> onToggleSelect()
                            hidden -> onReveal()
                            else -> onCopy(code)
                        }
                    },
                    onLongClick = { if (!selecting) menuOpen = true },
                )
                .padding(horizontal = 16.dp, vertical = 14.dp),
        ) {
            if (selecting) {
                Checkbox(
                    checked = selected,
                    onCheckedChange = { onToggleSelect() },
                    colors = CheckboxDefaults.colors(
                        checkedColor = palette.accent,
                        checkmarkColor = palette.accentText,
                        uncheckedColor = palette.borderStrong,
                    ),
                )
                Spacer(Modifier.width(6.dp))
            }
            Avatar(title = title, size = 42.dp, branded = brandIcons)
            Spacer(Modifier.width(14.dp))
            Column(modifier = Modifier.weight(1f)) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    HighlightedText(
                        text = title,
                        indices = titleIndices,
                        style = MaterialTheme.typography.titleMedium,
                        color = palette.textPrimary,
                    )
                    if (account.pinned) {
                        Spacer(Modifier.width(6.dp))
                        PinMarker()
                    }
                }
                if (account.issuer.isNotEmpty() && account.name.isNotEmpty()) {
                    HighlightedText(
                        text = account.name,
                        indices = result.nameIndices,
                        style = MaterialTheme.typography.bodySmall,
                        color = palette.textSecondary,
                    )
                }
                val formatted = formatCode(code)
                Text(
                    if (hidden) formatted.map { if (it.isWhitespace()) it else '•' }.joinToString("") else formatted,
                    style = CodeTextStyle,
                    color = palette.textPrimary,
                )
                if (!account.isCounterBased) {
                    Spacer(Modifier.height(6.dp))
                    val fraction = if (account.period > 0u) {
                        secondsRemaining.toFloat() / account.period.toFloat()
                    } else {
                        0f
                    }
                    CountdownBar(fraction = fraction)
                }
            }
            if (account.isCounterBased) {
                IconButton(onClick = onNextCode) {
                    Icon(
                        LiAuthIcons.Refresh,
                        contentDescription = Localization.t("home.nextCode"),
                        tint = palette.textPrimary,
                    )
                }
            } else {
                Text(
                    secondsRemaining.toString().padStart(2, '0'),
                    style = CodeTextStyle.copy(fontSize = MaterialTheme.typography.bodySmall.fontSize),
                    color = palette.textTertiary,
                )
            }
        }

        DropdownMenu(expanded = menuOpen, onDismissRequest = { menuOpen = false }) {
            DropdownMenuItem(
                text = { Text(Localization.t("menu.copyCode")) },
                onClick = { onCopy(code); menuOpen = false },
            )
            DropdownMenuItem(
                text = { Text(Localization.t("menu.select")) },
                onClick = { onStartSelection(); menuOpen = false },
            )
            DropdownMenuItem(
                text = { Text(Localization.t("menu.edit")) },
                onClick = { onEdit(); menuOpen = false },
            )
            DropdownMenuItem(
                text = {
                    Text(
                        if (account.pinned) Localization.t("menu.unpin") else Localization.t("menu.pin"),
                    )
                },
                onClick = { onTogglePin(); menuOpen = false },
            )
            DropdownMenuItem(
                text = { Text(Localization.t("menu.delete")) },
                onClick = { onDelete(); menuOpen = false },
            )
        }
    }
}

@Composable
private fun EmptyState(hasQuery: Boolean) {
    val palette = LocalPalette.current
    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        modifier = Modifier
            .fillMaxWidth()
            .padding(top = 72.dp),
    ) {
        androidx.compose.foundation.Image(
            painter = androidx.compose.ui.res.painterResource(com.liwidale.liauth.R.drawable.liauth_logo),
            contentDescription = null,
            colorFilter = androidx.compose.ui.graphics.ColorFilter.tint(palette.textPrimary),
            modifier = Modifier.size(72.dp),
        )
        Spacer(Modifier.height(18.dp))
        Text(
            if (hasQuery) Localization.t("home.noResultsTitle") else Localization.t("home.emptyTitle"),
            style = MaterialTheme.typography.titleLarge,
            color = palette.textPrimary,
        )
        Spacer(Modifier.height(4.dp))
        Text(
            if (hasQuery) Localization.t("home.noResultsSubtitle") else Localization.t("home.emptySubtitle"),
            style = MaterialTheme.typography.bodyMedium,
            color = palette.textSecondary,
        )
    }
}

@Composable
private fun DeleteDialog(account: AccountView, onConfirm: () -> Unit, onDismiss: () -> Unit) {
    val palette = LocalPalette.current
    val title = account.issuer.ifEmpty { account.name }
    AlertDialog(
        onDismissRequest = onDismiss,
        containerColor = palette.surface,
        shape = DialogShape,
        title = {
            Text(
                Localization.tf("delete.title", "name" to title),
                color = palette.textPrimary,
            )
        },
        text = { Text(Localization.t("delete.trashSubtitle"), color = palette.textSecondary) },
        confirmButton = {
            TextButton(onClick = onConfirm) {
                Text(Localization.t("delete.confirm"), color = palette.danger)
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) {
                Text(Localization.t("action.cancel"), color = palette.textSecondary)
            }
        },
    )
}

private fun copyCode(context: Context, code: String) {
    val clipboard = context.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
    clipboard.setPrimaryClip(ClipData.newPlainText("code", code))
}
