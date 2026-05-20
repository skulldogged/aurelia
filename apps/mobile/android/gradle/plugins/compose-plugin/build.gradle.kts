plugins {
  `kotlin-dsl`
}

gradlePlugin {
  plugins {
    create("aurelia-compose") {
      id = "aurelia.compose"
      implementationClass = "ComposePlugin"
    }
  }
}

repositories {
  google()
  mavenCentral()
}

dependencies {
  implementation("org.jetbrains.kotlin.plugin.compose:org.jetbrains.kotlin.plugin.compose.gradle.plugin:2.3.0")
}
