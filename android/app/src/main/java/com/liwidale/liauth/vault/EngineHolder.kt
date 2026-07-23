package com.liwidale.liauth.vault

import android.content.Context
import java.io.File
import uniffi.liauth.LiAuthEngine

/**
 * Process-wide engine instance. The main UI and the autofill / credential
 * services live in the same process, so they share the unlocked vault state
 * through this holder instead of opening the vault twice.
 */
object EngineHolder {

    @Volatile
    private var engine: LiAuthEngine? = null

    fun get(context: Context): LiAuthEngine {
        return engine ?: synchronized(this) {
            engine ?: run {
                val vaultPath = File(context.applicationContext.filesDir, "vault.liauth").absolutePath
                LiAuthEngine.newMobile(vaultPath).also { engine = it }
            }
        }
    }
}
