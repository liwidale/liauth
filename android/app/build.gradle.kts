import org.gradle.api.tasks.Copy

plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.compose.compiler)
}

android {
    namespace = "com.liwidale.liauth"
    compileSdk = 35

    defaultConfig {
        applicationId = "com.liwidale.liauth"
        minSdk = 26
        targetSdk = 35
        versionCode = 1
        versionName = rootProject.file("../Cargo.toml")
            .readLines()
            .firstOrNull { it.trim().startsWith("version") }
            ?.substringAfter('"')?.substringBefore('"') ?: "1.0.0"
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            isShrinkResources = false
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
            signingConfig = signingConfigs.getByName("debug")
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    kotlinOptions {
        jvmTarget = "17"
    }
    buildFeatures {
        compose = true
    }
    packaging {
        jniLibs {
            useLegacyPackaging = false
        }
    }
    sourceSets {
        getByName("main") {
            assets.srcDir(layout.buildDirectory.dir("generated/liauthAssets"))
            java.srcDir(layout.buildDirectory.dir("generated/uniffi/java"))
            kotlin.srcDir(layout.buildDirectory.dir("generated/uniffi/java"))
        }
    }
}

dependencies {
    implementation(libs.androidx.core.ktx)
    implementation(libs.androidx.lifecycle.runtime.ktx)
    implementation(libs.androidx.lifecycle.viewmodel.compose)
    implementation(libs.androidx.activity.compose)
    implementation(libs.androidx.fragment)
    implementation(platform(libs.androidx.compose.bom))
    implementation(libs.androidx.compose.ui)
    implementation(libs.androidx.compose.ui.graphics)
    implementation(libs.androidx.compose.material3)
    implementation(libs.androidx.navigation.compose)
    implementation(libs.androidx.biometric)
    implementation(libs.androidx.camera.core)
    implementation(libs.androidx.camera.camera2)
    implementation(libs.androidx.camera.lifecycle)
    implementation(libs.androidx.camera.view)
    implementation(libs.zxing.core)
    implementation("${libs.jna.get()}@aar")
}

val rustTargets = listOf(
    "aarch64-linux-android" to "arm64-v8a",
    "armv7-linux-androideabi" to "armeabi-v7a",
    "x86_64-linux-android" to "x86_64",
)

val cargoBuild = tasks.register<Exec>("cargoBuild") {
    workingDir = rootProject.file("..")
    environment("CARGO_PROFILE_RELEASE_STRIP", "none")
    commandLine(
        "cargo", "ndk",
        *rustTargets.flatMap { listOf("-t", it.second) }.toTypedArray(),
        "-o", project.file("src/main/jniLibs").absolutePath,
        "build", "--release", "-p", "liauth-ffi",
    )
}

val generateBindings = tasks.register<Exec>("generateUniffiBindings") {
    dependsOn(cargoBuild)
    workingDir = rootProject.file("..")
    outputs.dir(layout.buildDirectory.dir("generated/uniffi/java"))
    commandLine(
        "cargo", "run", "--release", "--bin", "uniffi-bindgen", "--",
        "generate",
        "--library", rootProject.file("../target/aarch64-linux-android/release/libliauth.so").absolutePath,
        "--language", "kotlin",
        "--no-format",
        "--out-dir", layout.buildDirectory.dir("generated/uniffi/java").get().asFile.absolutePath,
    )
}

val copyLocalization = tasks.register<Copy>("copyLocalization") {
    from(rootProject.file("../localization"))
    into(layout.buildDirectory.dir("generated/liauthAssets/localization"))
}

val copyBranding = tasks.register<Copy>("copyBranding") {
    from(rootProject.file("../branding/logo.png"))
    into(layout.buildDirectory.dir("generated/liauthAssets/branding"))
}

val copyBrandIcons = tasks.register<Copy>("copyBrandIcons") {
    from(rootProject.file("../assets/icons")) {
        include("*.png")
    }
    into(layout.buildDirectory.dir("generated/liauthAssets/icons"))
}

tasks.named("preBuild") {
    dependsOn(generateBindings, copyLocalization, copyBranding, copyBrandIcons)
}

tasks.withType<org.jetbrains.kotlin.gradle.tasks.KotlinCompile>().configureEach {
    dependsOn(generateBindings)
}

tasks.withType<JavaCompile>().configureEach {
    dependsOn(generateBindings)
}
