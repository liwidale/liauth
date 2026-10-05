package com.liwidale.liauth.core

import android.content.Context
import java.io.File
import java.util.Locale
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import org.json.JSONObject

data class LanguagePack(val code: String, val name: String, val strings: Map<String, String>)

object Localization {

    private const val PREFS = "liauth.prefs"
    private const val KEY_LANGUAGE = "language"

    private var packs: List<LanguagePack> = emptyList()
    private var fallback: Map<String, String> = emptyMap()

    private val activeCodeFlow = MutableStateFlow("en")
    val language: StateFlow<String> get() = activeCodeFlow

    fun initialize(context: Context) {
        val loaded = mutableListOf<LanguagePack>()
        val assetDir = "localization"
        val assetFiles = context.assets.list(assetDir).orEmpty().filter { it.endsWith(".json") }
        for (file in assetFiles) {
            val code = file.removeSuffix(".json")
            runCatching {
                context.assets.open("$assetDir/$file").bufferedReader().use { it.readText() }
            }.getOrNull()?.let { raw ->
                parsePack(code, raw)?.let(loaded::add)
            }
        }
        val userDir = File(context.filesDir, "languages")
        if (userDir.isDirectory) {
            userDir.listFiles { f -> f.extension == "json" }?.forEach { file ->
                parsePack(file.nameWithoutExtension, file.readText())?.let { pack ->
                    val index = loaded.indexOfFirst { it.code == pack.code }
                    if (index >= 0) loaded[index] = pack else loaded.add(pack)
                }
            }
        }
        packs = loaded
        fallback = loaded.firstOrNull { it.code == "en" }?.strings.orEmpty()

        val saved = context.getSharedPreferences(PREFS, Context.MODE_PRIVATE).getString(KEY_LANGUAGE, null)
        val system = Locale.getDefault().language.lowercase(Locale.ROOT)
        val requested = saved ?: system
        activeCodeFlow.value = when {
            loaded.any { it.code == requested } -> requested
            loaded.any { it.code == system } -> system
            else -> "en"
        }
    }

    fun availableLanguages(): List<Pair<String, String>> = packs.map { it.code to it.name }

    fun setLanguage(context: Context, code: String) {
        if (packs.any { it.code == code }) {
            activeCodeFlow.value = code
            context.getSharedPreferences(PREFS, Context.MODE_PRIVATE)
                .edit()
                .putString(KEY_LANGUAGE, code)
                .apply()
        }
    }

    fun t(key: String): String {
        val active = packs.firstOrNull { it.code == activeCodeFlow.value }
        return active?.strings?.get(key) ?: fallback[key] ?: key
    }

    fun tf(key: String, vararg args: Pair<String, String>): String {
        var value = t(key)
        for ((name, replacement) in args) {
            value = value.replace("{$name}", replacement)
        }
        return value
    }

    private fun parsePack(code: String, raw: String): LanguagePack? = runCatching {
        val json = JSONObject(raw)
        val strings = buildMap {
            json.keys().forEach { key ->
                val value = json.optString(key)
                if (value.isNotEmpty()) put(key, value)
            }
        }
        LanguagePack(code, strings["language.name"] ?: code.uppercase(Locale.ROOT), strings)
    }.getOrNull()
}
