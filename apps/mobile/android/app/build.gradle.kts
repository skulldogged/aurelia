plugins {
  id("com.android.application")
  id("aurelia.compose")
  id("org.jetbrains.kotlin.plugin.serialization") version "2.3.0"
}

val ktlint by configurations.creating

kotlin {
  jvmToolchain(17)
}

android {
  namespace = "com.aurelia.app"
  compileSdk = 36

  defaultConfig {
    applicationId = "com.aurelia.app"
    minSdk = 34
    targetSdk = 36
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

  buildTypes {
    debug {
    }
    create("fast") {
      initWith(getByName("debug"))
      isDebuggable = false
      isMinifyEnabled = true
      isDefault = true
      proguardFiles(
        getDefaultProguardFile("proguard-android-optimize.txt"),
        "proguard-rules.pro"
      )
      signingConfig = signingConfigs.getByName("debug")
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
  val composeBom = platform("androidx.compose:compose-bom:2026.01.00")
  implementation(composeBom)
  androidTestImplementation(composeBom)

  implementation("androidx.activity:activity-compose:1.12.2")
  implementation("androidx.compose.material3:material3:1.5.0-alpha12")
  implementation("androidx.compose.material:material-icons-core")
  implementation("androidx.compose.material:material-icons-extended")
  implementation("androidx.compose.ui:ui")
  implementation("androidx.compose.ui:ui-tooling-preview")
  implementation("androidx.compose.animation:animation-graphics")
  implementation("androidx.lifecycle:lifecycle-runtime-ktx:2.10.0")
  implementation("androidx.lifecycle:lifecycle-viewmodel-compose:2.10.0")
  implementation("androidx.navigation:navigation-compose:2.9.6")
  implementation("androidx.media3:media3-exoplayer:1.9.0")
  implementation("androidx.media3:media3-common:1.9.0")
  implementation("androidx.media3:media3-session:1.9.0")
  implementation("com.google.android.material:material:1.13.0")
  //noinspection Aligned16KB
  implementation("net.java.dev.jna:jna:5.18.1@aar")
  implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.10.2")
  implementation("io.coil-kt:coil-compose:2.7.0")
  implementation("com.squareup.okhttp3:okhttp:5.3.2")
  implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.10.0")
  implementation("androidx.graphics:graphics-shapes:1.1.0")
  implementation("androidx.work:work-runtime-ktx:2.10.1")

  debugImplementation("androidx.compose.ui:ui-tooling")

  ktlint("com.pinterest.ktlint:ktlint-cli:1.8.0") {
    attributes {
      attribute(Bundling.BUNDLING_ATTRIBUTE, objects.named(Bundling.EXTERNAL))
    }
  }
}
