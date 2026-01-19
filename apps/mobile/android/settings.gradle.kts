pluginManagement {
    includeBuild(File(settingsDir, "gradle/plugins/compose-plugin"))

    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}

dependencyResolutionManagement {
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.name = "aurelia"
include(":app")