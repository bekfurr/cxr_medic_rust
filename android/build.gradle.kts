plugins {
    id("com.android.application") version "8.3.0" apply false
}
// This file facilitates building via cargo-ndk or Android Studio.
// When building standalone Rust APK, simply run:
//   cargo apk build --release
// Or with cargo-ndk:
//   cargo ndk -t arm64-v8a -t armeabi-v7a -o ./android/app/src/main/jniLibs build --release

