# Add project specific ProGuard rules here.
# By default, the flags in this file are appended to flags specified
# in /usr/local/Cellar/android-sdk/24.3.3/tools/proguard/proguard-android.txt
# (or the directory containing the SDK)

# Keep line numbers, source file names, and Annotations (crucial for JNA and stack traces)
-keepattributes SourceFile,LineNumberTable,*Annotation*,Signature,EnclosingMethod

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

# Kotlinx Serialization Rules
-keepclassmembers class * {
    *** Companion;
}
-keepclassmembers class * {
    *** $serializer;
}

# Ignore missing AWT classes referenced by JNA on Android
-dontwarn java.awt.**

# Room Database keep rules (WorkManager database is instantiated via reflection)
-keep class * extends androidx.room.RoomDatabase {
    <init>(...);
}

# WorkManager Worker keep rules
-keep class * extends androidx.work.ListenableWorker {
    <init>(android.content.Context, androidx.work.WorkerParameters);
}

