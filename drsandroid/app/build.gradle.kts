plugins {
    id("com.android.application")
}

android {
    namespace = "io.github.doukutsu_rs"

    /**
     * NOTE: If you want to change the versions of packages required for the build
     * (e.g. build tools, compile SDK, NDK) or add new ones,
     * make the appropriate changes in packages.txt. Otherwise CI could break.
     */
    compileSdk = 35
    buildToolsVersion = "35.0.1"
    ndkVersion = "28.0.13004108"

    defaultConfig {
        applicationId = "io.github.doukutsu_rs"
        minSdk = 24
        targetSdk = 35
        versionCode = 3
        versionName = "0.102.0"

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"

        ndk {
            stl = "c++_shared"
        }

        externalNativeBuild {
            cmake {
                arguments.add("-DANDROID_STL=c++_shared")
            }
        }

        val documentsAuthorityValue = "$applicationId.documents"
        manifestPlaceholders["documentsAuthority"] = documentsAuthorityValue
        buildConfigField("String", "DOCUMENTS_AUTHORITY", "\"$documentsAuthorityValue\"")

        resValue("string", "app_name", "doukutsu-rs")
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )

            ndk {
                abiFilters.addAll(listOf("arm64-v8a", "armeabi-v7a", "x86", "x86_64"))
                stl = "c++_shared"
            }

            packagingOptions {
                resources {
                    excludes.add("**/DebugProbesKt.bin")
                }
            }
        }
        debug {
            applicationIdSuffix = ".debug"
            resValue("string", "app_name", "doukutsu-rs (debug)")

            isJniDebuggable = true

            val applicationId = defaultConfig.applicationId!!
            val documentsAuthorityValue = "$applicationId$applicationIdSuffix.documents"
            manifestPlaceholders["documentsAuthority"] = documentsAuthorityValue
            buildConfigField("String", "DOCUMENTS_AUTHORITY", "\"$documentsAuthorityValue\"")
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    buildFeatures {
        viewBinding = true
        prefab = true
    }

    externalNativeBuild {
        cmake {
            path = file("src/main/cpp/CMakeLists.txt")
        }
    }

    packagingOptions {
        jniLibs {
            useLegacyPackaging = true
        }
    }
}

dependencies {
    implementation("androidx.annotation:annotation:1.9.1")
    implementation("androidx.appcompat:appcompat:1.7.1")
    implementation("androidx.constraintlayout:constraintlayout:2.2.1")
    implementation("androidx.core:core:1.15.0")
    implementation("com.google.android.material:material:1.13.0")
    implementation("androidx.games:games-controller:2.0.2")
}
