package com.liwidale.liauth.core

import android.content.Context
import java.io.File

object CrashReporter {

    private const val FILE_NAME = "last-crash.txt"

    fun install(context: Context) {
        val appContext = context.applicationContext
        val previous = Thread.getDefaultUncaughtExceptionHandler()
        Thread.setDefaultUncaughtExceptionHandler { thread, throwable ->
            runCatching {
                File(appContext.filesDir, FILE_NAME).writeText(throwable.stackTraceToString())
            }
            previous?.uncaughtException(thread, throwable)
        }
    }

    fun consume(context: Context): String? {
        val file = File(context.filesDir, FILE_NAME)
        if (!file.exists()) return null
        val text = runCatching { file.readText() }.getOrNull()
        file.delete()
        return text?.take(4000)
    }
}
