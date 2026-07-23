package com.liwidale.liauth

import android.os.Bundle
import android.view.WindowManager
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.fragment.app.FragmentActivity
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import com.liwidale.liauth.core.Localization
import com.liwidale.liauth.ui.screens.AddScreen
import com.liwidale.liauth.ui.screens.HomeScreen
import com.liwidale.liauth.ui.screens.LockScreen
import com.liwidale.liauth.ui.screens.SettingsScreen
import com.liwidale.liauth.ui.screens.SyncScreen
import com.liwidale.liauth.ui.screens.TrashScreen
import com.liwidale.liauth.ui.theme.LiAuthTheme
import com.liwidale.liauth.ui.theme.LocalPalette
import com.liwidale.liauth.vault.VaultState
import com.liwidale.liauth.vault.VaultViewModel

class MainActivity : FragmentActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        applyCaptureProtection(readCaptureProtection())
        setContent {
            LiAuthRoot(
                onCaptureProtectionChanged = { enabled ->
                    saveCaptureProtection(enabled)
                    applyCaptureProtection(enabled)
                },
            )
        }
    }

    private fun applyCaptureProtection(enabled: Boolean) {
        if (enabled) {
            window.setFlags(
                WindowManager.LayoutParams.FLAG_SECURE,
                WindowManager.LayoutParams.FLAG_SECURE,
            )
        } else {
            window.clearFlags(WindowManager.LayoutParams.FLAG_SECURE)
        }
    }

    private fun readCaptureProtection(): Boolean =
        getSharedPreferences("liauth.prefs", MODE_PRIVATE).getBoolean("blockCapture", true)

    private fun saveCaptureProtection(enabled: Boolean) {
        getSharedPreferences("liauth.prefs", MODE_PRIVATE)
            .edit()
            .putBoolean("blockCapture", enabled)
            .apply()
    }
}

@Composable
fun LiAuthRoot(onCaptureProtectionChanged: (Boolean) -> Unit) {
    val viewModel: VaultViewModel = viewModel()
    val state by viewModel.state.collectAsState()
    val languageCode by Localization.language.collectAsState()
    val message by viewModel.message.collectAsState()
    val context = androidx.compose.ui.platform.LocalContext.current
    var themeMode by remember { mutableStateOf("system") }

    LaunchedEffect(state) {
        if (state == VaultState.Unlocked) {
            themeMode = viewModel.getSetting("theme") ?: "system"
        }
    }

    LaunchedEffect(message) {
        message?.let {
            android.widget.Toast.makeText(context, it, android.widget.Toast.LENGTH_SHORT).show()
            viewModel.consumeMessage()
        }
    }

    var crashReport by remember {
        mutableStateOf(com.liwidale.liauth.core.CrashReporter.consume(context))
    }
    crashReport?.let { report ->
        androidx.compose.material3.AlertDialog(
            onDismissRequest = { crashReport = null },
            title = { androidx.compose.material3.Text("Error report") },
            text = {
                androidx.compose.foundation.layout.Column(
                    modifier = Modifier.verticalScroll(rememberScrollState()),
                ) {
                    androidx.compose.material3.Text(
                        report,
                        style = androidx.compose.material3.MaterialTheme.typography.bodySmall,
                    )
                }
            },
            confirmButton = {
                androidx.compose.material3.TextButton(onClick = {
                    val clipboard = context.getSystemService(android.content.Context.CLIPBOARD_SERVICE)
                        as android.content.ClipboardManager
                    clipboard.setPrimaryClip(android.content.ClipData.newPlainText("crash", report))
                    crashReport = null
                }) { androidx.compose.material3.Text("Copy") }
            },
            dismissButton = {
                androidx.compose.material3.TextButton(onClick = { crashReport = null }) {
                    androidx.compose.material3.Text("Close")
                }
            },
        )
    }

    androidx.compose.runtime.key(languageCode) {
        LiAuthTheme(themeMode = themeMode) {
            val palette = LocalPalette.current
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .background(palette.background),
            ) {
                when (state) {
                    VaultState.Onboarding, VaultState.Locked -> LockScreen(viewModel)
                    VaultState.Unlocked -> MainNavigation(
                        viewModel = viewModel,
                        onThemeChanged = { themeMode = it },
                        onCaptureProtectionChanged = onCaptureProtectionChanged,
                    )
                }
            }
        }
    }
}

@Composable
fun MainNavigation(
    viewModel: VaultViewModel,
    onThemeChanged: (String) -> Unit,
    onCaptureProtectionChanged: (Boolean) -> Unit,
) {
    val navController = rememberNavController()
    // Subtle fade/slide between screens; a single setting turns it off. Read
    // as state so flipping the switch in Settings takes effect immediately.
    val animations by viewModel.animations.collectAsState()
    val enter = if (animations) {
        androidx.compose.animation.fadeIn(androidx.compose.animation.core.tween(180)) +
            androidx.compose.animation.slideInHorizontally(
                androidx.compose.animation.core.tween(180),
                initialOffsetX = { it / 12 },
            )
    } else {
        androidx.compose.animation.EnterTransition.None
    }
    val exit = if (animations) {
        androidx.compose.animation.fadeOut(androidx.compose.animation.core.tween(140))
    } else {
        androidx.compose.animation.ExitTransition.None
    }
    NavHost(
        navController = navController,
        startDestination = "home",
        enterTransition = { enter },
        exitTransition = { exit },
        popEnterTransition = { enter },
        popExitTransition = { exit },
    ) {
        composable("home") {
            HomeScreen(
                viewModel = viewModel,
                onAdd = { navController.navigate("add") },
                onSync = { navController.navigate("sync") },
                onSettings = { navController.navigate("settings") },
                onTrash = { navController.navigate("trash") },
            )
        }
        composable("add") {
            AddScreen(viewModel = viewModel, onDone = { navController.popBackStack() })
        }
        composable("sync") {
            SyncScreen(viewModel = viewModel, onDone = { navController.popBackStack() })
        }
        composable("settings") {
            SettingsScreen(
                viewModel = viewModel,
                onDone = { navController.popBackStack() },
                onThemeChanged = onThemeChanged,
                onCaptureProtectionChanged = onCaptureProtectionChanged,
            )
        }
        composable("trash") {
            TrashScreen(viewModel = viewModel, onDone = { navController.popBackStack() })
        }
    }
}
