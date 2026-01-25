# Add project specific ProGuard rules here.
# By default, the flags in this file are appended to flags specified
# in /usr/local/Cellar/android-sdk/24.3.3/tools/proguard/proguard-android.txt
# (or the directory containing the SDK)

# Don't obfuscate in debug to allow stack traces to be readable
-dontobfuscate

# Keep line numbers, source file names, and Annotations (crucial for JNA)
-keepattributes SourceFile,LineNumberTable,*Annotation*,Signature,EnclosingMethod

# Jetpack Compose specific rules
-keep class androidx.compose.** { *; }

# JNA (Java Native Access) Rules
# Prevent R8 from stripping JNA classes and native methods
-keep class com.sun.jna.** { *; }
-keep class * implements com.sun.jna.** { *; }
-keep class * extends com.sun.jna.** { *; }

# Specifically keep Structure fields as they are accessed via reflection
-keepclassmembers class * extends com.sun.jna.Structure {
    public <fields>;
}

# Uniffi generated code (which uses JNA)
-keep class uniffi.** { *; }
