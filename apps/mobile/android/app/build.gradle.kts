plugins {
    id("com.android.application")
    id("aurelia.compose")
}

val ktlint by configurations.creating

kotlin {
    jvmToolchain(17)
}

android {
    namespace = "com.aurelia.app"
    compileSdk = 34

    defaultConfig {
        applicationId = "com.aurelia.app"
        minSdk = 34
        targetSdk = 34
        versionCode = 1
        versionName = "0.1.0"
        vectorDrawables {
            useSupportLibrary = true
        }
    }

    buildFeatures {
        compose = true
    }

    composeOptions {
        kotlinCompilerExtensionVersion = "1.5.15"
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    packaging {
        resources {
            excludes += "/META-INF/{AL2.0,LGPL2.1}"
        }
    }
    buildToolsVersion = "36.0.0"
    ndkVersion = "29.0.13846066 rc3"
}

android.sourceSets["main"]
    .jniLibs.directories
    .add("src/main/jniLibs")

tasks.register<Exec>("buildRustHost") {
    workingDir = file("$rootDir/../../..")
    commandLine("cargo", "build", "-p", "aurelia-core")
}

tasks.register<JavaExec>("formatUniffiBindings") {
  mainClass.set("com.pinterest.ktlint.Main")
  classpath = ktlint
  args("-F", "--ignore-autocorrect-failures", "src/main/java/uniffi/aurelia_core/aurelia_core.kt")
  jvmArgs("--add-opens=java.base/java.lang=ALL-UNNAMED")
  isIgnoreExitValue = true
}

tasks.register<Exec>("generateUniffiBindings") {
    workingDir = file("$rootDir/../../..")
    commandLine(
        "cargo",
        "run",
        "-p",
        "uniffi-bindgen",
        "--",
        "generate",
        "--library",
        "target/debug/aurelia_core.dll",
        "--language",
        "kotlin",
        "--config",
        "apps/mobile/android/app/src/main/java/uniffi/aurelia_core/uniffi.toml",
        "--out-dir",
        "apps/mobile/android/app/src/main/java",
        "--no-format",
    )
    dependsOn("buildRustHost")
    finalizedBy("formatUniffiBindings")
}

tasks.register<Exec>("buildRustAndroid") {
    workingDir = file("$rootDir/../../..")
    commandLine(
        "cargo",
        "ndk",
        "-t",
        "arm64-v8a",
        "-t",
        "x86_64",
        "-o",
        "apps/mobile/android/app/src/main/jniLibs",
        "build",
        "-p",
        "aurelia-core",
        "--release",
    )
}

tasks.register("copyUniffiLibs") {
    dependsOn("buildRustAndroid")
    doLast {
        copy {
            from("src/main/jniLibs/arm64-v8a/libaurelia_core.so")
            into("src/main/jniLibs/arm64-v8a")
            rename("libaurelia_core.so", "libuniffi_aurelia_core.so")
        }
        copy {
            from("src/main/jniLibs/x86_64/libaurelia_core.so")
            into("src/main/jniLibs/x86_64")
            rename("libaurelia_core.so", "libuniffi_aurelia_core.so")
        }
    }
}

tasks.named("preBuild") {
    dependsOn("generateUniffiBindings", "copyUniffiLibs")
}

dependencies {
    val composeBom = platform("androidx.compose:compose-bom:2024.10.00")
    implementation(composeBom)
    androidTestImplementation(composeBom)

    implementation("androidx.activity:activity-compose:1.9.3")
    implementation("androidx.compose.material3:material3:1.3.1")
    implementation("androidx.compose.material:material-icons-core")
    implementation("androidx.compose.material:material-icons-extended")
    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.ui:ui-tooling-preview")
    implementation("androidx.compose.animation:animation-graphics")
    implementation("androidx.lifecycle:lifecycle-runtime-ktx:2.8.7")
    implementation("androidx.lifecycle:lifecycle-viewmodel-compose:2.8.7")
    implementation("androidx.navigation:navigation-compose:2.8.4")
    implementation("androidx.media3:media3-exoplayer:1.4.1")
    implementation("androidx.media3:media3-common:1.4.1")
    implementation("androidx.media3:media3-session:1.4.1")
    implementation("com.google.android.material:material:1.12.0")
    implementation("net.java.dev.jna:jna:5.15.0@aar")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.9.0")
    implementation("io.coil-kt:coil-compose:2.6.0")
    implementation("com.squareup.okhttp3:okhttp:4.12.0")

    debugImplementation("androidx.compose.ui:ui-tooling")

    ktlint("com.pinterest.ktlint:ktlint-cli:1.8.0") {
        attributes {
            attribute(Bundling.BUNDLING_ATTRIBUTE, objects.named(Bundling.EXTERNAL))
        }
    }
}
