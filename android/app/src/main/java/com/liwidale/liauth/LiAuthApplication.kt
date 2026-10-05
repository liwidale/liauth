package com.liwidale.liauth

import android.app.Application
import com.liwidale.liauth.core.CrashReporter
import com.liwidale.liauth.core.Localization

class LiAuthApplication : Application() {

    override fun onCreate() {
        super.onCreate()
        CrashReporter.install(this)
        Localization.initialize(this)
    }
}
