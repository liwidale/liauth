package com.liwidale.liauth.vault

import android.app.Application
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import com.liwidale.liauth.core.DeviceKeystore
import java.io.File
import java.security.SecureRandom
import javax.crypto.Cipher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.liauth.AccountView
import uniffi.liauth.CategoryView
import uniffi.liauth.CodeView
import uniffi.liauth.ImportSummary
import uniffi.liauth.LiAuthEngine
import uniffi.liauth.LiAuthException
import uniffi.liauth.SearchResultView
import uniffi.liauth.SyncPeerView
import uniffi.liauth.SyncReceiveStatus
import uniffi.liauth.SyncSession
import uniffi.liauth.TrashedAccountView

private const val ANIMATIONS_SETTING = "animations"

enum class SyncSendResult { Sent, WrongCode, Failed }

sealed interface VaultState {
    data object Onboarding : VaultState
    data object Locked : VaultState
    data object Unlocked : VaultState
}

class VaultViewModel(application: Application) : AndroidViewModel(application) {

    private val engine: LiAuthEngine

    private val stateFlow = MutableStateFlow<VaultState>(VaultState.Locked)
    val state: StateFlow<VaultState> = stateFlow.asStateFlow()

    private val accountsFlow = MutableStateFlow<List<AccountView>>(emptyList())
    val accounts: StateFlow<List<AccountView>> = accountsFlow.asStateFlow()

    private val codesFlow = MutableStateFlow<Map<String, CodeView>>(emptyMap())
    val codes: StateFlow<Map<String, CodeView>> = codesFlow.asStateFlow()

    private val categoriesFlow = MutableStateFlow<List<CategoryView>>(emptyList())
    val categories: StateFlow<List<CategoryView>> = categoriesFlow.asStateFlow()

    private val messageFlow = MutableStateFlow<String?>(null)
    val message: StateFlow<String?> = messageFlow.asStateFlow()

    private val trashFlow = MutableStateFlow<List<TrashedAccountView>>(emptyList())
    val trash: StateFlow<List<TrashedAccountView>> = trashFlow.asStateFlow()

    /// Mirrors the "animations" vault setting so screens react to the toggle
    /// without re-reading the engine on every recomposition.
    private val animationsFlow = MutableStateFlow(true)
    val animations: StateFlow<Boolean> = animationsFlow.asStateFlow()

    init {
        engine = EngineHolder.get(application)
        stateFlow.value = if (engine.vaultExists()) VaultState.Locked else VaultState.Onboarding
        viewModelScope.launch {
            while (true) {
                if (stateFlow.value == VaultState.Unlocked) {
                    refreshCodes()
                }
                delay(1000)
            }
        }
    }

    fun consumeMessage() {
        messageFlow.value = null
    }

    fun notify(text: String) {
        messageFlow.value = text
    }

    fun createVault(password: String, onError: (String) -> Unit) = launchCatching(onError) {
        withContext(Dispatchers.Default) { engine.createVault(password) }
        stateFlow.value = VaultState.Unlocked
        refreshAll()
    }

    fun unlock(password: String, onError: (String) -> Unit) = launchCatching(onError) {
        withContext(Dispatchers.Default) { engine.unlock(password) }
        stateFlow.value = VaultState.Unlocked
        refreshAll()
    }

    fun unlockWithDeviceSlot(key: ByteArray, onError: (String) -> Unit) = launchCatching(onError) {
        withContext(Dispatchers.Default) {
            engine.unlockWithSlot(DeviceKeystore.SLOT_NAME, key)
        }
        stateFlow.value = VaultState.Unlocked
        refreshAll()
    }

    fun enableBiometricSlot(encryptCipher: Cipher, onError: (String) -> Unit) = launchCatching(onError) {
        val slotKey = ByteArray(32).also { SecureRandom().nextBytes(it) }
        withContext(Dispatchers.Default) {
            engine.addKeySlot(DeviceKeystore.SLOT_NAME, slotKey)
        }
        DeviceKeystore.storeSlotKey(getApplication(), encryptCipher, slotKey)
    }

    fun disableBiometricSlot() {
        viewModelScope.launch {
            runCatching {
                withContext(Dispatchers.Default) { engine.removeKeySlot(DeviceKeystore.SLOT_NAME) }
            }
            DeviceKeystore.disable(getApplication())
        }
    }

    fun lock() {
        engine.lock()
        accountsFlow.value = emptyList()
        codesFlow.value = emptyMap()
        categoriesFlow.value = emptyList()
        stateFlow.value = VaultState.Locked
    }

    fun resetVault() {
        engine.lock()
        File(getApplication<Application>().filesDir, "vault.liauth").delete()
        DeviceKeystore.disable(getApplication())
        accountsFlow.value = emptyList()
        codesFlow.value = emptyMap()
        categoriesFlow.value = emptyList()
        stateFlow.value = VaultState.Onboarding
    }

    fun addFromUri(uri: String, onError: (String) -> Unit) = launchCatching(onError) {
        withContext(Dispatchers.Default) { engine.addAccountUri(uri) }
        refreshAll()
    }

    fun addManual(issuer: String, name: String, secret: String, onError: (String) -> Unit) =
        launchCatching(onError) {
            withContext(Dispatchers.Default) { engine.addAccountManual(issuer, name, secret) }
            refreshAll()
        }

    fun updateAccount(
        id: String,
        issuer: String,
        name: String,
        categoryId: String?,
        pinned: Boolean,
        onError: (String) -> Unit,
    ) = launchCatching(onError) {
        withContext(Dispatchers.Default) { engine.updateAccount(id, issuer, name, categoryId, pinned) }
        refreshAll()
    }

    fun updateAdvanced(id: String, algorithm: String, digits: UInt, period: UInt, onError: (String) -> Unit) =
        launchCatching(onError) {
            withContext(Dispatchers.Default) { engine.updateAccountAdvanced(id, algorithm, digits, period) }
            refreshAll()
        }

    fun deleteAccount(id: String, onError: (String) -> Unit) = launchCatching(onError) {
        withContext(Dispatchers.Default) { engine.deleteAccount(id) }
        refreshAll()
    }

    fun deleteAccounts(ids: List<String>, onDone: (UInt) -> Unit) {
        viewModelScope.launch {
            val trashed = runCatching {
                withContext(Dispatchers.Default) { engine.deleteAccounts(ids) }
            }.getOrDefault(0u)
            refreshAll()
            onDone(trashed)
        }
    }

    fun moveAccounts(ids: List<String>, categoryId: String?, onDone: (UInt) -> Unit) {
        viewModelScope.launch {
            val moved = runCatching {
                withContext(Dispatchers.Default) { engine.setAccountsCategory(ids, categoryId) }
            }.getOrDefault(0u)
            refreshAll()
            onDone(moved)
        }
    }

    fun updateNotes(id: String, notes: String, recoveryCodes: List<String>, onError: (String) -> Unit) =
        launchCatching(onError) {
            withContext(Dispatchers.Default) { engine.updateAccountNotes(id, notes, recoveryCodes) }
            refreshAll()
        }

    fun addManualAdvanced(
        issuer: String,
        name: String,
        secret: String,
        algorithm: String,
        digits: UInt,
        period: UInt,
        onError: (String) -> Unit,
    ) = launchCatching(onError) {
        withContext(Dispatchers.Default) {
            engine.addAccountManualAdvanced(issuer, name, secret, algorithm, digits, period)
        }
        refreshAll()
    }

    suspend fun searchAccounts(query: String): List<SearchResultView> = withContext(Dispatchers.Default) {
        runCatching { engine.searchAccounts(query) }.getOrDefault(emptyList())
    }

    fun lockoutRemainingSeconds(): ULong =
        runCatching { engine.lockoutRemainingSeconds() }.getOrDefault(0u)

    fun refreshTrash() {
        viewModelScope.launch {
            trashFlow.value = runCatching {
                withContext(Dispatchers.Default) { engine.trashedAccounts() }
            }.getOrDefault(emptyList())
        }
    }

    fun restoreAccount(id: String, onError: (String) -> Unit) = launchCatching(onError) {
        withContext(Dispatchers.Default) { engine.restoreAccount(id) }
        refreshAll()
        refreshTrash()
    }

    fun purgeAccount(id: String, onError: (String) -> Unit) = launchCatching(onError) {
        withContext(Dispatchers.Default) { engine.purgeAccount(id) }
        refreshTrash()
    }

    /** Measures clock drift over SNTP; the OS clock is never changed. */
    fun syncTimeDrift(onDone: (Long) -> Unit, onError: (String) -> Unit) = launchCatching(onError) {
        val offset = withContext(Dispatchers.IO) { engine.syncTimeDrift() }
        refreshAll()
        onDone(offset)
    }

    fun timeDriftSeconds(): Long = runCatching { engine.timeDriftSeconds() }.getOrDefault(0L)

    fun setAutoBackup(enabled: Boolean, onError: (String) -> Unit) = launchCatching(onError) {
        val dir = if (enabled) {
            getApplication<Application>().getExternalFilesDir("backups")?.absolutePath
        } else {
            null
        }
        withContext(Dispatchers.Default) { engine.setAutoBackupDir(dir) }
    }

    fun autoBackupEnabled(): Boolean =
        runCatching { engine.autoBackupDir() != null }.getOrDefault(false)

    fun webdavIsConfigured(): Boolean = runCatching { engine.webdavIsConfigured() }.getOrDefault(false)

    fun webdavConfigure(
        url: String,
        username: String,
        password: String,
        backupPassword: String,
        onDone: () -> Unit,
        onError: (String) -> Unit,
    ) {
        viewModelScope.launch {
            try {
                withContext(Dispatchers.IO) { engine.webdavConfigure(url, username, password, backupPassword) }
                onDone()
            } catch (e: Exception) {
                onError(e.message.orEmpty())
            }
        }
    }

    fun webdavSyncNow(onDone: () -> Unit, onError: (String) -> Unit) {
        viewModelScope.launch {
            try {
                withContext(Dispatchers.IO) { engine.webdavSyncNow() }
                onDone()
            } catch (e: Exception) {
                onError(e.message.orEmpty())
            }
        }
    }

    /** Fire-and-forget upload used after local changes. */
    private fun webdavSyncInBackground() {
        if (!webdavIsConfigured()) return
        viewModelScope.launch {
            runCatching { withContext(Dispatchers.IO) { engine.webdavSyncNow() } }
        }
    }

    fun advanceCounter(id: String) {
        viewModelScope.launch {
            runCatching {
                withContext(Dispatchers.Default) { engine.advanceCounter(id) }
            }
            refreshCodes()
        }
    }

    fun accountUri(id: String): String? = runCatching { engine.accountUri(id) }.getOrNull()

    fun addCategory(name: String, onError: (String) -> Unit) = launchCatching(onError) {
        withContext(Dispatchers.Default) { engine.addCategory(name) }
        refreshAll()
    }

    fun renameCategory(id: String, name: String, onError: (String) -> Unit) = launchCatching(onError) {
        withContext(Dispatchers.Default) { engine.renameCategory(id, name) }
        refreshAll()
    }

    fun deleteCategory(id: String, onError: (String) -> Unit) = launchCatching(onError) {
        withContext(Dispatchers.Default) { engine.deleteCategory(id) }
        refreshAll()
    }

    fun changePassword(current: String, new: String, onDone: () -> Unit, onError: (String) -> Unit) =
        launchCatching(onError) {
            withContext(Dispatchers.Default) { engine.changePassword(current, new) }
            DeviceKeystore.disable(getApplication())
            onDone()
        }

    fun exportBackup(password: String, onReady: (ByteArray) -> Unit, onError: (String) -> Unit) =
        launchCatching(onError) {
            val bytes = withContext(Dispatchers.Default) { engine.exportBackup(password) }
            onReady(bytes)
        }

    fun importData(
        data: ByteArray,
        password: String?,
        onDone: (ImportSummary) -> Unit,
        onPasswordRequired: () -> Unit,
        onError: (String) -> Unit,
    ) {
        viewModelScope.launch {
            try {
                val summary = withContext(Dispatchers.Default) { engine.importData(data, password) }
                refreshAll()
                onDone(summary)
            } catch (e: LiAuthException.PasswordRequired) {
                onPasswordRequired()
            } catch (e: LiAuthException.WrongPassword) {
                onPasswordRequired()
            } catch (e: Exception) {
                onError(e.message.orEmpty())
            }
        }
    }

    fun getSetting(key: String): String? = runCatching { engine.getSetting(key) }.getOrNull()

    fun setSetting(key: String, value: String) {
        if (key == ANIMATIONS_SETTING) animationsFlow.value = value != "false"
        viewModelScope.launch {
            runCatching { withContext(Dispatchers.Default) { engine.setSetting(key, value) } }
        }
    }

    fun syncStartReceiver(deviceName: String): SyncSession? =
        runCatching { engine.syncStartReceiver(deviceName) }.getOrNull()

    fun syncPollReceiver(): SyncReceiveStatus =
        runCatching { engine.syncPollReceiver() }.getOrDefault(SyncReceiveStatus.Waiting)

    fun syncStopReceiver() = engine.syncStopReceiver()

    suspend fun syncDiscover(timeoutMs: ULong): List<SyncPeerView> = withContext(Dispatchers.IO) {
        runCatching { engine.syncDiscover(timeoutMs) }.getOrDefault(emptyList())
    }

    suspend fun syncSend(addresses: List<String>, port: UShort, code: String): SyncSendResult =
        withContext(Dispatchers.IO) {
            try {
                engine.syncSend(addresses, port, code)
                SyncSendResult.Sent
            } catch (e: LiAuthException.WrongPassword) {
                SyncSendResult.WrongCode
            } catch (e: Exception) {
                SyncSendResult.Failed
            }
        }

    fun refreshAll() {
        viewModelScope.launch {
            runCatching {
                accountsFlow.value = withContext(Dispatchers.Default) { engine.accounts() }
                categoriesFlow.value = withContext(Dispatchers.Default) { engine.categories() }
            }
            animationsFlow.value = getSetting(ANIMATIONS_SETTING) != "false"
            refreshCodes()
            webdavSyncInBackground()
        }
    }

    private suspend fun refreshCodes() {
        runCatching {
            val list = withContext(Dispatchers.Default) { engine.codes() }
            codesFlow.value = list.associateBy { it.id }
        }
    }

    private fun launchCatching(onError: (String) -> Unit, block: suspend () -> Unit) {
        viewModelScope.launch {
            try {
                block()
            } catch (e: Exception) {
                onError(e.message.orEmpty())
            }
        }
    }
}
